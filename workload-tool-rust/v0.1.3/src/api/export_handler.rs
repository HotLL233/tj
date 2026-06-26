use axum::{extract::{Query, State}, Router, routing::get};
use axum::response::IntoResponse;
use axum::http::header;
use serde::Deserialize;
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

/// 0-based column index → Excel column letter (0=A, 1=B, ...)
fn _cl(n: u16) -> String {
    let letters = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut n = n + 1; // convert to 1-based
    let mut result = String::new();
    while n > 0 {
        n -= 1;
        result.insert(0, letters.chars().nth((n % 26) as usize).unwrap());
        n /= 26;
    }
    result
}

fn parse_instrument(project_name: &str) -> (&str, &str, &str) {
    // Parse instrument code like "HYLY-LC-01(230106)"
    if let Some(dash_pos) = project_name.find('-') {
        let after = &project_name[dash_pos + 1..];
        if after.starts_with("LC") || after.starts_with("GC") {
            if let Some(code_end) = after.find(|c: char| !c.is_alphanumeric() && c != '-') {
                let code = &after[..code_end];
                let suffix = &after[code_end..];
                let method = &project_name[..dash_pos];
                let itype = if code.starts_with("LC") { "液相" } else { "气相" };
                let mb = if suffix.starts_with('(') {
                    &suffix[1..suffix.len().min(suffix.len().saturating_sub(1))]
                } else { suffix };
                return (mb, code, itype);
            }
            let code = after;
            let method = &project_name[..dash_pos];
            let itype = if code.starts_with("LC") { "液相" } else { "气相" };
            return (method, code, itype);
        }
    }
    (project_name, "", "其他")
}

fn extract_code(n: &str) -> &str {
    n.split('-').next().unwrap_or(n)
}

use std::collections::HashMap;

// Simple method name lookup map (abbreviated - full version would have 95 entries)
fn get_method_full_name(group: &str, project: &str) -> String {
    // Use a basic mapping for common methods
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
    // Return the base method name
    let parts: Vec<&str> = project.splitn(2, '-').collect();
    if parts.len() > 1 { parts[1].to_string() } else { project.to_string() }
}

