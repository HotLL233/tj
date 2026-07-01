/// Excel export — 5 sheets, template-aligned format
/// Data aggregation → export_data.rs, Structure/write → export_write.rs
use axum::{extract::{Query, State}, Router, routing::get};
use axum::response::IntoResponse;
use axum::http::header;
use serde::Deserialize;
use std::collections::HashMap;
use crate::db::DbPool;
use crate::error::{Result, AppError};
use super::{export_data, export_write};

#[derive(Deserialize, utoipa::IntoParams)]
pub struct ExportQuery {
    /// Start date (YYYY-MM-DD), defaults to current month
    pub start: Option<String>,
    /// End date (YYYY-MM-DD)  
    pub end: Option<String>,
    /// Filter by group ID
    pub group_id: Option<i64>,
}

pub fn router(pool: DbPool) -> Router {
    Router::new()
        .route("/api/export/excel", get(export_excel))
        .with_state(pool)
}

// ─── Sheet 1: 月-汇总 ────────────────────────────────────────────

fn build_monthly(ws: &mut rust_xlsxwriter::Worksheet, conn: &rusqlite::Connection, start: Option<&str>, group_id: Option<i64>) -> Result<String> {
    ws.set_name("月-汇总").map_err(|e| AppError::Internal(e.to_string()))?;
    ws.set_tab_color(rust_xlsxwriter::Color::RGB(0x1976D2));
    export_write::set_col_widths(ws)?;
    let ref_date = start.unwrap_or("2026-06-01");
    let (d_start, d_end) = export_data::month_bounds(ref_date);
    let month_label = { let p: Vec<&str> = d_start.split('-').collect(); format!("{}年{}月", p[0], p[1].trim_start_matches('0')) };
    let (lab_order, lab_data) = export_data::build_lab_hierarchy(conn, group_id, &d_start, &d_end)?;
    let rows = export_data::flatten_lab(&lab_order, &lab_data);
    export_write::write_tree_sheet(ws, &rows, "使用实验室", "项目代号", "月检测数量", &export_write::Fmt::new())?;
    Ok(month_label)
}

// ─── Sheet 2: 每日工作量 ─────────────────────────────────────────

fn build_daily(ws: &mut rust_xlsxwriter::Worksheet, conn: &rusqlite::Connection, start: Option<&str>, end: Option<&str>, group_id: Option<i64>) -> Result<()> {
    ws.set_name("每日工作量").map_err(|e| AppError::Internal(e.to_string()))?;
    ws.set_tab_color(rust_xlsxwriter::Color::RGB(0x43A047));
    export_write::set_col_widths(ws)?;
    let fmt = export_write::Fmt::new();
    let s = start.unwrap_or("2026-01-01"); let e = end.unwrap_or("2026-12-31");
    let e_closed = format!("{}T23:59:59", e);
    let mut sql = String::from(
        "SELECT date(wr.recorded_at) AS wd, pg.name AS gn, p.name AS pn, SUM(wr.quantity) AS qty FROM work_records wr JOIN projects p ON wr.project_id=p.id JOIN project_groups pg ON p.group_id=pg.id WHERE wr.deleted_at IS NULL AND wr.recorded_at>=?1 AND wr.recorded_at<=?2");
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(s.to_string()), Box::new(e_closed)];
    if let Some(gid) = group_id { sql.push_str(" AND pg.id=?3"); params.push(Box::new(gid)); }
    sql.push_str(" GROUP BY date(wr.recorded_at), p.id ORDER BY wd, pg.sort_order, p.sort_order");
    let mut stmt = conn.prepare(&sql)?;
    let raw: Vec<(String, String, String, i64)> = stmt.query_map(rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())),
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)))?.
        collect::<std::result::Result<Vec<_>, _>>()?;
    let mut date_order: Vec<String> = vec![];
    let mut date_data: HashMap<String, HashMap<String, HashMap<String, export_data::ProjectRows>>> = HashMap::new();
    for (wd, gn, pn, qty) in &raw {
        let pc = export_data::extract_code(pn).to_string();
        let (_mb, ic, it) = export_data::parse_instrument(pn);
        let fn_name = export_data::get_method_full_name(gn, pn);
        date_data.entry(wd.clone()).or_insert_with(|| { date_order.push(wd.clone()); HashMap::new() });
        date_data.get_mut(wd).unwrap().entry(gn.clone()).or_default().entry(pc.clone()).or_default();
        let inst = date_data.get_mut(wd).unwrap().get_mut(gn).unwrap().get_mut(&pc).unwrap();
        if it == "气相" { inst.gc.push((ic.to_string(), fn_name, *qty)); } else { inst.lc.push((ic.to_string(), fn_name, *qty)); }
    }
    let mut rows: Vec<export_data::FlatRow> = vec![];
    for wd in &date_order { for gn in date_data[wd].keys() { for (pc, pr) in &date_data[wd][gn] {
        for (ic, ml, q) in &pr.lc { rows.push((wd.clone(), format!("{}/{}", gn, pc), ic.clone(), ml.clone(), *q, false)); }
        for (ic, ml, q) in &pr.gc { rows.push((wd.clone(), format!("{}/{}", gn, pc), ic.clone(), ml.clone(), *q, true)); }
    }}}
    export_write::write_tree_sheet(ws, &rows, "日期", "实验室/项目代号", "日检测数量", &fmt)?;
    Ok(())
}

