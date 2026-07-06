# v0.3.1 BugFix 方案 — 录入映射/时间/刷新/审计

> 基于 v0.3.0，2026-07-02

---

## Bug 清单 & 根因分析

### Bug 1: 录入页 — 实验室映射方法/项目失效

**现象**：点击实验室后不显示关联的方法，类型筛选也无效。

**根因**：v0.2.17 将方法从 `projects` 表拆分到独立的 `methods` 表，但 `EntryPage.tsx` 还是用旧方式工作：

```
当前逻辑:
  getProjects({ group_id: gid }) → projects 表 (只有研发项目)
  filtered = projects.filter(p.method_type !== '研发项目')  // ❌ method_type 字段已不在 ProjectResponse 中

正确逻辑:
  应同时加载:
    1. methods → 获取所有检测方法
    2. projects({ group_id }) → 获取该实验室关联的研发项目
    3. 按 method_type_links 中的类型过滤方法
    4. 方法标签上显示关联的研发项目名
```

**文件**：`frontend/src/pages/EntryPage.tsx` 第 38-53 行

---

### Bug 2: 录入时间不是北京时间

**现象**：录入页日期时间显示 UTC 时间，比北京时间慢 8 小时。

**根因**：第 26-28 行：
```typescript
const now = new Date();
return now.toISOString().slice(0, 16);  // ❌ toISOString() 返回 UTC
```

**文件**：`frontend/src/pages/EntryPage.tsx` 第 25-28 行

---

### Bug 3: 导入方法后项目和实验室不刷新

**现象**：导入 Excel 后，项目管理和实验室管理卡片看不到新数据，必须刷新页面。

**根因**：`ManagePage.tsx` 第 396 行导入成功回调只调了 `lm()`（reload methods），没调 `lp()` 和 `lg()`：
```typescript
try { 
  const r = await methodImport(f); 
  sm(`导入成功...`); 
  lm();  // ← 只刷新了方法
  // ❌ 缺少 lg() 刷新实验室分组
  // ❌ 缺少 lp() 刷新研发项目
}
```

**文件**：`frontend/src/pages/ManagePage.tsx` 第 393-398 行

---

### Bug 4: 项目编辑后不刷新

**现象**：编辑项目保存后，列表不更新，需要退出重进。

**根因**：`hps` 函数第 49 行已有 `lp()` 调用，但需确认实际是否执行到。检查后发现保存逻辑正常，但可能存在以下情况：
- 导入后创建的项目使用 `lg()` 刷新时，项目列表 `ps` 未同步更新
- 对话框关闭时 `pf` 状态未重置，导致下次打开仍是旧数据

**文件**：`frontend/src/pages/ManagePage.tsx` 第 49、396 行

---

### Bug 5: 回收站无记录

**现象**：软删除工作记录后，回收站看不到已删除记录。

**根因**：`soft_delete()` 使用 `datetime('now')`（UTC），但 `loadTrash` 可能在时区边缘无法正确匹配。更主要的是：`getRecords({ include_deleted: true })` 返回所有记录后前端过滤 `x.deleted_at != null`。如果删除操作使用 UTC 时间戳而数据库查询使用 localtime，可能产生偏移。

排查代码确认：
- `soft_delete` (L144): `UPDATE work_records SET deleted_at=datetime('now')` — UTC
- 前端 `loadTrash`: 正常过滤 `deleted_at != null`
- **应该能工作**，但如果用户从未删除过记录 → 自然为空

**文件**：`v0.3.0/src/repo/record_repo.rs` 第 144 行

---

### Bug 6: 审计日志无记录

**现象**：操作后审计卡片无数据。

**根因**：`group_repo.rs` **完全没有审计日志调用**。创建/更新/删除实验室分组时不会写入 `audit_log`。

已验证状态：
| 操作表 | 审计状态 |
|--------|---------|
| work_records | ✅ create/update/delete/restore 均有 |
| projects | ✅ create/update/delete 均有 |
| methods | ✅ create/update/delete 均有 |
| **project_groups** | ❌ **完全没有** |

另外 `audit_repo::log()` 不设置 `created_at`，依赖 DB DEFAULT `datetime('now')`（UTC 时间），与 `audit.rs` 中使用的 `datetime('now','localtime')` 不一致。

**文件**：`v0.3.0/src/repo/group_repo.rs`

---

## 修复方案（7 处改动）

