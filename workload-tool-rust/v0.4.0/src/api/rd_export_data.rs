/// 导出数据查询层 - v0.3.21 版本
/// 支持 10 个 Sheet 的数据查询
/// v0.3.21 关键修复：汇总表使用 group_concat 子查询获取实验室名（拼接显示），GROUP BY 不含实验室维度
///   这样每条 (项目, 方法) 只有一行，数量不会翻倍（修复 v0.3.19/0.3.20 的 JOIN 展开问题）

use rusqlite::Connection;
use crate::error::Result;

// ========== 通用数据结构 ==========

// 复用 export_data 的 Sheet 行结构类型，确保与 export_write::write_sheetN 期望的类型一致
// （rd 模块与分析检测共用同一套导出结构，仅底层查询表不同）
pub use super::export_data::{
    FlatRow, InstrumentDailyRow, ProjectSummaryRow, LabSummaryRow, PersonRecordRow,
    PersonSummaryRow, LabTotalRow, ProjectTotalRow, InstrumentSummaryRow, PhysChemRow,
};

// ========== 辅助函数 ==========

/// 从项目名称提取代号（取 - 前部分）
pub fn extract_code(name: &str) -> &str {
    name.split('-').next().unwrap_or(name)
}

/// 从方法全名提取仪器编号（@符号后的中括号内容）
/// 新规则：方法名格式为 "xxx@[仪器编号]"，例如 "QL-260211-DAD@[LC-04]"
pub fn extract_instrument(full_name: &str) -> String {
    let s = full_name.trim();
    tracing::debug!("extract_instrument 输入: '{}'", s);
    
    // 找 '@' 符号
    if let Some(at_pos) = s.find('@') {
        let after_at = &s[at_pos + 1..];
        // 在 @ 后面找 '[...]' 提取仪器编号
        if let Some(bracket_start) = after_at.find('[') {
            let inside_brackets = &after_at[bracket_start + 1..];
            if let Some(bracket_end) = inside_brackets.find(']') {
                let instrument = &inside_brackets[..bracket_end];
                if !instrument.is_empty() {
                    tracing::debug!("extract_instrument 提取结果(新规则): '{}'", instrument);
                    return instrument.to_string();
                }
            }
        }
        // 新规则：如果 @ 后面没有 []]，尝试直接取 @ 后面的内容作为仪器编号
        // 例如 "xxx@LC-01" 应该提取 "LC-01"
        let after_at_trimmed = after_at.trim();
        if !after_at_trimmed.is_empty() && !after_at_trimmed.starts_with('[') {
            tracing::debug!("extract_instrument 提取结果(@直接提取): '{}'", after_at_trimmed);
            return after_at_trimmed.to_string();
        }
    }
    // 兼容旧数据：如果没有 @ 符号，尝试直接从 [..] 提取
    if let Some(start) = s.rfind('[') {
        let rest = &s[start + 1..];
        if let Some(end_rel) = rest.find(']') {
            let instrument = &rest[..end_rel];
            if !instrument.is_empty() {
                tracing::debug!("extract_instrument 提取结果(旧兼容): '{}'", instrument);
                return instrument.to_string();
            }
        }
    }
    tracing::warn!("extract_instrument 无法识别仪器，输入: '{}'", s);
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

    // v0.3.25 修复：使用 wr.group_id 对应的 project_groups.name 显示单个实验室
    let mut sql = String::from(
        "SELECT COALESCE(pg.name, '未知') as lab_name,
                p.name, COALESCE(m.full_name, m.name), m.name, m.coefficient,
                SUM(wr.quantity)
         FROM rd_work_records wr
         LEFT JOIN project_groups pg ON pg.id = wr.group_id
         JOIN projects p ON wr.project_id = p.id
         LEFT JOIN methods m ON wr.method_id = m.id
         WHERE wr.deleted_at IS NULL
           AND wr.recorded_at >= ?1
           AND wr.recorded_at <= ?2"
    );

    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![
        Box::new(start.to_string()),
        Box::new(end_closed),
    ];

    if let Some(gid) = group_id {
        sql.push_str(&format!(" AND EXISTS (SELECT 1 FROM project_lab_links pll_f WHERE pll_f.project_id = p.id AND pll_f.group_id = {})", gid));
    }

    sql.push_str(" GROUP BY p.id, m.id ORDER BY lab_name, p.name, m.name");

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(
        rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())),
        |row| {
            let lab: String = row.get(0)?;
            let project: String = row.get(1)?;
            let _full_name: String = row.get(2).unwrap_or_default();
            let method: String = row.get(3).unwrap_or_default();
            let coefficient: f64 = row.get(4).unwrap_or(1.0);
            let quantity: i64 = row.get(5)?;

            let project_code = extract_code(&project).to_string();
            // 用 m.name（col 3，含@[仪器]格式）提取仪器，不用 full_name（col 2 可能只是"410/A001"）
            let instrument = extract_instrument(&method);
            let is_gc = instrument.starts_with("GC-");

            Ok((lab, project_code, instrument, method, quantity, is_gc, coefficient))
        }
    )?;

    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

