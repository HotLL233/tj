import json
import random
import sqlite3
import statistics
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from datetime import datetime, timedelta
from pathlib import Path

ROOT = Path(__file__).resolve().parent
DB = ROOT / "workload_loadtest.db"
REPORT = ROOT / "loadtest_report.json"

random.seed(4083)

ANALYSTS = [f"分析员{i:02d}" for i in range(1, 11)]
RD_SENDERS = [f"研发{i:02d}" for i in range(1, 11)]
OTHER_SENDERS = [f"部门送样{i:02d}" for i in range(1, 11)]
ALL_USERS = ANALYSTS + RD_SENDERS + OTHER_SENDERS
SAMPLERS = [f"取样员{i:02d}" for i in range(1, 6)]

DIVISIONS = ["研究院", "化工一部", "化工二部", "动保一部", "动保二部"]
GROUPS = ["液相室", "气相室", "理化室", "质谱室", "样品室"]
METHODS = [
    ("LC含量测定", "液相", 1.0, 80.0, 1.0),
    ("LC有关物质", "液相", 1.2, 120.0, 1.1),
    ("GC残留溶剂", "气相", 1.1, 100.0, 1.0),
    ("水分测定", "理化", 0.8, 50.0, 1.0),
    ("ICP元素分析", "ICP", 1.5, 180.0, 1.2),
]
PROJECTS = [
    ("LC-001", "LC项目001"),
    ("LC-002", "LC项目002"),
    ("GC-001", "GC项目001"),
    ("PH-001", "理化项目001"),
    ("ICP-001", "元素项目001"),
    ("RD-001", "研发项目001"),
    ("RD-002", "研发项目002"),
    ("RD-003", "研发项目003"),
]


def timed(name, fn):
    t0 = time.perf_counter()
    result = fn()
    return name, (time.perf_counter() - t0) * 1000.0, result


