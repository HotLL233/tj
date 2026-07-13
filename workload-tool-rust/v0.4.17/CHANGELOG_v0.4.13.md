## v0.4.13 — 2026-07-08

### 修复
- **修复 PDF 点击查看触发下载而非浏览器内嵌显示**
  - 根因：`Vec<u8>::into_response()` 默认注入 `Content-Type: application/octet-stream`，与 `HeaderMap` 中我们的 `Content-Type: application/pdf` 形成重复头，浏览器取第一个 `octet-stream` 触发下载
  - 修复：改用 `Response::new(Body::from(data))` 直接构造响应，绕过 `Vec<u8>` 的默认 Content-Type，确保只有一个正确的 Content-Type 头

### 变更文件
- `src/api/help_handler.rs` — `get_file` 函数返回值从 `(StatusCode, HeaderMap, Vec<u8>)` 改为直接构造 `Response`；移除未使用的 `HeaderMap` import

### 安装说明
- 本次更新覆盖安装即可，无需卸载旧版本
- 安装包文件名：`工作量统计工具_Rust_v0.4.13_Setup.exe`
