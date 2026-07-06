# v0.3.2 综合修复方案

> 基于 v0.3.1，2026-07-02

## 问题清单 & 根因分析

### Bug 1: 导入后项目不默认关联实验室

**根因**：`batch_import_column_split` 创建项目时强制挂到"研发项目"伪分组，但 `project_lab_pairs` 在项目插入之后才建立实际关联。插入时 `group_id` 指向"研发项目"分组，导入后用户需要在"关联实验室"中看不到实际实验室标签。

**修复**：`project_lab_pairs` 中 "研发项目" 不应成为唯一关联。需要两步：
1. 保持 `group_id = proj_gid`（FK 需要）
2. 确保同行的实验室列也正确关联到 `project_lab_links`

当前逻辑应该正确，但需验证配对构建是否真的执行到了 `INSERT OR IGNORE INTO project_lab_links`。

---

### Bug 2: "研发项目"分组泄漏到实验室管理

**根因**：导入时自动 `INSERT OR IGNORE INTO project_groups (name) VALUES ('研发项目')`，ManagePage 无过滤。

**修复**：`ManagePage.tsx` 的 `lg()` 函数获取所有分组后过滤掉内部分组。或在后端 `group_handler.rs` `list` 中排除该分组。

**文件**：`frontend/src/pages/ManagePage.tsx` 或 `v0.3.1/src/repo/group_repo.rs`

---

### Bug 3: 删除实验室报 FOREIGN KEY constraint failed

**根因**：`projects.group_id INTEGER NOT NULL REFERENCES project_groups(id)` 无 `ON DELETE` 动作 → 默认 RESTRICT。即使 `project_lab_links` 为空，FK 仍阻止删除。

**修复**：在 `group_repo::delete()` 中，删除前先将引用该分组的所有 projects 的 `group_id` 改为"研发项目"分组：

```rust
// 将所有引用该组的项目迁移到"研发项目"分组
let default_gid: i64 = conn.query_row(
    "SELECT id FROM project_groups WHERE name='研发项目'", [], |r| r.get(0)
)?;
conn.execute("UPDATE projects SET group_id=?1 WHERE group_id=?2", rusqlite::params![default_gid, id])?;
conn.execute("DELETE FROM project_groups WHERE id=?1", [id])?;
```

**文件**：`v0.3.1/src/repo/group_repo.rs` 第 64-72 行

---

### Bug 4: 方法类型只能新建不能删除

**根因**：`deleteMethodType` API 已实现（含使用计数校验），但 ManagePage 没有提供删除 UI 按钮。

**修复**：在"方法类型管理"对话框中列出已有类型，每行加删除按钮。

**文件**：`frontend/src/pages/ManagePage.tsx` 第 335-343 行的方法类型对话框

---

### Bug 5: 项目编辑时关联实验室/方法不显示

**根因**：Autocomplete 的 `value` 必须匹配 `options` 中的对象引用。`pf.lab_ids` 是 `number[]`，但 Autocomplete 的 `value` 需要是 `option` 对象数组。当前可能用到不完整的初始值。

**修复**：改用复选框列表（见 Bug 6 方案）。

---

### Bug 6: 项目编辑关联改为复选勾选模式

**当前**：嵌套 Autocomplete（选方法类型 → Autocomplete 选方法）

**期望**：实验室和方法都用 Checkbox 列表，类型筛选用 Select + Checkbox 列表：

```
┌─ 关联实验室 ────────────────────┐
│ [方法类型筛选: ▾ 全部]           │
│ ☑ A实验室  ☑ B实验室  ☐ C实验室 │
│ ☑ D实验室  ☐ E实验室            │
└────────────────────────────────┘

┌─ 关联方法 ──────────────────────┐
│ [方法类型筛选: ▾ 液相]           │
│ ☑ 甲醇测定  ☑ 乙酸乙酯  ☐ pH测定 │
│ ☑ 残留溶剂  ☐ 重金属             │
└────────────────────────────────┘
```

**修复**：重写项目编辑对话框中实验室和方法选择区域，用 `FormGroup` + `FormControlLabel` + `Checkbox` + `useMemo` 过滤。

**文件**：`frontend/src/pages/ManagePage.tsx` 项目编辑对话框部分

---

## 改动清单（6 处修复，8 文件）

| # | 文件 | 改动 | 类型 |
|---|------|------|------|
| 1 | `v0.3.1/src/repo/group_repo.rs` | delete() 迁移 project group_id → 研发项目再删 | 修复 |
| 2 | `v0.3.1/src/repo/group_repo.rs` | list() 排除"研发项目"分组 | 修复 |
| 3 | `frontend/src/pages/ManagePage.tsx` | 方法类型管理：列出已有类型 + 删除按钮 | 新增 UI |
| 4 | `frontend/src/pages/ManagePage.tsx` | 项目编辑：重写实验室选择为 Checkbox 列表 | 重写 |
| 5 | `frontend/src/pages/ManagePage.tsx` | 项目编辑：重写方法选择为类型筛选 + Checkbox 列表 | 重写 |
| 6 | `v0.3.1/src/repo/method_repo.rs` | batch_import：验证 project_lab_pairs 正确执行 | 验证+修复 |
| 7 | `v0.3.1/Cargo.toml` | 版本号 → 0.3.2 | 1 行 |
| 8 | `v0.3.1/build_installer.iss` | 路径 → 0.3.2 | 5 处 |

**共 8 个文件，≤ 10 个源文件 → ⚡ 快速模式**
