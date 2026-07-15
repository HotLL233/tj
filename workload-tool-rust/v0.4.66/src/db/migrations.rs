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

    // v0.4.19: 结构化帮助文章（Word/PDF 导入）
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
    // v0.4.22: 样品信息登记模块
    // ═══════════════════════════════════════════════════════════
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS sample_info_records (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            status TEXT NOT NULL DEFAULT '待检测',
            seq_no INTEGER NOT NULL,
            batch_no TEXT NOT NULL,
            user_name TEXT NOT NULL,
            lab_name TEXT NOT NULL,
            project_name TEXT NOT NULL,
            submitted_at TEXT NOT NULL,
            detection_date TEXT NOT NULL,
            main_components TEXT NOT NULL,
            detection_type TEXT NOT NULL,
            notes TEXT DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now','localtime')),
            updated_at TEXT DEFAULT (datetime('now','localtime')),
            deleted_at TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_sir_detection_type ON sample_info_records(detection_type);
        CREATE INDEX IF NOT EXISTS idx_sir_status ON sample_info_records(status);
        CREATE INDEX IF NOT EXISTS idx_sir_submitted ON sample_info_records(submitted_at);"
    ).ok();

    // v0.4.23: 检测类型表（软关联 sample_info_records.type_key）
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS sample_info_types (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            type_key TEXT NOT NULL UNIQUE,
            label TEXT NOT NULL,
            description TEXT DEFAULT '',
            color TEXT DEFAULT '#2e7d32',
            sort_order INTEGER DEFAULT 0,
            is_active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now','localtime'))
        );"
    ).ok();

    // v0.4.23: 种子数据（检测类型）
    conn.execute_batch(
        "INSERT OR IGNORE INTO sample_info_types (type_key, label, description, color, sort_order) VALUES
         ('icp', 'ICP', '电感耦合等离子体检测', '#2e7d32', 1),
         ('thermal', '热稳定性', '热稳定性 · TGA · DSC 检测', '#e65100', 2),
         ('mass', '质谱', '质谱分析检测', '#6a1b9a', 3),
         ('other', '其他', '液相 · 气相 · 理化等', '#0277bd', 4);"
    ).ok();

    // v0.4.23: sample_info_records 增加 type_key 软关联列
    conn.execute("ALTER TABLE sample_info_records ADD COLUMN type_key TEXT DEFAULT ''", []).ok();
    conn.execute("CREATE INDEX IF NOT EXISTS idx_sir_type_key ON sample_info_records(type_key)", []).ok();

    // ═══════════════════════════════════════════════════════════
    // v0.4.24: 事业部层级 — divisions 主数据表 + 8 类种子 + 三表 division_id 列
    // ═══════════════════════════════════════════════════════════
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS divisions (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            name        TEXT NOT NULL UNIQUE,
            sort_order  INTEGER NOT NULL DEFAULT 0,
            color       TEXT NOT NULL DEFAULT '#1976d2',
            is_active   INTEGER NOT NULL DEFAULT 1,
            deleted_at  TEXT,
            created_at  TEXT NOT NULL DEFAULT (datetime('now','localtime'))
        );
        CREATE INDEX IF NOT EXISTS idx_divisions_sort ON divisions(sort_order);"
    ).ok();

    // 种子（v0.4.24）：截图2 那 8 类，sort_order 与截图顺序一致
    conn.execute_batch(
        "INSERT OR IGNORE INTO divisions (name, sort_order) VALUES
            ('液相',1),('气相',2),('理化',3),('ICP',4),
            ('热分析',5),('质谱',6),('红外',7),('其他',99);"
    ).ok();

    // project_groups 增加 division_id（软关联；旧实验室自然为 NULL）
    conn.execute("ALTER TABLE project_groups ADD COLUMN division_id INTEGER REFERENCES divisions(id)", []).ok();
    conn.execute("CREATE INDEX IF NOT EXISTS idx_groups_division ON project_groups(division_id)", []).ok();
    // v0.4.40: 为 project_groups 补加 deleted_at 列（之前缺失导致 by_division SQL 报错）
    conn.execute("ALTER TABLE project_groups ADD COLUMN deleted_at TEXT", []).ok();

    // work_records / rd_work_records 增加 division_id（冗余快照，录入时锁定写入）
    conn.execute("ALTER TABLE work_records ADD COLUMN division_id INTEGER REFERENCES divisions(id)", []).ok();
    conn.execute("CREATE INDEX IF NOT EXISTS idx_records_division ON work_records(division_id)", []).ok();
    conn.execute("ALTER TABLE rd_work_records ADD COLUMN division_id INTEGER REFERENCES divisions(id)", []).ok();
    conn.execute("CREATE INDEX IF NOT EXISTS idx_rd_records_division ON rd_work_records(division_id)", []).ok();

    // ═══════════════════════════════════════════════════════════
    // v0.4.25: sample_info_records 增加 division_id 和 quantity
    // ═══════════════════════════════════════════════════════════
    conn.execute("ALTER TABLE sample_info_records ADD COLUMN division_id INTEGER DEFAULT NULL", []).ok();
    conn.execute("ALTER TABLE sample_info_records ADD COLUMN quantity INTEGER NOT NULL DEFAULT 1", []).ok();

    // ═══════════════════════════════════════════════════════════
    // v0.4.26: 列自定义 — sample_info_columns 表
    // ═══════════════════════════════════════════════════════════
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS sample_info_columns (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            field_key TEXT NOT NULL UNIQUE,
            label TEXT NOT NULL,
            data_type TEXT NOT NULL DEFAULT 'text',
            is_predefined INTEGER NOT NULL DEFAULT 0,
            is_required INTEGER NOT NULL DEFAULT 0,
            is_active INTEGER NOT NULL DEFAULT 1,
            width INTEGER DEFAULT 100,
            sort_order INTEGER DEFAULT 0,
            options TEXT,
            show_in_list INTEGER NOT NULL DEFAULT 1,
            show_in_export INTEGER NOT NULL DEFAULT 1,
            show_in_form INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now','localtime')),
            updated_at TEXT DEFAULT (datetime('now','localtime'))
        );"
    ).ok();

    // 预置数据（12 条内置列）
    conn.execute_batch(
        "INSERT OR IGNORE INTO sample_info_columns (field_key, label, data_type, is_predefined, is_required, sort_order, show_in_list, show_in_export, show_in_form)
         VALUES
          ('seq_no', '序号', 'number', 1, 0, 0, 1, 1, 0),
          ('user_name', '送样人', 'text', 1, 0, 1, 1, 1, 1),
          ('division_id', '所属部门', 'select', 1, 0, 2, 1, 1, 1),
          ('lab_name', '实验室/车间', 'text', 1, 0, 3, 1, 1, 1),
          ('project_name', '所属项目', 'text', 1, 0, 4, 1, 1, 1),
          ('quantity', '送样数量', 'number', 1, 0, 5, 1, 1, 1),
          ('batch_no', '样品批号', 'text', 1, 1, 6, 1, 1, 1),
          ('main_components', '样品主要成分', 'text', 1, 1, 7, 1, 1, 1),
          ('notes', '注意事项', 'text', 1, 0, 8, 1, 1, 1),
          ('submitted_at', '送样时间', 'date', 1, 0, 9, 1, 1, 0),
          ('detection_type', '检测类型', 'text', 1, 0, 10, 1, 1, 0),
          ('status', '状态', 'text', 1, 0, 11, 1, 1, 0),
          ('attachment_files', '附件', 'attachment', 1, 0, 12, 1, 1, 1);"
    ).ok();

    // v0.4.26: sample_info_records 加 extra_fields 列（存储自定义字段的 JSON）
    conn.execute("ALTER TABLE sample_info_records ADD COLUMN extra_fields TEXT DEFAULT '{}'", []).ok();

    // ═══════════════════════════════════════════════════════════
    // v0.4.27-A: 列可见性控制 + 用户系统 + 附件 + 研发送样扩展
    // ═══════════════════════════════════════════════════════════

    // 1. sample_info_columns 增加 type_key 列（自定义列绑定检测类型）
    conn.execute("ALTER TABLE sample_info_columns ADD COLUMN type_key TEXT DEFAULT NULL", []).ok();

    // 2. 列可见性桥接表
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS sample_info_column_visibility (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            type_key    TEXT NOT NULL,
            column_id   INTEGER NOT NULL REFERENCES sample_info_columns(id) ON DELETE CASCADE,
            is_visible  INTEGER NOT NULL DEFAULT 1,
            UNIQUE(type_key, column_id)
        );
        CREATE INDEX IF NOT EXISTS idx_scv_type ON sample_info_column_visibility(type_key);"
    )?;

    // 3. 用户表
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS users (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            username    TEXT NOT NULL UNIQUE,
            password    TEXT NOT NULL,
            division_id INTEGER REFERENCES divisions(id),
            group_id    INTEGER REFERENCES project_groups(id),
            is_admin    INTEGER NOT NULL DEFAULT 0,
            is_active   INTEGER NOT NULL DEFAULT 1,
            created_at  TEXT NOT NULL DEFAULT (datetime('now','localtime')),
            updated_at  TEXT DEFAULT (datetime('now','localtime'))
        );
        CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);"
    )?;

    // 4. 用户会话表
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS user_sessions (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id     INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            token       TEXT NOT NULL UNIQUE,
            expires_at  TEXT NOT NULL,
            created_at  TEXT NOT NULL DEFAULT (datetime('now','localtime'))
        );
        CREATE INDEX IF NOT EXISTS idx_sessions_token ON user_sessions(token);"
    )?;

    // 5. 附件表
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS sample_info_attachments (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            record_id   INTEGER NOT NULL REFERENCES sample_info_records(id) ON DELETE CASCADE,
            file_name   TEXT NOT NULL,
            stored_name TEXT NOT NULL,
            file_size   INTEGER NOT NULL,
            file_type   TEXT NOT NULL,
            created_at  TEXT NOT NULL DEFAULT (datetime('now','localtime'))
        );
        CREATE INDEX IF NOT EXISTS idx_sia_record ON sample_info_attachments(record_id);"
    )?;

    // 6. rd_work_records 扩展
    conn.execute("ALTER TABLE rd_work_records ADD COLUMN batch_no TEXT DEFAULT ''", []).ok();
    conn.execute("ALTER TABLE rd_work_records ADD COLUMN notes TEXT DEFAULT ''", []).ok();

    // 7. 种子数据：admin 用户（密码 admin123）
    conn.execute_batch(
        "INSERT OR IGNORE INTO users (username, password, is_admin, is_active)
         VALUES ('admin', '$2b$12$ASTL4icDakjN.9/MrnH.JeeSgoIbT3FpfBIUccI8ZObnZYfYcZfqa', 1, 1);"
    )?;
    // 修复已存在的 admin 记录密码（兼容旧安装）
    conn.execute(
        "UPDATE users SET password = ?1, updated_at = datetime('now','localtime')
         WHERE username = 'admin' AND password = ?2",
        rusqlite::params!["$2b$12$ASTL4icDakjN.9/MrnH.JeeSgoIbT3FpfBIUccI8ZObnZYfYcZfqa", "$2b$12$LJ3m4ys3Lk0TSwHlvL.5M.YiN.YiZZaQr3bGPlcFn8Y5NJMmROhfS"],
    )?;

    // 8. 种子数据：为现有 4 个检测类型批量建立预置列可见性
    conn.execute_batch(
        "INSERT OR IGNORE INTO sample_info_column_visibility (type_key, column_id, is_visible)
         SELECT t.type_key, c.id, 1
         FROM sample_info_types t
         CROSS JOIN sample_info_columns c
         WHERE c.is_predefined = 1
           AND t.type_key IN ('icp', 'thermal', 'mass', 'other');"
    )?;

    // ═══════════════════════════════════════════════════════════
    // v0.4.32: 用户分级（角色）+ 入口可见性后台可自定义
    // ═══════════════════════════════════════════════════════════

    // 角色表
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS roles (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            name        TEXT NOT NULL UNIQUE,
            description TEXT DEFAULT '',
            is_system   INTEGER NOT NULL DEFAULT 0,
            sort_order  INTEGER NOT NULL DEFAULT 0
        );
        CREATE TABLE IF NOT EXISTS role_permissions (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            role_id      INTEGER NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
            permission_key TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_role_perms_role ON role_permissions(role_id);"
    )?;

    // 用户表增加 role_id（老库升级：列已存在则忽略错误）
    conn.execute("ALTER TABLE users ADD COLUMN role_id INTEGER REFERENCES roles(id)", []).ok();

    // 种子角色 + 权限点
    seed_roles(conn)?;

    // v0.4.34: 为已有分析检测员角色补上 entry:sample + sample:collect 权限
    conn.execute_batch(
        "INSERT OR IGNORE INTO role_permissions (role_id, permission_key)
         SELECT r.id, 'entry:sample' FROM roles r WHERE r.name='分析检测员'
         UNION
         SELECT r.id, 'sample:collect' FROM roles r WHERE r.name='分析检测员';"
    ).ok();

    // ═══════════════════════════════════════════════════════════
    // v0.4.33: 研发送样列配置表
    // ═══════════════════════════════════════════════════════════
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS rd_record_columns (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            label TEXT NOT NULL,
            data_type TEXT NOT NULL DEFAULT 'text',
            width INTEGER DEFAULT 100,
            sort_order INTEGER DEFAULT 0,
            is_predefined INTEGER NOT NULL DEFAULT 1,
            show_in_list INTEGER NOT NULL DEFAULT 1,
            show_in_form INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now','localtime')),
            updated_at TEXT DEFAULT (datetime('now','localtime'))
        );"
    )?;

    // 预置 11 条内置列
    conn.execute_batch(
        "INSERT OR IGNORE INTO rd_record_columns (name, label, data_type, width, sort_order, show_in_list, show_in_form)
         VALUES
          ('seq_no', '序号', 'number', 60, 0, 1, 0),
          ('user_name', '送样人', 'text', 120, 1, 1, 1),
          ('division_id', '部门', 'text', 140, 2, 1, 1),
          ('lab_name', '实验室', 'text', 150, 3, 1, 0),
          ('project_name', '项目', 'text', 160, 4, 1, 1),
          ('detection_type', '检测类型', 'text', 120, 5, 1, 1),
          ('method_name', '方法', 'text', 200, 6, 1, 1),
          ('sampling_person', '取样人', 'text', 100, 7, 1, 0),
          ('sampling_time', '取样时间', 'text', 140, 8, 1, 0),
          ('status', '状态', 'text', 80, 9, 1, 0),
          ('notes', '注意事项', 'text', 150, 10, 1, 1);"
    )?;

    // v0.4.33: 更新 sample_info_columns 预置列宽度
    conn.execute(
        "UPDATE sample_info_columns SET width=60 WHERE field_key='seq_no' AND width!=60", []
    ).ok();
    conn.execute(
        "UPDATE sample_info_columns SET width=100 WHERE field_key='user_name' AND width!=100", []
    ).ok();
    conn.execute(
        "UPDATE sample_info_columns SET width=100 WHERE field_key='division_id' AND width!=100", []
    ).ok();
    conn.execute(
        "UPDATE sample_info_columns SET width=100 WHERE field_key='lab_name' AND width!=100", []
    ).ok();
    conn.execute(
        "UPDATE sample_info_columns SET width=120 WHERE field_key='project_name' AND width!=120", []
    ).ok();
    conn.execute(
        "UPDATE sample_info_columns SET width=80 WHERE field_key='quantity' AND width!=80", []
    ).ok();
    conn.execute(
        "UPDATE sample_info_columns SET width=120 WHERE field_key='batch_no' AND width!=120", []
    ).ok();
    conn.execute(
        "UPDATE sample_info_columns SET width=180 WHERE field_key='main_components' AND width!=180", []
    ).ok();
    conn.execute(
        "UPDATE sample_info_columns SET width=180 WHERE field_key='notes' AND width!=180", []
    ).ok();
    conn.execute(
        "UPDATE sample_info_columns SET width=140 WHERE field_key='submitted_at' AND width!=140", []
    ).ok();
    conn.execute(
        "UPDATE sample_info_columns SET width=100 WHERE field_key='detection_type' AND width!=100", []
    ).ok();
    conn.execute(
        "UPDATE sample_info_columns SET width=80 WHERE field_key='status' AND width!=80", []
    ).ok();

    // ═══════════════════════════════════════════════════════════
    // v0.4.35: 全 UI 自定义系统 — system_settings 表 + 种子数据
    // ═══════════════════════════════════════════════════════════
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS system_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TEXT
        );"
    )?;

    seed_default_settings(conn)?;

    Ok(())
}

