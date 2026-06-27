// 打包模式：隐藏控制台窗口（cargo build --release）
// 开发模式：cargo run --features console 保留控制台
#![cfg_attr(not(feature = "console"), windows_subsystem = "windows")]

mod db;
mod models;
mod repo;
mod api;
mod error;
mod tray;

use axum::http::header;
use axum::response::IntoResponse;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

const PORT: u16 = 8000;

fn get_data_dir() -> std::path::PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."))
}

#[tokio::main]
async fn main() {
    // 单实例检测
    if tray::is_port_in_use(PORT) {
        #[cfg(not(feature = "console"))]
        {
            // 托盘模式：已有实例运行，打开浏览器后退出
            open::that(format!("http://localhost:{}", PORT)).ok();
            return;
        }
        #[cfg(feature = "console")]
        {
            println!("已有实例在运行 → http://localhost:{}", PORT);
            open::that(format!("http://localhost:{}", PORT)).ok();
            return;
        }
    }

    // 初始化数据库（exe 同目录下的 data/ 文件夹）
    let data_dir = get_data_dir().join("data");
    std::fs::create_dir_all(&data_dir).ok();
    let db_path = data_dir.join("workload.db");
    let pool = db::init_pool(db_path.to_str().unwrap());
    {
        let conn = pool.get().expect("DB connection failed");
        db::migrations::run(&conn).expect("Migration failed");
        db::seed::ensure_seeded(&conn).expect("Seed failed");
    }

    // 构建路由 — API 优先，静态文件 + SPA fallback
    async fn serve_index() -> impl axum::response::IntoResponse {
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

    let addr = SocketAddr::from(([0, 0, 0, 0], PORT));

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
        tray::run_tray(PORT);
    }
    #[cfg(feature = "console")]
    {
        // 开发模式：保留控制台
        println!("🚀 工作量统计工具 v{} (Rust) — http://{}", env!("CARGO_PKG_VERSION"), addr);
        println!("按 Ctrl+C 退出");
        _server.await.unwrap();
    }
}
