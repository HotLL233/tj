# 工作量统计工具 — Rust 架构重构方案

---

## 一、现状分析

### 1.1 当前架构（Python 方案）

```
┌──────────────┐    HTTP/JSON     ┌──────────────┐    SQL     ┌──────────┐
│  React SPA   │ ◄──────────────► │  FastAPI 8080 │ ◄────────► │ SQLite   │
│  (Vite/MUI)  │    Axios         │  (uvicorn)    │           │  WAL     │
└──────────────┘                  └──────────────┘           └──────────┘
                                        │
                              ┌─────────┴─────────┐
                              │ PyInstaller .exe   │
                              │ + pystray 系统托盘  │
                              │ + Inno Setup 安装包 │
                              └───────────────────┘
```

### 1.2 核心模块清单

| 模块 | 当前实现 | 行数 | 功能 |
|------|---------|------|------|
| **HTTP Server** | FastAPI + uvicorn | 70 | REST API + SPA 静态文件 |
| **路由层** | 6 个 FastAPI Router | ~150 | 分组/项目/记录/统计/导出/审计 |
| **数据库** | sqlite3 直连 | 305 | 4 表 CRUD + 种子数据 + 迁移 |
| **统计引擎** | SQL GROUP BY | 300 | 7 维度聚合统计 |
| **Excel 导出** | openpyxl | 570 | 5 Sheet 模板格式导出 |
| **系统托盘** | pystray + PIL | 118 | 图标绘制 + 右键菜单 + 单实例 |
| **前端 SPA** | React/MUI | ~1000 | 4 页 SPA + 6 组件 |
| **打包** | PyInstaller + Inno | — | .exe 独立运行包 |

### 1.3 当前方案痛点

| 问题 | 影响 |
|------|------|
| Python .exe 体积大 | 安装包 44MB，启动慢 |
| FastAPI + uvicorn 异步开销 | 本地单机场景不需要 |
| PyInstaller 兼容脆弱 | 每次打包 adjust 隐藏导入 |
| 无类型安全 | 运行时才能发现 bugs |
| 前端 SPA 需要构建 | 前端依赖 node_modules |

---

## 二、Rust 重构目标架构

### 2.1 技术选型

| 层次 | 当前 | Rust 方案 | 理由 |
|------|------|-----------|------|
| **Web 框架** | FastAPI | **Axum** | 生态最成熟，tower 中间件体系 |
| **数据库** | sqlite3 | **rusqlite + SQLx 迁移** | 类型安全 SQL，编译期校验 |
| **序列化** | Pydantic | **serde + validator** | 零成本抽象 |
| **Excel 导出** | openpyxl | **rust_xlsxwriter** | 纯 Rust，性能高 |
| **前端** | React + Vite | **Leptos + Tailwind** 或 **保留 React** | 可选 WASM 全栈 / 保留现有前端 |
| **桌面打包** | PyInstaller | **Tauri v2** | WebView 壳，exe 体积小（~3MB） |
| **系统托盘** | pystray | **tray-icon crate** | Tauri 内置支持 |
| **配置/环境** | env vars | **figment / dotenvy** | Rust 惯用配置 |
| **日志** | print | **tracing** | 结构化日志 |

### 2.2 架构图

```
┌───────────────────────────────────────────────────────────┐
│                     Tauri v2 Shell                         │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐ │
│  │ WebView     │  │ Tauri IPC    │  │ System Tray      │ │
│  │ (前端 SPA)  │◄─┤ invoke/event ├─►│ tray-icon crate  │ │
│  └─────────────┘  └──────┬───────┘  └──────────────────┘ │
│                          │                                  │
│  ┌───────────────────────┴──────────────────────────────┐ │
│  │              Rust Backend (Axum embedded)              │ │
│  │                                                        │ │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐            │ │
│  │  │ Router   │  │ Service  │  │ Export   │            │ │
│  │  │ (axum)   │  │ (stats)  │  │ (xlsx)   │            │ │
│  │  └────┬─────┘  └────┬─────┘  └────┬─────┘            │ │
│  │       │              │              │                  │ │
│  │  ┌────┴──────────────┴──────────────┴─────┐           │ │
│  │  │          Repository Layer               │           │ │
│  │  │         (rusqlite connection pool)      │           │ │
│  │  └────────────────────┬───────────────────┘           │ │
│  └───────────────────────┼───────────────────────────────┘ │
│                          │                                  │
│                   ┌──────┴──────┐                           │
│                   │   SQLite    │                           │
│                   │  (WAL 模式)  │                           │
│                   └─────────────┘                           │
└───────────────────────────────────────────────────────────┘
```