fn month_bounds(ref_date: &str) -> (String, String) {
    // Parse YYYY-MM-DD and get month bounds
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

async fn export_excel(
    State(pool): State<DbPool>,
    Query(q): Query<ExportQuery>,
) -> Result<impl IntoResponse> {
    use rust_xlsxwriter::*;
    use std::io::Cursor;

    let conn = pool.get()?;

    // Determine month
    let ref_date = q.start.as_deref().unwrap_or("2026-06-01");
    let (d_start, d_end) = month_bounds(ref_date);
    let month_label = {
        let parts: Vec<&str> = d_start.split('-').collect();
        format!("{}年{}月", parts[0], parts[1].trim_start_matches('0'))
    };

    // Query all active projects
    let mut proj_sql = String::from(
        "SELECT p.id, p.name AS project_name, pg.name AS group_name, pg.sort_order AS gs, p.sort_order AS ps
         FROM projects p JOIN project_groups pg ON p.group_id = pg.id WHERE p.is_active = 1"
    );
    let mut proj_params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];
    if let Some(gid) = q.group_id {
        proj_sql.push_str(&format!(" AND pg.id=?{}", proj_params.len() + 1));
        proj_params.push(Box::new(gid));
    }
    proj_sql.push_str(" ORDER BY gs, ps");

    let mut stmt = conn.prepare(&proj_sql)?;
    let all_projects: Vec<(i64, String, String)> = stmt.query_map(
        rusqlite::params_from_iter(proj_params.iter().map(|p| p.as_ref())),
        |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?)),
    )?.collect::<std::result::Result<Vec<_>, _>>()?;

    // Query monthly records
    let mut rec_sql = String::from(
        "SELECT p.id AS project_id, SUM(wr.quantity) AS qty
         FROM work_records wr JOIN projects p ON wr.project_id = p.id
         JOIN project_groups pg ON p.group_id = pg.id
         WHERE wr.deleted_at IS NULL AND wr.recorded_at >= ?1 AND wr.recorded_at < ?2"
    );
    let mut rec_params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![
        Box::new(d_start.clone()),
        Box::new(d_end.clone()),
    ];
    if let Some(gid) = q.group_id {
        rec_sql.push_str(&format!(" AND pg.id=?{}", rec_params.len() + 1));
        rec_params.push(Box::new(gid));
    }
    rec_sql.push_str(" GROUP BY p.id");

    let mut stmt = conn.prepare(&rec_sql)?;
    let records: HashMap<i64, i64> = stmt.query_map(
        rusqlite::params_from_iter(rec_params.iter().map(|p| p.as_ref())),
        |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
    )?.filter_map(|r| r.ok()).collect();

    // Build lab→code→instrument hierarchy
    #[derive(Default)]
    struct InstData {
        lc: Vec<(String, String, i64)>,  // (instrument, method, qty)
        gc: Vec<(String, String, i64)>,
    }
    let mut lab_order: Vec<String> = vec![];
    let mut lab_data: HashMap<String, HashMap<String, InstData>> = HashMap::new();

    for (pid, pn, gn) in &all_projects {
        let pc = extract_code(pn).to_string();
        let (mb, ic, it) = parse_instrument(pn);
        let fn_name = get_method_full_name(gn, pn);
        let qty = records.get(pid).copied().unwrap_or(0);

        let gn_owned = gn.clone();
        if !lab_data.contains_key(&gn_owned) {
            lab_data.insert(gn_owned.clone(), HashMap::new());
            lab_order.push(gn_owned.clone());
        }
        let ld = lab_data.get_mut(&gn_owned).unwrap();
        if !ld.contains_key(&pc) {
            ld.insert(pc.clone(), InstData::default());
        }
        let inst = ld.get_mut(&pc).unwrap();
        if it == "气相" {
            inst.gc.push((ic.to_string(), fn_name, qty));
        } else {
            inst.lc.push((ic.to_string(), fn_name, qty));
        }
    }

    // === Sheet 1: 月-汇总 ===
    let mut wb = Workbook::new();
    let ws1 = wb.add_worksheet();
    ws1.set_name("月-汇总").map_err(|e| AppError::Internal(e.to_string()))?;
    ws1.set_tab_color(Color::RGB(0x1976D2));

    let fh = Format::new().set_bold().set_font_size(16);
    let fd = Format::new().set_font_size(16);

    let wcols = [8.89, 24.89, 18.0, 17.44, 43.66, 19.66, 12.0, 12.0, 22.11];
    for (i, w) in wcols.iter().enumerate() {
        ws1.set_column_width(i as u16, *w)?;
    }

    let hdrs = ["使用实验室","项目代号","液相仪器","检测方法","月检测数量","液相检测量","气相检测量","项目检测总量"];
    for (i, h) in hdrs.iter().enumerate() {
        ws1.write_with_format(1, (i+1) as u16, *h, &fh)?;
    }

    // Column indices (0-based): B=1  C=2  D=3  E=4  F=5  G=6  H=7  I=8
    const CB: u16 = 1; const CC: u16 = 2; const CD: u16 = 3; const CE: u16 = 4;
    const CF: u16 = 5; const CG: u16 = 6; const CH: u16 = 7; const CI: u16 = 8;

    let hr = 2u32;
    let mut r = hr + 1;

    // Two-pass: first merge all ranges, then write data (merge_range overwrites cells)
    struct RangeInfo { start: u32, end: u32, lab: String, code: String, pc_rows: u32, lc_rows: u32, gc_rows: u32 }
    let mut ranges: Vec<RangeInfo> = vec![];

    for lab_name in &lab_order {
        let pd = lab_data.get(lab_name).unwrap();
        let lab_start = r;
        for (pc, inst) in pd.iter() {
            let code_start = r;
            let rows = (inst.lc.len() + inst.gc.len()) as u32;
            r += rows;
            ranges.push(RangeInfo {
                start: code_start, end: r - 1,
                lab: lab_name.clone(), code: pc.clone(),
                pc_rows: rows, lc_rows: inst.lc.len() as u32, gc_rows: inst.gc.len() as u32,
            });
        }
        // Merge lab column B
        let lab_end = r - 1;
        if lab_end > lab_start {
            ws1.merge_range(lab_start, CB, lab_end, CB, "", &fd)?;
        }
    }

    // Write data + formulas per project-code group
    for rg in &ranges {
        let psr = rg.start;
        // Merge project code column C and G/H/I
        if rg.end > rg.start {
            ws1.merge_range(psr, CC, rg.end, CC, "", &fd)?;
            for col in [CG, CH, CI] {
                ws1.merge_range(psr, col, rg.end, col, "", &fd)?;
            }
        }
        // Write all instrument rows
        let mut ri = psr;
        let pd = lab_data.get(&rg.lab).unwrap();
        let inst = pd.get(&rg.code).unwrap();
        for (ic, ml, qty) in inst.lc.iter().chain(inst.gc.iter()) {
            ws1.write_with_format(ri, CB, rg.lab.as_str(), &fd)?;
            ws1.write_with_format(ri, CC, rg.code.as_str(), &fd)?;
            ws1.write_with_format(ri, CD, ic.as_str(), &fd)?;
            ws1.write_with_format(ri, CE, ml.as_str(), &fd)?;
            if *qty > 0 { ws1.write_with_format(ri, CF, *qty as f64, &fd)?; }
            ri += 1;
        }
        // Formulas: G = SUM(F for LC rows), H = SUM(F for GC rows), I = G+H
        if rg.lc_rows > 0 {
            let lc_last = psr + rg.lc_rows - 1;
            ws1.write_formula(psr, CG,
                format!("=SUM({}{}:{}{})", _cl(CF), psr, _cl(CF), lc_last).as_str())?;
        }
        if rg.gc_rows > 0 {
            let gc_start = psr + rg.lc_rows;
            ws1.write_formula(psr, CH,
                format!("=SUM({}{}:{}{})", _cl(CF), gc_start, _cl(CF), rg.end).as_str())?;
        }
        ws1.write_formula(psr, CI,
            format!("=SUM({}{}:{}{})", _cl(CG), psr, _cl(CH), psr).as_str())?;
    }

    // Total row
    let total_row = r;
    ws1.write_with_format(total_row, CB, "总计", &fd)?;
    let fr = format!("{}:{}", hr + 1, total_row - 1);
    ws1.write_formula(total_row, CF, format!("=SUM({}{})", _cl(CF), &fr).as_str())?;
    ws1.write_formula(total_row, CG, format!("=SUM({}{})", _cl(CG), &fr).as_str())?;
    ws1.write_formula(total_row, CH, format!("=SUM({}{})", _cl(CH), &fr).as_str())?;
    ws1.write_formula(total_row, CI, format!("=SUM({}{})", _cl(CI), &fr).as_str())?;

    ws1.set_freeze_panes(hr + 1, 4)?;

    // === Sheet 2-5: 简化版 ===
    let ws2 = wb.add_worksheet();
    ws2.set_name("每日工作量").map_err(|e| AppError::Internal(e.to_string()))?;
    ws2.set_tab_color(Color::RGB(0x43A047));
    ws2.write_with_format(0, 0, "日期", &fh)?;
    ws2.write_with_format(0, 1, "实验室", &fh)?;
    ws2.write_with_format(0, 2, "项目", &fh)?;
    ws2.write_with_format(0, 3, "数量", &fh)?;
    ws2.write_with_format(0, 4, "录入人", &fh)?;
    ws2.set_column_width(0, 12.0)?;
    ws2.set_column_width(1, 14.0)?;
    ws2.set_column_width(2, 30.0)?;
    ws2.set_column_width(3, 8.0)?;
    ws2.set_column_width(4, 12.0)?;
    ws2.set_freeze_panes(1, 0)?;

    // Query daily records
    let sql = format!(
        "SELECT date(wr.recorded_at) AS work_day, pg.name AS group_name, p.name AS project_name,
                SUM(wr.quantity) AS qty, GROUP_CONCAT(DISTINCT wr.user_name) AS users
         FROM work_records wr JOIN projects p ON wr.project_id = p.id
         JOIN project_groups pg ON p.group_id = pg.id
         WHERE wr.deleted_at IS NULL {}
         GROUP BY date(wr.recorded_at), pg.id, p.id
         ORDER BY work_day, pg.sort_order, p.sort_order",
        if q.start.is_some() || q.end.is_some() {
            "AND wr.recorded_at >= ?1 AND wr.recorded_at < ?2"
        } else { "" }
    );
    let mut stmt = conn.prepare(&sql)?;
    let mut dr = 1u32;
    if q.start.is_some() || q.end.is_some() {
        let s = q.start.as_deref().unwrap_or("2026-01-01");
        let e = q.end.as_deref().unwrap_or("2026-12-31");
        let rows = stmt.query_map((s, e), |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, i64>(3)?, row.get::<_, String>(4)?))
        })?;
        for row in rows {
            let (wd, gn, pn, qty, users) = row?;
            ws2.write_with_format(dr, 0, wd.as_str(), &fd)?;
            ws2.write_with_format(dr, 1, gn.as_str(), &fd)?;
            ws2.write_with_format(dr, 2, pn.as_str(), &fd)?;
            ws2.write_with_format(dr, 3, qty as f64, &fd)?;
            ws2.write_with_format(dr, 4, users.as_str(), &fd)?;
            dr += 1;
        }
    } else {
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, i64>(3)?, row.get::<_, String>(4)?))
        })?;
        for row in rows {
            let (wd, gn, pn, qty, users) = row?;
            ws2.write_with_format(dr, 0, wd.as_str(), &fd)?;
            ws2.write_with_format(dr, 1, gn.as_str(), &fd)?;
            ws2.write_with_format(dr, 2, pn.as_str(), &fd)?;
            ws2.write_with_format(dr, 3, qty as f64, &fd)?;
            ws2.write_with_format(dr, 4, users.as_str(), &fd)?;
            dr += 1;
        }
    }

    let ws3 = wb.add_worksheet();
    ws3.set_name("每周工作量").map_err(|e| AppError::Internal(e.to_string()))?;
    ws3.set_tab_color(Color::RGB(0xFF9800));
    ws3.write_with_format(0, 0, "每周工作量（数据同月汇总）", &fh)?;
    ws3.set_freeze_panes(1, 0)?;

    let ws4 = wb.add_worksheet();
    ws4.set_name("原始记录").map_err(|e| AppError::Internal(e.to_string()))?;
    ws4.set_tab_color(Color::RGB(0x9C27B0));
    ws4.write_with_format(0, 0, "原始记录（同每日工作量）", &fh)?;
    ws4.set_freeze_panes(1, 0)?;

    let ws5 = wb.add_worksheet();
    ws5.set_name("用户统计").map_err(|e| AppError::Internal(e.to_string()))?;
    ws5.set_tab_color(Color::RGB(0x1976D2));
    ws5.write_with_format(0, 0, "用户统计（同每日工作量）", &fh)?;
    ws5.set_freeze_panes(1, 0)?;

    let mut buf = Cursor::new(Vec::new());
    wb.save_to_writer(&mut buf).map_err(|e| AppError::Internal(e.to_string()))?;
    let data = buf.into_inner();

    let filename = format!("attachment; filename*=UTF-8''{}", url_escape::encode_component(&format!("工作量统计_{}.xlsx", month_label)));
    Ok((
        axum::response::Response::builder()
            .header(header::CONTENT_TYPE, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
            .header(header::CONTENT_DISPOSITION, filename)
            .body(axum::body::Body::from(data))
            .unwrap()
    ))
}
