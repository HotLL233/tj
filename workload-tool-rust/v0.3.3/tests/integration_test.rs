// Integration tests for export pipeline and import bugfixes
// Run: cargo test --tests

use std::collections::HashMap;
use workload_tool::api::{export_data, export_write};
use workload_tool::db::migrations;
use workload_tool::repo::method_repo;

/// Helper: create an in-memory SQLite database with all migrations applied.
fn setup_in_memory_db() -> rusqlite::Connection {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    migrations::run(&conn).unwrap();
    conn
}

/// Helper: get group_id by name from project_groups table.
fn get_group_id(conn: &rusqlite::Connection, name: &str) -> i64 {
    conn.query_row(
        "SELECT id FROM project_groups WHERE name=?1",
        rusqlite::params![name],
        |r| r.get(0),
    ).unwrap()
}

#[test]
fn test_parse_instrument_variants() {
    let (_, code, itype) = export_data::parse_instrument("HYLY-LC-01(230106)");
    assert_eq!(code, "LC-01");
    assert_eq!(itype, "液相");

    let (_, code, itype) = export_data::parse_instrument("E003-GC-02");
    assert_eq!(code, "GC-02");
    assert_eq!(itype, "气相");

    let (_, code, itype) = export_data::parse_instrument("SimpleName");
    assert_eq!(code, "");
    assert_eq!(itype, "其他");
}

#[test]
fn test_extract_code_variants() {
    assert_eq!(export_data::extract_code("HYLY-LC-01"), "HYLY");
    assert_eq!(export_data::extract_code("E003-GC-02"), "E003");
    assert_eq!(export_data::extract_code("OnlyOne"), "OnlyOne");
    assert_eq!(export_data::extract_code("A-B-C"), "A");
}

#[test]
fn test_month_bounds_normal() {
    let (s, e) = export_data::month_bounds("2026-06-15");
    assert_eq!(s, "2026-06-01");
    assert_eq!(e, "2026-07-01");
}

#[test]
fn test_month_bounds_december() {
    let (s, e) = export_data::month_bounds("2026-12-25");
    assert_eq!(s, "2026-12-01");
    assert_eq!(e, "2027-01-01");
}

#[test]
fn test_month_bounds_invalid() {
    // Invalid input falls back to default year=2026, month=1
    let (s, e) = export_data::month_bounds("not-a-date");
    assert!(s.contains("2026")); // defaults to 2026
    assert!(e.contains("2026"));
}

#[test]
fn test_week_ranges_has_data() {
    let weeks = export_data::week_ranges("2026-06-01");
    assert!(weeks.len() >= 4);
    assert!(weeks[0].0.contains("06"));
}

#[test]
fn test_method_full_name_known() {
    assert_eq!(
        export_data::get_method_full_name("410", "HYLY-LC-01(230106)"),
        "HYLY-230106-1-低温8℃-DAD"
    );
}

#[test]
fn test_method_full_name_unknown() {
    let fallback = export_data::get_method_full_name("410", "XXX-YY-ZZ");
    assert_eq!(fallback, "YY-ZZ");
}

#[test]
fn test_detect_groups_single() {
    let rows = vec![
        ("Lab".into(), "Code".into(), "IC".into(), "ML".into(), 10i64, false, 1.0f64),
        ("Lab".into(), "Code".into(), "IC2".into(), "ML2".into(), 5i64, true, 1.0f64),
    ];
    let groups = export_write::detect_groups(&rows);
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].2, 1); // lc_count
    assert_eq!(groups[0].3, 1); // gc_count
}

#[test]
fn test_detect_groups_multi() {
    let rows = vec![
        ("LabA".into(), "Code1".into(), "IC".into(), "ML".into(), 1i64, false, 1.0f64),
        ("LabA".into(), "Code2".into(), "IC".into(), "ML".into(), 2i64, false, 1.0f64),
        ("LabB".into(), "Code3".into(), "IC".into(), "ML".into(), 3i64, false, 1.0f64),
    ];
    let groups = export_write::detect_groups(&rows);
    assert_eq!(groups.len(), 3);
}

#[test]
fn test_cl_column_letters() {
    assert_eq!(export_write::_cl(0), "A");
    assert_eq!(export_write::_cl(5), "F");
    assert_eq!(export_write::_cl(8), "I");
    assert_eq!(export_write::_cl(25), "Z");
    assert_eq!(export_write::_cl(26), "AA");
}

#[test]
fn test_flat_row_order() {
    let lab_order = vec!["Lab1".to_string()];
    let mut lab_data: HashMap<String, HashMap<String, export_data::ProjectRows>> = HashMap::new();
    let mut inner = HashMap::new();
    let mut pr = export_data::ProjectRows::default();
    pr.lc.push(("LC-01".into(), "MethodA".into(), 10));
    pr.gc.push(("GC-01".into(), "MethodB".into(), 5));
    inner.insert("CODE1".into(), pr);
    lab_data.insert("Lab1".into(), inner);
    let rows = export_data::flatten_lab(&lab_order, &lab_data);
    assert_eq!(rows.len(), 2);
    assert!(!rows[0].5); // LC first
    assert!(rows[1].5);  // GC second
}

