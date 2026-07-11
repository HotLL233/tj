# v0.4.26 更新说明

> 发布日期：2026-07-10

## 新增功能

### 1. 样品信息登记 — 独立统计页（复刻分析检测模式）
- 新增 `SampleInfoStatsPage.tsx`，完整复刻分析检测 `StatsPage.tsx` 模式
- 8 个绿色主题统计维度卡片：按状态 / 按检测类型 / 按实验室/车间 / 按所属项目 / 按送样人 / 按月统计 / 送样人记录 / 导出 Excel
- 门户 `SampleInfoHome.tsx` 增加「查看统计」按钮
- 路由 `/sample-info/stats`

### 2. 样品信息登记 — 列纯自定义管理
- **管理页新增「④ 自定义列配置」**：可新增/删除/排序/重命名录入表单的列
- **数据存储**：自定义列的值存入 `sample_info_records.extra_fields` JSON 列
- **录入表单动态渲染**：根据列配置动态生成表头和单元格
- **记录列表动态列**：`SampleInfoEntry` 和 `ManagePage` 的记录列表根据 `show_in_list` 动态展示
- **导出同步**：导出 Excel 的列头和数据根据 `show_in_export` 动态生成
- 内置 12 个预置字段（不可删除），自定义字段以 `custom_` 开头
- 支持 text/number/select/date 四种列类型

## 技术变更

- 新建 `sample_info_columns` 数据库表
- 新建 `sample_info_column_handler.rs` 后端 CRUD API
- 改造 `sample_info_repo.rs`/`export_data.rs`/`export_write.rs` 支持动态列
- 新增 `SampleInfoStatsPage.tsx` 独立统计页
- 更新 Cargo.toml 版本号至 0.4.26

## 文件清单

| 文件 | 操作 | 说明 |
|------|------|------|
| `src/models/sample_info_column.rs` | 新建 | 列配置模型 |
| `src/repo/sample_info_column_repo.rs` | 新建 | 列配置 DAO |
| `src/api/sample_info_column_handler.rs` | 新建 | 列配置 API |
| `frontend/src/pages/SampleInfoStatsPage.tsx` | 新建 | 独立统计页 |
| `src/db/migrations.rs` | 修改 | 加 sample_info_columns 表 + extra_fields 列 |
| `src/models/sample_info.rs` | 修改 | 加 extra_fields |
| `src/repo/sample_info_repo.rs` | 修改 | 改造 SQL 支持 extra_fields |
| `src/api/sample_info_handler.rs` | 修改 | 透传 extra_fields |
| `src/api/sample_info_export_data.rs` | 修改 | 动态列查询 |
| `src/api/sample_info_export_write.rs` | 修改 | 动态列头/列宽 |
| `src/api/sample_info_export_handler.rs` | 修改 | 传递列配置 |
| `src/api/mod.rs` | 修改 | 注册新路由 |
| `frontend/src/types/index.ts` | 修改 | 加 SampleInfoColumn |
| `frontend/src/api/client.ts` | 修改 | 加列配置 API |
| `frontend/src/pages/SampleInfoEntry.tsx` | 修改 | 动态渲染 |
| `frontend/src/pages/ManagePage.tsx` | 修改 | 加列编辑器 + 动态列表 |
| `frontend/src/pages/SampleInfoHome.tsx` | 修改 | 加统计入口 |
| `frontend/src/pages/App.tsx` | 修改 | 加路由 |
| `Cargo.toml` | 修改 | 版本 0.4.26 |

## 安装说明
- Windows：运行安装包即可覆盖安装（AppId 与 v0.4.25 一致）
- Docker：`docker compose pull && docker compose up -d`

## v0.4.26 补丁（2026-07-10 14:17）
### 修复：Windows 安装包静态文件打包错误
- **根因**：`vite.config.ts` 构建输出到 `backend/static/`，但 `build_installer.iss` 从根目录 `static/` 取文件，导致安装包内置的是旧版前端页面
- **修复**：`build_installer.iss` Source 路径从 `static\*` 改为 `backend\static\*`，与 Dockerfile 对齐
- **影响**：仅影响 Windows 安装包。Docker 版本来就正确（Dockerfile 已用 `backend/static/`）
