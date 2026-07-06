# v0.3.0 数据导入系统灵活化方案

> 基于 v0.2.19，2026-07-01

## 改动清单（9 文件）

### 后端（v0.3.0/src/）

| # | 文件 | 类型 | 改动 |
|---|------|------|------|
| 1 | `db/migrations.rs` | 修改 | 新建 import_mappings 表 + 种子数据（11条规则） |
| 2 | `models/import.rs` | 新建 | ImportMapping 结构体 |
| 3 | `models/mod.rs` | 修改 | 添加 mod import |
| 4 | `repo/method_repo.rs` | 修改 | 新增 load_mappings()；batch_import_column_split 加 project_lab_pairs 参数 |
| 5 | `api/method_handler.rs` | 重写 | 列头路由改为映射表驱动；删除 classify_method_type；增加 /api/import/mappings 路由 |
| 6 | `api/mod.rs` | 修改 | 注册 import mappings 路由 |
| 7 | `Cargo.toml` | 修改 | 版本号 0.2.19 → 0.3.0 |
| 8 | `build_installer.iss` | 修改 | 路径 + 版本号 → 0.3.0 |

### 前端

| # | 文件 | 改动 |
|---|------|------|
| 9 | `frontend/src/types/index.ts` | 新增 ImportMapping 接口 |
| 10 | `frontend/src/api/client.ts` | 新增 getImportMappings() |
| 11 | `frontend/src/pages/ManagePage.tsx` | 方法级联：类型 Select → 过滤 Autocomplete；导入预览 |

---

## 核心变更

### A. import_mappings 表

```sql
CREATE TABLE IF NOT EXISTS import_mappings (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    header_pattern  TEXT NOT NULL,
    match_mode      TEXT NOT NULL DEFAULT 'contains',
    target_table    TEXT NOT NULL,
    default_type    TEXT DEFAULT '',
    priority        INTEGER NOT NULL DEFAULT 100,
    is_active       INTEGER NOT NULL DEFAULT 1,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 种子（覆盖当前所有硬编码）
INSERT INTO import_mappings (header_pattern, target_table, default_type, priority) VALUES
('*实验室*',   'project_groups', '',    10),
('*研发*',     'projects',       '',    20),
('*项目*',     'projects',       '',    21),
('*液相*',     'methods',        '液相',30),
('*气相*',     'methods',        '气相',31),
('*理化*',     'methods',        '理化',32),
('*ICP*',      'methods',        'ICP', 33),
('*热分析*',   'methods',        '热分析',34),
('*质谱*',     'methods',        '质谱',35),
('*方法*',     'methods',        '其他',90),
('*',          'methods',        '其他',999);
```

### B. 列头路由重写

```rust
// 替代 method_handler.rs 原 L99-141 的 5 行 if-else
let mappings = method_repo::load_mappings(&pool)?;

for col_idx in 0..ncols {
    let header = &headers[col_idx];
    let matched = mappings.iter().find(|m| wildcard_match(&m.header_pattern, header));
    
    match matched.map(|m| m.target_table.as_str()) {
        Some("project_groups") => { /* → group_items */ }
        Some("projects") => { /* → project_items */ }
        _ => { /* → method_items, type = m.default_type */ }
    }
}
```

### C. 项目-实验室 M:N 自动关联

导入时间行中同时存在项目列数据和实验室列数据 → 自动 INSERT INTO project_lab_links

### D. 方法级联选择

项目对话框：类型 Select（单选）→ 方法 Autocomplete（仅该类型方法，多选）
