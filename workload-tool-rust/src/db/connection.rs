use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

pub type DbPool = Pool<SqliteConnectionManager>;

pub fn init_pool(db_path: &str) -> DbPool {
    let manager = SqliteConnectionManager::file(db_path);
    Pool::builder()
        .max_size(4)
        .build(manager)
        .expect("Failed to create DB pool")
}