---

## 三、详细模块设计

### 3.1 Cargo 依赖

```toml
[package]
name = "workload-tool"
version = "1.0.0"
edition = "2021"

[dependencies]
# Web 框架
axum = "0.8"
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "fs", "trace"] }
tokio = { version = "1", features = ["full"] }

# 数据库
rusqlite = { version = "0.32", features = ["bundled", "vtab"] }
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite"] }

# 序列化
serde = { version = "1", features = ["derive"] }
serde_json = "1"
validator = { version = "0.20", features = ["derive"] }

# Excel 导出
rust_xlsxwriter = "0.80"

# Tauri 桌面
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-shell = "2"

# 工具
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4"] }
tracing = "0.1"
tracing-subscriber = "0.3"
thiserror = "2"
anyhow = "1"

# 日期解析
regex = "1"
```

### 3.2 项目目录结构

```
workload-tool/
├── Cargo.toml
├── tauri.conf.json
├── build_installer.iss
│
├── src/
│   ├── main.rs                    # Tauri 入口 + Axum 启动
│   ├── lib.rs                     # Tauri commands (IPC)
│   ├── config.rs                  # 配置读取
│   ├── error.rs                   # 统一错误类型
│   │
│   ├── db/
│   │   ├── mod.rs
│   │   ├── connection.rs          # SQLite 连接池 (r2d2)
│   │   ├── migrations.rs          # 建表 + 迁移 (嵌入 SQL)
│   │   └── seed.rs                # 种子数据 (8实验室/44项目)
│   │
│   ├── models/
│   │   ├── mod.rs
│   │   ├── group.rs               # ProjectGroup
│   │   ├── project.rs             # ProjectEntity
│   │   ├── record.rs              # WorkRecord (含软删除)
│   │   └── audit.rs               # AuditLog
│   │
│   ├── repo/
│   │   ├── mod.rs
│   │   ├── group_repo.rs          # project_groups CRUD
│   │   ├── project_repo.rs        # projects CRUD
│   │   ├── record_repo.rs         # work_records CRUD + 软删除
│   │   └── audit_repo.rs          # audit_log insert + query
│   │
│   ├── service/
│   │   ├── mod.rs
│   │   ├── group_svc.rs           # 分组业务逻辑（名称唯一校验）
│   │   ├── project_svc.rs         # 项目业务逻辑
│   │   ├── record_svc.rs          # 记录 + 审计日志
│   │   └── stats_svc.rs           # 7 维度统计查询
│   │
│   ├── api/
│   │   ├── mod.rs                 # 路由聚合
│   │   ├── group_handler.rs       # /api/groups
│   │   ├── project_handler.rs     # /api/projects
│   │   ├── record_handler.rs      # /api/records
│   │   ├── stats_handler.rs       # /api/stats/*
│   │   ├── export_handler.rs      # /api/export/excel
│   │   ├── audit_handler.rs       # /api/audit-logs
│   │   └── extractors.rs          # Query/Path 参数提取
│   │
│   ├── export/
│   │   ├── mod.rs
│   │   ├── monthly.rs             # Sheet 1: 月-汇总
│   │   ├── daily.rs               # Sheet 2: 每日工作量
│   │   ├── weekly.rs              # Sheet 3: 每周工作量
│   │   ├── raw_records.rs         # Sheet 4: 原始记录
│   │   └── user_stats.rs          # Sheet 5: 用户统计
│   │
│   └── tray/
│       ├── mod.rs
│       └── icon.rs                # 托盘图标 + 菜单
│
├── frontend/                      # 保留 React 前端（可选）
│   └── (现有代码)
│
└── migrations/
    └── 001_init.sql               # 初始表结构
```

### 3.3 数据库层

```rust
// db/connection.rs — SQLite 连接池
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

pub type DbPool = Pool<SqliteConnectionManager>;

pub fn init_pool(db_path: &str) -> DbPool {
    let manager = SqliteConnectionManager::file(db_path);
    let pool = Pool::builder()
        .max_size(4)  // 单机桌面应用，4 连接足够
        .build(manager)
        .expect("Failed to create DB pool");
    
    // 启用 WAL + 外键
    let conn = pool.get().unwrap();
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;").unwrap();
    pool
}
```