// ─── Sheet 3: 每周工作量 ─────────────────────────────────────────

fn build_weekly(ws: &mut rust_xlsxwriter::Worksheet, conn: &rusqlite::Connection, start: Option<&str>, group_id: Option<i64>) -> Result<()> {
    ws.set_name("每周工作量").map_err(|e| AppError::Internal(e.to_string()))?;
    ws.set_tab_color(rust_xlsxwriter::Color::RGB(0xFF9800));
    export_write::set_col_widths(ws)?;
    let fmt = export_write::Fmt::new();
    let ref_date = start.unwrap_or("2026-06-01");
    let (d_start, d_end) = export_data::month_bounds(ref_date);
    let weeks = export_data::week_ranges(&d_start);
    if weeks.is_empty() { return Ok(()); }
    let nw = weeks.len();
    let (lab_order, lab_data) = export_data::build_lab_hierarchy(conn, group_id, &d_start, &d_end)?;
    let rows = export_data::flatten_lab(&lab_order, &lab_data);
    let mut rec_sql = String::from("SELECT p.id, date(wr.recorded_at) AS wd, SUM(wr.quantity) AS qty FROM work_records wr JOIN projects p ON wr.project_id=p.id JOIN project_groups pg ON p.group_id=pg.id WHERE wr.deleted_at IS NULL AND wr.recorded_at>=?1 AND wr.recorded_at<?2");
    let mut rp: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(d_start.clone()), Box::new(d_end.clone())];
    if let Some(gid) = group_id { rec_sql.push_str(" AND pg.id=?3"); rp.push(Box::new(gid)); }
    rec_sql.push_str(" GROUP BY p.id, date(wr.recorded_at) ORDER BY wd");
    let mut stmt = conn.prepare(&rec_sql)?;
    let raw: Vec<(i64, String, i64)> = stmt.query_map(rusqlite::params_from_iter(rp.iter().map(|p| p.as_ref())),
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?.
        collect::<std::result::Result<Vec<_>, _>>()?;
    let mut pw: HashMap<i64, Vec<i64>> = HashMap::new();
    for (pid, wd_s, qty) in &raw {
        pw.entry(*pid).or_insert_with(|| vec![0i64; nw]);
        if let Ok(d) = chrono::NaiveDate::parse_from_str(wd_s, "%Y-%m-%d") {
            for (wi, (_, ws_s, we_s)) in weeks.iter().enumerate() {
                if let (Ok(ws), Ok(we)) = (chrono::NaiveDate::parse_from_str(ws_s, "%Y-%m-%d"), chrono::NaiveDate::parse_from_str(we_s, "%Y-%m-%d")) {
                    if d >= ws && d <= we { pw.get_mut(pid).unwrap()[wi] += qty; break; }
                }
            }
        }
    }
    let pid_sql = "SELECT p.id, p.name, pg.name FROM projects p JOIN project_groups pg ON p.group_id=pg.id WHERE p.is_active=1 ORDER BY pg.sort_order, p.sort_order";
    let mut stmt = conn.prepare(pid_sql)?;
    let ap: Vec<(i64, String, String)> = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?.
        collect::<std::result::Result<Vec<_>, _>>()?;
    let mut row_wk: Vec<Vec<i64>> = vec![];
    for (pid, _pn, _gn) in &ap { row_wk.push(pw.get(pid).cloned().unwrap_or_else(|| vec![0; nw])); }
    let hdrs: [&str; 8] = ["使用实验室","项目代号","液相仪器","检测方法","周检测数量","液相检测量","气相检测量","项目检测总量"];
    for (i, h) in hdrs.iter().enumerate() { ws.write_with_format(export_write::HR, (i+1) as u16, *h, &fmt.fh)?; }
    for (wi, (label, _, _)) in weeks.iter().enumerate() {
        let col = export_write::CI + 1 + wi as u16;
        ws.write_with_format(export_write::HR, col, label.as_str(), &fmt.fh)?; ws.set_column_width(col, 8.0)?;
    }
    export_write::write_tree_sheet(ws, &rows, "使用实验室", "项目代号", "周检测数量", &fmt)?;
    let dr = export_write::HR + 1;
    for (ri_offset, wq) in row_wk.iter().enumerate() { for wi in 0..nw { let v = wq[wi]; if v > 0 { ws.write_with_format(dr + ri_offset as u32, export_write::CI + 1 + wi as u16, v as f64, &fmt.fd)?; } } }
    ws.set_freeze_panes(export_write::HR + 1, 4)?;
    Ok(())
}

