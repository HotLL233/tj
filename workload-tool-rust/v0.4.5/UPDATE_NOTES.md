# v0.4.5 更新说明

## 发布日期
2026-07-07

## Bug 修复

### 修复研发送样记录跨实验室重复计数
- **问题**：一个项目关联多个实验室时，该项目下的研发送样记录会在所有关联实验室的卡片角标和记录列表中重复出现
- **根因**：`group_repo.rs` 的 `rd_record_count` 子查询和 `rd_record_repo.rs` 的 `list()` 过滤通过 `project_lab_links`（项目-实验室 M:N 关联表）推算记录所属实验室，而非直接使用 `rd_work_records.group_id`（录入时已存储）
- **修复**：3 处 SQL 条件改为直接用 `wr2.group_id = g.id` / `wr.group_id = gid`，不再绕路走项目关联表

---

## 修改文件清单

### 后端（Rust · `workload-tool-rust/v0.4.5/src`）
| 文件 | 改动 |
|------|------|
| `src/repo/group_repo.rs` | `list()` 和 `get_by_id()` 中 `rd_record_count` 子查询：移除 `JOIN projects ... JOIN project_lab_links`，改为 `WHERE wr2.group_id = g.id` |
| `src/repo/rd_record_repo.rs` | `list()` 中 `group_id` 过滤：从 `EXISTS (SELECT ... project_lab_links ...)` 改为 `wr.group_id = {gid}` |
| `src/api/rd_stats_handler.rs` | `summary()`、`by_user()`、`by_project()` 中 `group_id` EXISTS 子查询：从 `EXISTS (SELECT ... project_lab_links ...)` 改为 `wr.group_id = {gid}`（3处） |
| `src/api/rd_export_data.rs` | Sheet1 导出中 `group_id` 过滤：从 `EXISTS (SELECT ... project_lab_links ...)` 改为 `wr.group_id = {}`（1处） |

### 前端（React · `workload-tool-rust/v0.4.5/frontend/src`）
| 文件 | 改动 |
|------|------|
| `src/pages/SampleEntryPage.tsx` | 方法行的系数 Chip 替换为单价 Chip（`×系数` → `¥金额`，amount=0 不显示），颜色由橙色改为绿色 |

### 功能优化
- **研发送样入口**：方法列表中的系数标签改为显示单价（¥金额），未设置单价的不显示
- **研发送样记录**：按送样时间降序排列（后端 `ORDER BY wr.recorded_at DESC`，前端保持后端排序）

---

## 构建信息

- **Rust 编译（cargo check）**：✅ 零错误
- **前端**：无改动
- **安装包**：`installer/工作量统计工具_Rust_v0.4.5_Setup.exe`
- **覆盖安装**：✅ 支持（AppId 不变，覆盖旧版本安装；`data/` 保留）

---

**版本隔离说明**：`v0.4.4 → v0.4.5` 完整复制后增量修改；仅改 2 个 Rust 文件共 3 行 SQL，无前端改动。

---

# v0.4.4 更新说明

## 发布日期
2026-07-07

## 功能增强

### 新增「研发送样记录」独立卡片
- 在**研发送样门户**和**分析检测门户**的实验室卡片网格中，新增「研发送样记录」卡片，与实验室卡片同级显示
- 点击卡片进入独立页面，查看所有研发送样记录（含状态、取样人、取样时间）
- 该卡片不参与实验室编辑，仅用于查看和取样操作
- 卡片右上角显示全局待取样数量角标

### 实验室卡片待取样角标
- 实验室卡片右上角新增橙色圆形角标，显示该实验室当前待取样记录数
- 取样后自动减少（计数只统计 `status='待取样'` 的记录）
- 待取样数为 0 时角标自动隐藏

### 研发送样页面列头优化
- 研发送样录入页表头「录入时间」→「送样时间」，与研发送样模块语义一致

### 研发送样门户手机端显示优化
- 手机端实验室卡片排列从 1 列改为 3 列，与分析检测门户保持一致

---

## 修改文件清单

### 后端（Rust · `workload-tool-rust/v0.4.4/src`）
| 文件 | 改动 |
|------|------|
| `src/repo/group_repo.rs` | `rd_record_count` 子查询追加 `AND wr2.status='待取样'`，计数由「总记录数」改为「待取样数」 |

### 前端（React · `workload-tool-rust/v0.4.4/frontend/src`）
| 文件 | 改动 |
|------|------|
| `src/components/GroupCard.tsx` | 移除底部送样 Chip；新增右上角圆形角标显示待取样数 |
| `src/components/RecordsCard.tsx` | **新建**：独立记录卡片组件（含角标） |
| `src/pages/SamplePortal.tsx` | 插入 RecordsCard；手机端 grid 改为 3 列 |
| `src/pages/WorkloadPortal.tsx` | 插入 RecordsCard |
| `src/pages/SampleEntryPage.tsx` | 表头「录入时间」→「送样时间」 |
| `src/pages/RdRecordsPage.tsx` | **新建**：全量记录页（12列表格，支持取样人编辑） |
| `src/App.tsx` | 注册 `/sample-records` 路由 |

---

## 构建信息

- **Rust 编译（cargo check）**：✅ 零错误
- **前端类型检查（tsc --noEmit）**：✅ 零错误
- **安装包**：`installer/工作量统计工具_Rust_v0.4.4_Setup.exe`
- **覆盖安装**：✅ 支持（AppId 不变，覆盖旧版本安装；`data/` 保留）

---

**版本隔离说明**：`v0.4.3 → v0.4.4` 完整复制后增量修改；仅改研发送样相关代码，分析检测模块不受影响。