```rust
// db/migrations.rs — 建表 SQL（等价于 database.py 的 init_db）
const CREATE_TABLES: &str = r#"
CREATE TABLE IF NOT EXISTS project_groups (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL UNIQUE,
    sort_order  INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS projects (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    group_id    INTEGER NOT NULL REFERENCES project_groups(id),
    name        TEXT NOT NULL,
    full_name   TEXT DEFAULT '',
    notes       TEXT DEFAULT '',
    sort_order  INTEGER NOT NULL DEFAULT 0,
    is_active   INTEGER NOT NULL DEFAULT 1,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS work_records (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id  INTEGER NOT NULL REFERENCES projects(id),
    user_name   TEXT NOT NULL,
    quantity    INTEGER NOT NULL CHECK(quantity > 0),
    recorded_at TEXT NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at  TEXT
);

CREATE TABLE IF NOT EXISTS audit_log (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    action     TEXT NOT NULL,
    table_name TEXT NOT NULL,
    record_id  INTEGER,
    user_name  TEXT DEFAULT '',
    detail     TEXT DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_records_project ON work_records(project_id);
CREATE INDEX IF NOT EXISTS idx_records_date ON work_records(recorded_at);
CREATE INDEX IF NOT EXISTS idx_records_user ON work_records(user_name);
CREATE INDEX IF NOT EXISTS idx_records_deleted ON work_records(deleted_at);
"#;
```

### 3.4 API 层（Axum Router）

```rust
// api/mod.rs — 路由聚合（等价于 __init__.py）
use axum::{Router, routing::get};
use crate::db::DbPool;

pub fn api_router(pool: DbPool) -> Router {
    Router::new()
        .nest("/api/groups",      group_handler::router(pool.clone()))
        .nest("/api/projects",    project_handler::router(pool.clone()))
        .nest("/api/records",     record_handler::router(pool.clone()))
        .nest("/api/stats",       stats_handler::router(pool.clone()))
        .nest("/api/export",      export_handler::router(pool.clone()))
        .nest("/api/audit-logs",  audit_handler::router(pool))
}
```

```rust
// api/record_handler.rs — 示例：记录 CRUD
use axum::{
    extract::{Path, Query, State},
    Json, Router, routing::{get, post},
};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::db::DbPool;
use crate::error::AppError;
use crate::service::record_svc;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateRecordRequest {
    pub project_id: i64,
    #[validate(length(min = 1, max = 50))]
    pub user_name: String,
    #[validate(range(min = 1))]
    pub quantity: i32,
    pub recorded_at: String,  // ISO 8601
}

#[derive(Debug, Serialize)]
pub struct RecordResponse {
    pub id: i64,
    pub project_id: i64,
    pub project_name: String,
    pub group_name: String,
    pub user_name: String,
    pub quantity: i32,
    pub recorded_at: String,
    pub created_at: String,
}

pub fn router(pool: DbPool) -> Router {
    Router::new()
        .route("/", get(list_records).post(create_record))
        .route("/{id}", axum::routing::put(update_record).delete(soft_delete_record))
        .route("/{id}/restore", post(restore_record))
        .route("/by-user/{user_name}", axum::routing::delete(delete_by_user))
        .with_state(pool)
}

async fn create_record(
    State(pool): State<DbPool>,
    Json(body): Json<CreateRecordRequest>,
) -> Result<Json<ApiResponse<RecordResponse>>, AppError> {
    body.validate()?;
    let result = record_svc::create(&pool, body).await?;
    Ok(Json(ApiResponse { code: 0, message: "ok".into(), data: Some(result) }))
}
```

### 3.5 统计引擎（SQL 聚合）