def init_db():
    if DB.exists():
        DB.unlink()
    con = sqlite3.connect(DB)
    con.execute("PRAGMA journal_mode=WAL")
    con.execute("PRAGMA synchronous=NORMAL")
    con.executescript(
        """
        CREATE TABLE users (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          username TEXT NOT NULL UNIQUE,
          role TEXT NOT NULL,
          division_id INTEGER,
          group_id INTEGER
        );
        CREATE TABLE divisions (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          name TEXT NOT NULL,
          sort_order INTEGER NOT NULL DEFAULT 0
        );
        CREATE TABLE project_groups (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          name TEXT NOT NULL,
          division_id INTEGER,
          sort_order INTEGER NOT NULL DEFAULT 0
        );
        CREATE TABLE methods (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          name TEXT NOT NULL,
          method_type TEXT NOT NULL,
          coefficient REAL NOT NULL,
          amount REAL NOT NULL,
          multiplier REAL NOT NULL
        );
        CREATE TABLE projects (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          name TEXT NOT NULL,
          full_name TEXT,
          coefficient REAL NOT NULL DEFAULT 1.0,
          high_item TEXT
        );
        CREATE TABLE project_lab_links (
          project_id INTEGER NOT NULL,
          group_id INTEGER NOT NULL
        );
        CREATE TABLE work_records (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          project_id INTEGER NOT NULL,
          method_id INTEGER,
          user_name TEXT NOT NULL,
          quantity INTEGER NOT NULL,
          multiplier REAL NOT NULL DEFAULT 1.0,
          high_item TEXT,
          recorded_at TEXT NOT NULL,
          group_id INTEGER,
          division_id INTEGER,
          deleted_at TEXT
        );
        CREATE TABLE rd_work_records (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          project_id INTEGER NOT NULL,
          method_id INTEGER,
          user_name TEXT NOT NULL,
          quantity INTEGER NOT NULL,
          recorded_at TEXT NOT NULL,
          group_id INTEGER,
          division_id INTEGER,
          batch_no TEXT,
          notes TEXT,
          status TEXT NOT NULL DEFAULT '待取样',
          sampler TEXT,
          sampled_at TEXT,
          multiplier REAL NOT NULL DEFAULT 1.0,
          high_item TEXT,
          deleted_at TEXT
        );
        CREATE TABLE sample_info_records (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          status TEXT NOT NULL DEFAULT '待检测',
          seq_no INTEGER NOT NULL,
          batch_no TEXT NOT NULL,
          user_name TEXT NOT NULL,
          lab_name TEXT NOT NULL,
          project_name TEXT NOT NULL,
          submitted_at TEXT NOT NULL,
          detection_date TEXT,
          main_components TEXT,
          detection_type TEXT,
          type_key TEXT,
          division_id INTEGER,
          quantity INTEGER NOT NULL DEFAULT 1,
          notes TEXT,
          deleted_at TEXT
        );
        CREATE INDEX idx_work_date ON work_records(recorded_at);
        CREATE INDEX idx_work_user ON work_records(user_name);
        CREATE INDEX idx_work_project ON work_records(project_id);
        CREATE INDEX idx_rd_date ON rd_work_records(recorded_at);
        CREATE INDEX idx_rd_user ON rd_work_records(user_name);
        CREATE INDEX idx_rd_project ON rd_work_records(project_id);
        CREATE INDEX idx_sample_info_date ON sample_info_records(submitted_at);
        CREATE INDEX idx_sample_info_user ON sample_info_records(user_name);
        """
    )
    for i, name in enumerate(DIVISIONS, 1):
        con.execute("INSERT INTO divisions(name, sort_order) VALUES(?, ?)", (name, i))
    for i, name in enumerate(GROUPS, 1):
        con.execute("INSERT INTO project_groups(name, division_id, sort_order) VALUES(?, ?, ?)", (name, ((i - 1) % 5) + 1, i))
    for m in METHODS:
        con.execute("INSERT INTO methods(name, method_type, coefficient, amount, multiplier) VALUES(?,?,?,?,?)", m)
    for i, (name, full) in enumerate(PROJECTS, 1):
        con.execute("INSERT INTO projects(name, full_name, coefficient, high_item) VALUES(?,?,?,?)", (name, full, 1.0 + (i % 4) * 0.15, f"高项{i%3+1}"))
        for gid in range(1, 6):
            if (i + gid) % 2 == 0 or gid == ((i - 1) % 5) + 1:
                con.execute("INSERT INTO project_lab_links(project_id, group_id) VALUES(?,?)", (i, gid))
    for idx, username in enumerate(ALL_USERS, 1):
        role = "analysis" if username in ANALYSTS else "rd" if username in RD_SENDERS else "other"
        con.execute("INSERT INTO users(username, role, division_id, group_id) VALUES(?,?,?,?)", (username, role, ((idx - 1) % 5) + 1, ((idx - 1) % 5) + 1))
    con.commit()
    con.close()


def insert_work_records(n=1500):
    con = sqlite3.connect(DB, timeout=30)
    base = datetime(2026, 7, 1, 8, 30)
    rows = []
    for i in range(n):
        user = random.choice(ANALYSTS)
        project_id = random.randint(1, len(PROJECTS))
        method_id = random.randint(1, len(METHODS))
        group_id = random.randint(1, len(GROUPS))
        division_id = ((group_id - 1) % 5) + 1
        dt = base + timedelta(minutes=random.randint(0, 14 * 24 * 60))
        rows.append((project_id, method_id, user, random.randint(1, 12), round(random.uniform(0.8, 1.6), 2), f"高项{random.randint(1,3)}", dt.isoformat(timespec="seconds"), group_id, division_id))
    con.executemany(
        "INSERT INTO work_records(project_id, method_id, user_name, quantity, multiplier, high_item, recorded_at, group_id, division_id) VALUES(?,?,?,?,?,?,?,?,?)",
        rows,
    )
    con.commit()
    con.close()
    return n


