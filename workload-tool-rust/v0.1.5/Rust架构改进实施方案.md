---
AIGC:
    Label: "1"
    ContentProducer: 001191440300708461136T1XGW3
    ProduceID: 2864be058e4c066fbf5226e634b510c4_873d7860715711f1aabe5254007bceed
    ReservedCode1: /fgO+mGHZdvKXiCQbf3ih0QO4kwa4hOMkr9I+RmIOlkyKAkgauhr2efU4FYoVJF0AGl1wO8nXKS0/y6yBACoN618D6qb1QSqC3S36ZKqGTl3wFnVEl+Fc56zRVUPO/eUjQb72xdpEhDVnHqcMdEX95We0Q/WO/PKzQbOWTiIYZh3LIGVl/Mhuz+g200=
    ContentPropagator: 001191440300708461136T1XGW3
    PropagateID: 2864be058e4c066fbf5226e634b510c4_873d7860715711f1aabe5254007bceed
    ReservedCode2: /fgO+mGHZdvKXiCQbf3ih0QO4kwa4hOMkr9I+RmIOlkyKAkgauhr2efU4FYoVJF0AGl1wO8nXKS0/y6yBACoN618D6qb1QSqC3S36ZKqGTl3wFnVEl+Fc56zRVUPO/eUjQb72xdpEhDVnHqcMdEX95We0Q/WO/PKzQbOWTiIYZh3LIGVl/Mhuz+g200=
---

# Rust 工作量统计工具 — 架构改进实施方案

> **目标**：在不破坏现有功能的前提下，渐进式完成架构加固。
> **项目路径**：`D:\桌面\工作量统计工具项目\workload-tool-rust\`
> **任务交付对象**：Codex / AI 编码助手

---

## 总览

| 阶段 | 主题 | 预估改动量 | 优先级 |
|------|------|-----------|--------|
| Phase 1 | Service 层实现 + 审计抽象 + 配置化 + 日志 | ~500 行 | 🔴 高 |
| Phase 2 | Repo trait 抽象 + 查询辅助 | ~400 行 | 🟡 中 |
| Phase 3 | 种子数据外部化 + 健康检查 + 优雅关闭 | ~300 行 | 🟢 低 |

要求：每个 Phase 完成后 `cargo build` 必须通过，`cargo test` 全部通过，已有 API 行为不变。

---

## Phase 1：Service 层实现 + 审计抽象 + 配置化 + 日志

### 任务 1.1：创建配置模块

**新建文件** `src/config.rs`

```rust
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_port")]
    pub server_port: u16,

    #[serde(default = "default_db_dir")]
    pub db_dir: String,

    #[serde(default = "default_log_level")]
    pub log_level: String,

    #[serde(default)]
    pub log_file: Option<String>,
}

fn default_port() -> u16 { 8000 }
fn default_db_dir() -> String { "data".to_string() }
fn default_log_level() -> String { "info".to_string() }

impl AppConfig {
    pub fn load() -> Self {
        // 1. 先尝试读 exe 同目录的 config.toml
        // 2. 不存在则用默认值
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_default();

        let config_path = exe_dir.join("config.toml");
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path).unwrap_or_default();
            toml::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn db_path(&self) -> PathBuf {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_default();
        exe_dir.join(&self.db_dir).join("workload.db")
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server_port: default_port(),
            db_dir: default_db_dir(),
            log_level: default_log_level(),
            log_file: None,
        }
    }
}
```

**Cargo.toml 追加依赖**：
```toml
toml = "0.8"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tracing-appender = "0.2"
```

**修改 `main.rs`**：
- 在 `main()` 开头调用 `AppConfig::load()`，将配置传入 `AppState`
- 端口号、数据库路径从配置读取，不再硬编码
- 初始化 tracing subscriber：开发模式（console feature）输出到控制台，生产模式写文件日志

改动点：
- `AppState` 新增 `config: AppConfig` 字段
- `let addr = SocketAddr::from(([0, 0, 0, 0], state.config.server_port));`
- 数据库初始化路径改为 `state.config.db_path()`

---

### 任务 1.2：审计日志抽象

**新建文件** `src/audit.rs`

目标：用一个高阶函数统一包裹事务 + 审计日志写入，消除当前每个 repo 函数里手动调用 `log_on_conn` 的重复代码。

```rust
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Transaction;
use crate::models::AuditAction;