```rust
// service/stats_svc.rs — 统计查询
// 关键：保留现有 SQL GROUP BY 逻辑，不改为 Rust 内存计算
// SQLite 的聚合性能远优于应用层循环

pub struct StatsSummary {
    pub total_quantity: i64,
    pub total_records: i64,
    pub user_count: i64,
    pub project_count: i64,
    pub breakdown: Vec<PeriodBreakdown>,
}

pub struct PeriodBreakdown {
    pub period: String,       // "2026-06-23" 或 "2026-W25" 或 "2026-06"
    pub total_quantity: i64,
    pub record_count: i64,
}

impl StatsSummary {
    pub fn query(pool: &DbPool, start: Option<&str>, end: Option<&str>,
                 group_by: &str) -> Result<Self> {
        let conn = pool.get()?;
        
        // 1. 汇总统计（单次查询）
        let summary = conn.query_row(
            "SELECT COALESCE(SUM(quantity),0), COUNT(*), COUNT(DISTINCT user_name), COUNT(DISTINCT project_id)
             FROM work_records WHERE deleted_at IS NULL ...",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )?;

        // 2. 周期细分（按 group_by 动态构建 SQL）
        let period_expr = match group_by {
            "week" => "strftime('%Y-W%W', recorded_at)",
            "month" => "strftime('%Y-%m', recorded_at)",
            _ => "date(recorded_at)",
        };
        let breakdown = conn.prepare(&format!(
            "SELECT {} AS period, SUM(quantity), COUNT(*)
             FROM work_records WHERE deleted_at IS NULL ...
             GROUP BY period ORDER BY period", period_expr
        ))?.query_map([], /* map to PeriodBreakdown */)?;

        Ok(StatsSummary { ... })
    }
}
```

### 3.6 Excel 导出（rust_xlsxwriter）

```rust
// export/monthly.rs — 月-汇总（完全对齐 Python 版本）
use rust_xlsxwriter::*;

pub fn build_monthly_summary(
    workbook: &mut Workbook,
    pool: &DbPool,
    start: Option<&str>,
    end: Option<&str>,
    group_id: Option<i64>,
) -> Result<String> {
    let worksheet = workbook.add_worksheet().set_name("月-汇总")?;
    worksheet.set_tab_color(Color::RGB(0x1976D2));

    // 字体
    let font_header = Format::new()
        .set_font_name("仿宋").set_font_size(16).set_bold();
    let font_data = Format::new()
        .set_font_name("仿宋").set_font_size(16);
    let align_center = Format::new().set_align(FormatAlign::Center).set_vertical_align(FormatVerticalAlign::Center);
    let border = Format::new()
        .set_border(FormatBorder::Thin);

    // 表头
    let headers = ["使用实验室", "项目代号", "液相仪器", "检测方法",
                   "月检测数量", "液相检测量", "气相检测量", "项目检测总量"];
    for (ci, h) in headers.iter().enumerate() {
        worksheet.write_with_format(1, (ci + 1) as u16, h, &font_header)?;
    }

    // ... 数据行 + 合并 + SUM 公式

    // 列宽
    worksheet.set_column_width(0, 8.89)?;   // A
    worksheet.set_column_width(1, 24.89)?;  // B
    worksheet.set_column_width(2, 18.0)?;   // C
    worksheet.set_column_width(3, 17.44)?;  // D
    worksheet.set_column_width(4, 43.66)?;  // E
    worksheet.set_column_width(5, 19.66)?;  // F

    Ok(month_label)
}
```

### 3.7 Tauri 入口

