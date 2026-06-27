use crate::error::Result;

const DEFAULT_DATA: &[(&str, &[&str])] = &[
    ("410实验室", &[
        "HYLY-LC-01(230106)", "HYLY-LC-04(QL-230211)", "HYLY-LC-09(230106)",
        "YWJS-LC-11(250915)", "E003-LC-03(EF-241204-DAD)", "E003-LC-07(EF-241204-VWD)",
        "E003-GC-02(甲乙醇)", "E003-GC-04(顶空氯乙烷)"
    ]),
    ("415实验室", &[
        "YSLY-LC-12(T004-220909-VWD)", "YSLY-LC-12(YSLY-260325-VWD)",
        "YSLY-GC-02(甲乙醇)", "YSLY-GC-03(A003-210816)"
    ]),
    ("417实验室", &[
        "S002-LC-02(260108-DAD)", "S002-LC-02(T004-220909)", "S002-LC-02(260325)",
        "S002-LC-02(260410)", "S002-LC-08(SFBYS-251122)", "S002-LC-08(A003-251229)",
        "S002-LC-08(A003-251230)", "S002-LC-08(260413)", "S002-GC-02(甲乙醇)",
        "S002-GC-03(A003-210816)", "S002-GC-04(顶空氯乙烷)", "A003-GC-02(甲乙醇)",
        "JYAYSY-GC-03(251128)", "Q-LC-01(230106)", "Q008-LC-04(QL-260211)",
        "Q008-LC-09(230106)", "Q008-LC-11(Q002-230407)", "Q008-GC-04(顶空氯乙烷)"
    ]),
    ("418实验室", &[
        "三氟苯硼酸-LC-05(251108)", "三氟苯硼酸-GC-02(甲乙醇)"
    ]),
    ("生物合成", &[
        "环糊精-LC-19(240610)", "环糊精-LC-20(250310-RID)",
        "F008-LC-10(氟苯尼考)", "F008-LC-16(厂区-FB-250701)",
        "DC002-LC-10(250809)", "T001-LC-15(260302)", "T001-LC-15(260318)", "T001-LC-09(260408)"
    ]),
    ("车间", &["E003-GC-01(J001-240706)", "E003-GC-02(甲乙醇)"]),
    ("801厂区", &["F002-GC-04(顶空氯乙烷)"]),
    ("707实验室", &["D002-LC-10"]),
];

pub fn ensure_seeded(conn: &rusqlite::Connection) -> Result<()> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM project_groups", [], |r| r.get(0))?;
    if count > 0 { return Ok(()); }

    for (gi, (group_name, projects)) in DEFAULT_DATA.iter().enumerate() {
        conn.execute(
            "INSERT INTO project_groups (name, sort_order) VALUES (?1, ?2)",
            (group_name, gi as i64),
        )?;
        let group_id = conn.last_insert_rowid();
        for (pi, proj) in projects.iter().enumerate() {
            conn.execute(
                "INSERT INTO projects (group_id, name, sort_order) VALUES (?1, ?2, ?3)",
                rusqlite::params!(group_id, *proj, pi as i64),
            )?;
        }
    }
    Ok(())
}
