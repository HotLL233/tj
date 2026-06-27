use axum::{extract::{Query, State}, Router, routing::get};
use axum::response::IntoResponse;
use axum::http::header;
use serde::Deserialize;
use std::collections::HashMap;
use crate::db::DbPool;
use crate::error::{Result, AppError};

#[derive(Deserialize)]
pub struct ExportQuery {
    pub start: Option<String>,
    pub end: Option<String>,
    pub group_id: Option<i64>,
}

pub fn router(pool: DbPool) -> Router {
    Router::new()
        .route("/api/export/excel", get(export_excel))
        .with_state(pool)
}

// ─── helpers ────────────────────────────────────────────────────

fn _cl(n: u16) -> String {
    let letters = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut n = n + 1;
    let mut result = String::new();
    while n > 0 {
        n -= 1;
        result.insert(0, letters.chars().nth((n % 26) as usize).unwrap());
        n /= 26;
    }
    result
}

fn parse_instrument(project_name: &str) -> (&str, &str, &str) {
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

fn extract_code(n: &str) -> &str { n.split('-').next().unwrap_or(n) }

fn get_method_full_name(_group: &str, project: &str) -> String {
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

fn month_bounds(ref_date: &str) -> (String, String) {
    let parts: Vec<&str> = ref_date.split('-').collect();
    if parts.len() < 2 { return (ref_date.to_string(), ref_date.to_string()); }
    let year: i32 = parts[0].parse().unwrap_or(2026);
    let month: u32 = parts[1].parse().unwrap_or(1);
    let start = format!("{}-{:02}-01", year, month);
    let end = if month == 12 {
        format!("{}-01-01", year + 1)
    } else {
        format!("{}-{:02}-01", year, month + 1)
    };
    (start, end)
}

fn week_ranges(start_ymd: &str) -> Vec<(String, String, String)> {
    use chrono::{NaiveDate, Datelike};
    let parts: Vec<&str> = start_ymd.split('-').collect();
    if parts.len() < 2 { return vec![]; }
    let year: i32 = parts[0].parse().unwrap_or(2026);
    let month: u32 = parts[1].parse().unwrap_or(1);
    let first = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let wd = first.weekday().num_days_from_monday() as i64;
    let mut cur = first - chrono::Duration::days(wd);
    let last_day = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap() - chrono::Duration::days(1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap() - chrono::Duration::days(1)
    };
    let mut weeks = vec![];
    while cur <= last_day {
        let we = (cur + chrono::Duration::days(6)).min(last_day);
        let label = format!("{}.{:02}-{}.{:02}", cur.month(), cur.day(), we.month(), we.day());
        weeks.push((label, cur.format("%Y-%m-%d").to_string(), we.format("%Y-%m-%d").to_string()));
        cur = we + chrono::Duration::days(1);
    }
    weeks
}

// ─── structs ─────────────────────────────────────────────────────

#[derive(Default, Clone)]
struct ProjectRows { lc: Vec<(String, String, i64)>, gc: Vec<(String, String, i64)> }

/// Flattened row: (level1, level2, instrument, method, qty, is_gc)
type FlatRow = (String, String, String, String, i64, bool);

fn build_lab_hierarchy(
    conn: &rusqlite::Connection, group_id: Option<i64>, d_start: &str, d_end: &str,
) -> Result<(Vec<String>, HashMap<String, HashMap<String, ProjectRows>>)> {
    let mut proj_sql = String::from(
        "SELECT p.id, p.name AS pn, pg.name AS gn FROM projects p JOIN project_groups pg ON p.group_id=pg.id WHERE p.is_active=1"
    );
    let mut pp: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];
    if let Some(gid) = group_id { proj_sql.push_str(" AND pg.id=?1"); pp.push(Box::new(gid)); }
    proj_sql.push_str(" ORDER BY pg.sort_order, p.sort_order");
    let mut stmt = conn.prepare(&proj_sql)?;
    let ap: Vec<(i64, String, String)> = stmt.query_map(
        rusqlite::params_from_iter(pp.iter().map(|p| p.as_ref())),
        |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?)),
    )?.collect::<std::result::Result<Vec<_>, _>>()?;

    let mut rec_sql = String::from(
        "SELECT p.id, SUM(wr.quantity) FROM work_records wr JOIN projects p ON wr.project_id=p.id JOIN project_groups pg ON p.group_id=pg.id WHERE wr.deleted_at IS NULL AND wr.recorded_at>=?1 AND wr.recorded_at<?2"
    );
    let mut rp: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(d_start.to_string()), Box::new(d_end.to_string())];
    if let Some(gid) = group_id { rec_sql.push_str(" AND pg.id=?3"); rp.push(Box::new(gid)); }
    rec_sql.push_str(" GROUP BY p.id");
    let mut stmt = conn.prepare(&rec_sql)?;
    let rec_map: HashMap<i64, i64> = stmt.query_map(
        rusqlite::params_from_iter(rp.iter().map(|p| p.as_ref())),
        |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
    )?.filter_map(|r| r.ok()).collect();

    let mut lab_order: Vec<String> = vec![];
    let mut lab_data: HashMap<String, HashMap<String, ProjectRows>> = HashMap::new();
    for (pid, pn, gn) in &ap {
        let pc = extract_code(pn).to_string();
        let (_mb, ic, it) = parse_instrument(pn);
        let fn_name = get_method_full_name(gn, pn);
        let qty = rec_map.get(pid).copied().unwrap_or(0);
        lab_data.entry(gn.clone()).or_insert_with(|| { lab_order.push(gn.clone()); HashMap::new() });
        lab_data.get_mut(gn).unwrap().entry(pc.clone()).or_default();
        let inst = lab_data.get_mut(gn).unwrap().get_mut(&pc).unwrap();
        if it == "气相" { inst.gc.push((ic.to_string(), fn_name, qty)); }
        else { inst.lc.push((ic.to_string(), fn_name, qty)); }
    }
    Ok((lab_order, lab_data))
}

