# v0.4.75

## 修复内容

- 修复项目创建时 `no such table: main.high_items` 的 SQLite 迁移问题。
- 保留 `high_items` 兼容表，避免旧数据库中的 `high_item_id` 外键失效。
- 增加后端 `help:edit` 权限白名单，与前端权限配置保持一致。
- 清理重复角色权限，并增加 `(role_id, permission_key)` 唯一索引。
- 修复多角色用户删除角色时未检查 `user_roles` 的问题。
- 新增管理员专用 `POST /api/users` 用户创建接口，支持 `role_ids`。
- 管理页创建用户改用管理员接口；公开注册接口不允许自授角色。

## 验证

- `cargo test`
- `cargo build --features console`
- `npm.cmd run build`
- 用户多角色、角色权限、项目创建和角色删除回归测试
