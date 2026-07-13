# v0.4.2 更新说明

## 发布日期
2026-07-07

## Bug 修复

### ① 新建项目 FK 错误修复
- **问题**：管理界面点击「新建项目」保存时报 `FOREIGN KEY constraint failed` 数据库错误。
- **根因**：`project_repo::create` 硬编码 `group_id=0` 且 `method_type='研发项目'`，但不存在 id=0 的实验室。
- **修复**：`group_id` 改为取用户选择的第一个实验室 id；INSERT 同时写入 `full_name`/`sort_order`/`is_active`（前端已发送这些字段但后端原先静默忽略）；移除 `method_type` 硬编码（使用列默认值）。

### ② 全模块「人员/用户/用户名」→ 检测人/送样人
- 分析检测模块（录入页 + 统计页）内所有「人员」「用户」「用户名」→ **检测人**
- 研发送样模块（录入页 + 统计页）内所有「人员」「用户」「用户名」→ **送样人**
- 覆盖范围：输入框标签 / 校验提示 / 表格列头 / 筛选下拉 / 删除确认 / Sheet 说明文字

### ③ 卡片颜色按模块统一
- 门户页实验室卡片（`GroupCard`）：分析检测统一蓝色 `#1976d2`，研发送样统一橙色 `#e65100`（原彩虹十色 → 模块单色）
- 统计页汇总卡片（`StatsCards`）：同上统一模块色
- 两组件均通过可选 `themeColor` prop 控制，不传时保持原行为（向后兼容）

### ④ 研发送样返回标题回退修复
- **问题**：研发送样门户 → 实验室卡片 → 点返回后标题变成「工作量录入」。
- **根因**：`SampleEntryPage` 返回按钮写死 `navigate('/workload')`。
- **修复**：改为 `navigate('/sample')`，正确回到研发送样门户。

---

## 修改文件清单

### 后端（Rust · `workload-tool-rust/v0.4.2/src`）
| 文件 | 改动 |
|------|------|
| `src/models/project.rs` | `ProjectCreate` 新增 `full_name`、`sort_order`、`is_active` |
| `src/repo/project_repo.rs` | `create()` 重写：`group_id` 取 `lab_ids[0]`，INSERT 写入 full_name 等，移除 method_type 硬编码 |

### 前端（React · `workload-tool-rust/v0.4.2/frontend/src`）
| 文件 | 改动 |
|------|------|
| `src/pages/EntryPage.tsx` | 输入框标签 + 校验提示「用户名」→「检测人」 |
| `src/pages/StatsPage.tsx` | 14 处「人员/用户/用户名」→「检测人」 |
| `src/pages/SampleEntryPage.tsx` | 输入框标签 + 校验提示「用户名」→「送样人」；返回按钮 `/workload`→`/sample` |
| `src/pages/SampleStatsPage.tsx` | 14 处「人员/用户/用户名」→「送样人」 |
| `src/components/GroupCard.tsx` | 新增 `themeColor` prop；模块统一实验室卡片颜色 |
| `src/components/StatsCards.tsx` | 新增 `themeColor` prop；模块统一统计汇总卡片颜色 |
| `src/pages/WorkloadPortal.tsx` | `<GroupCard themeColor="#1976d2" />` |
| `src/pages/SamplePortal.tsx` | `<GroupCard themeColor="#e65100" />` |

---

## 构建信息

- **Rust 编译（cargo check）**：✅ 零错误
- **前端类型检查（tsc --noEmit）**：✅ 零错误
- **安装包**：`installer/工作量统计工具_Rust_v0.4.2_Setup.exe`
- **覆盖安装**：✅ 支持（AppId 与 v0.4.1 一致，覆盖旧版本安装；`data/` 目录保留）

---

**版本隔离说明**：`v0.4.1 → v0.4.2` 完整复制后端目录 + 前端源码目录后增量修改；仅改本次相关代码，未触碰无关逻辑。数据库表结构无变更。