fn flatten_lab(lab_order: &[String], lab_data: &HashMap<String, HashMap<String, ProjectRows>>) -> Vec<FlatRow> {
    let mut rows = vec![];
    for lab in lab_order {
        for (code, inst) in &lab_data[lab] {
            for (ic, ml, q) in &inst.lc { rows.push((lab.clone(), code.clone(), ic.clone(), ml.clone(), *q, false)); }
            for (ic, ml, q) in &inst.gc { rows.push((lab.clone(), code.clone(), ic.clone(), ml.clone(), *q, true)); }
        }
    }
    rows
}

// ─── Column constants ────────────────────────────────────────────
const CB: u16 = 1; const CC: u16 = 2; const CD: u16 = 3; const CE: u16 = 4;
const CF: u16 = 5; const CG: u16 = 6; const CH: u16 = 7; const CI: u16 = 8;
const HR: u32 = 2;

fn set_col_widths(ws: &mut rust_xlsxwriter::Worksheet) -> Result<()> {
    let w = [8.89, 24.89, 18.0, 17.44, 43.66, 19.66, 12.0, 12.0, 22.11];
    for (i, v) in w.iter().enumerate() { ws.set_column_width(i as u16, *v)?; }
    Ok(())
}

struct Fmt { fh: rust_xlsxwriter::Format, fd: rust_xlsxwriter::Format }

impl Fmt {
    fn new() -> Self {
        use rust_xlsxwriter::*;
        Self { fh: Format::new().set_bold().set_font_size(16), fd: Format::new().set_font_size(16) }
    }
}

/// Group indices for merge detection: Vec of (start_row_inclusive, end_row_inclusive, lc_count, gc_count)
fn detect_groups(rows: &[FlatRow]) -> Vec<(u32, u32, u32, u32)> {
    let mut groups: Vec<(u32, u32, u32, u32)> = vec![];
    let mut i = 0usize;
    while i < rows.len() {
        let (ref_l1, ref_l2, _, _, _, _) = &rows[i];
        let gs = (HR + 1) as u32 + i as u32;
        let mut ge = gs;
        let mut lc = 0u32; let mut gc = 0u32;
        while i < rows.len() {
            let (l1, l2, _, _, _, is_gc) = &rows[i];
            if l1 != ref_l1 || l2 != ref_l2 { break; }
            if *is_gc { gc += 1; } else { lc += 1; }
            ge = (HR + 1) as u32 + i as u32;
            i += 1;
        }
        groups.push((gs, ge, lc, gc));
    }

    // Also merge l1 column: find contiguous l1 blocks
    let mut l1_blocks: Vec<(u32, u32)> = vec![];
    let mut j = 0usize;
    while j < rows.len() {
        let ref_l1 = &rows[j].0;
        let bs = (HR + 1) as u32 + j as u32;
        let mut be = bs;
        while j < rows.len() && &rows[j].0 == ref_l1 { be = (HR + 1) as u32 + j as u32; j += 1; }
        l1_blocks.push((bs, be));
    }
    // Return groups + l1_blocks as a combined vec; l1_blocks are separate
    // We'll handle l1 merge in the writing logic directly.
    // For now, store l1 info in the groups for later or return as tuple.
    // Actually, the l1 merge is separate. Let me handle this differently.
    groups
}

