/// 导出数据查询层 - v0.3.6 重写版本
/// 支持 10 个 Sheet 的数据查询

use rusqlite::Connection;
use crate::error::Result;

// ========== 通用数据结构 ==========

/// 扁平行数据（Sheet 1 使用）
pub type FlatRow = (String, String, String, String, i64, bool, f64);
// (实验室, 项目代号, 仪器, 方法, 数量, 是否气相, 系数)

/// 仪器汇总行（Sheet 2）
#[derive(Debug, Clone, serde::Serialize)]
pub struct InstrumentDailyRow {
    pub date: String,
    pub instrument: String,
    pub lab: String,
    pub project: String,
    pub method: String,
    pub quantity: i64,
}

/// 项目汇总行（Sheet 3）
#[derive(Debug, Clone, serde::Serialize)]
pub struct ProjectSummaryRow {
    pub project: String,
    pub lab: String,
    pub instrument: String,
    pub method: String,
    pub quantity: i64,
    pub amount: f64,  // 方法金额
}

/// 实验室汇总行（Sheet 4）
#[derive(Debug, Clone, serde::Serialize)]
pub struct LabSummaryRow {
    pub lab: String,
    pub project: String,
    pub instrument: String,
    pub method: String,
    pub quantity: i64,
    pub amount: f64,
}

/// 人员原始记录行（Sheet 5）
#[derive(Debug, Clone, serde::Serialize)]
pub struct PersonRecordRow {
    pub recorded_at: String,
    pub lab: String,
    pub project: String,
    pub method: String,
    pub method_type: String,
    pub quantity: i64,
    pub user_name: String,
}

/// 人员汇总行（Sheet 6）
#[derive(Debug, Clone, serde::Serialize)]
pub struct PersonSummaryRow {
    pub user_name: String,
    pub method_type: String,  // 液相/气相/理化
    pub coefficient: f64,
    pub quantity: i64,
}

/// 实验室总表行（Sheet 7）
#[derive(Debug, Clone, serde::Serialize)]
pub struct LabTotalRow {
    pub lab: String,
    pub project: String,
    pub method_type: String,
    pub amount: f64,
    pub quantity: i64,
}

/// 项目总表行（Sheet 8）
#[derive(Debug, Clone, serde::Serialize)]
pub struct ProjectTotalRow {
    pub project: String,
    pub method_type: String,
    pub amount: f64,
    pub quantity: i64,
}

/// 仪器汇总表行（Sheet 9）
#[derive(Debug, Clone, serde::Serialize)]
pub struct InstrumentSummaryRow {
    pub instrument: String,
    pub quantity: i64,
    pub instrument_type: String,  // lc/gc/icp等
}

/// 理化汇总表行（Sheet 10）
#[derive(Debug, Clone, serde::Serialize)]
pub struct PhysChemRow {
    pub method: String,
    pub quantity: i64,
}

// ========== 辅助函数 ==========

/// 从项目名称提取代号（取 - 前部分）
pub fn extract_code(name: &str) -> &str {
    name.split('-').next().unwrap_or(name)
}

/// 从方法全名提取仪器编号（中括号内容）
pub fn extract_instrument(full_name: &str) -> String {
    if let Some(start) = full_name.find('[') {
        if let Some(end) = full_name.find(']') {
            if end > start {
                return full_name[start+1..end].to_string();
            }
        }
    }
    "未知".to_string()
}

/// 识别仪器类型（根据仪器编号前缀）
pub fn identify_instrument_type(instrument: &str) -> &str {
    if instrument.starts_with("LC-") || instrument.starts_with("HPLC-") {
        "液相"
    } else if instrument.starts_with("GC-") {
        "气相"
    } else if instrument.starts_with("ICP-") {
        "ICP"
    } else {
        "其他"
    }
}