/// 在事务中执行业务逻辑，自动写入审计日志并提交。
/// 业务逻辑返回 Err 时事务自动回滚，审计日志不写入。
pub fn execute_with_audit<F, T>(
    pool: &Pool<SqliteConnectionManager>,
    action: AuditAction,
    description: &str,
    user_name: Option<&str>,
    f: F,
) -> Result<T, crate::error::AppError>
where
    F: FnOnce(&Transaction) -> Result<T, crate::error::AppError>,
{
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;

    let result = f(&tx)?;

    // 写入审计日志（同一事务）
    tx.execute(
        "INSERT INTO audit_log (action, description, user_name, created_at) VALUES (?1, ?2, ?3, datetime('now','localtime'))",
        rusqlite::params![action.to_str(), description, user_name],
    )?;

    tx.commit()?;
    Ok(result)
}
```

**修改 `src/repo/` 下所有写操作函数**：

示例——改造 `create_record`：
```rust
// 改造前
pub fn create_record(pool: &Pool, ...) -> Result<Record> {
    let conn = pool.get()?;
    let tx = conn.transaction()?;
    // ... 插入逻辑
    log_on_conn(&tx, AuditAction::Create, ...)?;
    tx.commit()?;
}

// 改造后
pub fn create_record(pool: &Pool, ...) -> Result<Record> {
    execute_with_audit(pool, AuditAction::Create, "创建工作量记录", Some(&record.user_name), |tx| {
        // 只写纯业务 SQL，不再关心审计
        tx.execute("INSERT INTO work_records ...", params![...])?;
        let id = tx.last_insert_rowid();
        Ok(Record { id, ... })
    })
}
```

同样改造 `update_record`、`delete_record`、`restore_record`、`batch_delete_by_user`、`create_group`、`update_group`、`delete_group`、`create_project`、`update_project`、`delete_project`。

**删除** `src/repo/` 中原有的 `log_on_conn` 独立函数。

---

### 任务 1.3：实现 Service 层

**新建文件** `src/service/mod.rs`、`src/service/stats_service.rs`、`src/service/record_service.rs`

**原则**：Handler 只做参数提取和 HTTP 响应，业务编排、校验规则、聚合计算全部下沉到 Service。

#### `src/service/record_service.rs`

```rust
use crate::repo::{record_repo, audit_repo};
use crate::models::*;

pub struct RecordService;

impl RecordService {
    /// 创建一条工作量记录（含业务校验）
    pub fn create(pool: &Pool, input: CreateRecordInput) -> Result<Record, AppError> {
        // 1. 校验项目是否存在
        let project = project_repo::find_by_id(pool, input.project_id)?
            .ok_or(AppError::NotFound("项目不存在".into()))?;

        // 2. 校验数量 > 0
        if input.quantity <= 0 {
            return Err(AppError::Validation("数量必须大于0".into()));
        }

        // 3. 委托 repo 写入
        record_repo::create_record(pool, &input, &project)
    }

    /// 批量录入（一个事务内完成多条记录）
    pub fn batch_create(pool: &Pool, inputs: Vec<CreateRecordInput>) -> Result<Vec<Record>, AppError> {
        // 使用 execute_with_audit 包裹整个批次
        audit::execute_with_audit(pool, AuditAction::Create, "批量录入", None, |tx| {
            let mut records = Vec::new();
            for input in &inputs {
                let project = project_repo::find_by_id_on_tx(tx, input.project_id)?
                    .ok_or(AppError::NotFound("项目不存在".into()))?;
                let record = record_repo::create_record_on_tx(tx, input, &project)?;
                records.push(record);
            }
            Ok(records)
        })
    }

    /// 修改记录（纠错）—— 需判断是否真正有变更
    pub fn update(pool: &Pool, id: i64, input: UpdateRecordInput) -> Result<Record, AppError> {
        let old = record_repo::find_by_id(pool, id)?
            .ok_or(AppError::NotFound("记录不存在".into()))?;

        if old.user_name == input.user_name && old.quantity == input.quantity {
            return Err(AppError::Validation("未做任何修改".into()));
        }

        record_repo::update_record(pool, id, &input, &old)
    }
}
```

#### `src/service/stats_service.rs`

将当前 `src/api/stats.rs` 中的聚合逻辑移到这里：

```rust
pub struct StatsService;