fn write_tree_sheet(
    ws: &mut rust_xlsxwriter::Worksheet, rows: &[FlatRow],
    l1_hdr: &str, l2_hdr: &str, qty_hdr: &str, fmt: &Fmt,
) -> Result<()> {
    // Headers
    let hdrs = [l1_hdr, l2_hdr, "液相仪器", "检测方法", qty_hdr, "液相检测量", "气相检测量", "项目检测总量"];
    for (i, h) in hdrs.iter().enumerate() {
        ws.write_with_format(HR, (i + 1) as u16, *h, &fmt.fh)?;
    }

    // Detect groups for merge
    let groups = detect_groups(rows);

    // Merge l2(C) + G/H/I for each group
    for &(gs, ge, _, _) in &groups {
        if ge > gs {
            ws.merge_range(gs, CC, ge, CC, "", &fmt.fd)?;
            for col in [CG, CH, CI] { ws.merge_range(gs, col, ge, col, "", &fmt.fd)?; }
        }
    }
    // Merge l1(B): contiguous same-l1
    let mut i = 0usize;
    while i < rows.len() {
        let ref_l1 = &rows[i].0;
        let bs = (HR + 1) as u32 + i as u32;
        let mut be = bs;
        while i < rows.len() && &rows[i].0 == ref_l1 { be = (HR + 1) as u32 + i as u32; i += 1; }
        if be > bs { ws.merge_range(bs, CB, be, CB, "", &fmt.fd)?; }
    }

    // Write data
    let mut ri = HR + 1;
    for (l1, l2, ic, ml, qty, _) in rows {
        ws.write_with_format(ri, CB, l1.as_str(), &fmt.fd)?;
        ws.write_with_format(ri, CC, l2.as_str(), &fmt.fd)?;
        ws.write_with_format(ri, CD, ic.as_str(), &fmt.fd)?;
        ws.write_with_format(ri, CE, ml.as_str(), &fmt.fd)?;
        if *qty > 0 { ws.write_with_format(ri, CF, *qty as f64, &fmt.fd)?; }
        ri += 1;
    }

    // Formulas
    for &(gs, ge, lc_cnt, gc_cnt) in &groups {
        if lc_cnt > 0 {
            let lc_last = gs + lc_cnt - 1;
            ws.write_formula(gs, CG, format!("=SUM({}{}:{}{})", _cl(CF), gs, _cl(CF), lc_last).as_str())?;
        }
        if gc_cnt > 0 {
            let gc_start = gs + lc_cnt;
            ws.write_formula(gs, CH, format!("=SUM({}{}:{}{})", _cl(CF), gc_start, _cl(CF), ge).as_str())?;
        }
        ws.write_formula(gs, CI, format!("=SUM({}{}:{}{})", _cl(CG), gs, _cl(CH), gs).as_str())?;
    }

    // Total row
    let tr = ri;
    ws.write_with_format(tr, CB, "总计", &fmt.fd)?;
    for col in [CC, CD, CE] { ws.write_with_format(tr, col, "", &fmt.fd)?; }
    let dr = format!("{}:{}", HR + 1, tr - 1);
    for col in [CF, CG, CH, CI] {
        ws.write_formula(tr, col, format!("=SUM({}{})", _cl(col), &dr).as_str())?;
    }
    ws.set_freeze_panes(HR + 1, 4)?;
    Ok(())
}

// ─── Sheet 1: 月-汇总 ────────────────────────────────────────────

