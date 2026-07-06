# v0.3.25 更新说明

## 发布日期
2026-07-06

## 问题修复

### 修复：导出 Excel / 统计中实验室列仍显示多个实验室（v0.3.24 遗留问题）

**问题描述**：
- v0.3.24 已给 `work_records` 表添加 `group_id` 字段并保存录入时的实验室上下文
- 但导出 Excel 和统计页面中涉及实验室的数据列仍显示多个实验室（如"410,411"）
- 原因：v0.3.24 只改了数据保存层（`record_repo.rs` 的 `create` 和 `list`），但导出/统计读取层仍使用 `project_lab_links` 的 `group_concat` 或 `projects.group_id`

**根因**：
1. `src/api/export_data.rs`：所有 Sheet 的 `lab_name` 仍使用 `group_concat` 从 `project_lab_links` 查询
2. `src/api/stats_handler.rs`：`by_project` 的 `group_name` 仍使用 `group_concat` 从 `project_lab_links` 查询
3. `src/service/stats_service.rs`：`by_project` 和 `by_instrument` 使用 `p.group_id`（项目表的 group_id），而非 `wr.group_id`
4. `src/repo/record_repo.rs`：回退逻辑仍使用 `project_lab_links` 的 `group_concat`

**修复内容**（仅后端 4 个文件 + 版本配置 2 个文件）：

1. **`src/api/export_data.rs`**：
   - 所有 Sheet 查询添加 `LEFT JOIN project_groups pg ON pg.id = wr.group_id`
   - `lab_name` 改为 `COALESCE(pg.name, '未知')`

2. **`src/api/stats_handler.rs`**：
   - `by_project` 的 `group_name` 改为 `COALESCE(pg.name, '未分组')`
   - 添加 `LEFT JOIN project_groups pg ON pg.id = wr.group_id`

3. **`src/service/stats_service.rs`**：
   - `by_project`：从 `JOIN project_groups pg ON p.group_id=pg.id` 改为 `LEFT JOIN project_groups pg ON pg.id=wr.group_id`
   - `by_instrument`：同上

4. **`src/repo/record_repo.rs`**：
   - `list()` 和 `get_by_id_on_conn()`：去掉 `project_lab_links` 的 `group_concat` 回退
   - 旧数据（`group_id` 为 NULL）直接显示"未知"

5. **`Cargo.toml`**：版本号 `0.3.24` → `0.3.25`
6. **`build_installer.iss`**：所有 `v0.3.24` → `v0.3.25`

## 查询逻辑变更

**之前**（v0.3.24，错误）：
```sql
SELECT ...,
       COALESCE((SELECT group_concat(pg.name) 
                FROM project_lab_links pll 
                JOIN project_groups pg ON pll.group_id = pg.id 
                WHERE pll.project_id = p.id), '未知') AS lab_name
FROM work_records wr
```

**之后**（v0.3.25，正确）：
```sql
SELECT ...,
       COALESCE(pg.name, '未知') AS lab_name
FROM work_records wr
LEFT JOIN project_groups pg ON pg.id = wr.group_id
```

## 构建信息

- **Rust 编译**：✅ 成功
- **前端构建**：无需修改（本次仅后端修复）
- **安装包**：`installer/工作量统计工具_Rust_v0.3.25_Setup.exe`
- **覆盖安装**：✅ 支持（AppId 保持不变）

## 已知问题

1. **旧数据（v0.3.24 之前创建的记录）**：`group_id` 为 NULL，导出/统计时实验室列显示"未知"
   - **解决方案**：重新录入这些记录（删除旧记录，在新版本中重新创建）
2. **v0.3.24 创建的新记录**：`group_id` 已保存，导出/统计只显示在对应实验室下 ✅

## 升级建议

1. 直接安装 `工作量统计工具_Rust_v0.3.25_Setup.exe`
2. 覆盖安装（v0.3.24 或更高版本），数据和配置自动保留
3. 建议让用户重新录入最近的记录（使 `group_id` 被正确保存）

---
**版本隔离说明**：本版本为独立版本目录 `v0.3.25`，不修改旧版本代码。