impl StatsService {
    /// 周统计：按天汇总本周（周一～周日）
    pub fn weekly_summary(pool: &Pool) -> Result<Vec<DailySummary>, AppError> { ... }

    /// 月统计：按天汇总本月
    pub fn monthly_summary(pool: &Pool, year: i32, month: u32) -> Result<Vec<DailySummary>, AppError> { ... }

    /// 按用户统计
    pub fn by_user(pool: &Pool, start: &str, end: &str) -> Result<Vec<UserSummary>, AppError> { ... }

    /// 按项目统计
    pub fn by_project(pool: &Pool, start: &str, end: &str) -> Result<Vec<ProjectSummary>, AppError> { ... }

    /// 按仪器类型统计（液相/气相）
    pub fn by_type(pool: &Pool, start: &str, end: &str) -> Result<Vec<TypeSummary>, AppError> { ... }

    /// 按仪器统计
    pub fn by_instrument(pool: &Pool, start: &str, end: &str) -> Result<Vec<InstrumentSummary>, AppError> { ... }
}
```

**修改 `src/api/stats.rs`**：
- 删除聚合计算代码
- Handler 函数改为调用 `StatsService::xxx(pool, ...)`，返回 JSON
- 每个 handler 函数体 ≤ 15 行

**新建 `src/service/mod.rs`**：
```rust
pub mod stats_service;
pub mod record_service;
```

**修改 `src/main.rs`**：
- 添加 `mod service;`

---

### Phase 1 收尾检查清单

- [ ] `cargo build` 通过
- [ ] `cargo test` 通过
- [ ] 启动应用，所有页面功能正常
- [ ] `config.toml` 不存在时使用默认值正常启动
- [ ] 创建 `config.toml` 并修改端口号，重启后生效
- [ ] 生产模式（无 console feature）下日志写入文件

---

## Phase 2：Repo trait 抽象 + 查询辅助

### 任务 2.1：定义 Repository trait

**新建文件** `src/repo/traits.rs`

每个实体定义一个 trait + 一个 Filter/Query struct：

```rust
use async_trait::async_trait;  // 如果用异步
// 或不用 async_trait，直接用同步 trait

pub trait RecordRepository {
    fn create(&self, input: &CreateRecordInput, project: &Project) -> Result<Record, AppError>;
    fn find_by_id(&self, id: i64) -> Result<Option<Record>, AppError>;
    fn list(&self, filter: &RecordFilter) -> Result<PaginatedResult<Record>, AppError>;
    fn update(&self, id: i64, input: &UpdateRecordInput, old: &Record) -> Result<Record, AppError>;
    fn soft_delete(&self, id: i64) -> Result<(), AppError>;
    fn restore(&self, id: i64) -> Result<Record, AppError>;
    fn batch_delete_by_user(&self, user_name: &str) -> Result<usize, AppError>;
}

pub struct RecordFilter {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub user_name: Option<String>,
    pub project_id: Option<i64>,
    pub group_id: Option<i64>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub include_deleted: bool,
}
```

同样定义 `GroupRepository`、`ProjectRepository`、`AuditLogRepository`。

### 任务 2.2：SQLite 实现

**新建文件** `src/repo/sqlite_record_repo.rs` 等：

```rust
pub struct SqliteRecordRepo {
    pool: Pool<SqliteConnectionManager>,
}

impl RecordRepository for SqliteRecordRepo {
    fn create(&self, input: &CreateRecordInput, project: &Project) -> Result<Record, AppError> {
        audit::execute_with_audit(&self.pool, AuditAction::Create, "创建记录", Some(&input.user_name), |tx| {
            // ... 插入逻辑
        })
    }
    // ... 其他方法
}
```

### 任务 2.3：修改 AppState 支持依赖注入

```rust
// main.rs
pub struct AppState {
    pub config: AppConfig,
    pub record_repo: Arc<dyn RecordRepository>,
    pub group_repo: Arc<dyn GroupRepository>,
    pub project_repo: Arc<dyn ProjectRepository>,
    pub audit_repo: Arc<dyn AuditLogRepository>,
}
```

Handler 从 `State(state): State<AppState>` 中获取对应的 repo 调用。

### 任务 2.4：修改 Service 层使用 trait

```rust
pub struct RecordService {
    repo: Arc<dyn RecordRepository>,
    project_repo: Arc<dyn ProjectRepository>,
}