/// 写入 5 个种子角色（is_system=1 不可删除）及其权限点。
/// 使用 INSERT OR IGNORE 保证幂等，老库重复执行安全。
fn seed_roles(conn: &rusqlite::Connection) -> Result<()> {
    // (name, description, sort_order, permissions)
    let seeds: &[(&str, &str, i32, &[&str])] = &[
        ("系统管理员", "拥有全部权限", 0, &["*"]),
        (
            "分析检测员",
            "分析检测相关管理权限",
            1,
            &["entry:workload", "entry:sample", "sample:collect", "manage:projects", "manage:groups", "manage:methods", "manage:trash", "manage:audit", "manage:help"],
        ),
        (
            "研发送样员",
            "研发送样相关管理权限",
            2,
            &["entry:sample", "manage:sampleinfo", "manage:help"],
        ),
        (
            "样品登记员",
            "样品信息登记相关管理权限",
            3,
            &["entry:sample-info", "manage:sampleinfo", "manage:help"],
        ),
        (
            "查看者",
            "仅查看门户入口",
            4,
            &["entry:sample", "entry:workload", "entry:sample-info", "manage:help"],
        ),
    ];

    for (name, desc, sort_order, perms) in seeds {
        // 已存在同名角色则跳过（保留既有权限点）
        let exists: i64 = conn.query_row(
            "SELECT COUNT(*) FROM roles WHERE name = ?1",
            rusqlite::params![name],
            |r| r.get(0),
        )?;
        if exists > 0 {
            continue;
        }
        conn.execute(
            "INSERT INTO roles (name, description, is_system, sort_order) VALUES (?1, ?2, 1, ?3)",
            rusqlite::params![name, desc, sort_order],
        )?;
        let role_id = conn.last_insert_rowid();
        for p in *perms {
            conn.execute(
                "INSERT INTO role_permissions (role_id, permission_key) VALUES (?1, ?2)",
                rusqlite::params![role_id, p],
            )?;
        }
    }
    Ok(())
}

