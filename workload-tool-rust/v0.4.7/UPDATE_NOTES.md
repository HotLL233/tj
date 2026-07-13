# v0.4.7 更新说明

## 发布日期
2026-07-07

## 功能增强

### 新增「单价倍率」功能（仅分析检测模块）
- 方法管理表和编辑弹窗新增「单价倍率」列，默认 1.0
- 新建分析检测记录时自动带入方法单价倍率
- 检测人记录表新增可编辑「单价倍率」列，失焦自动保存，仅对单条记录生效
- 金额计算改为 `数量 × 单价 × 单价倍率`（Sheet3/4/5/6 及导出 Excel）
n### BugFix：导出 Excel 使用记录级单价倍率（v0.4.7）
- **问题**：用户在检测人记录中修改某条记录的单价倍率后，导出 Excel 仍读取方法表默认值，看不到修改
- **修复**：export_data.rs 全部 10 个 Sheet 的 SQL 从 m.multiplier 改为 wr.multiplier（优先取记录值，回退到方法默认值）

### 字段更名（v0.4.7）
- 全局「倍率」→「单价倍率」，语义更明确（绑定单价，不影响系数）

### 仪器 Chip 颜色优化
- 全局仪器标签颜色从亮紫色 `#9c27b0` 改为深青色 `#00897b`

### 取样人行内编辑
- 研发送样记录表取样人列改为行内输入，失焦自动保存，取消弹窗

### 统计入口按钮位置优化
- 研发送样门户「查看统计」按钮移到标题行右侧，与分析检测门户一致

### 统计页筛选修复
- 检测人/送样人筛选下拉框改为从后端获取全量人名列表
- 修复筛选不生效的 stale closure 问题

### 移动端表格 nowrap 修复
- 检测人记录/送样人记录表各列增加 `whiteSpace: nowrap`，消除手机端逐字换行

---

## 修改文件清单

### 后端（Rust · `workload-tool-rust/v0.4.7/src`）
| 文件 | 改动 |
|------|------|
| `db/migrations.rs` | `methods` 表加 `multiplier` 字段；`work_records` 表加 `multiplier` 字段 |
| `models/record.rs` | `RecordResponse`、`RecordUpdate`、`RecordCreate` 加 `multiplier` |
| `repo/method_repo.rs` | method 查询/创建/更新加 `multiplier` |
| `repo/record_repo.rs` | record 创建加 `multiplier`(默认取 method.multiplier)，更新支持 `multiplier` |
| `api/record_handler.rs` | 新增 `GET /records/users?start=&end=` 接口；列表/更新支持 multiplier |
| `api/stats_handler.rs` | 金额计算加入单价倍率 |
| `api/export_data.rs` | 导出 Excel 新增单价倍率列，金额加入单价倍率 |

### 前端（React · `workload-tool-rust/v0.4.7/frontend/src`）
| 文件 | 改动 |
|------|------|
| `types/index.ts` | `Method` 加 `multiplier`；`WorkRecord` 加 `multiplier`；`RecordUpdate` 加 `multiplier` |
| `api/client.ts` | `getRecords`/`getRdRecords` 补 `user_name` 参数；新增 `getRecordUsers` 接口 |
| `pages/ManagePage.tsx` | 方法管理加单价倍率列；仪器颜色 #9c27b0 → #00897b |
| `pages/EntryPage.tsx` | 仪器颜色 #9c27b0 → #00897b |
| `pages/StatsPage.tsx` | 检测人记录表加单价倍率列(行内编辑)；筛选改用后端全量人名；所有列 nowrap |
| `pages/SampleStatsPage.tsx` | 送样人记录表筛选修复；所有列 nowrap |
| `pages/SampleEntryPage.tsx` | 取样人行内编辑；仪器颜色 |
| `pages/RdRecordsPage.tsx` | 取样人行内编辑；仪器颜色 |
| `pages/SamplePortal.tsx` | 统计按钮移到标题右侧 |

---

## 构建信息

- **Rust 编译**：✅
- **前端**：✅
- **安装包**：`installer/工作量统计工具_Rust_v0.4.7_Setup.exe`
- **覆盖安装**：✅ 支持（AppId 不变）

---

**版本隔离**：`v0.4.5 → v0.4.7` 完整复制后增量修改。
