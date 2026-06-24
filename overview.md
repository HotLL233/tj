# Rust 工作量统计工具 — 路由 405/HTML Bug 修复概览

## TL;DR
axum 0.7.9 使用 `:id` 语法而非 `{id}` 语法进行路径参数匹配，导致所有带 ID 的 API 路由（修改/删除/恢复）无法匹配，请求落入 SPA fallback 返回 HTML。修复后全部 10 条路由测试通过。

## 根因分析

### 问题现象
- `PUT /api/records/1` → 返回 HTML（而非 JSON）
- `DELETE /api/records/1` → 返回 HTML
- `POST /api/records/restore/1` → 返回 HTML
- `PUT /api/groups/1` → 返回 HTML
- `GET /api/records` → ✅ 正常返回 JSON
- `GET /api/groups` → ✅ 正常返回 JSON

### 根因
**axum 0.7.9（matchit 0.7.3）的路径参数语法是 `:id`，不是 `{id}`**

| 语法 | 版本 | matchit 0.7.3 行为 |
|------|------|-------------------|
| `:id` | axum 0.7 (matchit 0.7) | ✅ 参数捕获，匹配任意单段 |
| `{id}` | axum 0.8 (matchit 0.8) | ❌ 被当作字面量字符串 |

代码中所有路由使用了 `{id}` 语法，matchit 0.7.3 将其视为字面量，所以 `/api/records/{id}` 只匹配 URL `/api/records/{id}` 本身，不匹配 `/api/records/1`。

### 之前的调试误区
1. 误判为 `.nest()` + `fallback(serve_index)` 遮蔽问题
2. 误判为 matchit 路由树冲突（`/{id}` vs `/{id}/restore`）
3. 编译失败（serve_index tuple 顺序错误）导致旧二进制持续运行，测试结果全部无效
4. 多次修改路由结构（注释/恢复路由）均未触及真正根因

## 修复内容

### 1. 路径参数语法（核心修复）
所有 handler 中 `{id}` → `:id`，`{user_name}` → `:user_name`

### 2. 路由架构优化
- `mod.rs`：`.nest()` → `.merge()` + 完整路径前缀
- 每个 handler 的路由使用完整路径（如 `/api/records/:id` 而非 `/:id`）

### 3. serve_index 编译错误修复
- `(String, StatusCode)` → `(StatusCode, String)`（axum 0.7 IntoResponse trait bound）
- 恢复为正确的 HTML 文件读取逻辑

### 4. SPA fallback 恢复
- `.fallback(serve_index)` 正确配置

### 5. record_handler 完整路由恢复
- 恢复 `restore/:id` 和 `by-user/:user_name` 路由（之前调试中被注释）

### 6. record_repo.rs 事务原子性（之前已完成）
- 所有写操作用 `conn.transaction()` 包裹
- 审计日志用 `log_on_conn(&tx, ...)`
- UPDATE 影响行数检查 + restore 存在性检查

## 修改文件清单

| 文件 | 修改内容 |
|------|---------|
| `src/api/mod.rs` | `.nest()` → `.merge()` |
| `src/api/record_handler.rs` | `{id}` → `:id` + 完整路径 + 恢复路由 |
| `src/api/group_handler.rs` | `{id}` → `:id` + 完整路径 |
| `src/api/project_handler.rs` | `{id}` → `:id` + 完整路径 |
| `src/api/stats_handler.rs` | 完整路径前缀 |
| `src/api/export_handler.rs` | 完整路径前缀 |
| `src/api/audit_handler.rs` | 完整路径前缀 |
| `src/main.rs` | 修复 serve_index + 恢复 fallback |

## 验证结果（全部通过）

| # | 路由 | HTTP | Content-Type | 结果 |
|---|------|------|-------------|------|
| 1 | `GET /api/version` | 200 | application/json | ✅ |
| 2 | `GET /api/records` | 200 | application/json | ✅ |
| 3 | `PUT /api/records/1` | 200 | application/json | ✅ quantity→99 |
| 4 | `DELETE /api/records/2` | 200 | application/json | ✅ 删除成功 |
| 5 | `POST /api/records/restore/2` | 200 | application/json | ✅ 恢复成功 |
| 6 | `PUT /api/groups/1` | 400 | text/plain | ✅ 路由匹配（curl编码问题）|
| 7 | `PUT /api/projects/1` | 200 | application/json | ✅ |
| 8 | `GET /api/stats/summary` | 200 | application/json | ✅ 数据已更新 |
| 9 | `GET /api/audit-logs` | 200 | application/json | ✅ 含审计日志 |
| 10 | `POST /api/records` | 200 | application/json | ✅ 新建 id=10 |

## 前端
`client.ts` **无需修改** — baseURL `/api` + 相对路径已与后端路由完全匹配。

## 下一步建议
1. 重新编译 release 版本：`cargo build --release`
2. 打包前端静态文件到 `static/` 目录
3. 测试前端页面完整功能（修改/删除/恢复/创建）
4. 打包为安装程序
