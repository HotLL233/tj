use std::collections::HashMap;
use rusqlite::Connection;
use crate::error::Result;

pub fn parse_instrument(project_name: &str) -> (&str, &str, &str) {
    if let Some(dash_pos) = project_name.find('-') {
        let after = &project_name[dash_pos + 1..];
        if after.starts_with("LC") || after.starts_with("GC") {
            if let Some(code_end) = after.find(|c: char| !c.is_alphanumeric() && c != '-') {
                let code = &after[..code_end];
                let itype = if code.starts_with("LC") { "液相" } else { "气相" };
                return (&after[code_end..], code, itype);
            }
            let itype = if after.starts_with("LC") { "液相" } else { "气相" };
            return (after, after, itype);
        }
    }
    (project_name, "", "其他")
}

pub fn extract_code(n: &str) -> &str { n.split('-').next().unwrap_or(n) }

pub fn get_method_full_name(_group: &str, project: &str) -> String {
    let key = project;
    if key.contains("HYLY-LC-01") { return "HYLY-230106-1-低温8℃-DAD".into(); }
    if key.contains("HYLY-LC-04") { return "QL-260211-DAD".into(); }
    if key.contains("HYLY-LC-09") { return "HYLY-230106-1-低温8℃-DAD".into(); }
    if key.contains("YWJS-LC-11") { return "Q002-230407-VWD".into(); }
    if key.contains("E003-LC-03") { return "EF-241204(38min)-DAD".into(); }
    if key.contains("E003-LC-07") { return "EF-241204(38min)-VWD".into(); }
    if key.contains("E003-GC-02") { return "甲乙醇".into(); }
    if key.contains("E003-GC-04") { return "顶空氯乙烷-乙醇200119-1".into(); }
    if key.contains("YSLY-LC-12") && key.contains("T004") { return "T004-220909-VWD".into(); }
    if key.contains("YSLY-LC-12") && key.contains("260325") { return "YSLY-260325-VWD".into(); }
    if key.contains("YSLY-GC-02") { return "甲乙醇".into(); }
    if key.contains("YSLY-GC-03") { return "A003-210816".into(); }
    let parts: Vec<&str> = project.splitn(2, '-').collect();
    if parts.len() > 1 { parts[1].to_string() } else { project.to_string() }
}

pub fn month_bounds(ref_date: &str) -> (String, String) {
    let parts: Vec<&str> = ref_date.split('-').collect();
    if parts.len() < 2 { return (ref_date.to_string(), ref_date.to_string()); }
    let year: i32 = parts[0].parse().unwrap_or(2026);
    let month: u32 = parts[1].parse().unwrap_or(1);
    let start = format!("{}-{:02}-01", year, month);
    let end = if month == 12 { format!("{}-01-01", year + 1) } else { format!("{}-{:02}-01", year, month + 1) };
    (start, end)
}

pub fn week_ranges(start_ymd: &str) -> Vec<(String, String, String)> {
    use chrono::{NaiveDate, Datelike};
    let parts: Vec<&str> = start_ymd.split('-').collect();
    if parts.len() < 2 { return vec![]; }
    let year: i32 = parts[0].parse().unwrap_or(2026);
    let month: u32 = parts[1].parse().unwrap_or(1);
    let first = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let wd = first.weekday().num_days_from_monday() as i64;
    let mut cur = first - chrono::Duration::days(wd);
    let last_day = if month == 12 { NaiveDate::from_ymd_opt(year+1,1,1).unwrap()-chrono::Duration::days(1) }
                  else { NaiveDate::from_ymd_opt(year,month+1,1).unwrap()-chrono::Duration::days(1) };
    let mut weeks = vec![];
    while cur <= last_day {
        let we = (cur + chrono::Duration::days(6)).min(last_day);
        weeks.push((format!("{:02}.{:02}-{:02}.{:02}",cur.month(),cur.day(),we.month(),we.day()),
            cur.format("%Y-%m-%d").to_string(), we.format("%Y-%m-%d").to_string()));
        cur = we + chrono::Duration::days(1);
    }
    weeks
}

#[derive(Default, Clone)]
pub struct ProjectRows { pub coeff: f64, pub lc: Vec<(String, String, i64)>, pub gc: Vec<(String, String, i64)> }

pub type FlatRow = (String, String, String, String, i64, bool, f64);