fn build_monthly(ws: &mut rust_xlsxwriter::Worksheet, conn: &rusqlite::Connection, start: Option<&str>, group_id: Option<i64>) -> Result<String> {
    ws.set_name("月-汇总").map_err(|e| AppError::Internal(e.to_string()))?;
    ws.set_tab_color(rust_xlsxwriter::Color::RGB(0x1976D2));
    set_col_widths(ws)?;
    let ref_date = start.unwrap_or("2026-06-01");
    let (d_start, d_end) = month_bounds(ref_date);
    let month_label = {
        let p: Vec<&str> = d_start.split('-').collect();
        format!("{}年{}月", p[0], p[1].trim_start_matches('0'))
    };
    let (lab_order, lab_data) = build_lab_hierarchy(conn, group_id, &d_start, &d_end)?;
    let rows = flatten_lab(&lab_order, &lab_data);
    let fmt = Fmt::new();
    write_tree_sheet(ws, &rows, "使用实验室", "项目代号", "月检测数量", &fmt)?;
    Ok(month_label)
}

// ─── Sheet 2: 每日工作量 ─────────────────────────────────────────

fn build_daily(ws: &mut rust_xlsxwriter::Worksheet, conn: &rusqlite::Connection, start: Option<&str>, end: Option<&str>, group_id: Option<i64>) -> Result<()> {
    ws.set_name("每日工作量").map_err(|e| AppError::Internal(e.to_string()))?;
    ws.set_tab_color(rust_xlsxwriter::Color::RGB(0x43A047));
    set_col_widths(ws)?;
    let fmt = Fmt::new();
    let s = start.unwrap_or("2026-01-01");
    let e = end.unwrap_or("2026-12-31");
    let e_closed = format!("{}T23:59:59", e);

    let mut sql = String::from(
        "SELECT date(wr.recorded_at) AS wd, pg.name AS gn, p.name AS pn, SUM(wr.quantity) AS qty
         FROM work_records wr JOIN projects p ON wr.project_id=p.id JOIN project_groups pg ON p.group_id=pg.id
         WHERE wr.deleted_at IS NULL AND wr.recorded_at>=?1 AND wr.recorded_at<=?2"
    );
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(s.to_string()), Box::new(e_closed)];
    if let Some(gid) = group_id { sql.push_str(" AND pg.id=?3"); params.push(Box::new(gid)); }
    sql.push_str(" GROUP BY date(wr.recorded_at), p.id ORDER BY wd, pg.sort_order, p.sort_order");
    let mut stmt = conn.prepare(&sql)?;
    let raw: Vec<(String, String, String, i64)> = stmt.query_map(
        rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())),
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    )?.collect::<std::result::Result<Vec<_>, _>>()?;

    let mut date_order: Vec<String> = vec![];
    let mut date_data: HashMap<String, HashMap<String, HashMap<String, ProjectRows>>> = HashMap::new();
    for (wd, gn, pn, qty) in &raw {
        let pc = extract_code(pn).to_string();
        let (_mb, ic, it) = parse_instrument(pn);
        let fn_name = get_method_full_name(gn, pn);
        if !date_data.contains_key(wd) { date_data.insert(wd.clone(), HashMap::new()); date_order.push(wd.clone()); }
        date_data.get_mut(wd).unwrap().entry(gn.clone()).or_default().entry(pc.clone()).or_default();
        let inst = date_data.get_mut(wd).unwrap().get_mut(gn).unwrap().get_mut(&pc).unwrap();
        if it == "气相" { inst.gc.push((ic.to_string(), fn_name, *qty)); }
        else { inst.lc.push((ic.to_string(), fn_name, *qty)); }
    }

    let mut rows: Vec<FlatRow> = vec![];
    for wd in &date_order {
        for gn in date_data[wd].keys() {
            for (pc, pr) in &date_data[wd][gn] {
                for (ic, ml, q) in &pr.lc { rows.push((wd.clone(), format!("{}/{}", gn, pc), ic.clone(), ml.clone(), *q, false)); }
                for (ic, ml, q) in &pr.gc { rows.push((wd.clone(), format!("{}/{}", gn, pc), ic.clone(), ml.clone(), *q, true)); }
            }
        }
    }
    write_tree_sheet(ws, &rows, "日期", "实验室/项目代号", "日检测数量", &fmt)?;
    Ok(())
}