/// 解析仪器信息（返回：方法基础名, 仪器编号, 仪器类型）
pub fn parse_instrument(project_name: &str) -> (String, String, String) {
    let parts: Vec<&str> = project_name.split('-').collect();
    if parts.len() >= 2 {
        let base = parts[0].to_string();
        let instrument = parts[1..].join("-");
        let itype = identify_instrument_type(&instrument).to_string();
        (base, instrument, itype)
    } else {
        (project_name.to_string(), "未知".to_string(), "其他".to_string())
    }
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

// ========== Sheet 1: 各实验室项目方法对应表 ==========

pub fn query_sheet1_data(
    conn: &Connection,
    start: &str,
    end: &str,
    group_id: Option<i64>
) -> Result<Vec<FlatRow>> {
    let end_closed = format!("{}T23:59:59", end);

    let mut sql = String::from(
        "SELECT pg.name, p.name, m.full_name, m.name, m.coefficient,
                COALESCE(SUM(wr.quantity), 0)
         FROM project_groups pg
         JOIN projects p ON p.group_id = pg.id
         LEFT JOIN work_records wr ON p.id = wr.project_id
            AND wr.deleted_at IS NULL
            AND wr.recorded_at >= ?1
            AND wr.recorded_at <= ?2
         LEFT JOIN methods m ON wr.method_id = m.id
         WHERE 1=1"
    );

    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![
        Box::new(start.to_string()),
        Box::new(end_closed),
    ];

    if let Some(gid) = group_id {
        sql.push_str(" AND pg.id = ?3");
        params.push(Box::new(gid));
    }

    sql.push_str(" GROUP BY pg.id, p.id, m.id ORDER BY pg.sort_order, p.name, m.name");

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(
        rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())),
        |row| {
            let lab: String = row.get(0)?;
            let project: String = row.get(1)?;
            let full_name: String = row.get(2).unwrap_or_default();
            let method: String = row.get(3).unwrap_or_default();
            let coefficient: f64 = row.get(4).unwrap_or(1.0);
            let quantity: i64 = row.get(5)?;

            let project_code = extract_code(&project).to_string();
            let instrument = extract_instrument(&full_name);
            let is_gc = instrument.starts_with("GC-");

            Ok((lab, project_code, instrument, method, quantity, is_gc, coefficient))
        }
    )?;

    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

// ========== Sheet 2: 仪器-汇总 ==========

