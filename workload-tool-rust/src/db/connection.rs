use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

pub type DbPool = Pool<SqliteConnectionManager>;

pub fn init_pool(db_path: &str) -> DbPool {
    let manager = SqliteConnectionManager::file(db_path)
        .with_init(|conn| {
            conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON; PRAGMA busy_timeout=5000;")
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
        });
    Pool::builder()
        .max_size(4)
        .build(manager)
        .expect("Failed to create DB pool")
}
