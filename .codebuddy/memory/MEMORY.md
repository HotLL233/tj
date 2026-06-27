# 项目长期记忆

## 开发策略 (2026-06-25 起生效)

1. **Python 版已冻结**：不再进行任何更新，仅保留 v0.2.5 作为历史版本。
2. **全部精力投入 Rust 版**：后续所有开发只针对 `workload-tool-rust/`。
3. **版本隔离原则**：每次代码变更前，将当前最新版本完整复制到新的版本号文件夹（如 `v0.1.3/` → `v0.1.4/`），在新文件夹内修改，**绝不覆盖原有代码**。
4. **文件夹命名**：以清晰版本号命名，如 `workload-tool-rust/v0.1.3/`、`v0.1.4/`。
5. **当前最新版本**：v0.2.1，路径 `workload-tool-rust/v0.2.1/`。

## 关键路径

- Python 最终版：`project-root/`
- Rust 版本基线：`workload-tool-rust/v0.1.7/`
- 安装包输出：`installer/`
- Inno Setup：`D:\APP\Inno Setup 6\ISCC.exe`

## 打包流程

1. 在新版本文件夹内 `cargo build --release`
2. 复制 exe 到 dist 文件夹
3. 复制前端 static 文件
4. 更新 `build_installer.iss`（文件源路径、版本号、输出文件名）
5. `ISCC.exe build_installer.iss`

## 技术约定

- Rust 版本号唯一来源：`Cargo.toml` → `env!("CARGO_PKG_VERSION")` 自动传播到 tray.rs/main.rs
- 前端版本号：通过 `/api/version` 端点动态获取
- 错误响应：统一 HTTP 200 + JSON code 字段（对齐前端 axios）
- 数据库：SQLite WAL 模式，连接池自动初始化 PRAGMA
- 托盘：tray-icon + winit，单实例保护，2 秒打开节流
