# 数据导入系统 V2 升级方案

> 基于 v0.2.19 实际架构，2026-07-01

---

## 一、现状痛点分析

| 痛点 | 当前实现 | 问题 |
|------|---------|------|
| 列头路由硬编码 | `h_clean.contains("实验室")` / `contains("研发")` 等 5 行 if-else | 新增业务字段需改源码重编译 |
| 关联方式单一 | 方法列 → `classify_method_type()` 7 项字符串匹配 | 无法按上下文选择模糊/精确匹配 |
| 项目-实验室 1:1 假设 | 导入时项目自动挂到"研发项目"分组 | 不支持多实验室协作场景 |
| 列头关键字泄露 | 列头文本直接参与分类逻辑 | 用户修改列头措辞可能破坏导入 |
| 无冲突处理 | 重名→跳过，无告警、无合并策略 | 数据静默丢失 |

---

## 二、总体架构

```
Excel 文件
    │
    ▼
┌─────────────────────────────────────┐
│  ① 列头解析层 (Header Parser)        │
│     列头 → 标准化签名 → 匹配映射表     │
│     完全不参与业务逻辑                  │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│  ② 映射决策层 (Mapping Engine)       │
│     ┌───────────────────────────┐   │
│     │  import_mappings 配置表    │   │
│     │  header → (target, strategy)│  │
│     └───────────────────────────┘   │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│  ③ 关联策略层 (Association Engine)   │
│     精确匹配 / 模糊匹配 / 组合匹配     │
│     冲突处理 / 去重策略 / 优先级        │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│  ④ 数据写入层 (Writer)              │
│     project_groups / projects /     │
│     methods / 关联表                │
│     事务包装 + 审计日志               │
└─────────────────────────────────────┘
```

### 核心原则

- **列头仅作为标识符**：列头通过映射表找到目标，列头文本本身不参与分类
- **策略可配置**：关联策略存储在数据库表中，用户可通过 UI 配置，无需改源码
- **向后兼容**：默认映射表覆盖当前所有硬编码规则，平滑升级

---

## 三、数据结构设计

### 3.1 新增表：`import_mappings`（导入映射配置）

```sql
CREATE TABLE import_mappings (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    -- 列头匹配规则
    header_pattern TEXT NOT NULL,          -- 列头匹配模式 (支持通配符 * = 任意字符)
    match_mode    TEXT NOT NULL DEFAULT 'contains',  -- contains | exact | prefix | suffix | regex
    -- 目标
    target_table  TEXT NOT NULL,           -- project_groups | projects | methods
    target_field  TEXT NOT NULL DEFAULT 'name',  -- 目标字段名
    -- 关联策略
    assoc_type    TEXT NOT NULL DEFAULT 'name',  -- 关联到哪个字段做匹配 (name | full_name | custom)
    assoc_strategy TEXT NOT NULL DEFAULT 'exact', -- exact | fuzzy | prefix | suffix
    -- 类型标注 (仅 methods)
    default_type  TEXT DEFAULT '',         -- 默认方法类型 (液相/气相/...)
    -- 优先级 & 状态
    priority      INTEGER NOT NULL DEFAULT 100,  -- 数值越小优先级越高
    is_active     INTEGER NOT NULL DEFAULT 1,
    notes         TEXT DEFAULT '',
    created_at    TEXT NOT NULL DEFAULT (datetime('now'))
);
```

**默认种子数据（覆盖当前硬编码规则）**：

```sql
-- 实验室管理列
INSERT INTO import_mappings (header_pattern, match_mode, target_table, priority, notes) VALUES
('*实验室*',  'contains', 'project_groups', 10, '列头含"实验室"→实验室管理'),
('*laboratory*', 'contains', 'project_groups', 11, '英文兜底');

-- 研发项目列
INSERT INTO import_mappings (header_pattern, match_mode, target_table, priority, notes) VALUES
('*研发*',    'contains', 'projects', 20, '列头含"研发"→研发项目'),
('*项目*',    'contains', 'projects', 21, '列头含"项目"→研发项目'),
('*project*', 'contains', 'projects', 22, '英文兜底');

-- 方法列（含类型标注）
INSERT INTO import_mappings (header_pattern, match_mode, target_table, default_type, priority, notes) VALUES
('*液相*',    'contains', 'methods', '液相',   30, ''),
('*气相*',    'contains', 'methods', '气相',   31, ''),
('*理化*',    'contains', 'methods', '理化',   32, ''),
('*ICP*',     'contains', 'methods', 'ICP',    33, '不区分大小写'),
('*热分析*',  'contains', 'methods', '热分析', 34, ''),
('*质谱*',    'contains', 'methods', '质谱',   35, ''),
('*方法*',    'contains', 'methods', '其他',   90, '兜底：所有含"方法"的列'),
('*method*',  'contains', 'methods', '其他',   91, '英文兜底');

-- 终极兜底
INSERT INTO import_mappings (header_pattern, match_mode, target_table, default_type, priority, notes) VALUES
('*',          'contains', 'methods', '其他',   999, '最终兜底：未匹配列→方法');
```

### 3.2 匹配优先级规则

