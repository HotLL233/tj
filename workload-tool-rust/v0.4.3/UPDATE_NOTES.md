# v0.4.3 更新说明

## 发布日期
2026-07-07

## 功能增强

### 研发送样记录 — 状态列 + 取样人

**状态列**：在研发送样录入页表格中新增「状态」列（第 2 列，序号之后），显示每条记录的当前状态：
- 🟡 **待取样**：黄色底，表示该记录尚未取样
- 🟢 **已取样**：绿色底，表示取样已完成

**取样人列**：在数量列后新增「取样人」列和「取样时间」列：
- 点击取样人列弹出对话框输入取样人姓名
- 确认后自动记录取样时间，状态自动变为「已取样」
- 支持修改已填写的取样人（仅改姓名，不改状态和时间）

**实验室卡片送样计数**：实验室卡片现在显示「X 条送样」，在分析检测门户和研发送样门户均可看到各实验室的研发送样记录数量。

---

## 修改文件清单

### 后端（Rust · `workload-tool-rust/v0.4.3/src`）
| 文件 | 改动 |
|------|------|
| `src/db/migrations.rs` | `rd_work_records` 新增 `status`/`sampler`/`sampled_at` 三列 |
| `src/models/rd_record.rs` | **新建**：`RdRecordResponse` + `RdSampleUpdate` |
| `src/models/mod.rs` | 注册 `rd_record` 模块 |
| `src/models/group.rs` | `GroupResponse` 新增 `rd_record_count` |
| `src/repo/rd_record_repo.rs` | 全部查询切换为 `RdRecordResponse`；SELECT/INSERT 含状态字段；新增 `sample()` |
| `src/repo/group_repo.rs` | `list()`/`get_by_id()` SQL 追加送样记录数子查询 |
| `src/api/rd_record_handler.rs` | 新增 `PUT /rd-records/:id/sample` |
| `src/service/rd_record_service.rs` | 切换为 `RdRecordResponse`；新增 `sample()` |

### 前端（React · `workload-tool-rust/v0.4.3/frontend/src`）
| 文件 | 改动 |
|------|------|
| `src/types/index.ts` | `WorkRecord` 加 `status?/sampler?/sampled_at?`；`ProjectGroup` 加 `rd_record_count?` |
| `src/api/client.ts` | 新增 `sampleRdRecord` |
| `src/components/GroupCard.tsx` | 实验室卡片加送样数量 Chip |
| `src/pages/SampleEntryPage.tsx` | 表格 8→12 列（序号/状态/录入时间/实验室/项目/送样人/方法/类型/仪器/数量/取样人/取样时间）；状态列彩色底；取样人列点击弹窗输入；取样时间只读 |

---

## 构建信息

- **Rust 编译（cargo check）**：✅ 零错误
- **前端类型检查（tsc --noEmit）**：✅ 零错误
- **安装包**：`installer/工作量统计工具_Rust_v0.4.3_Setup.exe`
- **覆盖安装**：✅ 支持（AppId 不变，覆盖旧版本安装；`data/` 保留，迁移自动加列）

---

**版本隔离说明**：`v0.4.2 → v0.4.3` 完整复制后增量修改；仅改研发送样相关代码，分析检测（work_records）一切未动。已存在的 `rd_work_records` 记录升级后 `status` 自动填充「待取样」。
