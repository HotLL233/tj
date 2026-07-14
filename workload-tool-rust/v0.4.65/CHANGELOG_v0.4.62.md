# v0.4.62 版本更新说明

## 更新日期
2026-07-14

## 更新内容

### 修复：PDF/Word 附件上传失败（> 2MB 文件）
- **根因**：axum 0.7 的 `Multipart` 提取器默认 body 限制为 2 MiB（2,097,152 字节），超过此大小的文件（如 3.2MB 的 PDF）被 axum 拒绝，导致 `field.bytes()` 返回 `"Error parsing multipart/form-data"` 错误
- **修复**：在 `sample_info_attachment_handler` 的 router 上添加 `.layer(DefaultBodyLimit::max(100 * 1024 * 1024))`，将所有上传 body 限制提升至 100MB
- **同时提高**：`MAX_SIZE` 常量从 10MB 改为 100MB，前端 `file.size` 检查从 10MB 改为 100MB
- **实测验证**：
  - 3MB PDF 上传 → ✅ 通过（此前旧代码报 code=5000）
  - 50MB PDF 上传 → ✅ 通过（multipart 解析成功，外键约束为正常业务错误）

### 改动文件清单

| 文件 | 改动 | MD5 变化 |
|---|---|---|
| `src/api/sample_info_attachment_handler.rs` | 加 `use DefaultBodyLimit` + router 加 `.layer(...)` + `MAX_SIZE` 10→100 + 错误提示 10→100MB | `3e09e713…` → `f2bd1e86…` |
| `frontend/src/pages/SampleInfoEntry.tsx` | 前端 `file.size` 检查 10→100MB + 提示文字 | `55d0f48f…` → `c356f99a…` |
| `Cargo.toml` / `frontend/package.json` / `build_installer.iss` | 版本号 0.4.61 → 0.4.62 | — |

**未改动文件**：所有其他源文件与 v0.4.61 完全一致。

## 打包说明
- 无 cmd 窗口（生产模式 `cargo build --release`）
- 支持覆盖安装（AppId 固定）
- 安装包：`D:\桌面\工作量统计工具项目\installer\样品管理系统_v0.4.62_Setup.exe`
- Docker：`ghcr.io/hotll233/tj/workload-tool:v0.4.62`