// ─── Sheet 3: 每周工作量 ─────────────────────────────────────────

fn build_weekly(ws: &mut rust_xlsxwriter::Worksheet, conn: &rusqlite::Connection, start: Option<&str>, group_id: Option<i64>) -> Result<()> {
    ws.set_name("每周工作量").map_err(|e| AppError::Internal(e.to_string()))?;
    ws.set_tab_color(rust_xlsxwriter::Color::RGB(0xFF9800));
    set_col_widths(ws)?;
    let fmt = Fmt::new();
    let ref_date = start.unwrap_or("2026-06-01");
    let (d_start, d_end) = month_bounds(ref_date);
    let weeks = week_ranges(&d_start);
    if weeks.is_empty() { return Ok(()); }
    let nw = weeks.len();

    let (lab_order, lab_data) = build_lab_hierarchy(conn, group_id, &d_start, &d_end)?;
    let rows = flatten_lab(&lab_order, &lab_data);

    // Query weekly breakdown
    let mut rec_sql = String::from(
        "SELECT p.id, date(wr.recorded_at) AS wd, SUM(wr.quantity) AS qty
         FROM work_records wr JOIN projects p ON wr.project_id=p.id JOIN project_groups pg ON p.group_id=pg.id
         WHERE wr.deleted_at IS NULL AND wr.recorded_at>=?1 AND wr.recorded_at<?2"
    );
    let mut rp: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(d_start.clone()), Box::new(d_end.clone())];
    if let Some(gid) = group_id { rec_sql.push_str(" AND pg.id=?3"); rp.push(Box::new(gid)); }
    rec_sql.push_str(" GROUP BY p.id, date(wr.recorded_at)");
    let mut stmt = conn.prepare(&rec_sql)?;
    let raw: Vec<(i64, String, i64)> = stmt.query_map(
        rusqlite::params_from_iter(rp.iter().map(|p| p.as_ref())),
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )?.collect::<std::result::Result<Vec<_>, _>>()?;

    // proj_id → Vec<week_qty>
    let mut pw: HashMap<i64, Vec<i64>> = HashMap::new();
    for (pid, wd_s, qty) in &raw {
        pw.entry(*pid).or_insert_with(|| vec![0i64; nw]);
        let wk = pw.get_mut(pid).unwrap();
        if let Ok(d) = chrono::NaiveDate::parse_from_str(wd_s, "%Y-%m-%d") {
            for (wi, (_, ws_s, we_s)) in weeks.iter().enumerate() {
                if let (Ok(ws), Ok(we)) = (chrono::NaiveDate::parse_from_str(ws_s, "%Y-%m-%d"), chrono::NaiveDate::parse_from_str(we_s, "%Y-%m-%d")) {
                    if d >= ws && d <= we { wk[wi] += qty; break; }
                }
            }
        }
    }

    // Query all project ids to match with rows
    let mut pid_sql = String::from("SELECT p.id, p.name, pg.name FROM projects p JOIN project_groups pg ON p.group_id=pg.id WHERE p.is_active=1");
    let mut pid_params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];
    if let Some(gid) = group_id { pid_sql.push_str(" AND pg.id=?1"); pid_params.push(Box::new(gid)); }
    pid_sql.push_str(" ORDER BY pg.sort_order, p.sort_order");
    let mut stmt = conn.prepare(&pid_sql)?;
    let all_pids: Vec<(i64, String, String)> = stmt.query_map(
        rusqlite::params_from_iter(pid_params.iter().map(|p| p.as_ref())),
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )?.collect::<std::result::Result<Vec<_>, _>>()?;

    // Map: (lab, code, instrument_type+instrument) → Vec<week_qty>
    let mut row_wk: Vec<Vec<i64>> = vec![];
    for (pid, _pn, _gn) in &all_pids {
        row_wk.push(pw.get(pid).cloned().unwrap_or_else(|| vec![0; nw]));
    }

    // Write headers
    let hdrs: [&str; 8] = ["使用实验室","项目代号","液相仪器","检测方法","周检测数量","液相检测量","气相检测量","项目检测总量"];
    for (i, h) in hdrs.iter().enumerate() { ws.write_with_format(HR, (i + 1) as u16, *h, &fmt.fh)?; }
    // Week sub-headers
    for (wi, (label, _, _)) in weeks.iter().enumerate() {
        let col = CI + 1 + wi as u16;
        ws.write_with_format(HR, col, label.as_str(), &fmt.fh)?;
        ws.set_column_width(col, 8.0)?;
    }

    // Detect groups + write data
    write_tree_sheet(ws, &rows, "使用实验室", "项目代号", "周检测数量", &fmt)?;

    // Fill weekly data (after write_tree_sheet to avoid overwrite)
    let dr = HR + 1;
    for (ri_offset, wk_qts) in row_wk.iter().enumerate() {
        for wi in 0..nw {
            let v = wk_qts[wi];
            if v > 0 { ws.write_with_format(dr + ri_offset as u32, CI + 1 + wi as u16, v as f64, &fmt.fd)?; }
        }
    }

    ws.set_freeze_panes(HR + 1, 4)?;
    Ok(())
}

