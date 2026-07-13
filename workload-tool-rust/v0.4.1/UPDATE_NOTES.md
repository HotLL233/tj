# v0.4.1 更新说明

## 发布日期
2026-07-06

## 功能增强

### 1. 录入人 / 人员 列头按模块改名
- 分析检测模块：所有「录入人」「人员」相关列头 / 选项卡名 → **「检测人」**
- 研发送样模块：所有「录入人」「人员」相关列头 / 选项卡名 → **「送样人」**
- 导出 Excel（Sheet5 人员原始记录、Sheet6 人员汇总）表头与 Sheet 名同步参数化：分析检测导出为「检测人…」、研发送样导出为「送样人…」
- 导入列名识别别名表补「检测人」，确保分析检测导出的 Excel 可回导

### 2. 模块背景色区分（一眼可辨）
- 分析检测模块：蓝色系（`#1976d2` / `rgba(25,118,210)`），统计页由紫调为蓝，导出按钮保持绿色
- 研发送样模块：橙色系（`#e65100` / `rgba(230,81,0)`），录入页（原蓝克隆残留）与统计页（原紫/绿）统一修正为橙
- 两模块在门户 / 录入 / 统计三页配色彻底分开

### 3. 删除「关联配置」卡片及功能
- 系统管理页移除「关联配置」卡片（含拖拽式关联配置组件 `AssociationConfig.tsx`），相关 import 与渲染块一并清理
- 项目编辑弹窗内的「关联实验室」「关联检测方法」多选框保留不动（属项目内联编辑）

### 4. 数据备份页中文说明
- 备份 / 恢复按钮、数据库状态、自动备份设置三处新增中文引导说明
- 「数据库状态」中各数据表记录数加中文标签：`分析检测记录` / `研发送样记录` / `送样记录(已退役)` / `项目` / `项目分组` / `检测方法` / `审计日志`（原显示原始表名）

### 5. 项目一览 / 实验室一览 / 方法一览优化
- **项目一览**：新增「关联实验室」「关联方法」「操作」列，每行提供编辑入口（打开已有项目编辑弹窗维护关联）
- **实验室一览**：新增「关联项目」列，显示该实验室关联的具体项目名称列表（后端 `group_repo` 通过关联子查询返回 `project_names`）
- **方法一览**：调整列宽（方法名称 / 类型加宽，保证长方法名如 `HYLY-230106-1-DAD@[LC-01]` 完整显示），表格容器支持横向滚动

---

## 修改文件清单

### 后端（Rust · `workload-tool-rust/v0.4.1/src`）
| 文件 | 改动 |
|------|------|
| `src/api/export_write.rs` | `write_sheet5` / `write_sheet6` 新增 `person_label: &str` 参数，表头与 Sheet 名参数化 |
| `src/api/export_handler.rs` | 调用传 `"检测人"` |
| `src/api/rd_export_handler.rs` | 调用传 `"送样人"` |
| `src/utils/excel_reader.rs` | `USER_KEYWORDS` 补 `"检测人"` |
| `src/models/group.rs` | `GroupResponse` 新增 `project_names: Option<String>` |
| `src/repo/group_repo.rs` | `list()` / `get_by_id()` SQL 关联子查询返回 `project_names` |
| `src/api/backup_handler.rs` | `TableCount` 新增 `label: String`；`table_counts()` 改用 (表名,中文标签) 规格表，新增 `rd_work_records`→研发送样记录，删除 audit_log 重复计数 |

### 前端（React · `workload-tool-rust/v0.4.1/frontend/src`）
| 文件 | 改动 |
|------|------|
| `src/pages/EntryPage.tsx` | 录入页表头「录入人」→「检测人」 |
| `src/pages/StatsPage.tsx` | 列头 / 选项卡名「人员」→「检测人」；统计页紫→蓝；绿色导出按钮保留 |
| `src/pages/SampleEntryPage.tsx` | 表头「录入人」→「送样人」；录入页蓝→橙 |
| `src/pages/SampleStatsPage.tsx` | 列头 / 选项卡名「人员」→「送样人」；统计页紫/绿→橙；导出按钮绿→橙 |
| `src/pages/ManagePage.tsx` | ①删关联配置卡片 ②备份区中文说明 ③项目一览加关联列+编辑入口 ④实验室一览加关联项目列 ⑤方法一览调列宽 ⑥数据状态表中文标签渲染 |
| `src/types/index.ts` | `ProjectGroup` 加 `project_names?`；`BackupStatus.tables` 加 `label?` |
| `src/components/AssociationConfig.tsx` | **删除**（关联配置组件） |

---

## 构建信息

- **Rust 编译（cargo check）**：✅ 零错误
- **前端类型检查（tsc --noEmit）**：✅ 零错误
- **安装包**：`installer/工作量统计工具_Rust_v0.4.1_Setup.exe`
- **覆盖安装**：✅ 支持（AppId 与 `v0.4.0` 一致，覆盖旧版本安装；`data/` 目录保留，升级自动迁移）

---

**版本隔离说明**：`v0.4.0 → v0.4.1` 完整复制后端目录 + 前端源码目录后增量修改；仅改本次相关代码，未触碰无关逻辑。数据库表结构无新增（沿用 v0.4.0 的 `rd_work_records` 与 `audit_log.module`），旧数据与 `sample_records` 表均保留。