// ─── Sheet 4: 原始记录 ───────────────────────────────────────────

fn build_raw(ws: &mut rust_xlsxwriter::Worksheet, conn: &rusqlite::Connection, start: Option<&str>, end: Option<&str>, group_id: Option<i64>) -> Result<()> {
    ws.set_name("原始记录").map_err(|e| AppError::Internal(e.to_string()))?;
    ws.set_tab_color(rust_xlsxwriter::Color::RGB(0x9C27B0));
    let fmt = export_write::Fmt::new();
    let s = start.unwrap_or("2026-01-01"); let e = end.unwrap_or("2026-12-31");
    let e_c = format!("{} 23:59:59", e);
    let mut sql = String::from("SELECT pg.name AS gn, p.name AS pn, wr.user_name, wr.quantity, wr.recorded_at FROM work_records wr JOIN projects p ON wr.project_id=p.id JOIN project_groups pg ON p.group_id=pg.id WHERE wr.deleted_at IS NULL AND wr.recorded_at>=?1 AND wr.recorded_at<=?2");
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(s), Box::new(e_c)];
    if let Some(gid) = group_id { sql.push_str(" AND pg.id=?3"); params.push(Box::new(gid)); }
    sql.push_str(" ORDER BY wr.recorded_at, pg.sort_order, p.sort_order");
    let mut stmt = conn.prepare(&sql)?;
    let ra: Vec<(String, String, String, i64, String)> = stmt.query_map(rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())),
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)))?.
        collect::<std::result::Result<Vec<_>, _>>()?;
    let hdrs = ["序号","日期","实验室","项目名称","仪器","检测方法","仪器类型","数量","录入人"];
    let cw = [6.0,12.0,14.0,30.0,12.0,30.0,8.0,8.0,12.0];
    for (i,h) in hdrs.iter().enumerate() { ws.write_with_format(0,i as u16,*h,&fmt.fh)?; ws.set_column_width(i as u16,cw[i])?; }
    for (i,(gn,pn,un,qty,ts)) in ra.iter().enumerate() {
        let ri = (i+1) as u32;
        let (_,ic,it) = export_data::parse_instrument(pn);
        let rd = &ts[..ts.len().min(10)];
        let vals = [(i+1).to_string(), rd.to_string(), gn.clone(), pn.clone(), ic.to_string(), export_data::get_method_full_name(gn,pn), it.to_string(), qty.to_string(), un.clone()];
        for (ci,v) in vals.iter().enumerate() { ws.write_with_format(ri,ci as u16,v.as_str(),&fmt.fd)?; }
    }
    ws.set_freeze_panes(1,0)?; Ok(())
}

// ─── Sheet 5: 用户统计 ───────────────────────────────────────────