// ══════════════════════════════════════════════════════════════════════
// v0.3.3: batch_import_column_split regression tests
// Bug: 导入后 projects.group_id 错误地指向"研发项目"而非实际实验室
// Fix: L260-287 — 去重不再限制 group_id, 删除伪链接, 新增 group_id 回写
// ══════════════════════════════════════════════════════════════════════

/// 场景1: 项目在 project_lab_pairs 中有实验室关联 → group_id 应指向实验室
#[test]
fn test_import_project_with_lab_gets_correct_group_id() {
    let conn = setup_in_memory_db();

    let group_names = vec!["实验室X".to_string()];
    let project_names = vec!["项目A".to_string()];
    let method_items: Vec<(String, String, String)> = vec![];
    let project_lab_pairs = vec![("项目A".to_string(), "实验室X".to_string())];

    let summary = method_repo::batch_import_column_split(
        &conn, &group_names, &project_names, &method_items, &project_lab_pairs,
    ).unwrap();

    // 验证项目已创建
    assert_eq!(summary.total_projects, 1);
    assert_eq!(summary.total_groups, 1);

    let lab_x_id = get_group_id(&conn, "实验室X");
    let rnd_id = get_group_id(&conn, "研发项目");

    // 核心断言：项目A的 group_id 应该是 实验室X，不是 研发项目
    let actual_gid: i64 = conn.query_row(
        "SELECT group_id FROM projects WHERE name='项目A'",
        [], |r| r.get(0),
    ).unwrap();

    assert_eq!(actual_gid, lab_x_id,
        "项目A 的 group_id 应指向 实验室X({})，实际为 {}", lab_x_id, actual_gid);
    assert_ne!(actual_gid, rnd_id,
        "项目A 的 group_id 不应指向'研发项目'({})", rnd_id);
}

/// 场景2: 项目不在 project_lab_pairs 中（无实验室关联）→ group_id 保持"研发项目"
#[test]
fn test_import_project_without_lab_keeps_default_group() {
    let conn = setup_in_memory_db();

    let group_names = vec!["实验室X".to_string()];
    let project_names = vec!["项目B".to_string()];
    let method_items: Vec<(String, String, String)> = vec![];
    let project_lab_pairs: Vec<(String, String)> = vec![]; // 空：无关联

    let summary = method_repo::batch_import_column_split(
        &conn, &group_names, &project_names, &method_items, &project_lab_pairs,
    ).unwrap();

    assert_eq!(summary.total_projects, 1);

    let rnd_id = get_group_id(&conn, "研发项目");

    let actual_gid: i64 = conn.query_row(
        "SELECT group_id FROM projects WHERE name='项目B'",
        [], |r| r.get(0),
    ).unwrap();

    assert_eq!(actual_gid, rnd_id,
        "项目B 无实验室关联，group_id 应保持'研发项目'({})，实际为 {}", rnd_id, actual_gid);
}

/// 场景3: 同一项目名出现在多个 project_lab_pairs 中 → 项目只创建一次，
///        group_id 取第一个匹配的实验室
#[test]
fn test_import_duplicate_project_in_pairs_first_lab_wins() {
    let conn = setup_in_memory_db();

    let group_names = vec!["实验室X".to_string(), "实验室Y".to_string()];
    let project_names = vec!["项目C".to_string()];
    let method_items: Vec<(String, String, String)> = vec![];
    let project_lab_pairs = vec![
        ("项目C".to_string(), "实验室X".to_string()),
        ("项目C".to_string(), "实验室Y".to_string()),
    ];

    let summary = method_repo::batch_import_column_split(
        &conn, &group_names, &project_names, &method_items, &project_lab_pairs,
    ).unwrap();

    // 项目只应创建一次
    assert_eq!(summary.total_projects, 1);

    let lab_x_id = get_group_id(&conn, "实验室X");
    let lab_y_id = get_group_id(&conn, "实验室Y");

    // group_id 应指向第一个匹配的实验室（实验室X）
    let actual_gid: i64 = conn.query_row(
        "SELECT group_id FROM projects WHERE name='项目C'",
        [], |r| r.get(0),
    ).unwrap();
    assert_eq!(actual_gid, lab_x_id,
        "项目C 的 group_id 应指向第一个匹配的 实验室X({})，实际为 {}", lab_x_id, actual_gid);
    assert_ne!(actual_gid, lab_y_id,
        "项目C 的 group_id 不应指向第二个匹配的 实验室Y");

    // 但 project_lab_links 应包含两条记录（M:N 关联）
    let proj_id: i64 = conn.query_row(
        "SELECT id FROM projects WHERE name='项目C'", [], |r| r.get(0),
    ).unwrap();

    let link_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM project_lab_links WHERE project_id=?1",
        [proj_id], |r| r.get(0),
    ).unwrap();
    assert_eq!(link_count, 2,
        "project_lab_links 应包含2条记录（实验室X 和 实验室Y），实际 {}", link_count);
}