```
匹配流程 (按 priority ASC):
  1. 遍历 import_mappings WHERE is_active=1 ORDER BY priority
  2. 对每个 mapping，用 match_mode 测试列头是否匹配 header_pattern
  3. 第一个匹配成功的 → 确定 target_table + assoc_strategy + default_type
  4. 无匹配 → 跳过该列（或按第999优先级兜底）
```

### 3.3 冲突矩阵

当同一个单元格值在目标表中已存在时：

| target_table | 冲突场景 | 策略 | 行为 |
|-------------|---------|------|------|
| project_groups | 同名分组 | upsert | 跳过，日志记录 |
| projects | 同名+同 group_id | skip | 跳过 |
| projects | 同名+不同 group_id | **merge-link** | 新增 project_lab_links |
| methods | 同名 | skip | 跳过 |
| methods | 同名+不同类型 | **update-type** | 追加 method_type_links（不删除已有） |

---

## 四、关联策略引擎设计

### 4.1 策略类型枚举

```
┌────────────────────────────────────────┐
│ Strategy        │ 匹配逻辑              │
├─────────────────┼──────────────────────┤
│ exact           │ header == pattern    │
│ contains         │ header.contains(pattern)│
│ prefix          │ header.starts_with(pattern)│
│ suffix          │ header.ends_with(pattern) │
│ regex           │ Regex::new(pattern)  │
│ wildcard        │ * → 任意字符          │
└────────────────────────────────────────┘
```

存储时统一用 `wildcard` 模式（`*` 通配），匹配引擎内部转换。用户界面显示为更友好的选择器。

### 4.2 关联判断流程

```
列头 "ICP检测项目"
    │
    ▼
① HeaderParser: 标准化 → "icp检测项目"
    │
    ▼
② MappingEngine: 遍历 import_mappings ORDER BY priority
    ├── priority=10 "*实验室*"  → contains("实验室")? NO  → 继续
    ├── priority=20 "*研发*"    → contains("研发")?  NO  → 继续
    ├── priority=21 "*项目*"    → contains("项目")?  YES → MATCH!
    │     target_table = "projects"
    │     assoc_strategy = "name" (default)
    │
    ▼
③ AssociationEngine: 生成项目记录
    列数据 → projects.name
    关联策略 = name → 用 name 字段与已有 projects 对比
    │
    ▼
④ Writer: INSERT OR 冲突处理
    事务开始
    ├── INSERT INTO projects (group_id, name, method_type)
    ├── INSERT INTO project_lab_links (project_id, group_id)
    ├── audit_repo::log_on_conn("import", "projects", ...)
    事务提交
```

### 4.3 项目-实验室多对多完整流程

```
Excel 列: "重点项目"        Excel 列: "A实验室"
  [A项目]                      [A实验室]
  [B项目]                      [B实验室]
  [C项目]                      [C实验室]
       │                            │
       ▼                            ▼
  projects 表                project_groups 表
  ┌────┬──────────┐          ┌────┬─────────┐
  │ id │ name     │          │ id │ name    │
  ├────┼──────────┤          ├────┼─────────┤
  │ 1  │ A项目    │          │ 1  │ A实验室  │
  │ 2  │ B项目    │          │ 2  │ B实验室  │
  │ 3  │ C项目    │          │ 3  │ C实验室  │
  └────┴──────────┘          └────┴─────────┘
       │                            │
       └────────┬───────────────────┘
                ▼
         project_lab_links (M:N)
         ┌────────────┬──────────┐
         │ project_id │ group_id │
         ├────────────┼──────────┤
         │     1      │    1     │  A项目→A实验室
         │     1      │    2     │  A项目→B实验室
         │     2      │    1     │  B项目→A实验室
         │  ...       │   ...    │
         └────────────┴──────────┘

导入时自动关联规则:
  IF 列A=projects 且 列B=project_groups
     AND 列A和列B在同一行都有数据
  THEN 自动建立 project_lab_links 关联
```

### 4.4 边界条件处理

| 边界场景 | 处理策略 |
|---------|---------|
| 列头匹配多个映射 | 取 priority 最小的（最高优先级） |
| 同名列头但不同含义 | 用户可在 UI 中临时覆盖映射 |
| 空单元格 | 跳过，不计入统计 |
| 部分列无匹配映射 | 跳过未匹配列，不影响已匹配列导入 |
| 数据库写入失败 | 整批回滚（事务），返回错误详情 |
| 超大 Excel（>1000行） | 分批提交，每 200 行一个子事务 |
| 方法类型不存在 | 自动创建 method_types 记录 |
| 列头含特殊字符 | 标准化：去空格、转小写、去重音 |

---

## 五、API 设计

### 5.1 映射配置 CRUD

```
GET    /api/import/mappings              → 列表（支持 is_active=true 筛选）
POST   /api/import/mappings              → 新增映射规则
PUT    /api/import/mappings/:id          → 修改映射规则
DELETE /api/import/mappings/:id          → 删除映射规则
PUT    /api/import/mappings/:id/toggle   → 启用/禁用
```

### 5.2 导入预览（不写入）

