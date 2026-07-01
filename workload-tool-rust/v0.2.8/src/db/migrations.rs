use crate::error::Result;

pub fn run(conn: &rusqlite::Connection) -> Result<()> {
    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA foreign_keys=ON;"
    )?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS project_groups (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE TABLE IF NOT EXISTS projects (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            group_id INTEGER NOT NULL REFERENCES project_groups(id),
            name TEXT NOT NULL,
            full_name TEXT DEFAULT '',
            notes TEXT DEFAULT '',
            sort_order INTEGER NOT NULL DEFAULT 0,
            is_active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE TABLE IF NOT EXISTS work_records (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER NOT NULL REFERENCES projects(id),
            user_name TEXT NOT NULL,
            quantity INTEGER NOT NULL CHECK(quantity > 0),
            recorded_at TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            deleted_at TEXT
        );
        CREATE TABLE IF NOT EXISTS audit_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            action TEXT NOT NULL,
            table_name TEXT NOT NULL,
            record_id INTEGER,
            user_name TEXT DEFAULT '',
            detail TEXT DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_records_project ON work_records(project_id);
        CREATE INDEX IF NOT EXISTS idx_records_date ON work_records(recorded_at);
        CREATE INDEX IF NOT EXISTS idx_records_user ON work_records(user_name);
        CREATE INDEX IF NOT EXISTS idx_records_deleted ON work_records(deleted_at);
        CREATE TABLE IF NOT EXISTS sample_records (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER NOT NULL REFERENCES projects(id),
            group_id INTEGER NOT NULL REFERENCES project_groups(id),
            user_name TEXT NOT NULL,
            sample_name TEXT NOT NULL,
            sample_count INTEGER NOT NULL DEFAULT 1 CHECK(sample_count > 0),
            unit TEXT NOT NULL DEFAULT '个',
            batch_no TEXT DEFAULT '',
            notes TEXT DEFAULT '',
            submitted_at TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now','localtime')),
            deleted_at TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_sample_project ON sample_records(project_id);
        CREATE INDEX IF NOT EXISTS idx_sample_group ON sample_records(group_id);
        CREATE INDEX IF NOT EXISTS idx_sample_user ON sample_records(user_name);
        CREATE INDEX IF NOT EXISTS idx_sample_date ON sample_records(submitted_at);
        CREATE INDEX IF NOT EXISTS idx_sample_deleted ON sample_records(deleted_at);"
    )?;

    // Import module support
    conn.execute("ALTER TABLE work_records ADD COLUMN batch_no TEXT DEFAULT ''", []).ok();
    conn.execute("ALTER TABLE work_records ADD COLUMN extra_info TEXT DEFAULT ''", []).ok();
    // Ensure UNIQUE constraint for upsert_project to work
    conn.execute("CREATE UNIQUE INDEX IF NOT EXISTS idx_projects_name_group ON projects(name, group_id)", []).ok();
    // v0.2.2: 项目管理系数
    conn.execute("ALTER TABLE projects ADD COLUMN coefficient REAL NOT NULL DEFAULT 1.0", []).ok();
    conn.execute("ALTER TABLE projects ADD COLUMN method_type TEXT NOT NULL DEFAULT '其他'", []).ok();
    conn.execute_batch("UPDATE projects SET method_type = CASE WHEN name LIKE '%LC-%' THEN '液相' WHEN name LIKE '%GC-%' THEN '气相' WHEN name LIKE '%理化%' THEN '理化' ELSE '其他' END WHERE method_type='其他'").ok();
    // v0.2.8: 方法类型表
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS method_types (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL UNIQUE, sort_order INTEGER DEFAULT 0);\
         INSERT OR IGNORE INTO method_types(name,sort_order) VALUES('检测类型',0),('液相',1),('气相',2),('理化',3),('其他',99);"
    ).ok();

    Ok(())
}
