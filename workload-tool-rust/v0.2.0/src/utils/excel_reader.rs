use calamine::{open_workbook, Reader, Xlsx, DataType};
use crate::models::import_record::ImportRecord;
use crate::error::AppError;

const DATE_KEYWORDS: &[&str] = &["日期","date","时间","time","检测日期","送样日期"];
const BATCH_KEYWORDS: &[&str] = &["批号","batch","批","批次","batch_no"];
const USER_KEYWORDS: &[&str] = &["录入人","user","操作人","实验员","送样人","人员","user_name"];
const QTY_KEYWORDS: &[&str] = &["数量","qty","quantity","件数","个数","检测数量","送样量"];
const PROJECT_KEYWORDS: &[&str] = &["项目","project","方法","project_name","检测项目","送样项目"];
const GROUP_KEYWORDS: &[&str] = &["实验室","group","分组","group_name"];

fn cell_to_string(val: &DataType) -> String {
    match val {
        DataType::String(s) => s.trim().to_string(),
        DataType::Float(f) => if f.fract() == 0.0 { format!("{}", *f as i64) } else { format!("{}", f) },
        DataType::Int(i) => i.to_string(),
        DataType::Bool(b) => if *b { "1".into() } else { "0".into() },
        _ => String::new(),
    }
}

fn cell_to_i64(val: &DataType) -> Option<i64> {
    match val {
        DataType::Float(f) => Some(*f as i64),
        DataType::Int(i) => Some(*i),
        DataType::String(s) => s.trim().parse().ok(),
        _ => None,
    }
}

fn normalize_date(raw: &str) -> Option<String> {
    let s = raw.trim();
    if s.is_empty() { return None; }
    if let Ok(n) = s.parse::<f64>() {
        if n > 40000.0 && n < 60000.0 {
            use chrono::NaiveDate;
            let base = NaiveDate::from_ymd_opt(1899, 12, 30).unwrap();
            let d = base + chrono::Duration::days(n as i64);
            return Some(d.format("%Y-%m-%d").to_string());
        }
    }
    if s.len() == 8 && s.chars().all(|c| c.is_ascii_digit()) {
        return Some(format!("{}-{}-{}", &s[0..4], &s[4..6], &s[6..8]));
    }
    let cleaned = s.replace('/', "-");
    if cleaned.len() >= 10 { return Some(cleaned[..10].to_string()); }
    let parts: Vec<&str> = cleaned.split('-').collect();
    if parts.len() == 3 {
        if let (Ok(y), Ok(m), Ok(d)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>(), parts[2].parse::<i32>()) {
            return Some(format!("{}-{:02}-{:02}", y, m, d));
        }
    }
    None
}

fn find_column(headers: &[String], keywords: &[&str]) -> Option<usize> {
    for kw in keywords {
        for (i, h) in headers.iter().enumerate() {
            if h.to_lowercase().contains(&kw.to_lowercase()) { return Some(i); }
        }
    }
    None
}

pub fn read_excel(file_path: &str) -> Result<Vec<ImportRecord>, AppError> {
    let mut workbook: Xlsx<_> = open_workbook(file_path)
        .map_err(|e| AppError::Validation(format!("无法打开Excel文件: {}", e)))?;

    let sheet_names = workbook.sheet_names().to_vec();
    if sheet_names.is_empty() {
        return Err(AppError::Validation("Excel文件中没有工作表".into()));
    }

    let range = workbook.worksheet_range(&sheet_names[0])
        .map_err(|e| AppError::Validation(format!("读取工作表失败: {}", e)))?;

    let rows_data: Vec<Vec<DataType>> = range.rows().map(|r| r.to_vec()).collect();
    if rows_data.len() < 2 {
        return Err(AppError::Validation("Excel文件至少需要表头行和一行数据".into()));
    }

    let headers: Vec<String> = rows_data[0].iter().map(cell_to_string).collect();
    let date_col = find_column(&headers, DATE_KEYWORDS)
        .ok_or(AppError::Validation("未找到日期列（关键字: 日期/时间）".into()))?;
    let batch_col = find_column(&headers, BATCH_KEYWORDS)
        .ok_or(AppError::Validation("未找到批号列（关键字: 批号/batch）".into()))?;
    let qty_col = find_column(&headers, QTY_KEYWORDS)
        .ok_or(AppError::Validation("未找到数量列（关键字: 数量/qty）".into()))?;
    let proj_col = find_column(&headers, PROJECT_KEYWORDS);
    let group_col = find_column(&headers, GROUP_KEYWORDS);
    let user_col = find_column(&headers, USER_KEYWORDS);

    let mut records: Vec<ImportRecord> = vec![];
    let mut skip_count = 0u32;

    for (ri, row) in rows_data.iter().skip(1).enumerate() {
        let m = date_col.max(batch_col).max(qty_col);
        if row.len() <= m { continue; }
        let _ = ri; // suppress unused

        let date_raw = cell_to_string(&row[date_col]);
        let date = normalize_date(&date_raw);
        if date.is_none() { skip_count += 1; continue; }

        let batch = cell_to_string(&row[batch_col]);
        if batch.is_empty() { skip_count += 1; continue; }

        let qty = cell_to_i64(&row[qty_col]).unwrap_or(0);
        if qty <= 0 { skip_count += 1; continue; }

        let project_name = proj_col.and_then(|c| {
            let v = cell_to_string(&row[c]);
            if v.is_empty() { None } else { Some(v) }
        }).unwrap_or_else(|| "未分类".to_string());

        let group_name = group_col.and_then(|c| {
            let v = cell_to_string(&row[c]);
            if v.is_empty() { None } else { Some(v) }
        }).unwrap_or_else(|| "默认".to_string());

        let user_name = user_col.and_then(|c| {
            let v = cell_to_string(&row[c]);
            if v.is_empty() { None } else { Some(v) }
        });

        records.push(ImportRecord {
            project_name, group_name,
            recorded_at: date.unwrap(),
            batch_no: batch, quantity: qty,
            user_name, extra_info: None,
        });
    }

    if records.is_empty() {
        return Err(AppError::Validation(format!("无有效数据（跳过{}行）", skip_count)));
    }

    Ok(records)
}
