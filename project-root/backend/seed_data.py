"""Built-in default project groups and methods (hardcoded, no external dependency).

Naming convention (from user):
  410-HYLY-LC-01(230106) = 410实验室 / HYLY项目 / LC-01仪器 / 230106方法
  417-S002-LC-02(260108) = 417实验室 / S002项目 / LC-02仪器 / 260108方法

Project name in app: "HYLY-LC-01(230106)" — full method identifier (minus lab prefix).
Group name: "410实验室", "415实验室", etc.
"""
import sqlite3
import os

DATABASE_PATH = os.path.join(os.path.dirname(__file__), "data", "workload.db")

# All groups and their projects — hardcoded, no Excel dependency
DEFAULT_DATA = [
    ("410实验室", [
        "HYLY-LC-01(230106)",
        "HYLY-LC-04(QL-230211)",
        "HYLY-LC-09(230106)",
        "YWJS-LC-11(Q002-230407)",
        "E003-LC-03(EF-241204)",
        "E003-LC-07(EF-241204)",
        "E003-GC-02甲乙醇",
        "E003-GC-04(顶空)",
    ]),
    ("415实验室", [
        "YSLY-LC-12(0909)",
        "YSLY-LC-12(YSLY-260325)",
        "YSLY-GC-02(甲乙醇)",
        "YSLY-GC-03(0816)",
    ]),
    ("417实验室", [
        "S002-LC-02(260108)",
        "S002-LC-02(0909)",
        "S002-LC-02(260325)",
        "S002-LC-02(260410)",
        "S002-LC-08(251122)",
        "S002-LC-08(251229)",
        "S002-LC-08(251230)",
        "S002-LC-08(260413)",
        "S002-GC-02(甲乙醇)",
        "S002-GC-03(0816)",
        "S002-GC-04(顶空)",
        "A003-GC-02(甲乙醇)",
        "JYAYSY-GC-03(1128)",
        "Q-LC-01(230106)",
        "Q008-LC-04(QL-260211)",
        "Q008-LC-09(230106)",
        "Q008-LC-13(PQ-SSA)",
        "Q008-GC-04(顶空)",
    ]),
    ("418实验室", [
        "三氟苯硼酸-LC-05(251108)",
        "三氟苯硼酸-GC-02(甲乙醇)",
    ]),
    ("生物合成", [
        "环糊精-LC-19(240610)",
        "F008-LC-20(250310-RID)",
        "F008-LC-16(厂区-FB-250701)",
        "F008-LC-16(F008)",
        "DC002-LC-10(250809)",
        "T001-LC-10(T001-260302)",
        "T001-LC-15(T001-260318)",
        "T001-LC-09(T001-260408)",
    ]),
    ("车间", [
        "E003-GC-01(240706)",
        "E003-GC-02(甲乙醇)",
    ]),
    ("801厂区", [
        "F002-GC-04(顶空)",
    ]),
    ("707实验室", [
        "D002-LC-10(200215)",
    ]),
]


def seed_default_data(db_path: str | None = None) -> int:
    """Seed built-in project groups and projects on first run.

    Returns the number of projects seeded.
    """
    db_path = db_path or DATABASE_PATH

    conn = sqlite3.connect(db_path)
    conn.execute("PRAGMA foreign_keys=ON")
    cursor = conn.cursor()

    # Check if already seeded
    cursor.execute("SELECT COUNT(*) FROM project_groups")
    group_count = cursor.fetchone()[0]
    if group_count > 0:
        conn.close()
        return 0

    total_projects = 0

    for sort_idx, (group_name, projects) in enumerate(DEFAULT_DATA):
        cursor.execute(
            "INSERT INTO project_groups (name, sort_order) VALUES (?, ?)",
            (group_name, sort_idx),
        )
        group_id = cursor.lastrowid

        for proj_idx, project_name in enumerate(projects):
            cursor.execute(
                "INSERT INTO projects (group_id, name, sort_order) VALUES (?, ?, ?)",
                (group_id, project_name, proj_idx),
            )
            total_projects += 1

    conn.commit()
    conn.close()
    print(f"[SEED] Built-in: {len(DEFAULT_DATA)} groups, {total_projects} projects")
    return total_projects


if __name__ == "__main__":
    seed_default_data()