```
POST /api/import/preview
  Body: multipart Excel 文件
  Response: {
    columns: [
      {
        header: "液相方法",
        matched_mapping: { id: 5, target_table: "methods", default_type: "液相" },
        sample_values: ["甲醇测定", "乙酸乙酯", ...],
        conflicts: []  // 与已有数据的冲突预览
      },
      ...
    ],
    summary: { total_rows: 42, total_columns: 5, will_create: 15, will_skip: 5 }
  }
```

### 5.3 导入执行

```
POST /api/import/execute  (替代当前 /api/methods/import)
  Body: multipart Excel + ?dry_run=true|false
  query params:
    - dry_run: bool          → 仅预览不写入
    - mapping_override: JSON → 临时覆盖映射（优先级高于 DB 配置）
    - link_projects_labs: bool → 是否自动建立项目-实验室关联
```

---

## 六、前端 UI 设计

### 6.1 导入对话框升级

```
┌─ 导入数据 ─────────────────────────────────┐
│  ┌─ 文件上传 ──────────────────────────┐    │
│  │ [选择文件]  重点项目统计.xlsx        │    │
│  └────────────────────────────────────┘    │
│                                             │
│  ┌─ 列映射预览 ────────────────────────┐    │
│  │ 列头          目标表        操作      │    │
│  │ ──────────────────────────────────  │    │
│  │ 重点项目    → 研发项目    [✎ 修改]   │    │
│  │ 液相方法    → 检测方法    [✎ 修改]   │    │
│  │ A实验室     → 实验室管理  [✎ 修改]   │    │
│  │ ICP检测     → 检测方法    [✎ 修改]   │    │
│  └────────────────────────────────────┘    │
│                                             │
│  ☑ 自动建立项目-实验室关联                  │
│  ☐ 预览模式（仅查看不导入）                  │
│                                             │
│                    [取消]  [预览]  [导入]    │
└─────────────────────────────────────────────┘
```

### 6.2 映射配置管理页

```
┌─ 映射规则管理 ──────────────────────────┐
│ [+ 新增规则]                              │
├────┬──────────┬────────┬──────┬──────┬───┤
│ 优 | 列头匹配  │ 目标表  │ 类型  │ 状态 │   │
│ 10 │ *实验室*  │ 实验室  │  -   │ ✓   │ ✎ │
│ 20 │ *研发*    │ 项目    │  -   │ ✓   │ ✎ │
│ 30 │ *液相*    │ 方法    │ 液相  │ ✓   │ ✎ │
│ 90 │ *方法*    │ 方法    │ 其他  │ ✓   │ ✎ │
│ 999│ *         │ 方法    │ 其他  │ ✓   │ ✎ │
└────┴──────────┴────────┴──────┴──────┴───┘
```

---

## 七、实施路径

### Phase 1（本次 v0.2.19 BugFix 基础上）— 最小修复

| 改动 | 文件 | 说明 |
|------|------|------|
| 导入列头匹配扩展 | `method_handler.rs` L117 | `"研发"` → `"研发"或"项目"` |
| 级联方法选择 | `ManagePage.tsx` | 项目对话框加类型筛选 |

**不改动数据结构，纯逻辑修复。**

### Phase 2（v0.3.0）— 映射表化

| 改动 | 说明 |
|------|------|
| 新建 `import_mappings` 表 | 迁移 + 种子数据 |
| MappingEngine 模块 | `src/import/mapping.rs` |
| 导入 API 重构 | `/api/import/*` 替代 `/api/methods/import` |
| 前端导入对话框重构 | 映射预览 + 手动覆盖 |

### Phase 3（v0.3.1）— 高级关联

| 改动 | 说明 |
|------|------|
| 策略引擎 | 支持 exact/fuzzy/prefix/suffix/regex 五种策略 |
| 项目-实验室自动关联 | 同行的项目和实验室列自动 link |
| 冲突预览 | 导入前展示与已有数据的冲突 |
| 分批事务 | 大数据量时每 200 行提交 |

---

## 八、异常处理策略

```
┌─────────────────────────────────────────────┐
│ 层次          │ 异常           │ 处理         │
├───────────────┼────────────────┼──────────────┤
│ 文件层        │ 格式错误       │ 拒绝导入     │
│               │ 文件过大(>10MB)│ 拒绝+提示    │
│               │ 空文件         │ 拒绝+提示    │
├───────────────┼────────────────┼──────────────┤
│ 列头层        │ 无匹配映射     │ 跳过该列+日志 │
│               │ 多映射冲突     │ 取最高优先级  │
│               │ 列头含特殊字符 │ 标准化后重试  │
├───────────────┼────────────────┼──────────────┤
│ 数据层        │ 空单元格       │ 跳过         │
│               │ 重复数据       │ 按冲突矩阵   │
│               │ 类型不存在     │ 自动创建     │
├───────────────┼────────────────┼──────────────┤
│ 写入层        │ DB 写入失败    │ 整批回滚     │
│               │ 中途中断       │ 已提交不回滚  │
└───────────────┴────────────────┴──────────────┘

所有异常均记入 audit_log，类型 = "import_error"
```
