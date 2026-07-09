/// 样品信息登记导出 — 数据查询层（独立模块，不引用分析检测表）
use rusqlite::Connection;
use crate::error::Result;

#[derive(Debug, Clone, serde::Serialize)]
pub struct SampleInfoExportRow {
    pub seq_no: i64,
    pub batch_no: String,
    pub user_name: String,
    pub lab_name: String,
    pub project_name: String,
    pub submitted_at: String,
    pub detection_date: String,
    pub detection_type: String,
    pub status: String,
    pub main_components: String,
    pub notes: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct NameCountRow {
    pub name: String,
    pub count: i64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TypeCountRow {
    pub type_key: String,
    pub label: String,
    pub count: i64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MonthCountRow {
    pub month: String,
    pub count: i64,
}

fn range_where(start: &str, end: &str) -> String {
    format!(
        "deleted_at IS NULL AND submitted_at >= '{}' AND submitted_at <= '{}T23:59:59'",
        start, end
    )
}

/// Sheet 1: 全部记录明细
pub fn query_detail(conn: &Connection, start: &str, end: &str) -> Result<Vec<SampleInfoExportRow>> {
    let wc = range_where(start, end);
    let mut stmt = conn.prepare(&format!(
        "SELECT sir.seq_no, sir.batch_no, sir.user_name, sir.lab_name, sir.project_name, \
                sir.submitted_at, sir.detection_date, COALESCE(sit.label, sir.detection_type), \
                sir.status, sir.main_components, sir.notes \
         FROM sample_info_records sir \
         LEFT JOIN sample_info_types sit ON sit.type_key = sir.type_key \
         WHERE {} ORDER BY sir.created_at DESC",
        wc
    ))?;
    let rows = stmt.query_map([], |row| {
        Ok(SampleInfoExportRow {
            seq_no: row.get(0)?,
            batch_no: row.get(1)?,
            user_name: row.get(2)?,
            lab_name: row.get(3)?,
            project_name: row.get(4)?,
            submitted_at: row.get(5)?,
            detection_date: row.get::<_, String>(6).unwrap_or_default(),
            detection_type: row.get::<_, String>(7).unwrap_or_default(),
            status: row.get(8)?,
            main_components: row.get(9)?,
            notes: row.get::<_, String>(10).unwrap_or_default(),
        })
    })?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

/// Sheet 2: 按状态
pub fn query_by_status(conn: &Connection, start: &str, end: &str) -> Result<Vec<NameCountRow>> {
    let wc = range_where(start, end);
    let mut stmt = conn.prepare(&format!(
        "SELECT status, COUNT(*) FROM sample_info_records WHERE {} GROUP BY status ORDER BY COUNT(*) DESC",
        wc
    ))?;
    let rows = stmt.query_map([], |row| {
        Ok(NameCountRow { name: row.get(0)?, count: row.get(1)? })
    })?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

/// Sheet 3: 按检测类型
pub fn query_by_type(conn: &Connection, start: &str, end: &str) -> Result<Vec<TypeCountRow>> {
    let wc = range_where(start, end);
    let mut stmt = conn.prepare(&format!(
        "SELECT COALESCE(sit.label, sir.detection_type) AS t, sir.type_key, COUNT(*) \
         FROM sample_info_records sir LEFT JOIN sample_info_types sit ON sit.type_key = sir.type_key \
         WHERE {} GROUP BY t, sir.type_key ORDER BY COUNT(*) DESC",
        wc
    ))?;
    let rows = stmt.query_map([], |row| {
        Ok(TypeCountRow {
            label: row.get(0)?,
            type_key: row.get::<_, String>(1).unwrap_or_default(),
            count: row.get(2)?,
        })
    })?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

/// Sheet 4: 按实验室
pub fn query_by_lab(conn: &Connection, start: &str, end: &str) -> Result<Vec<NameCountRow>> {
    let wc = range_where(start, end);
    let mut stmt = conn.prepare(&format!(
        "SELECT lab_name, COUNT(*) FROM sample_info_records WHERE {} GROUP BY lab_name ORDER BY COUNT(*) DESC",
        wc
    ))?;
    let rows = stmt.query_map([], |row| {
        Ok(NameCountRow { name: row.get(0)?, count: row.get(1)? })
    })?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

/// Sheet 5: 按项目
pub fn query_by_project(conn: &Connection, start: &str, end: &str) -> Result<Vec<NameCountRow>> {
    let wc = range_where(start, end);
    let mut stmt = conn.prepare(&format!(
        "SELECT project_name, COUNT(*) FROM sample_info_records WHERE {} GROUP BY project_name ORDER BY COUNT(*) DESC",
        wc
    ))?;
    let rows = stmt.query_map([], |row| {
        Ok(NameCountRow { name: row.get(0)?, count: row.get(1)? })
    })?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

/// Sheet 6: 按送样人
pub fn query_by_user(conn: &Connection, start: &str, end: &str) -> Result<Vec<NameCountRow>> {
    let wc = range_where(start, end);
    let mut stmt = conn.prepare(&format!(
        "SELECT user_name, COUNT(*) FROM sample_info_records WHERE {} GROUP BY user_name ORDER BY COUNT(*) DESC",
        wc
    ))?;
    let rows = stmt.query_map([], |row| {
        Ok(NameCountRow { name: row.get(0)?, count: row.get(1)? })
    })?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

/// Sheet 7: 按月份
pub fn query_by_month(conn: &Connection, start: &str, end: &str) -> Result<Vec<MonthCountRow>> {
    let wc = range_where(start, end);
    let mut stmt = conn.prepare(&format!(
        "SELECT strftime('%Y-%m', submitted_at) AS m, COUNT(*) FROM sample_info_records WHERE {} GROUP BY m ORDER BY m ASC",
        wc
    ))?;
    let rows = stmt.query_map([], |row| {
        Ok(MonthCountRow { month: row.get::<_, String>(0).unwrap_or_default(), count: row.get(1)? })
    })?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}