// ========= Sheet 2: 仪器-汇总 =========

pub fn query_sheet2_data(
    conn: &Connection,
    start: &str,
    end: &str
) -> Result<Vec<InstrumentDailyRow>> {
    let end_closed = format!("{}T23:59:59", end);

    // v0.3.25 修复：使用 wr.group_id 对应的 project_groups.name 显示单个实验室
    let sql =
        "SELECT date(wr.recorded_at) AS record_date,
                COALESCE(m.full_name, m.name),
                COALESCE(pg.name, '未知') AS lab_name,
                p.name AS project_name,
                m.name AS method_name,
                SUM(wr.quantity) AS total_qty
         FROM rd_work_records wr
         LEFT JOIN project_groups pg ON pg.id = wr.group_id
         JOIN projects p ON wr.project_id = p.id
         LEFT JOIN methods m ON wr.method_id = m.id
         WHERE wr.deleted_at IS NULL
           AND wr.recorded_at >= ?1
           AND wr.recorded_at <= ?2
         GROUP BY record_date, lab_name, m.id, p.id
         ORDER BY record_date, m.full_name, lab_name, p.name";

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([start, &end_closed], |row| {
        let date: String = row.get(0)?;
        let _full_name: String = row.get(1).unwrap_or_default();
        // 用 m.name（含@[仪器]格式）提取仪器，不用 full_name（可能只是"410/A001"）
        let method_for_instr: String = row.get(4).unwrap_or_default();
        let instrument = extract_instrument(&method_for_instr);

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

// ========= Sheet 3: 项目-汇总 =========

pub fn query_sheet3_data(
    conn: &Connection,
    start: &str,
    end: &str
) -> Result<Vec<ProjectSummaryRow>> {
    let end_closed = format!("{}T23:59:59", end);

    // v0.3.25 修复：使用 wr.group_id 对应的 project_groups.name 显示单个实验室
    let sql =
        "SELECT p.name AS project_name,
                COALESCE(pg.name, '未知') AS lab_name,
                COALESCE(m.full_name, m.name),
                m.name AS method_name,
                m.amount,
                SUM(wr.quantity) AS total_qty
         FROM rd_work_records wr
         LEFT JOIN project_groups pg ON pg.id = wr.group_id
         JOIN projects p ON wr.project_id = p.id
         LEFT JOIN methods m ON wr.method_id = m.id
         WHERE wr.deleted_at IS NULL
           AND wr.recorded_at >= ?1
           AND wr.recorded_at <= ?2
         GROUP BY p.id, m.id
         ORDER BY p.name, lab_name, m.name";

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([start, &end_closed], |row| {
        let _full_name: String = row.get(2).unwrap_or_default();
        // 用 m.name（col 3，含@[仪器]格式）提取仪器
        let method_for_instr: String = row.get(3).unwrap_or_default();
        let instrument = extract_instrument(&method_for_instr);

        Ok(ProjectSummaryRow {
            project: row.get(0)?,
            lab: row.get(1)?,
            instrument,
            method: row.get(3).unwrap_or_default(),
            quantity: row.get(5)?,
            unit_price: row.get::<_, f64>(4).unwrap_or(0.0),
        })
    })?;

    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

// ========= Sheet 4: 实验室-汇总 =========

pub fn query_sheet4_data(
    conn: &Connection,
    start: &str,
    end: &str
) -> Result<Vec<LabSummaryRow>> {
    let end_closed = format!("{}T23:59:59", end);

    // v0.3.25 修复：使用 wr.group_id 对应的 project_groups.name 显示单个实验室
    let sql =
        "SELECT COALESCE(pg.name, '未知') AS lab_name,
                p.name AS project_name,
                COALESCE(m.full_name, m.name),
                m.name AS method_name,
                m.amount,
                SUM(wr.quantity) AS total_qty
         FROM rd_work_records wr
         LEFT JOIN project_groups pg ON pg.id = wr.group_id
         JOIN projects p ON wr.project_id = p.id
         LEFT JOIN methods m ON wr.method_id = m.id
         WHERE wr.deleted_at IS NULL
           AND wr.recorded_at >= ?1
           AND wr.recorded_at <= ?2
         GROUP BY p.id, m.id
         ORDER BY lab_name, p.name, m.name";

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([start, &end_closed], |row| {
        let _full_name: String = row.get(2).unwrap_or_default();
        // 用 m.name（col 3，含@[仪器]格式）提取仪器
        let method_for_instr: String = row.get(3).unwrap_or_default();
        let instrument = extract_instrument(&method_for_instr);

        Ok(LabSummaryRow {
            lab: row.get(0)?,
            project: row.get(1)?,
            instrument,
            method: row.get(3).unwrap_or_default(),
            quantity: row.get(5)?,
            unit_price: row.get::<_, f64>(4).unwrap_or(0.0),
        })
    })?;

    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

// ========= Sheet 5: 人员-汇总（原始记录） ==========

pub fn query_sheet5_data(
    conn: &Connection,
    start: &str,
    end: &str
) -> Result<Vec<PersonRecordRow>> {
    let end_closed = format!("{}T23:59:59", end);

    // v0.3.25 修复：使用 wr.group_id 对应的 project_groups.name 显示单个实验室
    let sql =
        "SELECT wr.recorded_at,
                COALESCE(pg.name, '未知') AS lab_name,
                p.name AS project_name,
                m.name AS method_name,
                COALESCE(GROUP_CONCAT(DISTINCT mt.name), '其他') AS method_types,
                wr.quantity,
                wr.user_name
         FROM rd_work_records wr
         LEFT JOIN project_groups pg ON pg.id = wr.group_id
         JOIN projects p ON wr.project_id = p.id
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
         FROM rd_work_records wr
         LEFT JOIN project_groups pg ON pg.id = wr.group_id
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

// ========= Sheet 7: 实验室总表 ==========

pub fn query_sheet7_data(
    conn: &Connection,
    start: &str,
    end: &str
) -> Result<Vec<LabTotalRow>> {
    let end_closed = format!("{}T23:59:59", end);

    // v0.3.25 修复：使用 wr.group_id 对应的 project_groups.name 显示单个实验室
    let sql =
        "SELECT COALESCE(pg.name, '未知') AS lab_name,
                p.name AS project_name,
                COALESCE(mt.name, '其他') AS method_type,
                m.amount,
                SUM(wr.quantity) AS total_qty
         FROM rd_work_records wr
         LEFT JOIN project_groups pg ON pg.id = wr.group_id
         JOIN projects p ON wr.project_id = p.id
         LEFT JOIN methods m ON wr.method_id = m.id
         LEFT JOIN method_type_links mtl ON m.id = mtl.method_id
         LEFT JOIN method_types mt ON mtl.method_type_id = mt.id
         WHERE wr.deleted_at IS NULL
           AND wr.recorded_at >= ?1
           AND wr.recorded_at <= ?2
         GROUP BY p.id, mt.name, m.amount
         ORDER BY lab_name, p.name, mt.name";

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([start, &end_closed], |row| {
        Ok(LabTotalRow {
            lab: row.get(0)?,
            project: row.get(1)?,
            method_type: row.get(2)?,
            unit_price: row.get::<_, f64>(3).unwrap_or(0.0),
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
         FROM rd_work_records wr
         LEFT JOIN project_groups pg ON pg.id = wr.group_id
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
            unit_price: row.get::<_, f64>(2).unwrap_or(0.0),
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
        "SELECT COALESCE(NULLIF(m.full_name,''), m.name),
                SUM(wr.quantity) AS total_qty
         FROM rd_work_records wr
         LEFT JOIN project_groups pg ON pg.id = wr.group_id
         JOIN projects p ON wr.project_id = p.id
         LEFT JOIN methods m ON wr.method_id = m.id
         WHERE wr.deleted_at IS NULL
           AND wr.recorded_at >= ?1
           AND wr.recorded_at <= ?2
           AND (m.full_name IS NOT NULL OR m.name IS NOT NULL)
         GROUP BY COALESCE(NULLIF(m.full_name,''), m.name)
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
         FROM rd_work_records wr
         LEFT JOIN project_groups pg ON pg.id = wr.group_id
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
