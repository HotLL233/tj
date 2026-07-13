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
    conn.execute("CREATE UNIQUE INDEX IF NOT EXISTS idx_projects_name_group ON projects(name, group_id)", []).ok();
    conn.execute("ALTER TABLE projects ADD COLUMN coefficient REAL NOT NULL DEFAULT 1.0", []).ok();
    conn.execute("ALTER TABLE projects ADD COLUMN method_type TEXT NOT NULL DEFAULT '其他'", []).ok();
    conn.execute_batch("UPDATE projects SET method_type = CASE WHEN name LIKE '%LC-%' THEN '液相' WHEN name LIKE '%GC-%' THEN '气相' WHEN name LIKE '%理化%' THEN '理化' ELSE '其他' END WHERE method_type='其他'").ok();
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS method_types (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL UNIQUE, sort_order INTEGER DEFAULT 0);\
         INSERT OR IGNORE INTO method_types(name,sort_order) VALUES('检测类型',0),('液相',1),('气相',2),('理化',3),('ICP',4),('热分析',5),('质谱',6),('其他',99);"
    ).ok();
    conn.execute("ALTER TABLE projects ADD COLUMN associated_lab_ids TEXT NOT NULL DEFAULT ''", []).ok();
    conn.execute("ALTER TABLE projects ADD COLUMN associated_method_ids TEXT NOT NULL DEFAULT ''", []).ok();

    // v0.2.17: 卡片独立分离 — methods 表 + 3张关联表
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
    conn.execute_batch(
        "INSERT OR IGNORE INTO methods (name, full_name, coefficient, notes, created_at)
            SELECT p.name, COALESCE(p.full_name,''), COALESCE(p.coefficient,1.0), COALESCE(p.notes,''), p.created_at
            FROM projects p WHERE p.method_type != '研发项目' AND p.method_type != '实验室管理'
              AND NOT EXISTS (SELECT 1 FROM methods m WHERE m.name = p.name);"
    ).ok();
    conn.execute_batch(
        "INSERT OR IGNORE INTO method_type_links (method_id, method_type_id)
            SELECT m.id, mt.id FROM methods m
            JOIN projects p ON p.name = m.name AND p.method_type NOT IN ('研发项目','实验室管理')
            JOIN method_types mt ON mt.name = p.method_type AND mt.name != '研发项目'
            WHERE m.id IS NOT NULL;"
    ).ok();
    conn.execute_batch(
        "INSERT OR IGNORE INTO project_lab_links (project_id, group_id)
            SELECT id, group_id FROM projects
            WHERE method_type = '研发项目' OR method_type = '实验室管理';"
    ).ok();
    conn.execute_batch(
        "DELETE FROM projects WHERE method_type != '研发项目' AND method_type != '实验室管理';"
    ).ok();
    conn.execute("ALTER TABLE methods ADD COLUMN amount REAL NOT NULL DEFAULT 0.0", []).ok();
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
    conn.execute("ALTER TABLE work_records ADD COLUMN method_id INTEGER", []).ok();
    conn.execute(
        "UPDATE work_records SET method_id = (
            SELECT pml.method_id FROM project_method_links pml
            WHERE pml.project_id = work_records.project_id
            LIMIT 1
        ) WHERE method_id IS NULL",
        [],
    ).ok();
    conn.execute("CREATE INDEX IF NOT EXISTS idx_records_method ON work_records(method_id)", []).ok();
    conn.execute(
        "DELETE FROM project_lab_links WHERE group_id IN (SELECT id FROM project_groups WHERE name='研发项目')",
        [],
    ).ok();
    conn.execute("ALTER TABLE work_records ADD COLUMN group_id INTEGER REFERENCES project_groups(id)", []).ok();

    // v0.4.0: 研发送样录入数据
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

    // v0.4.0: 审计日志 module 隔离
    conn.execute("ALTER TABLE audit_log ADD COLUMN module TEXT NOT NULL DEFAULT 'shared'", []).ok();

    // v0.4.3: 研发送样状态+取样人
    conn.execute("ALTER TABLE rd_work_records ADD COLUMN status TEXT NOT NULL DEFAULT '待取样'", []).ok();
    conn.execute("ALTER TABLE rd_work_records ADD COLUMN sampler TEXT", []).ok();
    conn.execute("ALTER TABLE rd_work_records ADD COLUMN sampled_at TEXT", []).ok();

    // v0.4.6: 单价倍率字段
    conn.execute("ALTER TABLE methods ADD COLUMN multiplier REAL NOT NULL DEFAULT 1.0", []).ok();
    conn.execute("ALTER TABLE work_records ADD COLUMN multiplier REAL NOT NULL DEFAULT 1.0", []).ok();

    // v0.4.10: project_groups 模块关联字段
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
    conn.execute("ALTER TABLE help_documents ADD COLUMN page_count INTEGER DEFAULT 0", []).ok();

    // v0.4.19: 结构化帮助文章
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS help_articles (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            content_html TEXT NOT NULL DEFAULT '',
            toc_json TEXT,
            source_file TEXT,
            is_visible INTEGER DEFAULT 1,
            sort_order INTEGER DEFAULT 0,
            created_at TEXT DEFAULT (datetime('now','localtime')),
            updated_at TEXT DEFAULT (datetime('now','localtime'))
        );"
    ).ok();

    // ═══════════════════════════════════════════════════════════
    // v0.4.21: RBAC + 运营模块 + 审计 WORM
    // ═══════════════════════════════════════════════════════════

    // ---- RBAC ----
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS roles (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            name        TEXT NOT NULL UNIQUE,
            description TEXT DEFAULT '',
            is_system   INTEGER NOT NULL DEFAULT 0,
            sort_order  INTEGER DEFAULT 0
        );
        CREATE TABLE IF NOT EXISTS role_permissions (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            role_id    INTEGER NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
            permission TEXT NOT NULL,
            UNIQUE(role_id, permission)
        );
        CREATE TABLE IF NOT EXISTS users (
            id                   INTEGER PRIMARY KEY AUTOINCREMENT,
            username             TEXT NOT NULL UNIQUE,
            display_name         TEXT NOT NULL DEFAULT '',
            password_hash        TEXT NOT NULL,
            role_id              INTEGER NOT NULL REFERENCES roles(id),
            lab_id               INTEGER,
            must_change_password INTEGER NOT NULL DEFAULT 0,
            is_active            INTEGER NOT NULL DEFAULT 1,
            created_at           TEXT NOT NULL DEFAULT (datetime('now','localtime')),
            updated_at           TEXT NOT NULL DEFAULT (datetime('now','localtime'))
        );
        CREATE INDEX IF NOT EXISTS idx_users_role ON users(role_id);"
    ).ok();

    // ---- 仪器管理 ----
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS instruments (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            name        TEXT NOT NULL,
            model       TEXT DEFAULT '',
            location    TEXT DEFAULT '',
            manager     TEXT DEFAULT '',
            status      TEXT NOT NULL DEFAULT '正常',
            photo_path  TEXT DEFAULT '',
            qr_code_path TEXT DEFAULT '',
            notes       TEXT DEFAULT '',
            created_by  TEXT DEFAULT '',
            created_at  TEXT NOT NULL DEFAULT (datetime('now','localtime')),
            deleted_at  TEXT
        );
        CREATE TABLE IF NOT EXISTS instrument_bookings (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            instrument_id INTEGER NOT NULL REFERENCES instruments(id),
            applicant   TEXT NOT NULL,
            start_time  TEXT NOT NULL,
            end_time    TEXT NOT NULL,
            purpose     TEXT DEFAULT '',
            status      TEXT NOT NULL DEFAULT '待审批',
            approver    TEXT,
            approved_at TEXT,
            approver_note TEXT DEFAULT '',
            created_at  TEXT NOT NULL DEFAULT (datetime('now','localtime')),
            deleted_at  TEXT
        );
        CREATE TABLE IF NOT EXISTS instrument_maintenances (
            id            INTEGER PRIMARY KEY AUTOINCREMENT,
            instrument_id INTEGER NOT NULL REFERENCES instruments(id),
            maintainer    TEXT NOT NULL,
            maintained_at TEXT NOT NULL,
            content       TEXT DEFAULT '',
            cost          REAL DEFAULT 0,
            created_at    TEXT NOT NULL DEFAULT (datetime('now','localtime')),
            deleted_at  TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_inst_book_inst ON instrument_bookings(instrument_id);
        CREATE INDEX IF NOT EXISTS idx_inst_book_status ON instrument_bookings(status);"
    ).ok();

    // ---- 库存管理 ----
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS inventory_categories (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            name       TEXT NOT NULL,
            parent_id  INTEGER REFERENCES inventory_categories(id),
            sort_order INTEGER DEFAULT 0
        );
        CREATE TABLE IF NOT EXISTS inventory_items (
            id                   INTEGER PRIMARY KEY AUTOINCREMENT,
            name                 TEXT NOT NULL,
            brand                TEXT DEFAULT '',
            unit                 TEXT NOT NULL DEFAULT '个',
            category_id          INTEGER REFERENCES inventory_categories(id),
            tags                 TEXT DEFAULT '',
            location             TEXT DEFAULT '',
            spec                 TEXT DEFAULT '',
            safety_stock         REAL DEFAULT 0,
            expiry_threshold_days INTEGER DEFAULT 0,
            created_by           TEXT DEFAULT '',
            created_at           TEXT NOT NULL DEFAULT (datetime('now','localtime')),
            deleted_at           TEXT
        );
        CREATE TABLE IF NOT EXISTS inventory_batches (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            item_id      INTEGER NOT NULL REFERENCES inventory_items(id),
            batch_no     TEXT DEFAULT '',
            quantity     REAL NOT NULL DEFAULT 0,
            unit_price   REAL DEFAULT 0,
            produced_at  TEXT,
            expiry_date  TEXT,
            source_type  TEXT DEFAULT '',
            source_id    INTEGER,
            created_at   TEXT NOT NULL DEFAULT (datetime('now','localtime')),
            deleted_at   TEXT
        );
        CREATE TABLE IF NOT EXISTS inventory_transactions (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            item_id         INTEGER NOT NULL REFERENCES inventory_items(id),
            batch_id        INTEGER REFERENCES inventory_batches(id),
            tx_type         TEXT NOT NULL,
            quantity        REAL NOT NULL,
            applicant       TEXT DEFAULT '',
            approver        TEXT DEFAULT '',
            approval_task_id INTEGER,
            related_id      INTEGER,
            note            TEXT DEFAULT '',
            created_by      TEXT DEFAULT '',
            created_at      TEXT NOT NULL DEFAULT (datetime('now','localtime'))
        );
        CREATE INDEX IF NOT EXISTS idx_inv_item_cat ON inventory_items(category_id);
        CREATE INDEX IF NOT EXISTS idx_inv_batch_item ON inventory_batches(item_id);"
    ).ok();

    // ---- 采购管理 ----
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS suppliers (
            id            INTEGER PRIMARY KEY AUTOINCREMENT,
            name          TEXT NOT NULL,
            contact       TEXT DEFAULT '',
            phone         TEXT DEFAULT '',
            email         TEXT DEFAULT '',
            qualification TEXT DEFAULT '',
            status        TEXT NOT NULL DEFAULT '待审核',
            notes         TEXT DEFAULT '',
            created_at    TEXT NOT NULL DEFAULT (datetime('now','localtime')),
            deleted_at    TEXT
        );
        CREATE TABLE IF NOT EXISTS purchase_requisitions (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            requester       TEXT NOT NULL,
            item_name       TEXT NOT NULL,
            spec            TEXT DEFAULT '',
            quantity        REAL NOT NULL,
            unit            TEXT DEFAULT '',
            purpose         TEXT DEFAULT '',
            expected_supplier TEXT DEFAULT '',
            status          TEXT NOT NULL DEFAULT '待审批',
            approval_task_id INTEGER,
            created_by      TEXT DEFAULT '',
            created_at      TEXT NOT NULL DEFAULT (datetime('now','localtime')),
            deleted_at      TEXT
        );
        CREATE TABLE IF NOT EXISTS purchase_orders (
            id            INTEGER PRIMARY KEY AUTOINCREMENT,
            order_no      TEXT NOT NULL UNIQUE,
            supplier_id   INTEGER REFERENCES suppliers(id),
            requisition_ids TEXT DEFAULT '',
            total_amount  REAL DEFAULT 0,
            status        TEXT NOT NULL DEFAULT '待审批',
            approval_task_id INTEGER,
            sent_at       TEXT,
            received_at   TEXT,
            note          TEXT DEFAULT '',
            created_by    TEXT DEFAULT '',
            created_at    TEXT NOT NULL DEFAULT (datetime('now','localtime')),
            deleted_at    TEXT
        );
        CREATE TABLE IF NOT EXISTS purchase_order_items (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            order_id     INTEGER NOT NULL REFERENCES purchase_orders(id) ON DELETE CASCADE,
            item_name    TEXT NOT NULL,
            spec         TEXT DEFAULT '',
            quantity     REAL NOT NULL,
            unit_price   REAL DEFAULT 0,
            amount       REAL DEFAULT 0,
            requisition_id INTEGER
        );
        CREATE INDEX IF NOT EXISTS idx_pr_approval ON purchase_requisitions(approval_task_id);
        CREATE INDEX IF NOT EXISTS idx_po_approval ON purchase_orders(approval_task_id);"
    ).ok();

    // ---- 审批流 ----
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS approval_rules (
            id            INTEGER PRIMARY KEY AUTOINCREMENT,
            biz_type      TEXT NOT NULL,
            name          TEXT NOT NULL DEFAULT '',
            applicant_role TEXT,
            applicant     TEXT,
            object_type   TEXT,
            object_value  TEXT,
            approver_role TEXT,
            approver      TEXT,
            priority      INTEGER NOT NULL DEFAULT 100,
            is_active     INTEGER NOT NULL DEFAULT 1,
            created_at    TEXT NOT NULL DEFAULT (datetime('now','localtime'))
        );
        CREATE TABLE IF NOT EXISTS approval_tasks (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            biz_type    TEXT NOT NULL,
            biz_id      INTEGER NOT NULL,
            title       TEXT NOT NULL,
            applicant   TEXT NOT NULL,
            approver    TEXT,
            approver_role TEXT,
            status      TEXT NOT NULL DEFAULT '待审批',
            rule_id     INTEGER,
            decision_note TEXT DEFAULT '',
            decided_at  TEXT,
            created_at  TEXT NOT NULL DEFAULT (datetime('now','localtime'))
        );
        CREATE INDEX IF NOT EXISTS idx_appr_task_status ON approval_tasks(status, approver, approver_role);"
    ).ok();

    // ---- 通知 / 站内信 ----
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS notifications (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            recipient  TEXT NOT NULL,
            sender     TEXT DEFAULT 'system',
            title      TEXT NOT NULL,
            content    TEXT DEFAULT '',
            link       TEXT DEFAULT '',
            module     TEXT DEFAULT 'system',
            is_read    INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now','localtime'))
        );
        CREATE INDEX IF NOT EXISTS idx_notif_recipient ON notifications(recipient, is_read);"
    ).ok();

    // ---- 审计 WORM 改造 + module 修正（顺序：先回填 → 加列 → 触发器）----
    // 1) 历史 module 回填（必须在触发器创建之前执行）：work 侧原默认 shared → work
    conn.execute_batch(
        "UPDATE audit_log SET module='work'
          WHERE module='shared'
            AND table_name IN ('work_records','project_groups','projects','methods',
                               'project_lab_links','project_method_links','method_type_links',
                               'method_types','import_mappings');"
    ).ok();
    // 2) 字段级 diff 列
    conn.execute("ALTER TABLE audit_log ADD COLUMN before_json TEXT", []).ok();
    conn.execute("ALTER TABLE audit_log ADD COLUMN after_json TEXT", []).ok();
    // 3) WORM 触发器（物理只增，禁止 UPDATE/DELETE）
    conn.execute_batch(
        "CREATE TRIGGER IF NOT EXISTS audit_log_no_update BEFORE UPDATE ON audit_log
          BEGIN SELECT RAISE(ABORT, 'WORM: audit_log is append-only'); END;
         CREATE TRIGGER IF NOT EXISTS audit_log_no_delete BEFORE DELETE ON audit_log
          BEGIN SELECT RAISE(ABORT, 'WORM: audit_log is append-only'); END;"
    ).ok();

    Ok(())
}
