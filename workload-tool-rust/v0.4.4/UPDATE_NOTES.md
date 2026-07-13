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
