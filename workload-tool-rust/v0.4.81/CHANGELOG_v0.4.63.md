# v0.4.63 版本更新说明

## 更新日期
2026-07-14

## 更新内容

### 新增：高项管理 + 项目关联（需求1）
- **新增表** `high_items`（id, name, sort_order, is_active），种子数据初始化"高项"
- **projects 表加 `high_item_id`** 外键关联到 high_items
- **管理页**：高项 CRUD（新建/编辑/软删除/回收站彻底删除），项目编辑对话框新增"关联高项"下拉选择
- **统计**：所有统计 Sheet 加"高项"列（ABC方案全实现）
- 录入页不显示高项字段（"仅前端不显示"），用户记录/统计时显示

### 新增：批量导入用户（需求2）
- **后端**：`POST /api/users/import` 端点，接受 CSV 格式文件
- **前端**：管理页用户 Tab 新增「批量导入」按钮 + 弹窗
- 默认密码：`123456`；自动跳过已存在的用户名
- 解析结果实时展示：成功数 / 跳过数 / 详细错误列表

### 修复：回收站不显示类型/项目/方法（需求3）
- **根因**：`loadTrash` 只调用 `getRecords({ include_deleted: true })`（只查 work_records）
- **修复**：回收站重构为多子 Tab 视图，显示详细删除记录
- **审计日志**：所有操作记录统一由 `audit_log` 管理

## 改动文件清单

| 文件 | 改动 |
|------|------|
| `src/db/migrations.rs` | + high_items 表 + projects.high_item_id |
| `src/models/high_item.rs` | **新建**：HighItem / HighItemCreate / HighItemUpdate |
| `src/models/project.rs` | + high_item_id, high_item_name |
| `src/repo/high_item_repo.rs` | **新建**：完整 CRUD + 软删除 + 审计 |
| `src/repo/project_repo.rs` | + high_item_id SELECT / INSERT / UPDATE |
| `src/repo/mod.rs` | + pub mod high_item_repo |
| `src/api/high_item_handler.rs` | **新建**：完整 CRUD 路由 + require_admin |
| `src/api/user_handler.rs` | + POST /api/users/import 端点 + Multipart 支持 |
| `src/api/mod.rs` | + high_item_handler 路由 |
| `frontend/src/types/index.ts` | + high_item_id, high_item_name |
| `frontend/src/api/client.ts` | + 高项 API 8 函数 + importUsers 函数 |
| `frontend/src/pages/ManagePage.tsx` | + 高项管理 UI + 项目关联下拉 + 批量导入弹窗 |

## 打包说明
- 无 cmd 窗口（生产模式 `cargo build --release`）
- 支持覆盖安装（AppId 固定）
