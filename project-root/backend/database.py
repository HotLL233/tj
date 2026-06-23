"""SQLite database connection management and initialization."""
import os
import re
import sys
import sqlite3
from typing import Generator

# DB location: same directory as exe when frozen, or backend/ when dev
if getattr(sys, "frozen", False):
    BASE_DIR = os.path.dirname(sys.executable)
else:
    BASE_DIR = os.path.dirname(__file__)

DATABASE_DIR = os.path.join(BASE_DIR, "data")
DATABASE_PATH = os.path.join(DATABASE_DIR, "workload.db")

# Built-in project groups and methods (hardcoded, no external dependency)
DEFAULT_DATA = [
    ("410实验室", [
        "HYLY-LC-01(230106)", "HYLY-LC-04(QL-230211)", "HYLY-LC-09(230106)",
        "YWJS-LC-11(Q002-230407)", "E003-LC-03(EF-241204)", "E003-LC-07(EF-241204)",
        "E003-GC-02甲乙醇", "E003-GC-04(顶空)",
    ]),
    ("415实验室", [
        "YSLY-LC-12(0909)", "YSLY-LC-12(YSLY-260325)",
        "YSLY-GC-02(甲乙醇)", "YSLY-GC-03(0816)",
    ]),
    ("417实验室", [
        "S002-LC-02(260108)", "S002-LC-02(0909)", "S002-LC-02(260325)", "S002-LC-02(260410)",
        "S002-LC-08(251122)", "S002-LC-08(251229)", "S002-LC-08(251230)", "S002-LC-08(260413)",
        "S002-GC-02(甲乙醇)", "S002-GC-03(0816)", "S002-GC-04(顶空)",
        "A003-GC-02(甲乙醇)", "JYAYSY-GC-03(1128)",
        "Q-LC-01(230106)", "Q008-LC-04(QL-260211)", "Q008-LC-09(230106)", "Q008-LC-13(PQ-SSA)", "Q008-GC-04(顶空)",
    ]),
    ("418实验室", [
        "三氟苯硼酸-LC-05(251108)", "三氟苯硼酸-GC-02(甲乙醇)",
    ]),
    ("生物合成", [
        "环糊精-LC-19(240610)", "F008-LC-20(250310-RID)", "F008-LC-16(厂区-FB-250701)",
        "F008-LC-16(F008)", "DC002-LC-10(250809)", "T001-LC-10(T001-260302)",
        "T001-LC-15(T001-260318)", "T001-LC-09(T001-260408)",
    ]),
    ("车间", [
        "E003-GC-01(240706)", "E003-GC-02(甲乙醇)",
    ]),
    ("801厂区", [
        "F002-GC-04(顶空)",
    ]),
    ("707实验室", [
        "D002-LC-10(200215)",
    ]),
]

# ═══════════════════════════════════════════════════════════════════
#  方法简称 → 全称映射表（与汇总模板完全一致）
#  键格式: (实验室名称, 项目代码, 仪器编号)
# ═══════════════════════════════════════════════════════════════════

