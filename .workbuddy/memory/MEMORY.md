# 工作量统计工具项目 — 长期记忆

## 项目概况
- Rust (Axum 0.7.9 + rusqlite 0.31 + r2d2) 后端 + React 18 + TypeScript 前端
- 前端打包为静态文件由 Rust 后端 serve，SPA 模式
- 端口 8000，系统托盘运行（生产模式隐藏控制台）
- 数据库：SQLite，4 张表（project_groups, projects, work_records, audit_log）

## 关键技术决策
- **axum 0.7 路由语法**：使用 `:id`（不是 `{id}`，后者是 axum 0.8 语法）
- **路由架构**：使用 `.merge()` + 完整路径前缀（不用 `.nest()`，避免嵌套路由参数匹配问题）
- **错误处理**：统一返回 HTTP 200 + 业务错误码（与 Python 版一致）
- **事务控制**：所有写操作用 `conn.transaction()` 包裹，审计日志用 `log_on_conn(&tx, ...)`
- **SPA fallback**：`.fallback(serve_index)` 服务 `static/index.html`，仅对非 API 路径生效

## 版本历史
- v1.7.0/v1.7.1：Python (FastAPI) 版，PyInstaller + Inno Setup 打包
- v0.1.1 (Rust)：当前开发版本，正在移植 Python 版全部功能

## 已知问题与修复记录
- 2026-06-24：修复路由 405/HTML bug（根因：`{id}` 语法错误 → `:id`）
- 2026-06-24：修复 record_repo.rs 事务原子性（跨连接审计 → 同连接事务）
- 2026-06-24：修复 serve_index 编译错误（tuple 顺序 `(StatusCode, String)` 非 `(String, StatusCode)`）

## 文件结构
```
workload-tool-rust/
  src/
    main.rs          — 入口，路由组装，SPA fallback
    api/
      mod.rs         — API 路由汇总（.merge()）
      record_handler.rs  — 记录 CRUD + restore + by-user
      group_handler.rs   — 分组 CRUD
      project_handler.rs — 项目 CRUD
      stats_handler.rs   — 统计（summary/by-user/by-project/by-type/by-instrument）
      export_handler.rs  — Excel 导出
      audit_handler.rs   — 审计日志
    repo/
      record_repo.rs — 记录 DAO（事务原子性已修复）
      group_repo.rs  — 分组 DAO
      project_repo.rs— 项目 DAO
      audit_repo.rs  — 审计 DAO（log + log_on_conn）
    models/          — 数据模型
    error.rs         — 统一错误处理
    db/              — 数据库初始化 + 迁移 + 种子数据
    tray.rs          — 系统托盘
project-root/
  frontend/src/api/client.ts — 前端 API 客户端（baseURL: /api）

## 打包工具路径
- **Inno Setup 6.7.2**: `D:/APP/Inno Setup 6/ISCC.exe`
- 编译命令: `"D:/APP/Inno Setup 6/ISCC.exe" build_installer.iss`
```