/// 写入 5 条默认系统设置，使用 INSERT OR IGNORE 保证幂等
fn seed_default_settings(conn: &rusqlite::Connection) -> Result<()> {
    let settings = vec![
        ("theme", r##"{"primaryColor":"#667eea","secondaryColor":"#764ba2","bgColor":"#f8fafc","cardRadius":2,"loginBg":"linear-gradient(135deg, #f0f4f8, #e8f5e9)","loginButtonColor":"#f4511e","logoText":"知微"}"##),
        ("home_cards", r##"[{"key":"sample","title":"研发送样","subtitle":"送样录入 · 查看记录","path":"/sample","perm":"entry:sample","icon":"Science","gradient":"linear-gradient(145deg,#fff3e0,#ffe0b2)","border":"#e65100","titleColor":"#e65100"},{"key":"workload","title":"分析检测","subtitle":"检测录入 · 统计 · 管理","path":"/workload","perm":"entry:workload","icon":"BarChart","gradient":"linear-gradient(145deg,#e8eaf6,#c5cae9)","border":"#283593","titleColor":"#283593"},{"key":"sample-info","title":"样品信息登记","subtitle":"ICP · 热分析 · 质谱等样品信息填写","path":"/sample-info","perm":"entry:sample-info","icon":"Assignment","gradient":"linear-gradient(145deg,#e8f5e9,#c8e6c9)","border":"#2e7d32","titleColor":"#2e7d32"}]"##),
        ("portal_styles", r##"{"sampleColor":"#e65100","workloadColor":"#1976d2","brandName":"知微"}"##),
        ("manage_tabs", r##"[{"key":"projects","label":"项目管理","icon":"Folder","perm":"manage:projects","enabled":true},{"key":"groups","label":"分组管理","icon":"AccountTree","perm":"manage:groups","enabled":true},{"key":"divisions","label":"部门管理","icon":"Business","perm":"manage:divisions","enabled":true},{"key":"methods","label":"方法管理","icon":"Science","perm":"manage:methods","enabled":true},{"key":"trash","label":"回收站","icon":"Delete","perm":"manage:trash","enabled":true},{"key":"audit","label":"审计日志","icon":"History","perm":"manage:audit","enabled":true},{"key":"backup","label":"备份恢复","icon":"Backup","perm":"manage:backup","enabled":true},{"key":"help","label":"帮助页","icon":"Help","perm":"manage:help","enabled":true},{"key":"sampleinfo","label":"样品信息登记","icon":"Assignment","perm":"manage:sampleinfo","enabled":true},{"key":"users","label":"用户管理","icon":"People","perm":"manage:users","enabled":true},{"key":"roles","label":"角色管理","icon":"AdminPanelSettings","perm":"manage:roles","enabled":true}]"##),
        ("stats_cards", r##"[{"key":"total","label":"总数量","color":"#667eea","gradient":"linear-gradient(135deg,#667eea,#764ba2)"},{"key":"records","label":"总记录数","color":"#43a047","gradient":"linear-gradient(135deg,#43a047,#1b5e20)"},{"key":"users","label":"参与人数","color":"#fb8c00","gradient":"linear-gradient(135deg,#fb8c00,#e65100)"}]"##),
        // v0.4.36: 页面布局 — sample_entry 今日记录表格列定义
        ("layout_sample_entry_fields", r##"[{"key":"user_name","type":"text","label":"送样人","width":120,"required":false,"visible":true,"sort_order":1,"placeholder":""},{"key":"division_id","type":"select","label":"部门","width":140,"required":false,"visible":true,"sort_order":2,"options":"从用户分组读取"},{"key":"lab_name","type":"text","label":"实验室","width":150,"required":false,"visible":true,"sort_order":3,"placeholder":""},{"key":"project_name","type":"text","label":"项目","width":160,"required":false,"visible":true,"sort_order":4,"placeholder":""},{"key":"detection_type","type":"select","label":"检测类型","width":120,"required":false,"visible":true,"sort_order":5,"options":"从检测类型表读取"},{"key":"method_name","type":"text","label":"方法","width":200,"required":false,"visible":true,"sort_order":6,"placeholder":""},{"key":"quantity","type":"number","label":"数量","width":80,"required":false,"visible":true,"sort_order":7},{"key":"batch_no","type":"text","label":"批号","width":100,"required":false,"visible":true,"sort_order":8,"placeholder":""},{"key":"notes","type":"text","label":"注意事项","width":150,"required":false,"visible":true,"sort_order":9,"placeholder":""}]"##),
        // v0.4.37: 全站 UI 可视化编辑 — 各页面 section 布局
        ("layout_home", r##"{"sections":{"brand-title":{"visible":true,"label":"知微","color":"#667eea"},"entry-cards":{"visible":true},"footer-text":{"visible":true,"label":"选择功能入口，开始操作"}}}"##),
        ("layout_sample_entry", r##"{"sections":{"page-title":{"visible":true,"label":"研发送样录入"},"table-columns":{"visible":true},"submit-btn":{"visible":true,"label":"提交送样"},"reset-btn":{"visible":true,"label":"重置"},"sample-time":{"visible":true,"label":"送样时间（整单公共）"},"action-btns":{"visible":true,"actionButtons":{"addLabel":"添加行","deleteLabel":"删除选中","resetLabel":"重置"}},"entry-table":{"visible":true}}}"##),
        ("layout_sample_portal", r##"{"sections":{"page-title":{"visible":true,"label":"研发送样"},"division-chips":{"visible":true},"group-cards":{"visible":true}}}"##),
        ("layout_workload_portal", r##"{"sections":{"page-title":{"visible":true,"label":"分析检测"},"division-chips":{"visible":true},"group-cards":{"visible":true}}}"##),
        ("layout_sample_info_entry", r##"{"sections":{"page-title":{"visible":true,"label":"样品信息登记"},"form-fields":{"visible":true},"submit-btn":{"visible":true,"label":"提交"},"detection-type-chip":{"visible":true,"chipLabel":"icp"},"seq-no-chip":{"visible":true,"label":"序号: 自动生成"},"sample-time":{"visible":true,"label":"送样时间（整单公共）"},"action-btns":{"visible":true,"actionButtons":{"addLabel":"添加行","deleteLabel":"删除选中","resetLabel":"重置"}},"records-table":{"visible":true}}}"##),
        ("layout_sample_info_home", r##"{"sections":{"page-title":{"visible":true,"label":"样品信息登记"},"type-cards":{"visible":true}}}"##),
        ("layout_rd_records", r##"{"sections":{"page-title":{"visible":true,"label":"研发送样记录"},"table-columns":{"visible":true},"filters":{"visible":true}}}"##),
        ("layout_stats", r##"{"sections":{"page-title":{"visible":true,"label":"分析检测统计"},"stat-cards":{"visible":true},"charts":{"visible":true}}}"##),
        ("layout_sample_stats", r##"{"sections":{"page-title":{"visible":true,"label":"研发送样统计"},"stat-cards":{"visible":true},"charts":{"visible":true}}}"##),
        // v0.4.49: 统一表单字段定义（ManageFormConfig 管理页集中编辑）
        ("form_sample_entry", r##"{"table_config": {"row_height": 48, "seq_column_width": 50, "checkbox_column_width": 36}, "fields": [{"key": "user_name", "type": "text", "label": "送样人", "width": 120, "required": true, "visible": true, "sort_order": 1, "placeholder": ""}, {"key": "division_id", "type": "select", "label": "部门", "width": 140, "required": false, "visible": true, "sort_order": 2, "options": "从用户分组读取"}, {"key": "lab_name", "type": "text", "label": "实验室", "width": 150, "required": false, "visible": true, "sort_order": 3, "placeholder": ""}, {"key": "project_name", "type": "text", "label": "项目", "width": 160, "required": false, "visible": true, "sort_order": 4, "placeholder": ""}, {"key": "detection_type", "type": "select", "label": "检测类型", "width": 120, "required": false, "visible": true, "sort_order": 5, "options": "从检测类型表读取"}, {"key": "method_name", "type": "text", "label": "方法", "width": 200, "required": false, "visible": true, "sort_order": 6, "placeholder": ""}, {"key": "quantity", "type": "number", "label": "数量", "width": 80, "required": false, "visible": true, "sort_order": 7}, {"key": "batch_no", "type": "text", "label": "批号", "width": 100, "required": false, "visible": true, "sort_order": 8, "placeholder": ""}, {"key": "notes", "type": "text", "label": "注意事项", "width": 150, "required": false, "visible": true, "sort_order": 9, "placeholder": ""}]}"##),
        ("form_sample_info_entry", r##"{"table_config": {"row_height": 48, "seq_column_width": 50, "checkbox_column_width": 36}, "fields": [{"key": "sample_time", "type": "datetime", "label": "送样时间", "width": 160, "required": false, "visible": true, "sort_order": 1}, {"key": "submitted_by", "type": "text", "label": "送样人", "width": 120, "required": true, "visible": true, "sort_order": 2, "placeholder": ""}, {"key": "department", "type": "text", "label": "部门", "width": 140, "required": false, "visible": true, "sort_order": 3, "placeholder": ""}, {"key": "sample_type", "type": "select", "label": "检测类型", "width": 120, "required": true, "visible": true, "sort_order": 4, "options": "从检测类型表读取"}, {"key": "project_name", "type": "text", "label": "项目名称", "width": 160, "required": false, "visible": true, "sort_order": 5, "placeholder": ""}, {"key": "method_name", "type": "text", "label": "检测方法", "width": 200, "required": false, "visible": true, "sort_order": 6, "placeholder": ""}, {"key": "sample_name", "type": "text", "label": "样品名称", "width": 150, "required": false, "visible": true, "sort_order": 7, "placeholder": ""}, {"key": "sample_count", "type": "number", "label": "样品数", "width": 80, "required": false, "visible": true, "sort_order": 8}, {"key": "notes", "type": "text", "label": "备注", "width": 150, "required": false, "visible": true, "sort_order": 9, "placeholder": ""}]}"##),
        ("form_entry", r##"{"table_config": {"row_height": 48, "seq_column_width": 50, "checkbox_column_width": 36}, "fields": [{"key": "user_name", "type": "text", "label": "检测人", "width": 120, "required": true, "visible": true, "sort_order": 1, "placeholder": ""}, {"key": "group_id", "type": "select", "label": "实验室", "width": 150, "required": true, "visible": true, "sort_order": 2, "options": "实验室列表"}, {"key": "project_id", "type": "select", "label": "项目", "width": 160, "required": false, "visible": true, "sort_order": 3, "options": "项目列表"}, {"key": "method_id", "type": "select", "label": "方法", "width": 200, "required": false, "visible": true, "sort_order": 4, "options": "方法列表"}, {"key": "quantity", "type": "number", "label": "数量", "width": 80, "required": false, "visible": true, "sort_order": 5}, {"key": "recorded_at", "type": "datetime", "label": "检测时间", "width": 160, "required": false, "visible": true, "sort_order": 6}, {"key": "notes", "type": "text", "label": "备注", "width": 150, "required": false, "visible": true, "sort_order": 7, "placeholder": ""}]}"##),
        ("layout_entry", r##"{"sections":{"page-title":{"visible":true,"label":"检测录入"},"form-fields":{"visible":true},"submit-btn":{"visible":true,"label":"提交"}}}"##),
        ("layout_manage", r##"{"sections":{"page-title":{"visible":true,"label":"系统管理"},"tab-content":{"visible":true},"card-projects":{"visible":true,"label":"研发项目管理"},"card-groups":{"visible":true,"label":"实验室管理"},"card-divisions":{"visible":true,"label":"部门管理"},"card-methods":{"visible":true,"label":"检测方法管理"},"card-trash":{"visible":true,"label":"回收站"},"card-audit":{"visible":true,"label":"审计日志"},"card-backup":{"visible":true,"label":"数据备份"},"card-help":{"visible":true,"label":"教程与帮助"},"card-sampleinfo":{"visible":true,"label":"样品信息登记管理"},"card-users":{"visible":true,"label":"用户管理"},"card-roles":{"visible":true,"label":"角色管理"},"card-theme":{"visible":true,"label":"主题设置"},"card-home":{"visible":true,"label":"首页配置"},"card-stats":{"visible":true,"label":"统计卡片"}}}"##),
        // v0.4.51: 导出模板配置
        ("export_template_workload", r##"{"file_name": "样品管理_{s}_{e}", "sheets": {"sheet1": {"id": "sheet1", "title": "各实验室项目方法对应表", "color": "#1976D2", "enabled": true, "columns": {"lab_name": {"label": "使用实验室", "width": 14}, "project_name": {"label": "项目代号", "width": 18}, "lc_instrument": {"label": "液相仪器", "width": 18}, "method_name": {"label": "检测方法", "width": 30}, "coefficient": {"label": "单价倍率", "width": 10}, "quantity": {"label": "检测数量", "width": 12}, "lc_qty": {"label": "液相检测量", "width": 12}, "gc_qty": {"label": "气相检测量", "width": 12}, "total_qty": {"label": "项目检测总量", "width": 15}}}, "sheet2": {"id": "sheet2", "title": "仪器-汇总", "color": "#43A047", "enabled": true, "columns": {"date": {"label": "日期", "width": 12}, "instrument": {"label": "仪器", "width": 14}, "lab_name": {"label": "实验室", "width": 14}, "project_name": {"label": "项目", "width": 20}, "method_name": {"label": "方法", "width": 30}, "coefficient": {"label": "单价倍率", "width": 10}, "quantity": {"label": "数量", "width": 12}, "daily_total": {"label": "按天数量总计", "width": 15}}}, "sheet3": {"id": "sheet3", "title": "项目-汇总", "color": "#FF9800", "enabled": true, "columns": {"project_name": {"label": "项目", "width": 20}, "lab_name": {"label": "实验室", "width": 14}, "instrument": {"label": "仪器", "width": 14}, "method_name": {"label": "方法", "width": 30}, "quantity": {"label": "数量", "width": 12}, "total_qty": {"label": "项目总量", "width": 15}}}, "sheet4": {"id": "sheet4", "title": "实验室-汇总", "color": "#9C27B0", "enabled": true, "columns": {"lab_name": {"label": "使用实验室", "width": 14}, "project_name": {"label": "项目", "width": 18}, "method_name": {"label": "方法", "width": 30}, "quantity": {"label": "数量", "width": 12}, "person_qty": {"label": "人检测量", "width": 14}, "total_qty": {"label": "实验室总检测量", "width": 16}}}, "sheet5": {"id": "sheet5", "title": "人员-汇总", "color": "#E91E63", "enabled": true, "columns": {"date": {"label": "日期", "width": 12}, "project_name": {"label": "项目", "width": 18}, "method_name": {"label": "方法", "width": 30}, "quantity": {"label": "数量", "width": 12}, "user_name": {"label": "检测人", "width": 14}}}, "sheet6": {"id": "sheet6", "title": "人员汇总表", "color": "#00BCD4", "enabled": true, "columns": {"user_name": {"label": "检测人", "width": 14}, "quantity": {"label": "数量", "width": 12}, "count": {"label": "检测次数", "width": 12}, "instruments": {"label": "使用仪器", "width": 18}}}, "sheet7": {"id": "sheet7", "title": "实验室总表", "color": "#4CAF50", "enabled": true, "columns": {"lab_name": {"label": "实验室", "width": 14}, "project_name": {"label": "项目", "width": 18}, "method_name": {"label": "方法", "width": 30}, "multiplier": {"label": "单价倍率", "width": 10}, "quantity": {"label": "数量", "width": 12}, "coefficient": {"label": "系数", "width": 10}}}, "sheet8": {"id": "sheet8", "title": "项目总表", "color": "#FFC107", "enabled": true, "columns": {"project_name": {"label": "项目", "width": 18}, "lab_name": {"label": "实验室", "width": 14}, "method_name": {"label": "方法", "width": 30}, "multiplier": {"label": "单价倍率", "width": 10}, "quantity": {"label": "数量", "width": 12}}}, "sheet9": {"id": "sheet9", "title": "仪器汇总表", "color": "#9E9E9E", "enabled": true, "columns": {"instrument": {"label": "仪器", "width": 14}, "quantity": {"label": "检测量", "width": 12}, "total": {"label": "总数量", "width": 12}}}, "sheet10": {"id": "sheet10", "title": "理化汇总表", "color": "#795548", "enabled": true, "columns": {"project_name": {"label": "项目", "width": 18}, "method_name": {"label": "理化方法", "width": 30}, "quantity": {"label": "数量", "width": 12}}}, "sheet11": {"id": "sheet11", "title": "类型汇总表", "color": "#00BCD4", "enabled": true, "columns": {"type_name": {"label": "检测类型", "width": 14}, "quantity": {"label": "数量", "width": 12}}}}}"##),
        ("export_template_rd", r##"{"file_name": "研发送样统计_{s}_{e}", "sheets": {"sheet1": {"id": "sheet1", "title": "各实验室项目方法对应表", "color": "#1976D2", "enabled": true, "columns": {"lab_name": {"label": "使用实验室", "width": 14}, "project_name": {"label": "项目代号", "width": 18}, "lc_instrument": {"label": "液相仪器", "width": 18}, "method_name": {"label": "检测方法", "width": 30}, "coefficient": {"label": "单价倍率", "width": 10}, "quantity": {"label": "检测数量", "width": 12}, "lc_qty": {"label": "液相检测量", "width": 12}, "gc_qty": {"label": "气相检测量", "width": 12}, "total_qty": {"label": "项目检测总量", "width": 15}}}, "sheet2": {"id": "sheet2", "title": "仪器-汇总", "color": "#43A047", "enabled": true, "columns": {"date": {"label": "日期", "width": 12}, "instrument": {"label": "仪器", "width": 14}, "lab_name": {"label": "实验室", "width": 14}, "project_name": {"label": "项目", "width": 20}, "method_name": {"label": "方法", "width": 30}, "coefficient": {"label": "单价倍率", "width": 10}, "quantity": {"label": "数量", "width": 12}, "daily_total": {"label": "按天数量总计", "width": 15}}}, "sheet3": {"id": "sheet3", "title": "项目-汇总", "color": "#FF9800", "enabled": true, "columns": {"project_name": {"label": "项目", "width": 20}, "lab_name": {"label": "实验室", "width": 14}, "instrument": {"label": "仪器", "width": 14}, "method_name": {"label": "方法", "width": 30}, "quantity": {"label": "数量", "width": 12}, "total_qty": {"label": "项目总量", "width": 15}}}, "sheet4": {"id": "sheet4", "title": "实验室-汇总", "color": "#9C27B0", "enabled": true, "columns": {"lab_name": {"label": "使用实验室", "width": 14}, "project_name": {"label": "项目", "width": 18}, "method_name": {"label": "方法", "width": 30}, "quantity": {"label": "数量", "width": 12}, "person_qty": {"label": "人检测量", "width": 14}, "total_qty": {"label": "实验室总检测量", "width": 16}}}, "sheet5": {"id": "sheet5", "title": "人员-汇总", "color": "#E91E63", "enabled": true, "columns": {"date": {"label": "日期", "width": 12}, "project_name": {"label": "项目", "width": 18}, "method_name": {"label": "方法", "width": 30}, "quantity": {"label": "数量", "width": 12}, "user_name": {"label": "检测人", "width": 14}}}, "sheet6": {"id": "sheet6", "title": "人员���总表", "color": "#00BCD4", "enabled": true, "columns": {"user_name": {"label": "检测人", "width": 14}, "quantity": {"label": "数量", "width": 12}, "count": {"label": "检测次数", "width": 12}, "instruments": {"label": "使用仪器", "width": 18}}}, "sheet7": {"id": "sheet7", "title": "实验室总表", "color": "#4CAF50", "enabled": true, "columns": {"lab_name": {"label": "实验室", "width": 14}, "project_name": {"label": "项目", "width": 18}, "method_name": {"label": "方法", "width": 30}, "multiplier": {"label": "单价倍率", "width": 10}, "quantity": {"label": "数量", "width": 12}, "coefficient": {"label": "系数", "width": 10}}}, "sheet8": {"id": "sheet8", "title": "项目总表", "color": "#FFC107", "enabled": true, "columns": {"project_name": {"label": "项目", "width": 18}, "lab_name": {"label": "实验室", "width": 14}, "method_name": {"label": "方法", "width": 30}, "multiplier": {"label": "单价倍率", "width": 10}, "quantity": {"label": "数量", "width": 12}}}, "sheet9": {"id": "sheet9", "title": "仪器汇总表", "color": "#9E9E9E", "enabled": true, "columns": {"instrument": {"label": "仪器", "width": 14}, "quantity": {"label": "检测量", "width": 12}, "total": {"label": "总数量", "width": 12}}}, "sheet10": {"id": "sheet10", "title": "理化汇总表", "color": "#795548", "enabled": true, "columns": {"project_name": {"label": "项目", "width": 18}, "method_name": {"label": "理化方法", "width": 30}, "quantity": {"label": "数量", "width": 12}}}, "sheet11": {"id": "sheet11", "title": "类型汇总表", "color": "#00BCD4", "enabled": true, "columns": {"type_name": {"label": "检测类型", "width": 14}, "quantity": {"label": "数量", "width": 12}}}}}"##),
        ("export_template_sample_info", r##"{"file_name": "样品信息登记_{s}_{e}", "sheets": {}}"##),
    ];

    for (key, value) in settings {
        conn.execute(
            "INSERT OR IGNORE INTO system_settings (key, value, updated_at) VALUES (?1, ?2, datetime('now','localtime'))",
            rusqlite::params![key, value],
        )?;
    }

    // v0.4.63: 高项表 + 项目关联
    conn.execute("CREATE TABLE IF NOT EXISTS high_items (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL UNIQUE,
        sort_order INTEGER DEFAULT 0,
        is_active INTEGER NOT NULL DEFAULT 1,
        created_at TEXT NOT NULL DEFAULT (datetime('now','localtime'))
    )", []).ok();
    conn.execute("INSERT OR IGNORE INTO high_items (name, sort_order) VALUES ('高项', 1)", []).ok();
    conn.execute("ALTER TABLE projects ADD COLUMN high_item_id INTEGER DEFAULT NULL REFERENCES high_items(id) ON DELETE SET NULL", []).ok();
    // v0.4.64: 撤销 high_items 表架构，高项改为纯文本字段
    // 删除 high_items 表（迁移终止用 IF EXISTS）
    conn.execute("DROP TABLE IF EXISTS high_items", []).ok();
    // 清理旧的高项列，加新字段
    // SQLite 不支持 DROP COLUMN，保留 high_item_id 不用
    // 新加 high_item TEXT
    conn.execute("ALTER TABLE projects ADD COLUMN high_item TEXT DEFAULT NULL", []).ok();
    conn.execute("ALTER TABLE work_records ADD COLUMN high_item TEXT DEFAULT NULL", []).ok();
    conn.execute("ALTER TABLE rd_work_records ADD COLUMN high_item TEXT DEFAULT NULL", []).ok();

    Ok(())
}