METHOD_FULL_NAMES: dict = {
    # 410实验室
    ("410实验室", "HYLY", "LC-01", "230106"): "HYLY-230106-1-低温8℃-DAD",
    ("410实验室", "HYLY", "LC-04", "QL-230211"): "QL-260211-DAD",
    ("410实验室", "HYLY", "LC-09", "230106"): "HYLY-230106-1-低温8℃-DAD",
    ("410实验室", "YWJS", "LC-11", "Q002-230407"): "YWJS-250915-VWD",
    ("410实验室", "E003", "LC-03", "EF-241204"): "EF-241204(38min)-DAD",
    ("410实验室", "E003", "LC-07", "EF-241204"): "EF-241204(38min)-VWD",
    ("410实验室", "E003", "GC-02", "甲乙醇"): "甲乙醇",
    ("410实验室", "E003", "GC-04", "顶空"): "顶空氯乙烷-乙醇200119-1",
    # 415实验室
    ("415实验室", "YSLY", "LC-12", "0909"): "T004-220909-VWD",
    ("415实验室", "YSLY", "LC-12", "YSLY-260325"): "YSLY-260325-VWD",
    ("415实验室", "YSLY", "GC-02", "甲乙醇"): "甲乙醇",
    ("415实验室", "YSLY", "GC-03", "0816"): "A003-210816",
    # 417实验室
    ("417实验室", "S002", "LC-02", "260108"): "S002-260108-DAD",
    ("417实验室", "S002", "LC-02", "0909"): "T004-220909-DAD",
    ("417实验室", "S002", "LC-02", "260325"): "S002-260325-DAD",
    ("417实验室", "S002", "LC-02", "260410"): "S002-260410-DAD",
    ("417实验室", "S002", "LC-08", "251122"): "SFBYS-251122",
    ("417实验室", "S002", "LC-08", "251229"): "A003-251229",
    ("417实验室", "S002", "LC-08", "251230"): "A003-251230",
    ("417实验室", "S002", "LC-08", "260413"): "S002-260413",
    ("417实验室", "S002", "GC-02", "甲乙醇"): "甲乙醇",
    ("417实验室", "S002", "GC-03", "0816"): "A003-210816",
    ("417实验室", "S002", "GC-04", "顶空"): "顶空氯乙烷-乙醇200119-1",
    ("417实验室", "A003", "GC-02", "甲乙醇"): "甲乙醇",
    ("417实验室", "JYAYSY", "GC-03", "1128"): "JYAYSY-251128-GC-03",
    ("417实验室", "Q", "LC-01", "230106"): "HYLY-230106-1-低温8℃-DAD",
    ("417实验室", "Q008", "LC-04", "QL-260211"): "QL-260211-DAD",
    ("417实验室", "Q008", "LC-09", "230106"): "HYLY-230106-1-低温8℃-DAD",
    ("417实验室", "Q008", "LC-13", "PQ-SSA"): "PQ-SSA",
    ("417实验室", "Q008", "GC-04", "顶空"): "顶空氯乙烷-乙醇200119-1",
    # 418实验室
    ("418实验室", "三氟苯硼酸", "LC-05", "251108"): "三氟苯硼酸-251108-DAD",
    ("418实验室", "三氟苯硼酸", "GC-02", "甲乙醇"): "甲乙醇",
    # 生物合成
    ("生物合成", "环糊精", "LC-19", "240610"): "环糊精-240610",
    ("生物合成", "F008", "LC-20", "250310-RID"): "环糊精-250310-RID",
    ("生物合成", "F008", "LC-16", "厂区-FB-250701"): "厂区-FB-250701",
    ("生物合成", "F008", "LC-16", "F008"): "氟苯尼考-F008",
    ("生物合成", "DC002", "LC-10", "250809"): "DC002-250809-VWD",
    ("生物合成", "T001", "LC-10", "T001-260302"): "T001-260302",
    ("生物合成", "T001", "LC-15", "T001-260318"): "T001-260302",
    ("生物合成", "T001", "LC-09", "T001-260408"): "T001-260408",
    # 车间
    ("车间", "E003", "GC-01", "240706"): "J001-240706",
    ("车间", "E003", "GC-02", "甲乙醇"): "甲乙醇",
    # 801厂区
    ("801厂区", "F002", "GC-04", "顶空"): "顶空氯乙烷-乙醇200119-1",
    # 707实验室
    ("707实验室", "D002", "LC-10", "200215"): "",
}


def get_method_full_name(group_name: str, project_name: str) -> str:
    """从项目名称解析项目代码、仪器和版本，查询全称映射表。

    解析逻辑:
        "HYLY-LC-01(230106)" → code="HYLY", inst="LC-01", ver="230106"
        "E003-GC-02甲乙醇"    → code="E003", inst="GC-02", ver="甲乙醇"
        "三氟苯硼酸-LC-05(251108)" → code="三氟苯硼酸", inst="LC-05", ver="251108"

    Args:
        group_name: 实验室/分组名称
        project_name: 项目完整名称

    Returns:
        方法全称字符串，未匹配时返回空字符串
    """
    # 提取仪器编号
    m = re.search(r'(LC|GC)[-_](\d+)', project_name)
    if m:
        instrument = f"{m.group(1)}-{m.group(2)}"
        code = project_name[:m.start()].rstrip('-')
        rest = project_name[m.end():]
    else:
        instrument = ""
        code = project_name
        rest = ""

    # 提取版本号（括号内或尾部中文）
    version = ""
    ver_m = re.search(r'[（(]([^)）]+)[)）]', rest)
    if ver_m:
        version = ver_m.group(1)
    elif rest:
        # 尾部中文如 "甲乙醇"
        version = rest.strip()

    # 按精度逐步匹配
    # 优先精确匹配 (lab, code, inst, version)
    key4 = (group_name, code, instrument, version)
    if key4 in METHOD_FULL_NAMES:
        return METHOD_FULL_NAMES[key4]
    # 回退 (lab, code, inst)
    key3 = (group_name, code, instrument)
    if key3 in METHOD_FULL_NAMES:
        return METHOD_FULL_NAMES[key3]
    return ""


def _seed_builtin_data() -> None:
    """Seed built-in groups and projects on first run."""
    conn = sqlite3.connect(DATABASE_PATH)
    cursor = conn.cursor()
    cursor.execute("SELECT COUNT(*) FROM project_groups")
    if cursor.fetchone()[0] > 0:
        conn.close()
        return

    total = 0
    for sort_idx, (group_name, projects) in enumerate(DEFAULT_DATA):
        cursor.execute("INSERT INTO project_groups (name, sort_order) VALUES (?, ?)", (group_name, sort_idx))
        group_id = cursor.lastrowid
        for proj_idx, name in enumerate(projects):
            full_name = get_method_full_name(group_name, name)
            cursor.execute(
                "INSERT INTO projects (group_id, name, full_name, sort_order) VALUES (?, ?, ?, ?)",
                (group_id, name, full_name, proj_idx),
            )
            total += 1

    conn.commit()
    conn.close()
    print(f"[SEED] Built-in: {len(DEFAULT_DATA)} groups, {total} projects")