def insert_rd_records(n=1800):
    con = sqlite3.connect(DB, timeout=30)
    base = datetime(2026, 7, 1, 9, 0)
    rows = []
    for i in range(n):
        user = random.choice(RD_SENDERS + OTHER_SENDERS)
        project_id = random.randint(1, len(PROJECTS))
        method_id = random.randint(1, len(METHODS))
        group_id = random.randint(1, len(GROUPS))
        division_id = ((group_id - 1) % 5) + 1
        dt = base + timedelta(minutes=random.randint(0, 14 * 24 * 60))
        sampled = random.random() < 0.62
        sampler = random.choice(SAMPLERS) if sampled else None
        sampled_at = (dt + timedelta(hours=random.randint(1, 48))).isoformat(timespec="seconds") if sampled else None
        status = "已取样" if sampled else "待取样"
        rows.append((project_id, method_id, user, random.randint(1, 10), dt.isoformat(timespec="seconds"), group_id, division_id, f"RD{i+1:05d}", "压力测试", status, sampler, sampled_at, round(random.uniform(0.8, 1.6), 2), f"高项{random.randint(1,3)}"))
    con.executemany(
        "INSERT INTO rd_work_records(project_id, method_id, user_name, quantity, recorded_at, group_id, division_id, batch_no, notes, status, sampler, sampled_at, multiplier, high_item) VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
        rows,
    )
    con.commit()
    con.close()
    return n


def insert_sample_info(n=900):
    con = sqlite3.connect(DB, timeout=30)
    base = datetime(2026, 7, 1, 10, 0)
    rows = []
    users = RD_SENDERS + OTHER_SENDERS
    for i in range(n):
        user = random.choice(users)
        group = random.choice(GROUPS)
        project = random.choice(PROJECTS)[0]
        dt = base + timedelta(minutes=random.randint(0, 14 * 24 * 60))
        status = random.choice(["待检测", "检测中", "已完成"])
        type_key = random.choice(["lc", "gc", "physchem", "icp"])
        rows.append((status, i + 1, f"S{i+1:05d}", user, group, project, dt.isoformat(timespec="seconds"), dt.date().isoformat(), "主成分A/B", type_key.upper(), type_key, random.randint(1, 5), random.randint(1, 8), "压力测试"))
    con.executemany(
        "INSERT INTO sample_info_records(status, seq_no, batch_no, user_name, lab_name, project_name, submitted_at, detection_date, main_components, detection_type, type_key, division_id, quantity, notes) VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
        rows,
    )
    con.commit()
    con.close()
    return n


def query_one(sql, params=()):
    con = sqlite3.connect(DB, timeout=30)
    con.row_factory = sqlite3.Row
    rows = [dict(r) for r in con.execute(sql, params).fetchall()]
    con.close()
    return rows


def stats_queries():
    start = "2026-07-01T00:00:00"
    end = "2026-07-15T23:59:59"
    return {
        "analysis_summary": ("SELECT COUNT(*) record_count, COALESCE(SUM(wr.quantity),0) total_quantity, COALESCE(SUM(wr.quantity * p.coefficient * wr.multiplier),0) score FROM work_records wr JOIN projects p ON p.id=wr.project_id WHERE wr.deleted_at IS NULL AND wr.recorded_at BETWEEN ? AND ?", (start, end)),
        "analysis_by_user": ("SELECT wr.user_name, COUNT(*) record_count, SUM(wr.quantity) total_quantity FROM work_records wr WHERE wr.deleted_at IS NULL AND wr.recorded_at BETWEEN ? AND ? GROUP BY wr.user_name ORDER BY total_quantity DESC", (start, end)),
        "rd_summary": ("SELECT COUNT(*) record_count, COALESCE(SUM(wr.quantity),0) total_quantity, COALESCE(SUM(wr.quantity * p.coefficient),0) score FROM rd_work_records wr JOIN projects p ON p.id=wr.project_id WHERE wr.deleted_at IS NULL AND wr.recorded_at BETWEEN ? AND ?", (start, end)),
        "rd_by_user": ("SELECT wr.user_name, COUNT(*) record_count, SUM(wr.quantity) total_quantity FROM rd_work_records wr WHERE wr.deleted_at IS NULL AND wr.recorded_at BETWEEN ? AND ? GROUP BY wr.user_name ORDER BY total_quantity DESC", (start, end)),
        "rd_by_division": ("SELECT d.name division_name, COUNT(*) record_count, SUM(wr.quantity) total_quantity FROM rd_work_records wr LEFT JOIN divisions d ON d.id=wr.division_id WHERE wr.deleted_at IS NULL AND wr.recorded_at BETWEEN ? AND ? GROUP BY d.name ORDER BY total_quantity DESC", (start, end)),
        "sample_info_stats": ("SELECT status, COUNT(*) record_count, SUM(quantity) total_quantity FROM sample_info_records WHERE deleted_at IS NULL AND submitted_at BETWEEN ? AND ? GROUP BY status", (start, end)),
        "rd_pending_samples": ("SELECT status, COUNT(*) record_count FROM rd_work_records WHERE deleted_at IS NULL GROUP BY status", ()),
        "rd_sampler_load": ("SELECT sampler, COUNT(*) record_count FROM rd_work_records WHERE sampler IS NOT NULL GROUP BY sampler ORDER BY record_count DESC", ()),
    }