pub fn query_sheet2_data(
    conn: &Connection,
    start: &str,
    end: &str
) -> Result<Vec<InstrumentDailyRow>> {
    let end_closed = format!("{}T23:59:59", end);

    let sql =
        "SELECT date(wr.recorded_at) AS record_date,
                m.full_name,
                pg.name AS lab_name,
                p.name AS project_name,
                m.name AS method_name,
                SUM(wr.quantity) AS total_qty
         FROM work_records wr
         JOIN projects p ON wr.project_id = p.id
         JOIN project_groups pg ON p.group_id = pg.id
         LEFT JOIN methods m ON wr.method_id = m.id
         WHERE wr.deleted_at IS NULL
           AND wr.recorded_at >= ?1
           AND wr.recorded_at <= ?2
         GROUP BY record_date, m.id, p.id
         ORDER BY record_date, m.full_name, pg.sort_order, p.name";

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([start, &end_closed], |row| {
        let date: String = row.get(0)?;
        let full_name: String = row.get(1).unwrap_or_default();
        let instrument = extract_instrument(&full_name);

        Ok(InstrumentDailyRow {
            date,
            instrument,
            lab: row.get(2)?,
            project: row.get(3)?,
            method: row.get(4).unwrap_or_default(),
            quantity: row.get(5)?,
        })
    })?;

    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

// ========== Sheet 3: 项目-汇总 ==========

pub fn query_sheet3_data(
    conn: &Connection,
    start: &str,
    end: &str
) -> Result<Vec<ProjectSummaryRow>> {
    let end_closed = format!("{}T23:59:59", end);

    let sql =
        "SELECT p.name AS project_name,
                pg.name AS lab_name,
                m.full_name,
                m.name AS method_name,
                m.amount,
                SUM(wr.quantity) AS total_qty
         FROM work_records wr
         JOIN projects p ON wr.project_id = p.id
         JOIN project_groups pg ON p.group_id = pg.id
         LEFT JOIN methods m ON wr.method_id = m.id
         WHERE wr.deleted_at IS NULL
           AND wr.recorded_at >= ?1
           AND wr.recorded_at <= ?2
         GROUP BY p.id, m.id
         ORDER BY p.name, pg.sort_order, m.name";

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([start, &end_closed], |row| {
        let full_name: String = row.get(2).unwrap_or_default();
        let instrument = extract_instrument(&full_name);

        Ok(ProjectSummaryRow {
            project: row.get(0)?,
            lab: row.get(1)?,
            instrument,
            method: row.get(3).unwrap_or_default(),
            quantity: row.get(5)?,
            amount: row.get::<_, f64>(4).unwrap_or(0.0),
        })
    })?;

    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

// ========== Sheet 4: 实验室-汇总 ==========

pub fn query_sheet4_data(
    conn: &Connection,
    start: &str,
    end: &str
) -> Result<Vec<LabSummaryRow>> {
    let end_closed = format!("{}T23:59:59", end);

    let sql =
        "SELECT pg.name AS lab_name,
                p.name AS project_name,
                m.full_name,
                m.name AS method_name,
                m.amount,
                SUM(wr.quantity) AS total_qty
         FROM work_records wr
         JOIN projects p ON wr.project_id = p.id
         JOIN project_groups pg ON p.group_id = pg.id
         LEFT JOIN methods m ON wr.method_id = m.id
         WHERE wr.deleted_at IS NULL
           AND wr.recorded_at >= ?1
           AND wr.recorded_at <= ?2
         GROUP BY pg.id, p.id, m.id
         ORDER BY pg.sort_order, p.name, m.name";

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([start, &end_closed], |row| {
        let full_name: String = row.get(2).unwrap_or_default();
        let instrument = extract_instrument(&full_name);

        Ok(LabSummaryRow {
            lab: row.get(0)?,
            project: row.get(1)?,
            instrument,
            method: row.get(3).unwrap_or_default(),
            quantity: row.get(5)?,
            amount: row.get::<_, f64>(4).unwrap_or(0.0),
        })
    })?;

    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

// ========== Sheet 5: 人员-汇总（原始记录） ==========

pub fn query_sheet5_data(
    conn: &Connection,
    start: &str,
    end: &str
) -> Result<Vec<PersonRecordRow>> {
    let end_closed = format!("{}T23:59:59", end);

    let sql =
        "SELECT wr.recorded_at,
                pg.name AS lab_name,
                p.name AS project_name,
                m.name AS method_name,
                COALESCE(GROUP_CONCAT(DISTINCT mt.name), '其他') AS method_types,
                wr.quantity,
                wr.user_name
         FROM work_records wr
         JOIN projects p ON wr.project_id = p.id
         JOIN project_groups pg ON p.group_id = pg.id
         LEFT JOIN methods m ON wr.method_id = m.id
         LEFT JOIN method_type_links mtl ON m.id = mtl.method_id
         LEFT JOIN method_types mt ON mtl.method_type_id = mt.id
         WHERE wr.deleted_at IS NULL
           AND wr.recorded_at >= ?1
           AND wr.recorded_at <= ?2
         GROUP BY wr.id
         ORDER BY wr.recorded_at DESC";

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([start, &end_closed], |row| {
        Ok(PersonRecordRow {
            recorded_at: row.get(0)?,
            lab: row.get(1)?,
            project: row.get(2)?,
            method: row.get(3).unwrap_or_default(),
            method_type: row.get(4)?,
            quantity: row.get(5)?,
            user_name: row.get(6)?,
        })
    })?;

    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

// ========== Sheet 6: 人员汇总表 ==========

pub fn query_sheet6_data(
    conn: &Connection,
    start: &str,
    end: &str
) -> Result<Vec<PersonSummaryRow>> {
    let end_closed = format!("{}T23:59:59", end);

    let sql =
        "SELECT wr.user_name,
                COALESCE(mt.name, '其他') AS method_type,
                m.coefficient,
                SUM(wr.quantity) AS total_qty
         FROM work_records wr
         JOIN projects p ON wr.project_id = p.id
         LEFT JOIN methods m ON wr.method_id = m.id
         LEFT JOIN method_type_links mtl ON m.id = mtl.method_id
         LEFT JOIN method_types mt ON mtl.method_type_id = mt.id
         WHERE wr.deleted_at IS NULL
           AND wr.recorded_at >= ?1
           AND wr.recorded_at <= ?2
         GROUP BY wr.user_name, mt.name, m.coefficient, m.id
         ORDER BY wr.user_name, mt.name, m.coefficient";

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([start, &end_closed], |row| {
        Ok(PersonSummaryRow {
            user_name: row.get(0)?,
            method_type: row.get(1)?,
            coefficient: row.get::<_, f64>(2).unwrap_or(1.0),
            quantity: row.get(3)?,
        })
    })?;

    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

// ========== Sheet 7: 实验室总表 ==========

pub fn query_sheet7_data(
    conn: &Connection,
    start: &str,
    end: &str
) -> Result<Vec<LabTotalRow>> {
    let end_closed = format!("{}T23:59:59", end);

    let sql =
        "SELECT pg.name AS lab_name,
                p.name AS project_name,
                COALESCE(mt.name, '其他') AS method_type,
                m.amount,
                SUM(wr.quantity) AS total_qty
         FROM work_records wr
         JOIN projects p ON wr.project_id = p.id
         JOIN project_groups pg ON p.group_id = pg.id
         LEFT JOIN methods m ON wr.method_id = m.id
         LEFT JOIN method_type_links mtl ON m.id = mtl.method_id
         LEFT JOIN method_types mt ON mtl.method_type_id = mt.id
         WHERE wr.deleted_at IS NULL
           AND wr.recorded_at >= ?1
           AND wr.recorded_at <= ?2
         GROUP BY pg.id, p.id, mt.name, m.amount
         ORDER BY pg.sort_order, p.name, mt.name";

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([start, &end_closed], |row| {
        Ok(LabTotalRow {
            lab: row.get(0)?,
            project: row.get(1)?,
            method_type: row.get(2)?,
            amount: row.get::<_, f64>(3).unwrap_or(0.0),
            quantity: row.get(4)?,
        })
    })?;

    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

// ========== Sheet 8: 项目总表 ==========

pub fn query_sheet8_data(
    conn: &Connection,
    start: &str,
    end: &str
) -> Result<Vec<ProjectTotalRow>> {
    let end_closed = format!("{}T23:59:59", end);

    let sql =
        "SELECT p.name AS project_name,
                COALESCE(mt.name, '其他') AS method_type,
                m.amount,
                SUM(wr.quantity) AS total_qty
         FROM work_records wr
         JOIN projects p ON wr.project_id = p.id
         LEFT JOIN methods m ON wr.method_id = m.id
         LEFT JOIN method_type_links mtl ON m.id = mtl.method_id
         LEFT JOIN method_types mt ON mtl.method_type_id = mt.id
         WHERE wr.deleted_at IS NULL
           AND wr.recorded_at >= ?1
           AND wr.recorded_at <= ?2
         GROUP BY p.id, mt.name, m.amount
         ORDER BY p.name, mt.name";

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([start, &end_closed], |row| {
        Ok(ProjectTotalRow {
            project: row.get(0)?,
            method_type: row.get(1)?,
            amount: row.get::<_, f64>(2).unwrap_or(0.0),
            quantity: row.get(3)?,
        })
    })?;

    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

// ========== Sheet 9: 仪器汇总表 ==========

pub fn query_sheet9_data(
    conn: &Connection,
    start: &str,
    end: &str
) -> Result<Vec<InstrumentSummaryRow>> {
    let end_closed = format!("{}T23:59:59", end);

    let sql =
        "SELECT m.full_name,
                SUM(wr.quantity) AS total_qty
         FROM work_records wr
         JOIN projects p ON wr.project_id = p.id
         LEFT JOIN methods m ON wr.method_id = m.id
         WHERE wr.deleted_at IS NULL
           AND wr.recorded_at >= ?1
           AND wr.recorded_at <= ?2
           AND m.full_name IS NOT NULL
         GROUP BY m.full_name
         ORDER BY total_qty DESC";

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([start, &end_closed], |row| {
        let full_name: String = row.get(0)?;
        let instrument = extract_instrument(&full_name);
        let instrument_type = identify_instrument_type(&instrument).to_string();

        Ok(InstrumentSummaryRow {
            instrument,
            quantity: row.get(1)?,
            instrument_type,
        })
    })?;

    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

// ========== Sheet 10: 理化汇总表 ==========

pub fn query_sheet10_data(
    conn: &Connection,
    start: &str,
    end: &str
) -> Result<Vec<PhysChemRow>> {
    let end_closed = format!("{}T23:59:59", end);

    let sql =
        "SELECT m.name AS method_name,
                SUM(wr.quantity) AS total_qty
         FROM work_records wr
         JOIN projects p ON wr.project_id = p.id
         LEFT JOIN methods m ON wr.method_id = m.id
         LEFT JOIN method_type_links mtl ON m.id = mtl.method_id
         LEFT JOIN method_types mt ON mtl.method_type_id = mt.id
         WHERE wr.deleted_at IS NULL
           AND wr.recorded_at >= ?1
           AND wr.recorded_at <= ?2
           AND mt.name = '理化'
         GROUP BY m.name
         ORDER BY total_qty DESC";

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([start, &end_closed], |row| {
        Ok(PhysChemRow {
            method: row.get(0).unwrap_or_default(),
            quantity: row.get(1)?,
        })
    })?;

    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}
