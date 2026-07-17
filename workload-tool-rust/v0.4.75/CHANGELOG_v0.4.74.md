# v0.4.74 版本更新说明

## 更新日期
2026-07-15

## 新增：用户多角色 + 文档排序

### 多角色绑定
- **新表 `user_roles`**：用户可绑定多个角色，权限取所有角色的并集
- 迁移回填：已有 `users.role_id` 自动同步到 `user_roles`
- 用户管理页角色选择改为多选 Checkbox 列表
- 创建用户时支持传 `role_ids` 数组
- 种子角色「文档编辑员」仅含 `help:edit` 权限，可分配给任意用户

### 文档/文章排序
- 新增 `PUT /api/help-documents/sort` 和 `PUT /api/help-articles/sort` 端点
- 管理页文档列表和文章列表支持拖拽排序（使用现有 sort_order 字段）

## 改动文件
- `src/db/migrations.rs` — +user_roles 表 + 回填 + 文档编辑员种子角色
- `src/models/user.rs` — User +role_ids、UserCreate/UserUpdate +role_ids
- `src/repo/user_repo.rs` — 权限读取改为 user_roles JOIN、create/update 写 user_roles
- `src/service/auth_service.rs` — register 传空 role_ids
- `src/repo/help_repo.rs` — +reorder_documents
- `src/repo/article_repo.rs` — +reorder
- `src/api/help_handler.rs` — +PUT /api/help-documents/sort
- `src/api/article_handler.rs` — +PUT /api/help-articles/sort