// ─── Sheet 4: 原始记录 ───────────────────────────────────────────

fn build_raw(ws: &mut rust_xlsxwriter::Worksheet, conn: &rusqlite::Connection, start: Option<&str>, end: Option<&str>, group_id: Option<i64>) -> Result<()> {
    ws.set_name("原始记录").map_err(|e| AppError::Internal(e.to_string()))?;
    ws.set_tab_color(rust_xlsxwriter::Color::RGB(0x9C27B0));
    let fmt = Fmt::new();
    let s = start.unwrap_or("2026-01-01");
    let e = end.unwrap_or("2026-12-31");
    let e_sql = format!("{} 23:59:59", e);

    let mut sql = String::from(
        "SELECT pg.name AS gn, p.name AS pn, wr.user_name, wr.quantity, wr.recorded_at
         FROM work_records wr JOIN projects p ON wr.project_id=p.id JOIN project_groups pg ON p.group_id=pg.id
         WHERE wr.deleted_at IS NULL AND wr.recorded_at>=?1 AND wr.recorded_at<=?2"
    );
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(s.to_string()), Box::new(e_sql)];
    if let Some(gid) = group_id { sql.push_str(" AND pg.id=?3"); params.push(Box::new(gid)); }
    sql.push_str(" ORDER BY wr.recorded_at, pg.sort_order, p.sort_order");
    let mut stmt = conn.prepare(&sql)?;
    let rows: Vec<(String, String, String, i64, String)> = stmt.query_map(
        rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())),
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
    )?.collect::<std::result::Result<Vec<_>, _>>()?;

    let hdrs = ["序号","日期","实验室","项目名称","仪器","检测方法","仪器类型","数量","录入人"];
    let col_w = [6.0, 12.0, 14.0, 30.0, 12.0, 30.0, 8.0, 8.0, 12.0];
    for (i, h) in hdrs.iter().enumerate() { ws.write_with_format(0, i as u16, *h, &fmt.fh)?; ws.set_column_width(i as u16, col_w[i])?; }

    for (i, (gn, pn, un, qty, ts)) in rows.iter().enumerate() {
        let ri = (i + 1) as u32;
        let (_mb, ic, it) = parse_instrument(pn);
        let rd = &ts[..ts.len().min(10)];
        let seq = &(i + 1).to_string();
        let qs = &qty.to_string();
        let vals: [&dyn std::fmt::Display; 9] = [seq, &rd, gn, pn, &ic, &get_method_full_name(gn, pn), &it, qs, un];
        for (ci, v) in vals.iter().enumerate() {
            ws.write(ri, ci as u16, v.to_string())?;
        }
    }
    ws.set_freeze_panes(1, 0)?;
    Ok(())
}

// ─── Sheet 5: 用户统计 ───────────────────────────────────────────

