# 工作量统计工具 (Rust)

局域网多用户工作量录入与统计系统，Rust 重写版。

## 快速启动

```bash
# 开发模式（带控制台）
cargo run --features console

# 生产构建（隐藏控制台，系统托盘运行）
cargo build --release
.\target\release\workload-tool.exe
```

启动后访问 `http://127.0.0.1:8000` 或局域网 IP:8000。

## 配置项

在 exe 同目录创建 `config.toml`（可选，不存在则用默认值）：

```toml
server_port = 8000       # 监听端口（默认 8000）
db_dir = "data"          # 数据库目录（默认 data/）
log_level = "info"       # 日志级别: trace/debug/info/warn/error
log_file = "app.log"     # 日志文件名（不设则仅控制台输出）
```

## API 列表

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/version` | 服务端版本号 |
| GET | `/api/health` | 健康检查（DB 连通性） |
| GET | `/swagger-ui` | Swagger 文档 UI |
| GET | `/api-docs/openapi.json` | OpenAPI JSON |
| **分组** | | |
| GET | `/api/groups` | 分组列表 |
| POST | `/api/groups` | 创建分组 |
| PUT | `/api/groups/:id` | 更新分组 |
| DELETE | `/api/groups/:id` | 删除分组 |
| **项目** | | |
| GET | `/api/projects` | 项目列表 `?group_id=&active_only=1` |
| POST | `/api/projects` | 创建项目 |
| PUT | `/api/projects/:id` | 更新项目 |
| DELETE | `/api/projects/:id` | 删除项目 |
| **记录** | | |
| GET | `/api/records` | 记录列表 `?page=&page_size=&user_name=&start=&end=` |
| POST | `/api/records` | 录入记录 `{project_id,user_name,quantity,recorded_at}` |
| PUT | `/api/records/:id` | 修改记录 |
| DELETE | `/api/records/:id` | 软删除单条 |
| POST | `/api/records/restore/:id` | 恢复记录 |
| DELETE | `/api/records/by-user/:name` | 删除用户所有记录 `?start=&end=` |
| **统计** | | |
| GET | `/api/stats/summary` | 统计汇总 `?start=&end=&group_by=week` |
| GET | `/api/stats/by-user` | 按用户统计 |
| GET | `/api/stats/by-project` | 按项目统计 |
| GET | `/api/stats/by-type` | 按仪器类型统计 |
| GET | `/api/stats/by-instrument` | 按仪器统计 |
| **导出** | | |
| GET | `/api/export/excel` | 导出 Excel `?start=&end=&group_id=` |
| **审计** | | |
| GET | `/api/audit-logs` | 审计日志 `?page=&page_size=` |

### GET /api/export/excel 参数

| 参数 | 类型 | 说明 |
|------|------|------|
| `start` | string | 起始日期 (YYYY-MM-DD)，默认当月 |
| `end` | string | 结束日期 |
| `group_id` | int | 筛选分组 ID |

导出包含 5 个 Sheet：月-汇总、每日工作量、每周工作量、原始记录、用户统计。

## 技术栈

- **Web**: Axum 0.7 + Tokio
- **数据库**: SQLite (rusqlite + r2d2 连接池)
- **Excel**: rust_xlsxwriter
- **序列化**: serde + serde_json
- **文档**: utoipa + Swagger UI
- **日志**: tracing + tracing-subscriber
- **配置**: toml