fn build_user_stats(ws: &mut rust_xlsxwriter::Worksheet, conn: &rusqlite::Connection, start: Option<&str>, end: Option<&str>, group_id: Option<i64>) -> Result<()> {
    ws.set_name("用户统计").map_err(|e| AppError::Internal(e.to_string()))?;
    ws.set_tab_color(rust_xlsxwriter::Color::RGB(0x1976D2));
    export_write::set_col_widths(ws)?;
    let fmt = export_write::Fmt::new();
    let s = start.unwrap_or("2026-01-01"); let e = end.unwrap_or("2026-12-31");
    let e_c = format!("{}T23:59:59", e);
    let mut sql = String::from("SELECT wr.user_name, pg.name AS gn, p.name AS pn, SUM(wr.quantity) AS qty FROM work_records wr JOIN projects p ON wr.project_id=p.id JOIN project_groups pg ON p.group_id=pg.id WHERE wr.deleted_at IS NULL AND wr.recorded_at>=?1 AND wr.recorded_at<=?2");
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(s.to_string()), Box::new(e_c)];
    if let Some(gid) = group_id { sql.push_str(" AND pg.id=?3"); params.push(Box::new(gid)); }
    sql.push_str(" GROUP BY wr.user_name, p.id ORDER BY wr.user_name, pg.sort_order, p.sort_order");
    let mut stmt = conn.prepare(&sql)?;
    let raw: Vec<(String,String,String,i64)> = stmt.query_map(rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())),
        |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?)))?.
        collect::<std::result::Result<Vec<_>,_>>()?;
    let mut user_order: Vec<String> = vec![];
    let mut user_data: HashMap<String, HashMap<String, HashMap<String, export_data::ProjectRows>>> = HashMap::new();
    for (un,gn,pn,qty) in &raw {
        let pc = export_data::extract_code(pn).to_string();
        let (_,ic,it) = export_data::parse_instrument(pn);
        let fn_name = export_data::get_method_full_name(gn,pn);
        user_data.entry(un.clone()).or_insert_with(|| { user_order.push(un.clone()); HashMap::new() });
        user_data.get_mut(un).unwrap().entry(gn.clone()).or_default().entry(pc.clone()).or_default();
        let inst = user_data.get_mut(un).unwrap().get_mut(gn).unwrap().get_mut(&pc).unwrap();
        if it == "气相" { inst.gc.push((ic.to_string(),fn_name,*qty)); } else { inst.lc.push((ic.to_string(),fn_name,*qty)); }
    }
    let mut rows: Vec<export_data::FlatRow> = vec![];
    for un in &user_order { for gn in user_data[un].keys() { for (pc, pr) in &user_data[un][gn] {
        for (ic,ml,q) in &pr.lc { rows.push((un.clone(),format!("{}/{}",gn,pc),ic.clone(),ml.clone(),*q,false)); }
        for (ic,ml,q) in &pr.gc { rows.push((un.clone(),format!("{}/{}",gn,pc),ic.clone(),ml.clone(),*q,true)); }
    }}}
    export_write::write_tree_sheet(ws, &rows, "用户名", "实验室/项目代号", "月检测数量", &fmt)?;
    Ok(())
}

// ─── Entry point ─────────────────────────────────────────────────

async fn export_excel(State(pool): State<DbPool>, Query(q): Query<ExportQuery>) -> Result<impl IntoResponse> {
    use std::io::Cursor;
    let conn = pool.get()?;
    let mut wb = rust_xlsxwriter::Workbook::new();
    let ws1 = wb.add_worksheet();
    let month_label = build_monthly(ws1, &conn, q.start.as_deref(), q.group_id)?;
    let ws2 = wb.add_worksheet(); build_daily(ws2, &conn, q.start.as_deref(), q.end.as_deref(), q.group_id)?;
    let ws3 = wb.add_worksheet(); build_weekly(ws3, &conn, q.start.as_deref(), q.group_id)?;
    let ws4 = wb.add_worksheet(); build_raw(ws4, &conn, q.start.as_deref(), q.end.as_deref(), q.group_id)?;
    let ws5 = wb.add_worksheet(); build_user_stats(ws5, &conn, q.start.as_deref(), q.end.as_deref(), q.group_id)?;
    let mut buf = Cursor::new(Vec::new());
    wb.save_to_writer(&mut buf).map_err(|e| AppError::Internal(e.to_string()))?;
    let data = buf.into_inner();
    let filename = format!("attachment; filename*=UTF-8''{}", url_escape::encode_component(&format!("工作量统计_{}.xlsx", month_label)));
    Ok(axum::response::Response::builder()
        .header(header::CONTENT_TYPE, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
        .header(header::CONTENT_DISPOSITION, filename)
        .body(axum::body::Body::from(data)).unwrap())
}
