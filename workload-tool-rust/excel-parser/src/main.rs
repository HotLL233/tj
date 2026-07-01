//! excel-parser: 独立 CLI 工具 — 解析 Excel 并输出 JSON
//! 
//! 用法: excel-parser.exe <file.xlsx>
//! 输出: JSON 到 stdout, 日志到 stderr
//!
//! 代码直接来自 D:\桌面\py\rust-tool\src-tauri\src\lib.rs, 零改动移植

use calamine::{open_workbook, Reader, Xlsx, DataType};
use regex::Regex;
use serde::Serialize;
use std::env;
use std::path::Path;

// ===================== 常量（与 rust-tool 一致） =====================

const SKIP_ROWS: usize = 2;
const DATE_KEYWORDS: &[&str] = &[
    "日期","date","时间","time","检测日期","采样日期",
    "报告日期","分析日期","送检日期","检验日期",
];
const BATCH_KEYWORDS: &[&str] = &[
    "批号","batch","lot","编号","序号",
    "样品批号","样品编号","样品号","样品名称",
    "实验编号","样本号","code","id",
];

// ===================== 输出结构 =====================

#[derive(Serialize)]
struct SheetResult {
    name: String,
    rows_read: usize,
    rows_parsed: usize,
    rows_skipped: usize,
    date_col: usize,
    batch_col: usize,
    records: Vec<RecordEntry>,
}

#[derive(Serialize)]
struct RecordEntry {
    date: String,
    batch_no: String,
}

#[derive(Serialize)]
struct Output {
    success: bool,
    file_name: String,
    sheets: Vec<SheetResult>,
    error: String,
}

// ===================== 日期/数值处理（与 rust-tool 一致） =====================

fn year_ok(d: &chrono::NaiveDate) -> bool {
    let y: i32 = d.format("%Y").to_string().parse().unwrap_or(0);
    y >= 1900 && y <= 2100
}

fn try_parse_yyyymmdd(s: &str) -> Option<String> {
    let clean = if let Some(dot) = s.find('.') { &s[..dot] } else { s };
    if clean.len() == 8 && clean.chars().all(|c| c.is_ascii_digit()) {
        if let Ok(d) = chrono::NaiveDate::parse_from_str(clean, "%Y%m%d") {
            if year_ok(&d) { return Some(d.format("%Y-%m-%d").to_string()); }
        }
    }
    None
}

fn ensure_date_str(v: &DataType) -> String {
    if let Some(s) = v.as_string() {
        let s = s.trim().to_string();
        if s.is_empty() { return String::new(); }
        if let Some(d) = try_parse_yyyymmdd(&s) { return d; }
        for fmt in &["%Y-%m-%d", "%Y/%m/%d", "%Y.%m.%d"] {
            if let Ok(d) = chrono::NaiveDate::parse_from_str(&s, fmt) { return d.format("%Y-%m-%d").to_string(); }
        }
        return String::new();
    }
    if let Some(f) = v.as_f64() {
        let s = format!("{:.0}", f);
        if let Some(d) = try_parse_yyyymmdd(&s) { return d; }
        if f >= 40000.0 && f <= 200000.0 {
            if let Some(d) = chrono::NaiveDate::from_ymd_opt(1899, 12, 30).unwrap()
                .checked_add_signed(chrono::Duration::days(f as i64)) {
                if year_ok(&d) { return d.format("%Y-%m-%d").to_string(); }
            }
        }
        return String::new();
    }
    String::new()
}

fn get_as_string(v: &DataType) -> String {
    if let Some(s) = v.as_string() { return s.trim().to_string(); }
    if let Some(f) = v.as_f64() {
        if f.fract() == 0.0 && f >= 0.0 { return format!("{:.0}", f); }
        return f.to_string();
    }
    if let Some(i) = v.as_i64() { return i.to_string(); }
    String::new()
}

// ===================== 检测逻辑（与 rust-tool 一致） =====================

fn detect_data_start_row(range: &calamine::Range<DataType>, max_scan: usize) -> usize {
    let re = Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();
    let ncols = (range.width() as usize).min(30);
    let nrows = (range.height() as usize).min(max_scan);
    for row_idx in 0..nrows {
        for col_idx in 0..ncols {
            if let Some(val) = range.get((row_idx, col_idx)) {
                if re.is_match(&ensure_date_str(val)) { return row_idx; }
            }
        }
    }
    SKIP_ROWS
}

