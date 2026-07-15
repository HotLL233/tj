// 打包模式：隐藏控制台窗口（cargo build --release）
// 开发模式：cargo run --features console 保留控制台
#![cfg_attr(all(not(feature = "console"), target_os = "windows"), windows_subsystem = "windows")]

use workload_tool::{config, db, api, repo};
use axum::http::header;

// tray module kept in binary crate (not library) — Windows only
#[cfg(target_os = "windows")]
mod tray;
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

/// 单实例检测：检查端口是否已被占用
pub fn is_port_in_use(port: u16) -> bool {
    std::net::TcpListener::bind(("127.0.0.1", port)).is_err()
}

#[tokio::main]
async fn main() {
    let app_config = config::AppConfig::load();

    #[cfg(not(feature = "console"))]
    {
        use tracing_subscriber::fmt;
        let level: tracing::Level = app_config.log_level.parse().unwrap_or(tracing::Level::INFO);
        if let Some(ref log_file) = app_config.log_file {
            let file_appender = tracing_appender::rolling::never(get_data_dir(), log_file);
            fmt().with_max_level(level).with_writer(file_appender).with_target(false).init();
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
    if is_port_in_use(port) {
        tracing::info!("已有实例运行在端口 {}", port);
        open::that(format!("http://localhost:{}", port)).ok();
        return;
    }

    let db_path = app_config.db_path();
    if let Some(parent) = db_path.parent() { std::fs::create_dir_all(parent).ok(); }
    tracing::info!("数据库路径: {}", db_path.display());

    let pool = db::init_pool(db_path.to_str().unwrap_or("data/workload.db"));
    {
        let conn = pool.get().expect("DB connection failed");
        db::migrations::run(&conn).expect("Migration failed");
        db::seed::ensure_seeded(&conn).expect("Seed failed");
        // v0.3.9: 修复 methods 表中 name/full_name 为空的脏数据
        conn.execute(
            "UPDATE methods SET name=id||'_auto', full_name=name WHERE (name IS NULL OR name='') AND id>0",
            [],
        ).ok();
    }
    tracing::info!("数据库初始化完成");

    async fn serve_index() -> impl IntoResponse {
        match tokio::fs::read_to_string("static/index.html").await {
            Ok(html) => ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], html).into_response(),
            Err(_) => (axum::http::StatusCode::NOT_FOUND, "index.html not found").into_response(),
        }
    }

    let config_arc = std::sync::Arc::new(app_config);
    let config_clone = config_arc.clone();
    let app = api::api_router(pool, config_arc)
        .nest_service("/assets", ServeDir::new("static/assets"))
        .fallback(serve_index)
        .layer(CorsLayer::permissive());

    if config_clone.backup_enabled {
        let bk_cfg = config_clone.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(bk_cfg.backup_interval_hours.max(1) * 3600)).await;
                let db = bk_cfg.db_path().to_string_lossy().to_string();
                let dir = bk_cfg.backup_dir();
                let max_count = bk_cfg.max_backup_count;
                let ts = chrono::Local::now().format("%Y%m%d_%H%M%S");
                let dst = dir.join(format!("workload_auto_{}.db", ts));
                let _ = std::fs::create_dir_all(&dir);
                let db_clone = db.clone();
                let backup_result = tokio::task::spawn_blocking(move || {
                    rusqlite::Connection::open(&db_clone).and_then(|c| c.execute_batch(&format!("VACUUM INTO '{}'", dst.to_string_lossy().replace('\'', "''"))))
                }).await;
                match backup_result {
                    Ok(Ok(_)) => {
                        tracing::info!("Auto-backup completed");
                        // 清理旧备份
                        if let Ok(es) = std::fs::read_dir(&dir) {
                            let mut files: Vec<_> = es.flatten().filter_map(|e| {
                                let name = e.file_name().into_string().ok()?;
                                if !name.ends_with(".db") { return None; }
                                let time = e.metadata().ok()?.modified().ok()?;
                                Some((name, time))
                            }).collect();
                            if files.len() as u64 > max_count && max_count > 0 {
                                files.sort_by(|a,b| b.1.cmp(&a.1));
                                for (name, _) in files.iter().skip(max_count as usize) {
                                    let _ = std::fs::remove_file(dir.join(name));
                                }
                            }
                        }
                        // 记录审计日志
                        let _ = repo::audit_repo::log_for_backup("backup", "自动备份完成");
                    }
                    Err(e) => tracing::error!("Auto-backup failed: {}", e),
                    _ => tracing::error!("Auto-backup: VACUUM failed"),
                }
            }
        });
    }

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("启动服务器 → http://{}", addr);

    let _server = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    #[cfg(all(not(feature = "console"), target_os = "windows"))] { tray::run_tray(port); }
    #[cfg(feature = "console")] {
        println!("🚀 样品管理系统 v{} (Rust) — http://{}", env!("CARGO_PKG_VERSION"), addr);
        println!("按 Ctrl+C 退出");
        _server.await.unwrap();
    }
}
