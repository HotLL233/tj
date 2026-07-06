# v0.4.0 更新说明

## 发布日期
2026-07-06

## 功能增强

### 新增「研发送样」模块（完整克隆分析检测）

与分析检测同级入口，逻辑完全一致，但录入数据独立存储，主数据（项目 / 实验室 / 方法）两个模块共用。

- 入口：首页「研发送样」卡片 → 选择实验室 → 录入 / 统计 / 管理
- 录入页 `研发送样录入`：克隆分析检测录入页，写入独立的 `rd_work_records` 表
- 统计页 `研发送样统计`：完整克隆 10 个 Sheet 预览 + 导出（文件名「研发送样统计_…xlsx」）
- 管理页：两模块共用同一套主数据，任一处更新即时同步

### 审计日志按模块隔离

- `audit_log` 新增 `module` 字段：`work`（分析检测）/ `rd`（研发送样）/ `shared`（共享主数据）
- 历史审计记录默认归为 `shared`，分析检测与研发送样审计页各自可见「本模块 + 共享」
- 退役旧「送样」模块（原 `sample_records` 相关页面与接口）

---

## 修改文件清单

### 后端（Rust）
| 文件 | 改动 |
|------|------|
| `src/db/migrations.rs` | 新增 `rd_work_records` 表；`audit_log` 新增 `module` 列（默认 `shared`） |
| `src/repo/audit_repo.rs` | 新增 `log_with_module` / `log_on_conn_with_module`；`list` 支持 module 过滤 |
| `src/models/audit.rs` | `AuditLogResponse` 新增 `module` 字段 |
| `src/repo/rd_record_repo.rs` | 新建：研发送样记录 DAO（写入 `rd_work_records`，审计带 `rd` 模块） |
| `src/service/rd_record_service.rs` | 新建：研发送样记录服务 |
| `src/api/rd_record_handler.rs` | 新建：`/api/rd-records` 系列路由 |
| `src/api/rd_stats_handler.rs` | 新建：`/api/rd-stats/*` 统计路由 |
| `src/api/rd_export_data.rs` | 新建：研发送样导出 SQL（复用 `export_data` 行结构类型） |
| `src/api/rd_export_handler.rs` | 新建：`/api/rd-export/excel` 导出 |
| `src/api/rd_export_preview_handler.rs` | 新建：`/api/rd-export/preview/sheet1~10` 预览 |
| `src/api/mod.rs` | 注册 rd_* 路由；移除 `sample_handler` |
| `src/repo/mod.rs` / `src/service/mod.rs` / `src/models/mod.rs` | 移除 sample 模块声明 |
| 退役文件 | `sample_handler.rs` / `sample_repo.rs` / `sample_service.rs` / `models/sample.rs` 已删除 |

### 前端（React）
| 文件 | 改动 |
|------|------|
| `src/api/client.ts` | 新增 rd 系列 API（records / stats / export / preview） |
| `src/pages/SampleEntryPage.tsx` | 克隆分析检测录入页，调用 rd API |
| `src/pages/SampleStatsPage.tsx` | 克隆分析检测统计页（10 Sheet 预览），调用 rd API |
| `src/pages/SamplePortal.tsx` | 标题改为「研发送样」，入口指向 `/sample/:id` 与 `/sample/stats` |
| `src/App.tsx` | 移除 `/sample/list` 路由；删除 `SampleListPage` 引用 |
| `src/pages/SampleListPage.tsx` | 退役删除 |
| `src/pages/ManagePage.tsx` | 审计日志表名映射补充 `rd_work_records`；`samples` 标注「已退役」 |
| `src/components/Layout.tsx` | 关于页功能列表与 v0.4.0 更新说明 |
| `src/pages/HomePage.tsx` | 首页「研发送样」入口（沿用） |

---

## 构建信息

- **Rust 编译**：✅ 成功
- **前端构建**：✅ 成功
- **安装包**：`installer/工作量统计工具_Rust_v0.4.0_Setup.exe`
- **覆盖安装**：✅ 支持（安装目录 `{autopf}\工作量统计工具`，覆盖旧版本）

---

**版本隔离说明**：`v0.3.33 → v0.4.0` 完整复制后端目录后增量修改；前端源码在 `project-root/frontend` 统一构建后复制到 `v0.4.0/static`。数据库表结构新增 `rd_work_records` 与 `audit_log.module`，旧数据与旧 `sample_records` 表均保留。
