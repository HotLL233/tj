## v0.4.22 — 新增「样品信息登记」独立模块

### 新增功能
在主页面新增第三个独立门户入口 **「样品信息登记」**，用于 ICP、热分析、质谱等检测类型的样品基础信息填写与状态管理。

### 功能详情

**前端（5 个文件）**
- 新增 `SampleInfoHome.tsx` — 检测类型卡片导航页（ICP / 热分析 / 质谱 / 其他 四张卡片）
- 新增 `SampleInfoEntry.tsx` — 登记卡片表单 + 可展开记录列表 + 状态流转
- 修改 `HomePage.tsx` — 新增第三张绿色主题卡片 "样品信息登记"
- 修改 `App.tsx` — 新增路由 `/sample-info` 和 `/sample-info/entry`
- 修改 `types/index.ts` — 新增 `SampleInfoRecord` 接口
- 修改 `client.ts` — 新增 5 个 API 函数

**后端（3 个文件）**
- 新增 `models/sample_info.rs` — 数据模型（含查询参数、创建/更新请求）
- 新增 `repo/sample_info_repo.rs` — CRUD + 状态流转 + 序号自动生成
- 新增 `api/sample_info_handler.rs` — REST API 路由（5 个端点）
- 修改 `db/migrations.rs` — 建表 `sample_info_records`（11 列 + 3 个索引）
- 修改 `models/mod.rs`, `repo/mod.rs`, `api/mod.rs` — 注册新模块

**数据表字段**
| 字段 | 说明 |
|------|------|
| status | 待检测/待取样/已取样/检测完成 |
| seq_no | 按检测类型+日期自动递增 |
| batch_no | 样品批号* |
| user_name | 送样人* |
| lab_name | 实验室/车间* |
| project_name | 所属项目* |
| submitted_at | 送样时间*（自动抓取） |
| detection_date | 检测时间* |
| main_components | 样品主要成分* |
| detection_type | 检测类型*（从卡片入口带入） |
| notes | 注意事项 |

**状态流转规则**
待检测 → 待取样 → 已取样 → 检测完成（不可跳转，检测完成不可再操作）

### 变更文件清单
| 文件 | 类型 |
|------|------|
| `src/db/migrations.rs` | 修改 |
| `src/models/sample_info.rs` | 新增 |
| `src/models/mod.rs` | 修改 |
| `src/repo/sample_info_repo.rs` | 新增 |
| `src/repo/mod.rs` | 修改 |
| `src/api/sample_info_handler.rs` | 新增 |
| `src/api/mod.rs` | 修改 |
| `frontend/src/types/index.ts` | 修改 |
| `frontend/src/api/client.ts` | 修改 |
| `frontend/src/pages/HomePage.tsx` | 修改 |
| `frontend/src/App.tsx` | 修改 |
| `frontend/src/pages/SampleInfoHome.tsx` | 新增 |
| `frontend/src/pages/SampleInfoEntry.tsx` | 新增 |

### 安装说明
- 版本：v0.4.22
- 安装包：`工作量统计工具_Rust_v0.4.22_Setup.exe`
- 安装方式：直接覆盖安装 v0.4.21，无需卸载旧版本
- 数据库自动迁移，已有数据不受影响
