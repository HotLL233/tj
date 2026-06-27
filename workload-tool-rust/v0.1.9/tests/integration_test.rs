// Integration tests for export pipeline
// Run: cargo test --tests

use std::collections::HashMap;
use workload_tool::api::{export_data, export_write};

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
        ("Lab".into(), "Code".into(), "IC".into(), "ML".into(), 10i64, false),
        ("Lab".into(), "Code".into(), "IC2".into(), "ML2".into(), 5i64, true),
    ];
    let groups = export_write::detect_groups(&rows);
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].2, 1); // lc_count
    assert_eq!(groups[0].3, 1); // gc_count
}

#[test]
fn test_detect_groups_multi() {
    let rows = vec![
        ("LabA".into(), "Code1".into(), "IC".into(), "ML".into(), 1i64, false),
        ("LabA".into(), "Code2".into(), "IC".into(), "ML".into(), 2i64, false),
        ("LabB".into(), "Code3".into(), "IC".into(), "ML".into(), 3i64, false),
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
