# v0.4.72 版本更新说明

## 更新日期
2026-07-15

## 修复
### Bug 1：新列在录入页不显示
- 根因：list_active_by_type SQL 要求 c.type_key=?1，新列 type_key=NULL 永远不匹配
- 修复：改为 LEFT JOIN visibility 表统一过滤，无 visibility 行=全部可见

### Bug 2：保存 ICP 后返回管理页变成"全部"
- 根因：前端 PUT /api/sample-info/columns/:id/types 后端无对应路由 → 404 静默失败
- 修复：加 PUT /api/sample-info/columns/:id/types 端点，调 set_visibility 持久化

### 优化：新列默认全类型可见
- 创建列时自动 INSERT sample_info_column_visibility 为所有活跃类型写可见行

## 改动文件
- `src/api/sample_info_column_handler.rs` — +update_column_types 端点
- `src/repo/sample_info_column_repo.rs` — list_active_by_type 统一 LEFT JOIN + create 自动可见
