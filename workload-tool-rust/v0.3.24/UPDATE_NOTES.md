# v0.3.24 更新说明

## 发布日期
2026-07-06

## 问题修复

### 修复：导出 Excel / 分类统计中项目与实验室对应关系错误

**问题描述**：
- 项目 A001 同时关联 410 和 411 两个实验室（配置关系）
- 用户在 **410 实验室** 选项卡下录入了 A001 的记录
- 统计页面显示正常（实验室列只显示 410）✅
- **导出 Excel 后**，A001 的数据同时出现在 410 和 411 下 ❌

**根因**：
1. `work_records` 表**没有 `group_id` 字段**，不存储录入时的实验室上下文
2. 导出/统计查询时，通过 `project_lab_links` 关联表获取实验室名称（返回项目关联的**所有**实验室）
3. 导致一条记录被"分配"到多个实验室，数量翻倍

**修复方案**：
1. **数据库迁移**：`work_records` 表新增 `group_id INTEGER REFERENCES project_groups(id)` 字段
2. **录入时保存实验室上下文**：创建记录时，将当前选项卡 `groupId` 存入 `work_records.group_id`
3. **查询时优先使用 `wr.group_id`**：
   - 如果 `wr.group_id` 非空：直接获取该实验室名称（单一名称）
   - 如果 `wr.group_id` 为空（旧数据）：回退到 `project_lab_links` 逻辑（显示"未知"）
4. **影响文件**：
   - `src/db/migrations.rs`：新增迁移（v0.3.24）
   - `src/models/record.rs`：`RecordCreate` 新增 `group_id` 字段
   - `src/repo/record_repo.rs`：`create()` 保存 `group_id`；`list()` 和 `get_by_id_on_conn()` 使用 `wr.group_id`
   - `src/api/record_handler.rs`：`create()` 接受并传递 `group_id`
   - `src/api/export_data.rs`：所有 Sheet 查询改为 `LEFT JOIN project_groups ON wr.group_id`
   - `src/api/stats_handler.rs`：所有统计查询改为 `LEFT JOIN project_groups ON wr.group_id`
   - `project-root/frontend/src/api/client.ts`：`createRecord` API 新增 `group_id` 参数
   - `project-root/frontend/src/pages/EntryPage.tsx`：创建记录时传递当前 `groupId`

## 技术细节

### 数据库变更
```sql
-- v0.3.24 迁移
ALTER TABLE work_records ADD COLUMN group_id INTEGER REFERENCES project_groups(id);
```

### 查询逻辑变更

**之前**（错误）：
```sql
SELECT ...,
       COALESCE((SELECT group_concat(pg.name) 
                FROM project_lab_links pll 
                JOIN project_groups pg ON pll.group_id = pg.id 
                WHERE pll.project_id = p.id), '未知') AS group_name
FROM work_records wr
```

**之后**（正确）：
```sql
SELECT ...,
       COALESCE(pg.name, '未知') AS group_name
FROM work_records wr
LEFT JOIN project_groups pg ON pg.id = wr.group_id
```

### 前端变更
- 创建记录时，从 URL 参数 `?groupId=xxx` 获取当前实验室 ID
- 传递给后端 API：`createRecord({ ..., group_id: gid })`

## 构建信息

- **Rust 编译**：✅ 成功（6个 warning，无 error）
- **前端构建**：✅ 成功
- **安装包**：`installer/工作量统计工具_Rust_v0.3.24_Setup.exe`
- **覆盖安装**：✅ 支持（AppId 保持不变）

## 已知问题

1. **旧数据（v0.3.24 之前创建的记录）**：`group_id` 为 NULL，导出/统计时实验室列显示"未知"
   - **解决方案**：重新录入这些记录（删除旧记录，在新版本中重新创建）
   - 或者：接受"未知"显示（比之前的错误对应关系更好）

2. **多实验室项目的新记录**：如果项目关联多个实验室，记录只在**实际录入的实验室**下显示（符合预期）

## 升级建议

1. 直接安装 `工作量统计工具_Rust_v0.3.24_Setup.exe`
2. 覆盖安装（v0.3.23 或更高版本），数据和配置自动保留
3. 建议让用户重新录入最近的记录（使 `group_id` 被正确保存）

---
**版本隔离说明**：本版本为独立版本目录 `v0.3.24`，不修改旧版本代码。