fn build_user_stats(ws: &mut rust_xlsxwriter::Worksheet, conn: &rusqlite::Connection, start: Option<&str>, end: Option<&str>, group_id: Option<i64>) -> Result<()> {
    ws.set_name("用户统计").map_err(|e| AppError::Internal(e.to_string()))?;
    ws.set_tab_color(rust_xlsxwriter::Color::RGB(0x1976D2));
    set_col_widths(ws)?;
    let fmt = Fmt::new();
    let s = start.unwrap_or("2026-01-01");
    let e = end.unwrap_or("2026-12-31");
    let e_closed = format!("{}T23:59:59", e);

    let mut sql = String::from(
        "SELECT wr.user_name, pg.name AS gn, p.name AS pn, SUM(wr.quantity) AS qty
         FROM work_records wr JOIN projects p ON wr.project_id=p.id JOIN project_groups pg ON p.group_id=pg.id
         WHERE wr.deleted_at IS NULL AND wr.recorded_at>=?1 AND wr.recorded_at<=?2"
    );
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(s.to_string()), Box::new(e_closed)];
    if let Some(gid) = group_id { sql.push_str(" AND pg.id=?3"); params.push(Box::new(gid)); }
    sql.push_str(" GROUP BY wr.user_name, p.id ORDER BY wr.user_name, pg.sort_order, p.sort_order");
    let mut stmt = conn.prepare(&sql)?;
    let raw: Vec<(String, String, String, i64)> = stmt.query_map(
        rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())),
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    )?.collect::<std::result::Result<Vec<_>, _>>()?;

    let mut user_order: Vec<String> = vec![];
    let mut user_data: HashMap<String, HashMap<String, HashMap<String, ProjectRows>>> = HashMap::new();
    for (un, gn, pn, qty) in &raw {
        let pc = extract_code(pn).to_string();
        let (_mb, ic, it) = parse_instrument(pn);
        let fn_name = get_method_full_name(gn, pn);
        if !user_data.contains_key(un) { user_data.insert(un.clone(), HashMap::new()); user_order.push(un.clone()); }
        user_data.get_mut(un).unwrap().entry(gn.clone()).or_default().entry(pc.clone()).or_default();
        let inst = user_data.get_mut(un).unwrap().get_mut(gn).unwrap().get_mut(&pc).unwrap();
        if it == "气相" { inst.gc.push((ic.to_string(), fn_name, *qty)); }
        else { inst.lc.push((ic.to_string(), fn_name, *qty)); }
    }

    let mut rows: Vec<FlatRow> = vec![];
    for un in &user_order {
        for gn in user_data[un].keys() {
            for (pc, pr) in &user_data[un][gn] {
                for (ic, ml, q) in &pr.lc { rows.push((un.clone(), format!("{}/{}", gn, pc), ic.clone(), ml.clone(), *q, false)); }
                for (ic, ml, q) in &pr.gc { rows.push((un.clone(), format!("{}/{}", gn, pc), ic.clone(), ml.clone(), *q, true)); }
            }
        }
    }
    write_tree_sheet(ws, &rows, "用户名", "实验室/项目代号", "月检测数量", &fmt)?;
    Ok(())
}

// ─── Entry point ─────────────────────────────────────────────────

async fn export_excel(
    State(pool): State<DbPool>,
    Query(q): Query<ExportQuery>,
) -> Result<impl IntoResponse> {
    use rust_xlsxwriter::*;
    use std::io::Cursor;

    let conn = pool.get()?;
    let mut wb = Workbook::new();

    let ws1 = wb.add_worksheet();
    let month_label = build_monthly(ws1, &conn, q.start.as_deref(), q.group_id)?;

    let ws2 = wb.add_worksheet();
    build_daily(ws2, &conn, q.start.as_deref(), q.end.as_deref(), q.group_id)?;

    let ws3 = wb.add_worksheet();
    build_weekly(ws3, &conn, q.start.as_deref(), q.group_id)?;

    let ws4 = wb.add_worksheet();
    build_raw(ws4, &conn, q.start.as_deref(), q.end.as_deref(), q.group_id)?;

    let ws5 = wb.add_worksheet();
    build_user_stats(ws5, &conn, q.start.as_deref(), q.end.as_deref(), q.group_id)?;

    let mut buf = Cursor::new(Vec::new());
    wb.save_to_writer(&mut buf).map_err(|e| AppError::Internal(e.to_string()))?;
    let data = buf.into_inner();

    let filename = format!("attachment; filename*=UTF-8''{}",
        url_escape::encode_component(&format!("工作量统计_{}.xlsx", month_label)));
    Ok(axum::response::Response::builder()
        .header(header::CONTENT_TYPE, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
        .header(header::CONTENT_DISPOSITION, filename)
        .body(axum::body::Body::from(data))
        .unwrap())
}
