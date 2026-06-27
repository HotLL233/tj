// 打包模式：隐藏控制台窗口（cargo build --release）
// 开发模式：cargo run --features console 保留控制台
#![cfg_attr(not(feature = "console"), windows_subsystem = "windows")]

mod config;
mod db;
mod models;
mod repo;
mod api;
mod audit;
mod service;
mod error;
mod tray;

use axum::http::header;
use axum::response::IntoResponse;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

fn get_data_dir() -> std::path::PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."))
}

#[tokio::main]
async fn main() {
    // 加载配置
    let app_config = config::AppConfig::load();

    // 初始化 tracing（日志）
    #[cfg(not(feature = "console"))]
    {
        use tracing_subscriber::fmt;
        let level: tracing::Level = app_config.log_level.parse().unwrap_or(tracing::Level::INFO);
        if let Some(ref log_file) = app_config.log_file {
            let file_appender = tracing_appender::rolling::never(
                get_data_dir(), log_file,
            );
            fmt().with_max_level(level)
                .with_writer(file_appender)
                .with_target(false)
                .init();
        } else {
            fmt().with_max_level(level).with_target(false).init();
        }
    }
    #[cfg(feature = "console")]
    {
        use tracing_subscriber::fmt;
        let level: tracing::Level = app_config.log_level.parse().unwrap_or(tracing::Level::INFO);
        fmt().with_max_level(level).init();
    }

    let port = app_config.server_port;

    // 单实例检测
    if tray::is_port_in_use(port) {
        tracing::info!("已有实例运行在端口 {}", port);
        open::that(format!("http://localhost:{}", port)).ok();
        return;
    }

    // 初始化数据库（从配置读取路径）
    let db_path = app_config.db_path();
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    tracing::info!("数据库路径: {}", db_path.display());

    let pool = db::init_pool(db_path.to_str().unwrap_or("data/workload.db"));
    {
        let conn = pool.get().expect("DB connection failed");
        db::migrations::run(&conn).expect("Migration failed");
        db::seed::ensure_seeded(&conn).expect("Seed failed");
    }
    tracing::info!("数据库初始化完成");

    // 构建路由 — API 优先，静态文件 + SPA fallback
    async fn serve_index() -> impl IntoResponse {
        match tokio::fs::read_to_string("static/index.html").await {
            Ok(html) => (
                [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
                html,
            ).into_response(),
            Err(_) => (axum::http::StatusCode::NOT_FOUND, "index.html not found").into_response(),
        }
    }

    let app = api::api_router(pool)
        .nest_service("/assets", ServeDir::new("static/assets"))
        .fallback(serve_index)
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("启动服务器 → http://{}", addr);

    // 启动服务器（后台线程）
    let _server = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });

    // 短暂等待服务器就绪
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    #[cfg(not(feature = "console"))]
    {
        // 生产模式：系统托盘
        tray::run_tray(port);
    }
    #[cfg(feature = "console")]
    {
        // 开发模式：保留控制台
        println!("🚀 工作量统计工具 v{} (Rust) — http://{}", env!("CARGO_PKG_VERSION"), addr);
        println!("按 Ctrl+C 退出");
        _server.await.unwrap();
    }
}