pub fn build_lab_hierarchy(conn: &Connection, group_id: Option<i64>, d_start: &str, d_end: &str) -> Result<(Vec<String>, HashMap<String, HashMap<String, ProjectRows>>)> {
    let mut proj_sql = String::from("SELECT p.id, p.name AS pn, pg.name AS gn, p.coefficient FROM projects p JOIN project_groups pg ON p.group_id=pg.id WHERE p.is_active=1");
    let mut pp: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];
    if let Some(gid) = group_id { proj_sql.push_str(" AND pg.id=?1"); pp.push(Box::new(gid)); }
    proj_sql.push_str(" ORDER BY pg.sort_order, p.sort_order");
    let mut stmt = conn.prepare(&proj_sql)?;
    let ap: Vec<(i64, String, String, f64)> = stmt.query_map(
        rusqlite::params_from_iter(pp.iter().map(|p| p.as_ref())),
        |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, f64>(3).unwrap_or(1.0))),
    )?.collect::<std::result::Result<Vec<_>, _>>()?;

    let mut rec_sql = String::from("SELECT p.id, SUM(wr.quantity) FROM work_records wr JOIN projects p ON wr.project_id=p.id JOIN project_groups pg ON p.group_id=pg.id WHERE wr.deleted_at IS NULL AND wr.recorded_at>=?1 AND wr.recorded_at<?2");
    let mut rp: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(d_start.to_string()), Box::new(d_end.to_string())];
    if let Some(gid) = group_id { rec_sql.push_str(" AND pg.id=?3"); rp.push(Box::new(gid)); }
    rec_sql.push_str(" GROUP BY p.id");
    let mut stmt = conn.prepare(&rec_sql)?;
    let rec_map: HashMap<i64, i64> = stmt.query_map(
        rusqlite::params_from_iter(rp.iter().map(|p| p.as_ref())), |row| Ok((row.get::<_,i64>(0)?, row.get::<_,i64>(1)?)),
    )?.filter_map(|r| r.ok()).collect();

    let mut lab_order: Vec<String> = vec![];
    let mut lab_data: HashMap<String, HashMap<String, ProjectRows>> = HashMap::new();
    for (pid, pn, gn, coeff) in &ap {
        let pc = extract_code(pn).to_string();
        let (_mb, ic, it) = parse_instrument(pn);
        let fn_name = get_method_full_name(gn, pn);
        let qty = rec_map.get(pid).copied().unwrap_or(0);
        lab_data.entry(gn.clone()).or_insert_with(|| { lab_order.push(gn.clone()); HashMap::new() });
        lab_data.get_mut(gn).unwrap().entry(pc.clone()).or_insert_with(|| ProjectRows { coeff: *coeff, ..Default::default() });
        let inst = lab_data.get_mut(gn).unwrap().get_mut(&pc).unwrap();
        if it == "气相" { inst.gc.push((ic.to_string(), fn_name, qty)); }
        else { inst.lc.push((ic.to_string(), fn_name, qty)); }
    }
    Ok((lab_order, lab_data))
}

pub fn flatten_lab(lab_order: &[String], lab_data: &HashMap<String, HashMap<String, ProjectRows>>) -> Vec<FlatRow> {
    let mut rows = vec![];
    for lab in lab_order {
        for (code, inst) in &lab_data[lab] {
            let coeff = inst.coeff;
            for (ic, ml, q) in &inst.lc { rows.push((lab.clone(), code.clone(), ic.clone(), ml.clone(), *q, false, coeff)); }
            for (ic, ml, q) in &inst.gc { rows.push((lab.clone(), code.clone(), ic.clone(), ml.clone(), *q, true, coeff)); }
        }
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_parse_instrument_lc() { let (_, code, itype) = parse_instrument("HYLY-LC-01(230106)"); assert_eq!(code, "LC-01"); assert_eq!(itype, "液相"); }
    #[test] fn test_parse_instrument_gc() { let (_, code, itype) = parse_instrument("E003-GC-02"); assert_eq!(code, "GC-02"); assert_eq!(itype, "气相"); }
    #[test] fn test_parse_instrument_other() { let (_, code, itype) = parse_instrument("SomeProject"); assert_eq!(code, ""); assert_eq!(itype, "其他"); }
    #[test] fn test_extract_code() { assert_eq!(extract_code("HYLY-LC-01"), "HYLY"); assert_eq!(extract_code("E003-GC-02"), "E003"); }
    #[test] fn test_month_bounds() { let (s, e) = month_bounds("2026-06-15"); assert_eq!(s, "2026-06-01"); assert_eq!(e, "2026-07-01"); }
    #[test] fn test_month_bounds_december() { let (s, e) = month_bounds("2026-12-10"); assert_eq!(s, "2026-12-01"); assert_eq!(e, "2027-01-01"); }
    #[test] fn test_week_ranges() { let weeks = week_ranges("2026-06-01"); assert!(!weeks.is_empty()); assert!(weeks[0].0.contains("06")); }
    #[test] fn test_method_full_name_known() { assert_eq!(get_method_full_name("410", "HYLY-LC-01(230106)"), "HYLY-230106-1-低温8℃-DAD"); }
}
