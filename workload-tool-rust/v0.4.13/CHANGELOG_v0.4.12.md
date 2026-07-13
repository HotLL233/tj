## v0.4.12 — 2026-07-08

### 修复
- **修复帮助文档上传失败**：上传超过 2MB 的文档时报「上传失败」错误
  - 根因：Axum 默认 body 限制为 2MB，未配置 `DefaultBodyLimit`
  - 修复：help_handler 路由添加 `DefaultBodyLimit::max(100MB)` 层
  - 辅助修复：multipart 错误处理改为容错模式（`while let Ok(Some(field))`，与其他 handler 一致）
  - 辅助修复：前端上传失败提示改为显示具体错误原因（不再吞掉异常消息）

### 变更文件
- `src/api/help_handler.rs` — handler 路由添加 100MB body limit；multipart 错误处理改为容错模式
- `frontend/src/pages/ManagePage.tsx` — 上传失败 catch 块显示具体错误信息

### 安装说明
- 本次更新覆盖安装即可，无需卸载旧版本
- 安装包文件名：`工作量统计工具_Rust_v0.4.12_Setup.exe`
