# v0.4.33 更新说明

> **版本**: 0.4.33 | **日期**: 2026-07-11 | **基线**: v0.4.32

## 功能新增

### 1. 岗位分级预设模板
- **新增 2 个预设角色模板**：分析员（`entry:workload`）和实验员（`entry:sample` + `entry:sample-info`）
- 在角色管理页的编辑对话框中新增"基于模板"下拉选项，选择后自动填充角色名称和入口权限
- 预置模板完整列表：系统管理员/分析检测员/研发送样员/样品登记员/查看者/分析员/实验员

### 2. 研发送样列宽全面优化
- 列宽全面调整：送样人 80→120px、部门 90→140px、项目 120→150px、类型 100→110px、方法 140→180px、数量 65→80px、批号 80→100px、注意事项 minWidth 100→130px
- 表格整体 minWidth 1100→1280px，添加水平滚动兜底

### 3. 研发送样列自定义
- **新建 `rd_record_columns` 表**：支持 11 个预置列的宽度和显隐配置
- 后端 API：`GET /api/rd-record-columns`（列表）、`PUT /api/rd-record-columns/:id`（更新宽度/显隐）
- 管理页新增「研发送样列配置」子 Tab，可编辑每列的宽度和是否在列表/表单中显示
- 研发送样录入和记录列表改为从 API 读取列配置动态渲染

### 4. 样品信息登记自动填充
- 登记表单的送样人和部门字段自动填充当前登录用户的信息
- 新增行默认继承当前用户

### 5. 样品信息登记列宽可配置
- `sample_info_columns` 种子宽度全面更新为更合理的默认值（main_components 180px、notes 180px 等）
- 管理页列编辑器新增 width 字段，每列宽度可单独调整

### 6. 部门软删除
- **物理删除改为软删除**：`DELETE` → `UPDATE is_active=0, deleted_at=datetime('now')`
- 正常列表过滤已删除的部门（`WHERE is_active=1`）
- 新增「已删除部门列表」和恢复功能（`restore` API）

### 7. 部门说明文案优化
- 文案更新，对齐软删除行为

### 8. 审计记录全覆盖
- 所有关键写操作（角色 CRUD/权限设置、部门 CRUD、研发送样 CRUD/取样、样品信息登记 CRUD/状态变更）均写入审计日志
- 审计记录自动抓取当前操作用户（从 JWT Claims 提取）

## 技术细节
- **版本隔离**：基于 v0.4.32 创建独立文件夹 v0.4.33，不修改历史版本
- **数据库变更**：新增 `rd_record_columns` 表（11 个种子列）
- **新建文件**（4 个）：`rd_record_column.rs`、`rd_record_column_repo.rs`、`rd_record_column_handler.rs`、`AdminRdRecordColumns.tsx`
- **修改文件**（17 个）：`migrations.rs`、`api/mod.rs`、`models/mod.rs`、`repo/mod.rs`、`division_repo.rs`、`division_handler.rs`、`role_handler.rs`、`rd_record_handler.rs`、`sample_info_handler.rs`、`AdminRolesPage.tsx`、`SampleEntryPage.tsx`、`SampleInfoEntry.tsx`、`RdRecordsPage.tsx`、`ManagePage.tsx`、`types/index.ts`、`api/client.ts`

## 升级说明
- 安装此版本后首次启动会自动创建 `rd_record_columns` 表并插入种子数据
- 已有数据无损迁移
- 安装包可覆盖安装 v0.4.30/v0.4.31/v0.4.32（AppId 固定）
