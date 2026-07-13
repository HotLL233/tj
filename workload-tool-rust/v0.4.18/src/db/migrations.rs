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
            show_in_work INTEGER NOT NULL DEFAULT 1,
            show_in_rd INTEGER NOT NULL DEFAULT 1,
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

    // v0.2.19: methods 表增加 amount 字段
    conn.execute("ALTER TABLE methods ADD COLUMN amount REAL NOT NULL DEFAULT 0.0", []).ok();

    // v0.3.0: 导入映射配置表
    conn.execute_batch("CREATE TABLE IF NOT EXISTS import_mappings (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        header_pattern TEXT NOT NULL,
        match_mode TEXT NOT NULL DEFAULT 'contains',
        target_table TEXT NOT NULL,
        default_type TEXT DEFAULT '',
        priority INTEGER NOT NULL DEFAULT 100,
        is_active INTEGER NOT NULL DEFAULT 1,
        created_at TEXT NOT NULL DEFAULT (datetime('now'))
    )").ok();

    // 种子数据：列头→目标表映射
    conn.execute_batch("INSERT OR IGNORE INTO import_mappings (header_pattern, target_table, default_type, priority) VALUES
    ('*实验室*','project_groups','',10),
    ('*研发*','projects','',20),
    ('*项目*','projects','',21),
    ('*液相*','methods','液相',30),
    ('*气相*','methods','气相',31),
    ('*理化*','methods','理化',32),
    ('*ICP*','methods','ICP',33),
    ('*热分析*','methods','热分析',34),
    ('*质谱*','methods','质谱',35),
    ('*方法*','methods','其他',90),
    ('*','methods','其他',999)").ok();

    // v0.3.7: work_records 增加 method_id 精确关联
    conn.execute("ALTER TABLE work_records ADD COLUMN method_id INTEGER", []).ok();
    // 对已有记录：尝试用项目关联的第一个方法回填
    conn.execute(
        "UPDATE work_records SET method_id = (
            SELECT pml.method_id FROM project_method_links pml
            WHERE pml.project_id = work_records.project_id
            LIMIT 1
        ) WHERE method_id IS NULL",
        [],
    ).ok();
    conn.execute("CREATE INDEX IF NOT EXISTS idx_records_method ON work_records(method_id)", []).ok();

    // v0.3.7: 清理研发项目伪分组关联的 lab links
    conn.execute(
        "DELETE FROM project_lab_links WHERE group_id IN (SELECT id FROM project_groups WHERE name='研发项目')",
        [],
    ).ok();

    // v0.3.24: work_records 增加 group_id 保存录入时的实验室上下文
    conn.execute("ALTER TABLE work_records ADD COLUMN group_id INTEGER REFERENCES project_groups(id)", []).ok();
    // 注意：已有记录的 group_id 设为 NULL（无法可靠推断录入时的实验室）
    // 查询时若 wr.group_id IS NULL，回退到 project_lab_links 逻辑

    // ═══════════════════════════════════════════════════════════
    // v0.4.0: 研发送样录入数据 — 与分析检测 work_records 完全独立存储
    // 结构镜像 work_records；主数据(projects/groups/methods)两个模块共用
    // ═══════════════════════════════════════════════════════════
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS rd_work_records (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER NOT NULL REFERENCES projects(id),
            method_id INTEGER,
            user_name TEXT NOT NULL,
            quantity INTEGER NOT NULL CHECK(quantity > 0),
            recorded_at TEXT NOT NULL,
            group_id INTEGER REFERENCES project_groups(id),
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            deleted_at TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_rd_records_project ON rd_work_records(project_id);
        CREATE INDEX IF NOT EXISTS idx_rd_records_method ON rd_work_records(method_id);
        CREATE INDEX IF NOT EXISTS idx_rd_records_date ON rd_work_records(recorded_at);
        CREATE INDEX IF NOT EXISTS idx_rd_records_user ON rd_work_records(user_name);
        CREATE INDEX IF NOT EXISTS idx_rd_records_deleted ON rd_work_records(deleted_at);"
    ).ok();

    // v0.4.0: 审计日志 module 隔离（work/rd/shared）
    // 旧记录无 module 列 → 默认 'shared'，两个模块审计页都能看到
    conn.execute("ALTER TABLE audit_log ADD COLUMN module TEXT NOT NULL DEFAULT 'shared'", []).ok();

    // v0.4.3: 研发送样状态+取样人
    conn.execute("ALTER TABLE rd_work_records ADD COLUMN status TEXT NOT NULL DEFAULT '待取样'", []).ok();
    conn.execute("ALTER TABLE rd_work_records ADD COLUMN sampler TEXT", []).ok();
    conn.execute("ALTER TABLE rd_work_records ADD COLUMN sampled_at TEXT", []).ok();

    // v0.4.6: 单价倍率字段
    conn.execute("ALTER TABLE methods ADD COLUMN multiplier REAL NOT NULL DEFAULT 1.0", []).ok();
    conn.execute("ALTER TABLE work_records ADD COLUMN multiplier REAL NOT NULL DEFAULT 1.0", []).ok();

    // v0.4.10: project_groups 增加模块关联字段
    conn.execute("ALTER TABLE project_groups ADD COLUMN show_in_work INTEGER NOT NULL DEFAULT 1", []).ok();
    conn.execute("ALTER TABLE project_groups ADD COLUMN show_in_rd INTEGER NOT NULL DEFAULT 1", []).ok();

    // v0.4.11: 帮助文档管理
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS help_documents (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            filename TEXT NOT NULL,
            file_path TEXT NOT NULL,
            file_type TEXT NOT NULL,
            file_size INTEGER DEFAULT 0,
            is_visible INTEGER DEFAULT 1,
            sort_order INTEGER DEFAULT 0,
            created_at TEXT DEFAULT (datetime('now','localtime')),
            updated_at TEXT DEFAULT (datetime('now','localtime'))
        );"
    ).ok();

    // v0.4.18: 帮助文档 PDF → PNG page count
    conn.execute("ALTER TABLE help_documents ADD COLUMN page_count INTEGER DEFAULT 0", []).ok();

    Ok(())
}
