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
         INSERT OR IGNORE INTO method_types(name,sort_order) VALUES('检测类型',0),('液相',1),('气相',2),('理化',3),('ICP',4),('热分析',5),('质谱',6),('其他',99);"
    ).ok();

    // v0.2.16: 项目多选关联
    conn.execute("ALTER TABLE projects ADD COLUMN associated_lab_ids TEXT NOT NULL DEFAULT ''", []).ok();
    conn.execute("ALTER TABLE projects ADD COLUMN associated_method_ids TEXT NOT NULL DEFAULT ''", []).ok();

    // ═══════════════════════════════════════════════════════════
    // v0.2.17: 卡片独立分离 — methods 表 + 3张关联表
    // ═══════════════════════════════════════════════════════════

    // 新建 methods 表
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS methods (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            full_name TEXT DEFAULT '',
            coefficient REAL NOT NULL DEFAULT 1.0,
            notes TEXT DEFAULT '',
            is_active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );"
    ).ok();

    // 3张关联表
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS project_lab_links (
            project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            group_id INTEGER NOT NULL REFERENCES project_groups(id) ON DELETE CASCADE,
            PRIMARY KEY (project_id, group_id)
        );"
    ).ok();
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS project_method_links (
            project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            method_id INTEGER NOT NULL REFERENCES methods(id) ON DELETE CASCADE,
            PRIMARY KEY (project_id, method_id)
        );"
    ).ok();
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS method_type_links (
            method_id INTEGER NOT NULL REFERENCES methods(id) ON DELETE CASCADE,
            method_type_id INTEGER NOT NULL REFERENCES method_types(id) ON DELETE CASCADE,
            PRIMARY KEY (method_id, method_type_id)
        );"
    ).ok();

    // 数据迁移：旧 projects 表中 method_type!='研发项目' 且 !='实验室管理' 的迁移到 methods
    conn.execute_batch(
        "INSERT OR IGNORE INTO methods (name, full_name, coefficient, notes, created_at)
            SELECT p.name, COALESCE(p.full_name,''), COALESCE(p.coefficient,1.0), COALESCE(p.notes,''), p.created_at
            FROM projects p WHERE p.method_type != '研发项目' AND p.method_type != '实验室管理'
              AND NOT EXISTS (SELECT 1 FROM methods m WHERE m.name = p.name);"
    ).ok();

    // 迁移 method_type → method_type_links
    conn.execute_batch(
        "INSERT OR IGNORE INTO method_type_links (method_id, method_type_id)
            SELECT m.id, mt.id FROM methods m
            JOIN projects p ON p.name = m.name AND p.method_type NOT IN ('研发项目','实验室管理')
            JOIN method_types mt ON mt.name = p.method_type AND mt.name != '研发项目'
            WHERE m.id IS NOT NULL;"
    ).ok();

    // 迁移 v0.2.16 associated_lab_ids → project_lab_links (仅研发项目)
    // 简单处理：对每个研发项目，将其 group_id 作为 lab link
    conn.execute_batch(
        "INSERT OR IGNORE INTO project_lab_links (project_id, group_id)
            SELECT id, group_id FROM projects
            WHERE method_type = '研发项目' OR method_type = '实验室管理';"
    ).ok();

    // 删除 projects 表中 method_type!='研发项目' 且 !='实验室管理' 的旧记录
    conn.execute_batch(
        "DELETE FROM projects WHERE method_type != '研发项目' AND method_type != '实验室管理';"
    ).ok();

    Ok(())
}
