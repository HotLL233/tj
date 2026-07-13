# v0.4.10 更新说明

## 发布日期
2026-07-08

## 功能增强

### 实验室模块关联
- `project_groups` 表新增 `show_in_work`（分析检测）和 `show_in_rd`（研发送样）布尔字段
- 新建/编辑实验室时可选择在哪个模块显示，默认两个都勾选
- 实验室一览表新增「关联分组」列，采用 Chip 点击切换（蓝底=选中，灰框=未选）
- 分析检测门户仅显示 `show_in_work!=false` 的实验室
- 研发送样门户仅显示 `show_in_rd!=false` 的实验室
- 现有数据自动默认全选，不受影响

### 方法一览表类型列自适应换行
- 修复类型列 `minWidth: 160` 导致 column 过宽、所有 Chip 单行排开的问题
- 去掉 `minWidth` 约束后列宽自动收紧，Chip 自动换行（约 4 个一行）

---

## 修改文件清单

### 后端（Rust · `workload-tool-rust/v0.4.10/src`）
| 文件 | 改动 |
|------|------|
| `db/migrations.rs` | project_groups 加 show_in_work/show_in_rd 迁移 |
| `models/group.rs` | GroupResponse/Create/Update 加字段 |
| `repo/group_repo.rs` | list/get/create/update 读写新字段 |

### 前端（React · `workload-tool-rust/v0.4.10/frontend/src`）
| 文件 | 改动 |
|------|------|
| `types/index.ts` | ProjectGroup 加 show_in_work/show_in_rd |
| `api/client.ts` | createGroup/updateGroup 类型加字段 |
| `pages/ManagePage.tsx` | 实验室编辑弹窗加 Chip；实验室一览加关联分组 Chip 列；方法一览类型列去 minWidth |
| `pages/WorkloadPortal.tsx` | 按 show_in_work 过滤 |
| `pages/SamplePortal.tsx` | 按 show_in_rd 过滤 |

---

## 构建信息

- **Rust 编译**：待构建
- **前端**：需重新构建
- **安装包**：`installer/工作量统计工具_Rust_v0.4.10_Setup.exe`
- **覆盖安装**：✅ 支持（AppId 不变）

---

**版本隔离**：`v0.4.9 → v0.4.10` 完整复制后增量修改。