def run_query_pressure(rounds=80, workers=12):
    queries = stats_queries()
    latencies = []
    errors = []
    def task(i):
        name = list(queries.keys())[i % len(queries)]
        sql, params = queries[name]
        try:
            qname, ms, rows = timed(name, lambda: query_one(sql, params))
            return {"name": qname, "ms": ms, "rows": len(rows), "error": None}
        except Exception as exc:
            return {"name": name, "ms": None, "rows": 0, "error": str(exc)}

    with ThreadPoolExecutor(max_workers=workers) as ex:
        futures = [ex.submit(task, i) for i in range(rounds)]
        for f in as_completed(futures):
            r = f.result()
            if r["error"]:
                errors.append(r)
            else:
                latencies.append(r["ms"])
    return {
        "rounds": rounds,
        "workers": workers,
        "errors": errors,
        "latency_ms": {
            "min": min(latencies) if latencies else None,
            "avg": statistics.mean(latencies) if latencies else None,
            "p95": sorted(latencies)[int(len(latencies) * 0.95) - 1] if latencies else None,
            "max": max(latencies) if latencies else None,
        },
    }


def db_counts():
    tables = ["users", "work_records", "rd_work_records", "sample_info_records"]
    return {t: query_one(f"SELECT COUNT(*) count FROM {t}")[0]["count"] for t in tables}


def main():
    t0 = time.perf_counter()
    init_db()
    load_steps = []
    for name, fn in [
        ("insert_analysis_work_records", lambda: insert_work_records(1500)),
        ("insert_rd_work_records", lambda: insert_rd_records(1800)),
        ("insert_sample_info_records", lambda: insert_sample_info(900)),
    ]:
        step, ms, count = timed(name, fn)
        load_steps.append({"step": step, "ms": ms, "rows": count})

    samples = {name: query_one(*qp)[:10] for name, qp in stats_queries().items()}
    pressure = run_query_pressure(rounds=120, workers=16)
    report = {
        "scenario": {
            "total_people": 30,
            "analysts": len(ANALYSTS),
            "rd_senders": len(RD_SENDERS),
            "other_senders": len(OTHER_SENDERS),
            "samplers": len(SAMPLERS),
            "analysis_records": 1500,
            "rd_records": 1800,
            "sample_info_records": 900,
        },
        "database": str(DB),
        "counts": db_counts(),
        "load_steps": load_steps,
        "query_pressure": pressure,
        "sample_results": samples,
        "total_elapsed_ms": (time.perf_counter() - t0) * 1000.0,
        "notes": [
            "This is a database-level workload simulation using the same logical tables and query patterns.",
            "The packaged app login endpoint only accepts admin credentials in this build, so per-user token/API login pressure was not simulated.",
        ],
    }
    REPORT.write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")
    print(json.dumps(report, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
