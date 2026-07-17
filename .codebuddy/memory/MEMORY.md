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

## Git 同步约定 (2026-07-13 起)

- 用户频繁要求"同步 git，更改数太多"：直接 `git add -A` + 单笔提交 + 推送，无需逐文件确认。
- **提交前必须排查构建/缓存类垃圾**：Rust `target/`、`vite.config.ts.timestamp-*.mjs`、`*.bak`、`pdfium*.zip`、`.workbuddy/artifacts/`、`deliverables/**/target/`。
- 若误将大型产物（如 `deliverables/_build-cache-*/target/` 约 5GB）加入暂存，**在推送前**用 `git reset --mixed HEAD~N` 回退并重写历史，使大文件不进远程。
- 用户工作流会保留大量 `v0.x.y/` 整份版本拷贝与 `deliverables/` 交付物目录，均按正常文件提交。

## 技术约定

- Rust 版本号唯一来源：`Cargo.toml` → `env!("CARGO_PKG_VERSION")` 自动传播到 tray.rs/main.rs
- 前端版本号：通过 `/api/version` 端点动态获取
- 错误响应：统一 HTTP 200 + JSON code 字段（对齐前端 axios）
- 数据库：SQLite WAL 模式，连接池自动初始化 PRAGMA
- 托盘：tray-icon + winit，单实例保护，2 秒打开节流