def _populate_full_names() -> None:
    """Populate full_name for all projects that have an empty full_name field.

    This handles both new seed data that may have been inserted with '' and
    migration from older databases that lacked the full_name column.
    """
    conn = sqlite3.connect(DATABASE_PATH)
    conn.row_factory = sqlite3.Row
    cursor = conn.cursor()
    cursor.execute(
        """
        SELECT p.id, p.name, pg.name AS group_name
        FROM projects p
        JOIN project_groups pg ON p.group_id = pg.id
        WHERE p.full_name = ''
        """
    )
    rows = cursor.fetchall()
    updated = 0
    for row in rows:
        full_name = get_method_full_name(row["group_name"], row["name"])
        if full_name:
            cursor.execute("UPDATE projects SET full_name = ? WHERE id = ?", (full_name, row["id"]))
            updated += 1
    conn.commit()
    conn.close()
    if updated:
        print(f"[MIGRATE] Populated full_name for {updated} projects")


def get_db() -> Generator[sqlite3.Connection, None, None]:
    """FastAPI dependency injection: provide a database connection per request."""
    os.makedirs(DATABASE_DIR, exist_ok=True)
    conn = sqlite3.connect(DATABASE_PATH, check_same_thread=False)
    conn.row_factory = sqlite3.Row
    conn.execute("PRAGMA journal_mode=WAL")
    conn.execute("PRAGMA foreign_keys=ON")
    try:
        yield conn
    finally:
        conn.close()


def init_db() -> None:
    """Initialize database tables and indexes."""
    os.makedirs(DATABASE_DIR, exist_ok=True)
    conn = sqlite3.connect(DATABASE_PATH, check_same_thread=False)
    conn.execute("PRAGMA journal_mode=WAL")
    conn.execute("PRAGMA foreign_keys=ON")

    cursor = conn.cursor()
    cursor.executescript("""
        CREATE TABLE IF NOT EXISTS project_groups (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            name       TEXT    NOT NULL UNIQUE,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at DATETIME NOT NULL DEFAULT (datetime('now', 'localtime'))
        );

        CREATE TABLE IF NOT EXISTS projects (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            group_id   INTEGER NOT NULL,
            name       TEXT    NOT NULL,
            full_name  TEXT    NOT NULL DEFAULT '',
            notes      TEXT    NOT NULL DEFAULT '',
            sort_order INTEGER NOT NULL DEFAULT 0,
            is_active  INTEGER NOT NULL DEFAULT 1,
            created_at DATETIME NOT NULL DEFAULT (datetime('now', 'localtime')),
            FOREIGN KEY (group_id) REFERENCES project_groups(id)
        );

        CREATE TABLE IF NOT EXISTS work_records (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id  INTEGER  NOT NULL,
            user_name   TEXT     NOT NULL,
            quantity    INTEGER  NOT NULL,
            recorded_at DATETIME NOT NULL,
            created_at  DATETIME NOT NULL DEFAULT (datetime('now', 'localtime')),
            deleted_at  DATETIME NULL,
            FOREIGN KEY (project_id) REFERENCES projects(id)
        );

        CREATE TABLE IF NOT EXISTS audit_log (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            action     TEXT     NOT NULL,
            table_name TEXT     NOT NULL,
            record_id  INTEGER,
            user_name  TEXT     NOT NULL DEFAULT 'system',
            detail     TEXT,
            created_at DATETIME NOT NULL DEFAULT (datetime('now', 'localtime'))
        );

        CREATE INDEX IF NOT EXISTS idx_projects_group ON projects(group_id);
        CREATE INDEX IF NOT EXISTS idx_records_project ON work_records(project_id);
        CREATE INDEX IF NOT EXISTS idx_records_date ON work_records(recorded_at);
        CREATE INDEX IF NOT EXISTS idx_records_user ON work_records(user_name);
        CREATE INDEX IF NOT EXISTS idx_records_deleted ON work_records(deleted_at);
    """)

    # Migration: add full_name and notes columns if missing (for existing databases)
    try:
        cursor.execute("ALTER TABLE projects ADD COLUMN full_name TEXT NOT NULL DEFAULT ''")
    except sqlite3.OperationalError:
        pass  # Column already exists
    try:
        cursor.execute("ALTER TABLE projects ADD COLUMN notes TEXT NOT NULL DEFAULT ''")
    except sqlite3.OperationalError:
        pass  # Column already exists

    conn.commit()
    conn.close()

    # Auto-seed built-in data on first run
    _seed_builtin_data()

    # Populate full_name for all projects that have empty full_name
    _populate_full_names()