/// 场景4: project_lab_pairs 引用了不存在的实验室 → group_id 保持"研发项目"
#[test]
fn test_import_project_with_nonexistent_lab_falls_back() {
    let conn = setup_in_memory_db();

    let group_names: Vec<String> = vec![];  // 不创建任何实验室
    let project_names = vec!["项目D".to_string()];
    let method_items: Vec<(String, String, String)> = vec![];
    let project_lab_pairs = vec![("项目D".to_string(), "不存在的实验室".to_string())];

    let summary = method_repo::batch_import_column_split(
        &conn, &group_names, &project_names, &method_items, &project_lab_pairs,
    ).unwrap();

    assert_eq!(summary.total_projects, 1);

    let rnd_id = get_group_id(&conn, "研发项目");

    let actual_gid: i64 = conn.query_row(
        "SELECT group_id FROM projects WHERE name='项目D'",
        [], |r| r.get(0),
    ).unwrap();

    // 实验室不存在 → UPDATE 失败 → group_id 保持为"研发项目"
    assert_eq!(actual_gid, rnd_id,
        "项目D 的实验室不存在，group_id 应保持'研发项目'({})，实际为 {}", rnd_id, actual_gid);
}

/// 场景5: 去重逻辑 — 项目已存在时不重复创建，也不覆盖已有 group_id
#[test]
fn test_import_dedup_existing_project_not_overwritten() {
    let conn = setup_in_memory_db();

    // 第一次导入：创建项目E，关联实验室X
    let group_names1 = vec!["实验室X".to_string()];
    let project_names1 = vec!["项目E".to_string()];
    let method_items1: Vec<(String, String, String)> = vec![];
    let pairs1 = vec![("项目E".to_string(), "实验室X".to_string())];

    let s1 = method_repo::batch_import_column_split(
        &conn, &group_names1, &project_names1, &method_items1, &pairs1,
    ).unwrap();
    assert_eq!(s1.total_projects, 1);

    let lab_x_id = get_group_id(&conn, "实验室X");

    // 第二次导入：相同项目名 项目E，但这次关联实验室Y
    let group_names2 = vec!["实验室Y".to_string()];
    let project_names2 = vec!["项目E".to_string()];
    let pairs2 = vec![("项目E".to_string(), "实验室Y".to_string())];

    let s2 = method_repo::batch_import_column_split(
        &conn, &group_names2, &project_names2, &method_items1, &pairs2,
    ).unwrap();

    // 项目已存在，不应再创建
    assert_eq!(s2.total_projects, 0);

    // group_id 应保持为第一次导入时的实验室X（不被覆盖）
    let actual_gid: i64 = conn.query_row(
        "SELECT group_id FROM projects WHERE name='项目E'",
        [], |r| r.get(0),
    ).unwrap();
    assert_eq!(actual_gid, lab_x_id,
        "已存在项目的 group_id 不应被后续导入覆盖，应保持 实验室X({})", lab_x_id);

    // 但 project_lab_links 应新增实验室Y的关联
    let proj_id: i64 = conn.query_row(
        "SELECT id FROM projects WHERE name='项目E'", [], |r| r.get(0),
    ).unwrap();
    let link_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM project_lab_links WHERE project_id=?1",
        [proj_id], |r| r.get(0),
    ).unwrap();
    assert!(link_count >= 1);
}

/// 场景6: 规模验证 — 多项目混合（有/无实验室关联）
#[test]
fn test_import_mixed_projects_lab_and_no_lab() {
    let conn = setup_in_memory_db();

    let group_names = vec!["实验室A".to_string(), "实验室B".to_string()];
    let project_names = vec![
        "项目_with_lab".to_string(),
        "项目_no_lab".to_string(),
    ];
    let method_items: Vec<(String, String, String)> = vec![];
    let project_lab_pairs = vec![
        ("项目_with_lab".to_string(), "实验室A".to_string()),
    ];

    let summary = method_repo::batch_import_column_split(
        &conn, &group_names, &project_names, &method_items, &project_lab_pairs,
    ).unwrap();

    assert_eq!(summary.total_projects, 2);

    let lab_a_id = get_group_id(&conn, "实验室A");
    let rnd_id = get_group_id(&conn, "研发项目");

    // 有实验室的项目
    let gid_with: i64 = conn.query_row(
        "SELECT group_id FROM projects WHERE name='项目_with_lab'",
        [], |r| r.get(0),
    ).unwrap();
    assert_eq!(gid_with, lab_a_id);

    // 无实验室的项目
    let gid_no: i64 = conn.query_row(
        "SELECT group_id FROM projects WHERE name='项目_no_lab'",
        [], |r| r.get(0),
    ).unwrap();
    assert_eq!(gid_no, rnd_id);
}