| # | 文件 | 改动 | 类型 |
|---|------|------|------|
| 1 | `frontend/src/pages/EntryPage.tsx` | 重写数据加载：加载 methods + 过滤含该 lab 的方法 | 重写 |
| 2 | `frontend/src/pages/EntryPage.tsx` | 修复时间：`toISOString()` → 本地时间 | 1行修复 |
| 3 | `frontend/src/pages/ManagePage.tsx` | 导入成功后加 `lg()` + `lp()` | 2行新增 |
| 4 | `frontend/src/pages/ManagePage.tsx` | 项目保存后加 `lg()`（lab关联变化时） | 1行新增 |
| 5 | `v0.3.0/src/repo/group_repo.rs` | create/update/delete 增加 audit_repo::log | 3处新增 |
| 6 | `v0.3.0/src/repo/audit_repo.rs` | log/log_on_conn 统一使用 `datetime('now','localtime')` | 2处修改 |
| 7 | `v0.3.0/Cargo.toml` | 版本号 → 0.3.1 | 1行修改 |

**共 7 个文件，≤ 10 个源文件 → ⚡ 快速模式。**

---

### 详细修复

#### Fix 1: EntryPage.tsx 数据加载重写

当前逻辑（错误）:
```typescript
const loadProjects = async () => {
  const r = await getProjects({ group_id: gid, active_only: true });
  setProjects(r.data);
};
const methods = projects.filter(p => p.method_type !== '研发项目');  // ❌ method_type 不存在
const filtered = typeFilter === '全部' ? methods : methods.filter(p => p.method_type === typeFilter);  // ❌
```

新逻辑（正确）:
```typescript
const loadMethods = async () => {
  const r = await getMethods();
  setAllMethods(r.data || []);
};
const loadProjects = async () => {
  const r = await getProjects({ group_id: gid });  // 获取该实验室关联的研发项目
  setProjects(r.data || []);
};

// 过滤：显示已被该实验室项目所关联的方法
const linkedMethods = useMemo(() => {
  if (!projects.length || !allMethods.length) return [];
  const linkedIds: Set<number> = new Set();
  projects.forEach(p => (p.method_ids || []).forEach(id => linkedIds.add(id)));
  return allMethods.filter(m => linkedIds.has(m.id));
}, [projects, allMethods]);

const filtered = useMemo(() => {
  if (typeFilter === '全部') return linkedMethods;
  return linkedMethods.filter(m => (m.type_names || []).includes(typeFilter));
}, [linkedMethods, typeFilter]);
```

#### Fix 2: EntryPage.tsx 本地时间

```diff
- const [dateTime, setDateTime] = useState(() => {
-   const now = new Date();
-   return now.toISOString().slice(0, 16);
- });
+ const [dateTime, setDateTime] = useState(() => {
+   const now = new Date();
+   const y = now.getFullYear();
+   const m = String(now.getMonth() + 1).padStart(2, '0');
+   const d = String(now.getDate()).padStart(2, '0');
+   const hh = String(now.getHours()).padStart(2, '0');
+   const mm = String(now.getMinutes()).padStart(2, '0');
+   return `${y}-${m}-${d}T${hh}:${mm}`;
+ });
```

#### Fix 3: ManagePage.tsx 导入后刷新

```diff
  try { 
    const r = await methodImport(f); 
    sm(`导入成功: ${r.data?.total_methods || 0}条方法, ${r.data?.total_groups || 0}个分组`); 
    lm(); 
+   lg();  // 刷新实验室分组
+   lp();  // 刷新研发项目
  }
```

#### Fix 4: group_repo.rs 审计日志

在 create (第 48 行后)、update (第 53 行后)、delete (第 66 行后) 增加：
```rust
audit_repo::log(pool, "create", "project_groups", Some(gid), "system", &format!("创建实验室: {}", body.name))?;
audit_repo::log(pool, "update", "project_groups", Some(id), "system", &format!("更新实验室: {}", body.name.as_deref().unwrap_or("")))?;
audit_repo::log(pool, "delete", "project_groups", Some(id), "system", "删除实验室")?;
```

并在文件顶部添加 `use crate::repo::audit_repo;`

#### Fix 5: audit_repo.rs 统一 localtime

```diff
- pool.get()?.execute(
-   "INSERT INTO audit_log (action, table_name, record_id, user_name, detail) VALUES (?1,?2,?3,?4,?5)",
+ pool.get()?.execute(
+   "INSERT INTO audit_log (action, table_name, record_id, user_name, detail, created_at) VALUES (?1,?2,?3,?4,?5, datetime('now','localtime'))",
```

`log_on_conn` 同理。

#### Fix 6: 所有 `datetime('now')` → `datetime('now','localtime')`

`record_repo.rs` 第 144、180 行：
```diff
- UPDATE work_records SET deleted_at=datetime('now')
+ UPDATE work_records SET deleted_at=datetime('now','localtime')
```