impl RecordService {
    pub fn new(
        repo: Arc<dyn RecordRepository>,
        project_repo: Arc<dyn ProjectRepository>,
    ) -> Self { ... }

    pub fn create(&self, input: CreateRecordInput) -> Result<Record, AppError> {
        let project = self.project_repo.find_by_id(input.project_id)?;
        self.repo.create(&input, &project)
    }
}
```

---

## Phase 3：种子数据外部化 + 健康检查 + 优雅关闭

### 任务 3.1：种子数据外部化

**新建目录和文件** `data/seed_groups.json`、`data/seed_projects.json`

```json
// data/seed_groups.json
[
  { "id": 1, "name": "410", "display_order": 1 },
  { "id": 2, "name": "415", "display_order": 2 }
  // ...
]
```

```json
// data/seed_projects.json
[
  { "id": 1, "name": "方法A", "full_name": "方法A完整名称", "group_id": 1, "instrument_type": "LC", "instrument_no": "LC-001" }
  // ...
]
```

**修改 `src/db/seed.rs`**：

```rust
const SEED_GROUPS: &str = include_str!("../../data/seed_groups.json");
const SEED_PROJECTS: &str = include_str!("../../data/seed_projects.json");

pub fn seed(pool: &Pool) -> Result<(), AppError> {
    let groups: Vec<SeedGroup> = serde_json::from_str(SEED_GROUPS)?;
    let projects: Vec<SeedProject> = serde_json::from_str(SEED_PROJECTS)?;
    // 插入逻辑不变
}
```

### 任务 3.2：健康检查端点

**新增路由** `GET /api/health`

```rust
// src/api/health.rs
pub async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    match state.pool.get() {
        Ok(_) => Json(json!({ "status": "ok", "version": env!("CARGO_PKG_VERSION") })),
        Err(e) => Json(json!({ "status": "error", "message": e.to_string() })),
    }
}
```

### 任务 3.3：优雅关闭

**修改 `main.rs`**，将 `axum::serve` 包装在 `tokio::select!` 中：

```rust
let listener = tokio::net::TcpListener::bind(&addr).await?;
let (tx, rx) = tokio::sync::oneshot::channel::<()>();

// 托盘退出或 Ctrl+C 时发送关闭信号
let shutdown_signal = async move {
    tokio::signal::ctrl_c().await.ok();
    tracing::info!("收到关闭信号，正在优雅退出...");
    let _ = tx.send(());
};

tokio::select! {
    result = axum::serve(listener, app) => {
        if let Err(e) = result {
            tracing::error!("服务器错误: {}", e);
        }
    }
    _ = shutdown_signal => {
        tracing::info!("服务器已关闭");
    }
}
```

---

## 通用约束（所有 Phase 生效）

1. **编译通过**：每个 Phase 结束必须 `cargo build` 成功，不允许有 warning（`RUSTFLAGS="-D warnings"`）
2. **测试通过**：`cargo test` 全部通过，新增代码应有对应测试
3. **API 兼容**：22 个端点路径和方法不变，返回 JSON 结构不变
4. **不破坏前端**：前端代码不动，SPA 路由和 API 调用不受影响
5. **代码风格**：保持项目现有风格——英文注释、snake_case、模块化文件组织
6. **提交粒度**：每个子任务完成后独立 commit，commit message 格式 `phase(N): 简短描述`

---

## 可选增强（低优先级，视情况执行）

- [ ] **API 版本化**：在路由前加 `/api/v1/` 前缀，旧路径保留并重定向
- [ ] **前端 dev proxy**：`build.rs` 检测 `static/index.html` 不存在时输出 warning
- [ ] **Docker 支持**：写 `Dockerfile` 便于部署
- [ ] **CI/CD**：`.github/workflows/build.yml` 自动构建 + 测试
- [ ] **前端独立仓库**：将 React 源码拆出为独立 git 仓库，`static/` 以 git submodule 引入
*（内容由AI生成，仅供参考）*