```rust
// main.rs — Tauri 启动 Axum 嵌入式服务器
use tauri::Manager;
use axum::Router;
use std::net::SocketAddr;
use tokio::net::TcpListener;

mod config;
mod db;
mod models;
mod repo;
mod service;
mod api;
mod export;
mod error;

#[tokio::main]
async fn main() {
    // 初始化数据库
    let data_dir = get_app_data_dir();
    let db_path = data_dir.join("workload.db");
    let pool = db::connection::init_pool(db_path.to_str().unwrap());
    db::migrations::run(&pool.get().unwrap()).unwrap();
    db::seed::ensure_seeded(&pool.get().unwrap()).unwrap();

    // 启动 Axum
    let app = api::api_router(pool);

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(move |app| {
            let app_handle = app.handle().clone();

            // 设置系统托盘
            tray::setup(&app_handle)?;

            // 在后台启动 HTTP 服务
            tokio::spawn(async move {
                let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
                let listener = TcpListener::bind(addr).await.unwrap();
                axum::serve(listener, app).await.unwrap();
            });

            // 自动打开浏览器
            open::that("http://localhost:8000")?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 3.8 前端方案选择

**方案 A：保留 React SPA（推荐 — 改动最小）**

```
优点：现有前端完全复用，只需改 API base URL
改动：client.ts 中 baseURL 改为 http://127.0.0.1:8000
打包：Vite build → static/ 目录 → Tauri resources
```

**方案 B：Leptos 全栈（长期目标）**

```
优点：纯 Rust 全栈，无 JS 运行时，WASM 体积小
缺点：学习成本高，组件库不如 MUI
成本：完全重写前端（~1000 行 TSX → ~2000 行 Leptos）
```

**建议：第一阶段用方案 A，第二版迭代时考虑方案 B。**

---

## 四、数据库迁移策略

### 4.1 旧数据兼容

```rust
// db/migrations.rs
// 现有 SQLite 文件直接读取，无需迁移
// 表结构完全一致：
//   project_groups / projects / work_records / audit_log
// 唯一差异：Python 用 TEXT 存日期，Rust 继续用 TEXT（chrono::NaiveDateTime）
```

### 4.2 种子数据嵌入

```rust
// db/seed.rs
const DEFAULT_DATA: &[(&str, &[&str])] = &[
    ("410实验室", &["HYLY-LC-01(230106)", "HYLY-LC-04(QL-230211)", ...]),
    ("415实验室", &["YSLY-LC-12(T004-220909)", ...]),
    // ... 同 database.py 的 DEFAULT_DATA
];
```

### 4.3 95 条方法全称映射

```rust
// models/project.rs
use std::collections::HashMap;
use once_cell::sync::Lazy;

static METHOD_FULL_NAMES: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    HashMap::from([
        ("HYLY-LC-01(230106)", "HYLY-230106-1-低温8℃-DAD"),
        ("QL-LC-04(230211)", "QL-260211-DAD"),
        // ... 95 条
    ])
});

pub fn get_method_full_name(group: &str, project: &str) -> Option<&str> {
    METHOD_FULL_NAMES.get(project).copied()
}
```

---

## 五、对比评估

| 维度 | Python 方案 | Rust 方案 | 提升 |
|------|------------|-----------|------|
| **安装包大小** | 44 MB | ~8 MB (Tauri 3MB + 5MB 资源) | **5x 减小** |
| **启动速度** | 3-5s (Python 解压) | <500ms | **6-10x 加速** |
| **内存占用** | ~80 MB | ~30 MB | **2.5x 降低** |
| **API 响应** | ~2ms (FastAPI) | ~0.3ms (Axum) | **6x 加速** |
| **类型安全** | 运行时 | 编译期 | **零运行时错误** |
| **打包复杂度** | 需调整 hiddenimports | cargo tauri build 一键 | **大幅简化** |
| **前端复用** | — | 保留 React SPA | **零前端改动** |

---

## 六、实施路线图

### Phase 1：核心后端（2-3 天）
- [ ] 初始化 Cargo workspace + Tauri 项目骨架
- [ ] 数据库层（rusqlite + 建表 + 种子数据）
- [ ] models + repo 层（CRUD）
- [ ] API 路由（6 个 handler）
- [ ] 统计引擎（7 个查询）

### Phase 2：导出 + 托盘（1-2 天）
- [ ] Excel 导出（5 个 Sheet，完全对齐 v4 模板）
- [ ] 系统托盘集成
- [ ] 单实例检测

### Phase 3：前端集成 + 打包（1 天）
- [ ] 集成现有 React SPA（static 文件 serve）
- [ ] Tauri 配置 + 图标
- [ ] Inno Setup 安装包
- [ ] 端到端测试

### Phase 4：优化（可选）
- [ ] Leptos 前端重写
- [ ] 自动更新（tauri-plugin-updater）
- [ ] 备份/恢复

**总计工时估量：4-6 天（单人全职）**

---

## 七、风险与对策

| 风险 | 对策 |
|------|------|
| rust_xlsxwriter 不支持仿宋字体 | 字体名写死为 "仿宋"，由系统字体提供（与 openpyxl 相同） |
| 合并单元格 API 差异 | rust_xlsxwriter 有 `merge_range()`，功能对齐 |
| Tauri WebView 兼容性 | 指定 Edge WebView2（Win10+ 内置） |
| 旧数据库兼容 | 表结构不变，直接复用 .db 文件 |
| 学习曲线 | 先保留 React 前端，只替换后端 |