fn detect_columns(range: &calamine::Range<DataType>, header_end: usize) -> (usize, usize) {
    let max_header_scan: usize = 3;
    let mut date_col = 0usize; let mut batch_col = 1usize;
    let mut df = false; let mut bf = false;
    let h = header_end.min(range.height() as usize);
    let w = (range.width() as usize).min(30);
    if h == 0 || w < 2 { return (date_col, batch_col); }
    let scan_start = if h > max_header_scan { h - max_header_scan } else { 0 };
    for row_idx in (scan_start..h).rev() {
        for col_idx in 0..w {
            if let Some(val) = range.get((row_idx, col_idx)) {
                let s = val.as_string().unwrap_or_default().to_lowercase();
                if s.trim().is_empty() { continue; }
                if !df && DATE_KEYWORDS.iter().any(|kw| s.contains(kw)) { date_col = col_idx; df = true; }
                if !bf && BATCH_KEYWORDS.iter().any(|kw| s.contains(kw)) { batch_col = col_idx; bf = true; }
            }
        }
        if df && bf { break; }
    }
    (date_col, batch_col)
}

fn process_sheet(range: &calamine::Range<DataType>, sheet_name: &str) -> Option<SheetResult> {
    let auto_skip = detect_data_start_row(range, 50);
    let h = range.height() as usize;
    let w = range.width() as usize;

    let header_rows = auto_skip.min(h);
    let (dc, bc) = if header_rows > 0 {
        detect_columns(range, header_rows)
    } else {
        (0usize, 1usize)
    };
    if dc >= w || bc >= w { return None; }

    let re = Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();
    let mut records: Vec<RecordEntry> = Vec::new();
    let mut last_date = String::new();
    let prefixes = ["例","示例","sample"];
    let mut rows_read = 0usize;
    let mut rows_skipped = 0usize;

    for row_idx in auto_skip..h {
        rows_read += 1;
        let dv = range.get((row_idx, dc)).map(|v| ensure_date_str(v)).unwrap_or_default();
        let date = if dv.is_empty() { last_date.clone() } else { last_date = dv.clone(); dv };
        if !re.is_match(&date) { rows_skipped += 1; continue; }

        let bv = range.get((row_idx, bc)).map(|v| get_as_string(v)).unwrap_or_default();
        if bv.is_empty() { rows_skipped += 1; continue; }
        if prefixes.iter().any(|p| bv.starts_with(p)) { rows_skipped += 1; continue; }

        records.push(RecordEntry { date, batch_no: bv });
    }

    if records.is_empty() { None } else {
        Some(SheetResult {
            name: sheet_name.to_string(),
            rows_read,
            rows_parsed: records.len(),
            rows_skipped,
            date_col: dc,
            batch_col: bc,
            records,
        })
    }
}

// ===================== 主函数 =====================

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("用法: excel-parser.exe <file.xlsx>");
        println!("{}", serde_json::to_string(&Output {
            success: false,
            file_name: String::new(),
            sheets: vec![],
            error: "缺少参数: 请提供 Excel 文件路径".to_string(),
        }).unwrap());
        std::process::exit(1);
    }

    let file_path = &args[1];
    let file_name = Path::new(file_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    // 打开文件
    let mut workbook: Xlsx<_> = match open_workbook(file_path) {
        Ok(wb) => wb,
        Err(e) => {
            let output = Output {
                success: false,
                file_name,
                sheets: vec![],
                error: format!("无法打开 Excel 文件: {}", e),
            };
            println!("{}", serde_json::to_string(&output).unwrap());
            std::process::exit(1);
        }
    };

    let sheet_names = workbook.sheet_names().to_vec();
    let mut sheets: Vec<SheetResult> = vec![];

    for name in &sheet_names {
        if name == "目录" { continue; }
        let range = match workbook.worksheet_range(name) {
            Ok(r) => r,
            Err(_) => continue,
        };
        if let Some(result) = process_sheet(&range, name) {
            sheets.push(result);
        }
    }

    if sheets.is_empty() {
        let output = Output {
            success: false,
            file_name,
            sheets: vec![],
            error: format!("未能从任何 Sheet 中解析到有效数据。Sheet 列表: {:?}", sheet_names),
        };
        println!("{}", serde_json::to_string(&output).unwrap());
        std::process::exit(1);
    }

    let total_records: usize = sheets.iter().map(|s| s.records.len()).sum();
    eprintln!("解析完成: {} 个 Sheet, {} 条记录", sheets.len(), total_records);

    let output = Output {
        success: true,
        file_name,
        sheets,
        error: String::new(),
    };
    println!("{}", serde_json::to_string(&output).unwrap());
}
