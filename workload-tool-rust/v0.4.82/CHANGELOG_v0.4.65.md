# v0.4.65 版本更新说明

## 更新日期
2026-07-14

## 更新内容

### 修复：导入用户部门/实验室/角色显示"未分配"
- **根因**：`import_users` 只读 CSV 前 2 列（用户名/密码），INSERT 只插 4 个字段（缺 division_id/group_id/role_id）
- **修复**：逐行解析部门名/实验室名/角色名，查对应 ID 后写入

### 新增：回收站用户 Tab + 彻底删除
- **回收站新增"用户"子 Tab**：显示已停用的用户（is_active=0）
- **回收站所有子 Tab 新增"彻底删除"按钮**（confirm 确认后物理删除）
- **后端新增** `DELETE /api/users/:id/permanent` 路由

### 改动文件

| 文件 | 改动 |
|------|------|
| `src/api/user_handler.rs` | import_users 加 dept/lab/role 解析；+permanent_delete_user 路由 |
| `src/repo/user_repo.rs` | 新增 permanent_delete 函数 |
| `frontend/src/api/client.ts` | +deleteUserPermanent |
| `frontend/src/pages/ManagePage.tsx` | 回收站+用户Tab+4Tab彻底删除 |
