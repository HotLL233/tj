use axum::{
    body::Body,
    extract::{Multipart, State},
    http::{header, Response, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use calamine::{open_workbook_auto, DataType, Range, Reader};
use rusqlite::{Connection, OptionalExtension, Transaction};
use rust_xlsxwriter::{Color, DataValidation, Format, FormatAlign, FormatBorder, Workbook, XlsxError};
use serde::Serialize;
use std::collections::{HashMap, HashSet};

use crate::db::DbPool;
use crate::error::{AppError, Result};
use crate::models::ApiResponse;

const MAX_UPLOAD_BYTES: usize = 20 * 1024 * 1024;
const DATA_SHEETS: [&str; 7] = ["部门", "实验室", "检测类型", "检测方法", "研发项目", "项目关联", "预检结果"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImportMode {
    Upsert,
    Skip,
}

impl ImportMode {
    fn parse(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "" | "upsert" => Ok(Self::Upsert),
            "skip" => Ok(Self::Skip),
            _ => Err(AppError::Validation("导入策略仅支持 upsert 或 skip".into())),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportIssue {
    pub sheet: String,
    pub row: usize,
    pub entity_type: String,
    pub name: String,
    pub action: String,
    pub level: String,
    pub message: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ImportCounts {
    pub total_rows: usize,
    pub departments: usize,
    pub labs: usize,
    pub method_types: usize,
    pub methods: usize,
    pub projects: usize,
    pub relations: usize,
    pub creates: usize,
    pub updates: usize,
    pub skips: usize,
    pub errors: usize,
    pub warnings: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct MasterImportPreview {
    pub valid: bool,
    pub mode: String,
    pub counts: ImportCounts,
    pub issues: Vec<ImportIssue>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MasterImportResult {
    pub success: bool,
    pub created: usize,
    pub updated: usize,
    pub skipped: usize,
    pub relation_sets: usize,
    pub message: String,
}

#[derive(Debug, Clone)]
struct DepartmentRow {
    row: usize,
    name: String,
    sort_order: i64,
    color: String,
    is_active: bool,
}

#[derive(Debug, Clone)]
struct LabRow {
    row: usize,
    name: String,
    department: String,
    sort_order: i64,
    show_in_work: bool,
    show_in_rd: bool,
}

#[derive(Debug, Clone)]
struct MethodTypeRow {
    row: usize,
    name: String,
    sort_order: i64,
}

#[derive(Debug, Clone)]
struct MethodRow {
    row: usize,
    name: String,
    full_name: String,
    method_types: Vec<String>,
    coefficient: f64,
    multiplier: f64,
    amount: f64,
    is_active: bool,
    notes: String,
}

#[derive(Debug, Clone)]
struct ProjectRow {
    row: usize,
    name: String,
    full_name: String,
    high_item: Option<String>,
    sort_order: i64,
    is_active: bool,
    notes: String,
}

#[derive(Debug, Clone)]
struct RelationRow {
    row: usize,
    project: String,
    labs: Vec<String>,
    methods: Vec<String>,
    primary_lab: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct ParsedData {
    departments: Vec<DepartmentRow>,
    labs: Vec<LabRow>,
    method_types: Vec<MethodTypeRow>,
    methods: Vec<MethodRow>,
    projects: Vec<ProjectRow>,
    relations: Vec<RelationRow>,
}

#[derive(Debug, Clone, Default)]
struct ApplyCounts {
    created: usize,
    updated: usize,
    skipped: usize,
    relation_sets: usize,
}

pub fn router(pool: DbPool) -> Router {
    Router::new()
        .route("/api/master-import/template", get(download_template))
        .route("/api/master-import/precheck", post(precheck))
        .route("/api/master-import/execute", post(execute))
        .with_state(pool)
}

async fn download_template() -> Result<impl IntoResponse> {
    let bytes = build_template()?;
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        )
        .header(
            header::CONTENT_DISPOSITION,
            "attachment; filename*=UTF-8''%E4%B8%BB%E6%95%B0%E6%8D%AE%E4%B8%80%E9%94%AE%E5%AF%BC%E5%85%A5%E6%A8%A1%E6%9D%BF_v0.4.76.xlsx",
        )
        .body(Body::from(bytes))
        .map_err(|e| AppError::Internal(format!("构建模板响应失败: {e}")))?;
    Ok(response)
}

async fn precheck(State(pool): State<DbPool>, multipart: Multipart) -> Result<Json<ApiResponse<MasterImportPreview>>> {
    let (bytes, mode) = read_upload(multipart).await?;
    let (data, mut issues) = parse_uploaded_workbook(&bytes)?;
    let conn = pool.get()?;
    let preview = build_preview(&conn, &data, mode, &mut issues)?;
    Ok(Json(ApiResponse::ok(preview)))
}

async fn execute(State(pool): State<DbPool>, multipart: Multipart) -> Result<Json<ApiResponse<MasterImportResult>>> {
    let (bytes, mode) = read_upload(multipart).await?;
    let (data, mut issues) = parse_uploaded_workbook(&bytes)?;
    let mut conn = pool.get()?;
    let preview = build_preview(&conn, &data, mode, &mut issues)?;
    if !preview.valid {
        return Err(AppError::Validation(format!(
            "预检未通过：{} 个错误，请修正模板后重试",
            preview.counts.errors
        )));
    }

    let tx = conn.transaction()?;
    let applied = apply_import(&tx, &data, mode)?;
    tx.execute(
        "INSERT INTO audit_log (action, table_name, user_name, detail) VALUES ('import','master_data','system',?1)",
        [format!(
            "主数据一键导入：新增{}，更新{}，跳过{}，关联{}",
            applied.created, applied.updated, applied.skipped, applied.relation_sets
        )],
    )?;
    tx.commit()?;

    let message = format!(
        "导入完成：新增 {}，更新 {}，跳过 {}，项目关联 {}",
        applied.created, applied.updated, applied.skipped, applied.relation_sets
    );
    Ok(Json(ApiResponse::ok(MasterImportResult {
        success: true,
        created: applied.created,
        updated: applied.updated,
        skipped: applied.skipped,
        relation_sets: applied.relation_sets,
        message,
    })))
}

async fn read_upload(mut multipart: Multipart) -> Result<(Vec<u8>, ImportMode)> {
    let mut file: Option<Vec<u8>> = None;
    let mut mode = ImportMode::Upsert;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("读取上传内容失败: {e}")))?
    {
        match field.name() {
            Some("file") => {
                let filename = field.file_name().unwrap_or_default().to_ascii_lowercase();
                if !filename.is_empty() && !filename.ends_with(".xlsx") {
                    return Err(AppError::Validation("请选择 .xlsx 模板文件".into()));
                }
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::Validation(format!("读取上传文件失败: {e}")))?;
                if bytes.is_empty() {
                    return Err(AppError::Validation("上传文件为空".into()));
                }
                if bytes.len() > MAX_UPLOAD_BYTES {
                    return Err(AppError::Validation("上传文件不能超过 20 MB".into()));
                }
                if !bytes.starts_with(b"PK") {
                    return Err(AppError::Validation("文件不是有效的 xlsx 工作簿".into()));
                }
                file = Some(bytes.to_vec());
            }
            Some("mode") => {
                let value = field
                    .text()
                    .await
                    .map_err(|e| AppError::Validation(format!("读取导入策略失败: {e}")))?;
                mode = ImportMode::parse(&value)?;
            }
            _ => {}
        }
    }
    Ok((
        file.ok_or_else(|| AppError::Validation("未收到模板文件".into()))?,
        mode,
    ))
}

fn build_template() -> std::result::Result<Vec<u8>, XlsxError> {
    let mut workbook = Workbook::new();
    let title = Format::new()
        .set_bold()
        .set_font_size(18)
        .set_font_color(Color::RGB(0x0B6E69));
    let section = Format::new()
        .set_bold()
        .set_font_color(Color::White)
        .set_background_color(Color::RGB(0x1976D2))
        .set_border(FormatBorder::Thin)
        .set_align(FormatAlign::Center);
    let wrap = Format::new().set_text_wrap().set_align(FormatAlign::Top);

    {
        let ws = workbook.add_worksheet();
        ws.set_name("使用说明")?;
        ws.set_column_width(0, 20)?;
        ws.set_column_width(1, 90)?;
        ws.merge_range(0, 0, 0, 1, "主数据一键导入模板 v0.4.76", &title)?;
        let rows = [
            ("导入顺序", "部门 → 实验室 → 检测类型 → 检测方法 → 研发项目 → 项目关联。系统会按该顺序自动处理。"),
            ("填写要求", "带 * 的字段必填；数据表从第 2 行开始填写，不要修改工作表名称和表头。"),
            ("多值分隔", "检测类型、关联实验室、关联检测方法支持使用中文分号、英文分号、顿号或逗号分隔。"),
            ("导入策略", "覆盖更新：同名数据更新字段和关联；跳过已有：同名数据保持不变，仅新增不存在的数据。"),
            ("高项逻辑", "高项直接写入研发项目的高项文本字段，与当前项目管理逻辑一致。"),
            ("安全机制", "请先在管理页面执行预检。正式导入使用单个事务，任一写入失败会整批回滚。"),
        ];
        for (idx, (name, description)) in rows.iter().enumerate() {
            ws.write_with_format((idx + 2) as u32, 0, *name, &section)?;
            ws.write_with_format((idx + 2) as u32, 1, *description, &wrap)?;
        }
    }

    {
        let ws = workbook.add_worksheet();
        ws.set_name("字段字典")?;
        let headers = ["工作表", "字段", "必填", "落库字段", "说明"];
        write_headers(ws, &headers)?;
        let rows = [
            ["部门", "部门名称*", "是", "divisions.name", "按名称匹配"],
            ["实验室", "所属部门*", "是", "project_groups.division_id", "可引用同一模板内的部门"],
            ["检测方法", "检测类型*", "是", "method_type_links", "支持多个类型"],
            ["研发项目", "高项", "否", "projects.high_item", "纯文本，不使用旧高项表"],
            ["项目关联", "关联实验室*", "是", "project_lab_links", "覆盖更新模式下替换该项目关联"],
            ["项目关联", "关联检测方法*", "是", "project_method_links", "覆盖更新模式下替换该项目关联"],
        ];
        for (r, values) in rows.iter().enumerate() {
            for (c, value) in values.iter().enumerate() {
                ws.write((r + 1) as u32, c as u16, *value)?;
            }
        }
        for (col, width) in [18.0, 24.0, 10.0, 30.0, 48.0].iter().enumerate() {
            ws.set_column_width(col as u16, *width)?;
        }
        ws.set_freeze_panes(1, 0)?;
    }

    add_data_sheet(&mut workbook, "部门", &["部门名称*", "排序", "颜色", "启用"], &[24.0, 10.0, 14.0, 10.0], &[3])?;
    add_data_sheet(&mut workbook, "实验室", &["实验室名称*", "所属部门*", "排序", "工作量显示", "研发送样显示"], &[24.0, 24.0, 10.0, 14.0, 14.0], &[3, 4])?;
    add_data_sheet(&mut workbook, "检测类型", &["类型名称*", "排序"], &[24.0, 10.0], &[])?;
    add_data_sheet(&mut workbook, "检测方法", &["方法简称*", "方法全称", "检测类型*", "系数", "倍率", "金额", "启用", "备注"], &[26.0, 36.0, 30.0, 10.0, 10.0, 12.0, 10.0, 36.0], &[6])?;
    add_data_sheet(&mut workbook, "研发项目", &["项目简称*", "项目全称", "高项", "排序", "启用", "备注"], &[26.0, 38.0, 20.0, 10.0, 10.0, 36.0], &[4])?;
    add_data_sheet(&mut workbook, "项目关联", &["项目简称*", "关联实验室*", "关联检测方法*", "主实验室"], &[26.0, 42.0, 50.0, 24.0], &[])?;
    add_data_sheet(&mut workbook, "预检结果", &["工作表", "行号", "对象类型", "对象名称", "处理方式", "状态", "提示"], &[18.0, 10.0, 18.0, 28.0, 16.0, 12.0, 56.0], &[])?;

    workbook.save_to_buffer()
}

fn write_headers(ws: &mut rust_xlsxwriter::Worksheet, headers: &[&str]) -> std::result::Result<(), XlsxError> {
    let format = Format::new()
        .set_bold()
        .set_font_color(Color::White)
        .set_background_color(Color::RGB(0x1976D2))
        .set_border(FormatBorder::Thin)
        .set_align(FormatAlign::Center)
        .set_text_wrap();
    for (col, header) in headers.iter().enumerate() {
        ws.write_with_format(0, col as u16, *header, &format)?;
    }
    Ok(())
}

fn add_data_sheet(
    workbook: &mut Workbook,
    name: &str,
    headers: &[&str],
    widths: &[f64],
    yes_no_columns: &[u16],
) -> std::result::Result<(), XlsxError> {
    let ws = workbook.add_worksheet();
    ws.set_name(name)?;
    write_headers(ws, headers)?;
    for (col, width) in widths.iter().enumerate() {
        ws.set_column_width(col as u16, *width)?;
    }
    ws.set_freeze_panes(1, 0)?;
    ws.autofilter(0, 0, 1000, (headers.len() - 1) as u16)?;
    if !yes_no_columns.is_empty() {
        let validation = DataValidation::new().allow_list_strings(&["是", "否"])?;
        for col in yes_no_columns {
            ws.add_data_validation(1, *col, 1000, *col, &validation)?;
        }
    }
    Ok(())
}

fn parse_uploaded_workbook(bytes: &[u8]) -> Result<(ParsedData, Vec<ImportIssue>)> {
    let path = std::env::temp_dir().join(format!("master_import_{}.xlsx", uuid::Uuid::new_v4()));
    std::fs::write(&path, bytes)
        .map_err(|e| AppError::Internal(format!("写入临时模板失败: {e}")))?;
    let result = parse_workbook_path(&path);
    let _ = std::fs::remove_file(path);
    result
}

fn parse_workbook_path(path: &std::path::Path) -> Result<(ParsedData, Vec<ImportIssue>)> {
    let mut workbook = open_workbook_auto(path)
        .map_err(|e| AppError::Validation(format!("无法打开 Excel 文件: {e}")))?;
    let sheet_names: HashSet<String> = workbook.sheet_names().iter().cloned().collect();
    for sheet in DATA_SHEETS {
        if !sheet_names.contains(sheet) {
            return Err(AppError::Validation(format!("缺少工作表「{sheet}」，请使用系统下载的模板")));
        }
    }

    let mut data = ParsedData::default();
    let mut issues = Vec::new();

    let departments = read_sheet_rows(&workbook.worksheet_range("部门").map_err(workbook_error)?)?;
    for row in departments {
        let name = required(&row, "部门名称*", "部门", "部门", &mut issues);
        if name.is_empty() { continue; }
        let sort_order = parse_i64(&row, "排序", 0, "部门", &name, &mut issues);
        let color = value(&row, "颜色");
        let color = if color.is_empty() { "#1976d2".into() } else if is_hex_color(&color) { color } else {
            push_error(&mut issues, "部门", row.row, "部门", &name, "颜色必须是 #RRGGBB 格式");
            "#1976d2".into()
        };
        let is_active = parse_bool(&row, "启用", true, "部门", &name, &mut issues);
        data.departments.push(DepartmentRow { row: row.row, name, sort_order, color, is_active });
    }

    let labs = read_sheet_rows(&workbook.worksheet_range("实验室").map_err(workbook_error)?)?;
    for row in labs {
        let name = required(&row, "实验室名称*", "实验室", "实验室", &mut issues);
        let department = required(&row, "所属部门*", "实验室", "实验室", &mut issues);
        if name.is_empty() || department.is_empty() { continue; }
        data.labs.push(LabRow {
            row: row.row,
            sort_order: parse_i64(&row, "排序", 0, "实验室", &name, &mut issues),
            show_in_work: parse_bool(&row, "工作量显示", true, "实验室", &name, &mut issues),
            show_in_rd: parse_bool(&row, "研发送样显示", true, "实验室", &name, &mut issues),
            name,
            department,
        });
    }

    let method_types = read_sheet_rows(&workbook.worksheet_range("检测类型").map_err(workbook_error)?)?;
    for row in method_types {
        let name = required(&row, "类型名称*", "检测类型", "检测类型", &mut issues);
        if name.is_empty() { continue; }
        data.method_types.push(MethodTypeRow {
            row: row.row,
            sort_order: parse_i64(&row, "排序", 0, "检测类型", &name, &mut issues),
            name,
        });
    }

    let methods = read_sheet_rows(&workbook.worksheet_range("检测方法").map_err(workbook_error)?)?;
    for row in methods {
        let name = required(&row, "方法简称*", "检测方法", "检测方法", &mut issues);
        let method_types = split_multi(&required(&row, "检测类型*", "检测方法", "检测方法", &mut issues));
        if name.is_empty() || method_types.is_empty() { continue; }
        data.methods.push(MethodRow {
            row: row.row,
            full_name: value(&row, "方法全称"),
            coefficient: parse_f64(&row, "系数", 1.0, "检测方法", &name, &mut issues),
            multiplier: parse_f64(&row, "倍率", 1.0, "检测方法", &name, &mut issues),
            amount: parse_f64(&row, "金额", 0.0, "检测方法", &name, &mut issues),
            is_active: parse_bool(&row, "启用", true, "检测方法", &name, &mut issues),
            notes: value(&row, "备注"),
            name,
            method_types,
        });
    }

    let projects = read_sheet_rows(&workbook.worksheet_range("研发项目").map_err(workbook_error)?)?;
    for row in projects {
        let name = required(&row, "项目简称*", "研发项目", "研发项目", &mut issues);
        if name.is_empty() { continue; }
        let high_item = value(&row, "高项");
        data.projects.push(ProjectRow {
            row: row.row,
            full_name: value(&row, "项目全称"),
            high_item: if high_item.is_empty() { None } else { Some(high_item) },
            sort_order: parse_i64(&row, "排序", 0, "研发项目", &name, &mut issues),
            is_active: parse_bool(&row, "启用", true, "研发项目", &name, &mut issues),
            notes: value(&row, "备注"),
            name,
        });
    }

    let relations = read_sheet_rows(&workbook.worksheet_range("项目关联").map_err(workbook_error)?)?;
    for row in relations {
        let project = required(&row, "项目简称*", "项目关联", "项目关联", &mut issues);
        let labs = split_multi(&required(&row, "关联实验室*", "项目关联", "项目关联", &mut issues));
        let methods = split_multi(&required(&row, "关联检测方法*", "项目关联", "项目关联", &mut issues));
        if project.is_empty() || labs.is_empty() || methods.is_empty() { continue; }
        let primary = value(&row, "主实验室");
        if !primary.is_empty() && !labs.iter().any(|x| x == &primary) {
            push_error(&mut issues, "项目关联", row.row, "项目关联", &project, "主实验室必须包含在关联实验室中");
        }
        data.relations.push(RelationRow {
            row: row.row,
            project,
            labs,
            methods,
            primary_lab: if primary.is_empty() { None } else { Some(primary) },
        });
    }

    check_duplicates(&data.departments, |x| (&x.name, x.row), "部门", "部门", &mut issues);
    check_duplicates(&data.labs, |x| (&x.name, x.row), "实验室", "实验室", &mut issues);
    check_duplicates(&data.method_types, |x| (&x.name, x.row), "检测类型", "检测类型", &mut issues);
    check_duplicates(&data.methods, |x| (&x.name, x.row), "检测方法", "检测方法", &mut issues);
    check_duplicates(&data.projects, |x| (&x.name, x.row), "研发项目", "研发项目", &mut issues);
    check_duplicates(&data.relations, |x| (&x.project, x.row), "项目关联", "项目关联", &mut issues);

    Ok((data, issues))
}

#[derive(Debug)]
struct SheetRow {
    row: usize,
    values: HashMap<String, String>,
}

fn read_sheet_rows(range: &Range<DataType>) -> Result<Vec<SheetRow>> {
    let mut rows = range.rows();
    let headers: Vec<String> = rows
        .next()
        .ok_or_else(|| AppError::Validation("工作表缺少表头".into()))?
        .iter()
        .map(cell_to_string)
        .collect();
    let mut result = Vec::new();
    for (index, cells) in rows.enumerate() {
        let mut values = HashMap::new();
        let mut has_value = false;
        for (col, header) in headers.iter().enumerate() {
            if header.is_empty() { continue; }
            let cell = cells.get(col).map(cell_to_string).unwrap_or_default();
            if !cell.is_empty() { has_value = true; }
            values.insert(header.clone(), cell);
        }
        if has_value {
            result.push(SheetRow { row: index + 2, values });
        }
    }
    Ok(result)
}

fn workbook_error(error: calamine::Error) -> AppError {
    AppError::Validation(format!("读取工作表失败: {error}"))
}

fn cell_to_string(cell: &DataType) -> String {
    match cell {
        DataType::String(value) | DataType::DateTimeIso(value) | DataType::DurationIso(value) => value.trim().to_string(),
        DataType::Float(value) | DataType::DateTime(value) | DataType::Duration(value) => {
            if value.fract() == 0.0 { format!("{}", *value as i64) } else { value.to_string() }
        }
        DataType::Int(value) => value.to_string(),
        DataType::Bool(value) => if *value { "是".into() } else { "否".into() },
        DataType::Empty | DataType::Error(_) => String::new(),
    }
}

fn value(row: &SheetRow, header: &str) -> String {
    row.values.get(header).map(|x| x.trim().to_string()).unwrap_or_default()
}

fn required(row: &SheetRow, header: &str, sheet: &str, entity_type: &str, issues: &mut Vec<ImportIssue>) -> String {
    let result = value(row, header);
    if result.is_empty() {
        push_error(issues, sheet, row.row, entity_type, "", &format!("必填字段「{header}」不能为空"));
    }
    result
}

fn parse_i64(row: &SheetRow, header: &str, default: i64, entity_type: &str, name: &str, issues: &mut Vec<ImportIssue>) -> i64 {
    let raw = value(row, header);
    if raw.is_empty() { return default; }
    raw.parse::<i64>().unwrap_or_else(|_| {
        push_error(issues, entity_type, row.row, entity_type, name, &format!("字段「{header}」必须是整数"));
        default
    })
}

fn parse_f64(row: &SheetRow, header: &str, default: f64, entity_type: &str, name: &str, issues: &mut Vec<ImportIssue>) -> f64 {
    let raw = value(row, header);
    if raw.is_empty() { return default; }
    match raw.parse::<f64>() {
        Ok(value) if value >= 0.0 => value,
        _ => {
            push_error(issues, entity_type, row.row, entity_type, name, &format!("字段「{header}」必须是非负数字"));
            default
        }
    }
}

fn parse_bool(row: &SheetRow, header: &str, default: bool, entity_type: &str, name: &str, issues: &mut Vec<ImportIssue>) -> bool {
    let raw = value(row, header);
    if raw.is_empty() { return default; }
    match raw.to_ascii_lowercase().as_str() {
        "是" | "启用" | "true" | "1" | "yes" => true,
        "否" | "停用" | "false" | "0" | "no" => false,
        _ => {
            push_error(issues, entity_type, row.row, entity_type, name, &format!("字段「{header}」只能填写是或否"));
            default
        }
    }
}

fn split_multi(raw: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    raw.split([';', '；', '、', ',', '，', '\n'])
        .map(str::trim)
        .filter(|x| !x.is_empty())
        .filter(|x| seen.insert((*x).to_string()))
        .map(ToString::to_string)
        .collect()
}

fn is_hex_color(value: &str) -> bool {
    value.len() == 7 && value.starts_with('#') && value[1..].chars().all(|x| x.is_ascii_hexdigit())
}

fn push_error(issues: &mut Vec<ImportIssue>, sheet: &str, row: usize, entity_type: &str, name: &str, message: &str) {
    issues.push(ImportIssue {
        sheet: sheet.into(), row, entity_type: entity_type.into(), name: name.into(),
        action: "阻止导入".into(), level: "error".into(), message: message.into(),
    });
}

fn check_duplicates<T, F>(items: &[T], key: F, sheet: &str, entity_type: &str, issues: &mut Vec<ImportIssue>)
where
    F: Fn(&T) -> (&String, usize),
{
    let mut seen = HashMap::<String, usize>::new();
    for item in items {
        let (name, row) = key(item);
        if let Some(first_row) = seen.insert(name.clone(), row) {
            push_error(issues, sheet, row, entity_type, name, &format!("模板内名称重复，首次出现在第 {first_row} 行"));
        }
    }
}

fn load_names(conn: &Connection, sql: &str) -> Result<HashSet<String>> {
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
    Ok(rows.filter_map(|row| row.ok()).collect())
}

fn build_preview(conn: &Connection, data: &ParsedData, mode: ImportMode, issues: &mut Vec<ImportIssue>) -> Result<MasterImportPreview> {
    let existing_departments = load_names(conn, "SELECT name FROM divisions")?;
    let existing_labs = load_names(conn, "SELECT name FROM project_groups")?;
    let existing_types = load_names(conn, "SELECT name FROM method_types")?;
    let existing_methods = load_names(conn, "SELECT name FROM methods")?;
    let existing_projects = load_names(conn, "SELECT name FROM projects")?;

    let imported_departments: HashSet<_> = data.departments.iter().map(|x| x.name.clone()).collect();
    let imported_labs: HashSet<_> = data.labs.iter().map(|x| x.name.clone()).collect();
    let imported_types: HashSet<_> = data.method_types.iter().map(|x| x.name.clone()).collect();
    let imported_methods: HashSet<_> = data.methods.iter().map(|x| x.name.clone()).collect();
    let imported_projects: HashSet<_> = data.projects.iter().map(|x| x.name.clone()).collect();

    for row in &data.labs {
        if !existing_departments.contains(&row.department) && !imported_departments.contains(&row.department) {
            push_error(issues, "实验室", row.row, "实验室", &row.name, &format!("所属部门「{}」不存在", row.department));
        }
    }
    for row in &data.methods {
        for type_name in &row.method_types {
            if !existing_types.contains(type_name) && !imported_types.contains(type_name) {
                push_error(issues, "检测方法", row.row, "检测方法", &row.name, &format!("检测类型「{type_name}」不存在"));
            }
        }
    }
    let relation_projects: HashSet<_> = data.relations.iter().map(|x| x.project.clone()).collect();
    for row in &data.projects {
        if !existing_projects.contains(&row.name) && !relation_projects.contains(&row.name) {
            push_error(issues, "研发项目", row.row, "研发项目", &row.name, "新增项目必须在「项目关联」工作表中配置至少一个实验室和检测方法");
        }
    }
    for row in &data.relations {
        if !existing_projects.contains(&row.project) && !imported_projects.contains(&row.project) {
            push_error(issues, "项目关联", row.row, "项目关联", &row.project, "引用的研发项目不存在");
        }
        for lab in &row.labs {
            if !existing_labs.contains(lab) && !imported_labs.contains(lab) {
                push_error(issues, "项目关联", row.row, "项目关联", &row.project, &format!("关联实验室「{lab}」不存在"));
            }
        }
        for method in &row.methods {
            if !existing_methods.contains(method) && !imported_methods.contains(method) {
                push_error(issues, "项目关联", row.row, "项目关联", &row.project, &format!("关联检测方法「{method}」不存在"));
            }
        }
    }

    let mut counts = ImportCounts {
        departments: data.departments.len(), labs: data.labs.len(), method_types: data.method_types.len(),
        methods: data.methods.len(), projects: data.projects.len(), relations: data.relations.len(),
        ..ImportCounts::default()
    };
    counts.total_rows = counts.departments + counts.labs + counts.method_types + counts.methods + counts.projects + counts.relations;

    append_actions(issues, "部门", "部门", data.departments.iter().map(|x| (x.row, &x.name)), &existing_departments, mode, &mut counts);
    append_actions(issues, "实验室", "实验室", data.labs.iter().map(|x| (x.row, &x.name)), &existing_labs, mode, &mut counts);
    append_actions(issues, "检测类型", "检测类型", data.method_types.iter().map(|x| (x.row, &x.name)), &existing_types, mode, &mut counts);
    append_actions(issues, "检测方法", "检测方法", data.methods.iter().map(|x| (x.row, &x.name)), &existing_methods, mode, &mut counts);
    append_actions(issues, "研发项目", "研发项目", data.projects.iter().map(|x| (x.row, &x.name)), &existing_projects, mode, &mut counts);
    for row in &data.relations {
        issues.push(ImportIssue {
            sheet: "项目关联".into(), row: row.row, entity_type: "项目关联".into(), name: row.project.clone(),
            action: if mode == ImportMode::Skip && existing_projects.contains(&row.project) { "跳过".into() } else { "写入关联".into() },
            level: "info".into(), message: format!("{} 个实验室，{} 个检测方法", row.labs.len(), row.methods.len()),
        });
    }

    counts.errors = issues.iter().filter(|x| x.level == "error").count();
    counts.warnings = issues.iter().filter(|x| x.level == "warning").count();
    let valid = counts.errors == 0 && counts.total_rows > 0;
    if counts.total_rows == 0 {
        issues.push(ImportIssue {
            sheet: "模板".into(), row: 0, entity_type: "模板".into(), name: String::new(), action: "阻止导入".into(),
            level: "error".into(), message: "模板中没有可导入数据".into(),
        });
        counts.errors += 1;
    }

    Ok(MasterImportPreview {
        valid,
        mode: if mode == ImportMode::Upsert { "upsert" } else { "skip" }.into(),
        counts,
        issues: issues.clone(),
    })
}

fn append_actions<'a, I>(
    issues: &mut Vec<ImportIssue>, sheet: &str, entity_type: &str, rows: I,
    existing: &HashSet<String>, mode: ImportMode, counts: &mut ImportCounts,
)
where
    I: Iterator<Item = (usize, &'a String)>,
{
    for (row, name) in rows {
        let exists = existing.contains(name);
        let action = if !exists {
            counts.creates += 1;
            "新增"
        } else if mode == ImportMode::Upsert {
            counts.updates += 1;
            "更新"
        } else {
            counts.skips += 1;
            "跳过"
        };
        issues.push(ImportIssue {
            sheet: sheet.into(), row, entity_type: entity_type.into(), name: name.clone(), action: action.into(),
            level: "info".into(), message: if exists { "数据库中已存在同名数据".into() } else { "名称校验通过".into() },
        });
    }
}

fn apply_import(tx: &Transaction<'_>, data: &ParsedData, mode: ImportMode) -> Result<ApplyCounts> {
    let mut counts = ApplyCounts::default();
    let existing_project_names = load_names(tx, "SELECT name FROM projects")?;
    for row in &data.departments {
        let existing = id_by_name(tx, "divisions", &row.name)?;
        match (existing, mode) {
            (Some(id), ImportMode::Upsert) => {
                tx.execute("UPDATE divisions SET sort_order=?1,color=?2,is_active=?3,deleted_at=NULL WHERE id=?4", rusqlite::params![row.sort_order, row.color, row.is_active, id])?;
                counts.updated += 1;
            }
            (Some(_), ImportMode::Skip) => counts.skipped += 1,
            (None, _) => {
                tx.execute("INSERT INTO divisions (name,sort_order,color,is_active) VALUES (?1,?2,?3,?4)", rusqlite::params![row.name, row.sort_order, row.color, row.is_active])?;
                counts.created += 1;
            }
        }
    }
    for row in &data.labs {
        let division_id = id_by_name(tx, "divisions", &row.department)?.ok_or_else(|| AppError::Validation(format!("部门不存在: {}", row.department)))?;
        let existing = id_by_name(tx, "project_groups", &row.name)?;
        match (existing, mode) {
            (Some(id), ImportMode::Upsert) => {
                tx.execute("UPDATE project_groups SET sort_order=?1,show_in_work=?2,show_in_rd=?3,division_id=?4,deleted_at=NULL WHERE id=?5", rusqlite::params![row.sort_order, row.show_in_work, row.show_in_rd, division_id, id])?;
                counts.updated += 1;
            }
            (Some(_), ImportMode::Skip) => counts.skipped += 1,
            (None, _) => {
                tx.execute("INSERT INTO project_groups (name,sort_order,show_in_work,show_in_rd,division_id) VALUES (?1,?2,?3,?4,?5)", rusqlite::params![row.name, row.sort_order, row.show_in_work, row.show_in_rd, division_id])?;
                counts.created += 1;
            }
        }
    }
    for row in &data.method_types {
        let existing = id_by_name(tx, "method_types", &row.name)?;
        match (existing, mode) {
            (Some(id), ImportMode::Upsert) => {
                tx.execute("UPDATE method_types SET sort_order=?1 WHERE id=?2", rusqlite::params![row.sort_order, id])?;
                counts.updated += 1;
            }
            (Some(_), ImportMode::Skip) => counts.skipped += 1,
            (None, _) => {
                tx.execute("INSERT INTO method_types (name,sort_order) VALUES (?1,?2)", rusqlite::params![row.name, row.sort_order])?;
                counts.created += 1;
            }
        }
    }
    for row in &data.methods {
        let existing = id_by_name(tx, "methods", &row.name)?;
        let method_id = match (existing, mode) {
            (Some(id), ImportMode::Upsert) => {
                tx.execute("UPDATE methods SET full_name=?1,coefficient=?2,multiplier=?3,amount=?4,is_active=?5,notes=?6 WHERE id=?7", rusqlite::params![row.full_name, row.coefficient, row.multiplier, row.amount, row.is_active, row.notes, id])?;
                counts.updated += 1;
                id
            }
            (Some(id), ImportMode::Skip) => {
                counts.skipped += 1;
                id
            }
            (None, _) => {
                tx.execute("INSERT INTO methods (name,full_name,coefficient,multiplier,amount,is_active,notes) VALUES (?1,?2,?3,?4,?5,?6,?7)", rusqlite::params![row.name, row.full_name, row.coefficient, row.multiplier, row.amount, row.is_active, row.notes])?;
                counts.created += 1;
                tx.last_insert_rowid()
            }
        };
        if existing.is_none() || mode == ImportMode::Upsert {
            tx.execute("DELETE FROM method_type_links WHERE method_id=?1", [method_id])?;
            for type_name in &row.method_types {
                let type_id = id_by_name(tx, "method_types", type_name)?.ok_or_else(|| AppError::Validation(format!("检测类型不存在: {type_name}")))?;
                tx.execute("INSERT OR IGNORE INTO method_type_links (method_id,method_type_id) VALUES (?1,?2)", rusqlite::params![method_id, type_id])?;
            }
        }
    }

    let relation_map: HashMap<&str, &RelationRow> = data.relations.iter().map(|x| (x.project.as_str(), x)).collect();
    for row in &data.projects {
        let existing = id_by_name(tx, "projects", &row.name)?;
        let primary_group_id = if let Some(relation) = relation_map.get(row.name.as_str()) {
            let lab_name = relation.primary_lab.as_ref().unwrap_or(&relation.labs[0]);
            Some(id_by_name(tx, "project_groups", lab_name)?.ok_or_else(|| AppError::Validation(format!("实验室不存在: {lab_name}")))?)
        } else { None };
        match (existing, mode) {
            (Some(id), ImportMode::Upsert) => {
                tx.execute("UPDATE projects SET full_name=?1,high_item=?2,sort_order=?3,is_active=?4,notes=?5,group_id=COALESCE(?6,group_id) WHERE id=?7", rusqlite::params![row.full_name, row.high_item, row.sort_order, row.is_active, row.notes, primary_group_id, id])?;
                counts.updated += 1;
            }
            (Some(_), ImportMode::Skip) => counts.skipped += 1,
            (None, _) => {
                let group_id = primary_group_id.ok_or_else(|| AppError::Validation(format!("新增项目缺少实验室关联: {}", row.name)))?;
                tx.execute("INSERT INTO projects (group_id,name,full_name,high_item,sort_order,is_active,notes,method_type) VALUES (?1,?2,?3,?4,?5,?6,?7,'研发项目')", rusqlite::params![group_id, row.name, row.full_name, row.high_item, row.sort_order, row.is_active, row.notes])?;
                counts.created += 1;
            }
        }
    }

    for row in &data.relations {
        let project_id = id_by_name(tx, "projects", &row.project)?.ok_or_else(|| AppError::Validation(format!("项目不存在: {}", row.project)))?;
        if mode == ImportMode::Skip && existing_project_names.contains(&row.project) {
            counts.skipped += 1;
            continue;
        }
        tx.execute("DELETE FROM project_lab_links WHERE project_id=?1", [project_id])?;
        tx.execute("DELETE FROM project_method_links WHERE project_id=?1", [project_id])?;
        for lab in &row.labs {
            let group_id = id_by_name(tx, "project_groups", lab)?.ok_or_else(|| AppError::Validation(format!("实验室不存在: {lab}")))?;
            tx.execute("INSERT OR IGNORE INTO project_lab_links (project_id,group_id) VALUES (?1,?2)", rusqlite::params![project_id, group_id])?;
        }
        for method in &row.methods {
            let method_id = id_by_name(tx, "methods", method)?.ok_or_else(|| AppError::Validation(format!("检测方法不存在: {method}")))?;
            tx.execute("INSERT OR IGNORE INTO project_method_links (project_id,method_id) VALUES (?1,?2)", rusqlite::params![project_id, method_id])?;
        }
        let primary = row.primary_lab.as_ref().unwrap_or(&row.labs[0]);
        let primary_id = id_by_name(tx, "project_groups", primary)?.ok_or_else(|| AppError::Validation(format!("主实验室不存在: {primary}")))?;
        tx.execute("UPDATE projects SET group_id=?1 WHERE id=?2", rusqlite::params![primary_id, project_id])?;
        counts.relation_sets += 1;
    }
    Ok(counts)
}

fn id_by_name(conn: &Connection, table: &str, name: &str) -> Result<Option<i64>> {
    let sql = match table {
        "divisions" => "SELECT id FROM divisions WHERE name=?1 ORDER BY id LIMIT 1",
        "project_groups" => "SELECT id FROM project_groups WHERE name=?1 ORDER BY id LIMIT 1",
        "method_types" => "SELECT id FROM method_types WHERE name=?1 ORDER BY id LIMIT 1",
        "methods" => "SELECT id FROM methods WHERE name=?1 ORDER BY id LIMIT 1",
        "projects" => "SELECT id FROM projects WHERE name=?1 ORDER BY id LIMIT 1",
        _ => return Err(AppError::Internal("不支持的主数据表".into())),
    };
    Ok(conn.query_row(sql, [name], |row| row.get(0)).optional()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_template_has_all_required_sheets() {
        let bytes = build_template().expect("template");
        let path = std::env::temp_dir().join(format!("master_template_test_{}.xlsx", uuid::Uuid::new_v4()));
        std::fs::write(&path, bytes).expect("write template");
        let workbook = open_workbook_auto(&path).expect("open template");
        let names: HashSet<String> = workbook.sheet_names().iter().cloned().collect();
        for sheet in ["使用说明", "字段字典", "部门", "实验室", "检测类型", "检测方法", "研发项目", "项目关联", "预检结果"] {
            assert!(names.contains(sheet), "missing sheet {sheet}");
        }
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn split_multi_accepts_supported_separators() {
        assert_eq!(split_multi("A；B,C、D;A"), vec!["A", "B", "C", "D"]);
    }

    #[test]
    fn transaction_import_writes_master_data_and_links() {
        let mut conn = Connection::open_in_memory().expect("memory db");
        crate::db::migrations::run(&conn).expect("migrations");
        let data = ParsedData {
            departments: vec![DepartmentRow { row: 2, name: "测试部门".into(), sort_order: 20, color: "#1976d2".into(), is_active: true }],
            labs: vec![LabRow { row: 2, name: "测试实验室".into(), department: "测试部门".into(), sort_order: 20, show_in_work: true, show_in_rd: true }],
            method_types: vec![MethodTypeRow { row: 2, name: "测试类型".into(), sort_order: 20 }],
            methods: vec![MethodRow { row: 2, name: "测试方法".into(), full_name: "测试方法全称".into(), method_types: vec!["测试类型".into()], coefficient: 1.5, multiplier: 2.0, amount: 3.0, is_active: true, notes: "".into() }],
            projects: vec![ProjectRow { row: 2, name: "测试项目".into(), full_name: "测试项目全称".into(), high_item: Some("高项A".into()), sort_order: 20, is_active: true, notes: "".into() }],
            relations: vec![RelationRow { row: 2, project: "测试项目".into(), labs: vec!["测试实验室".into()], methods: vec!["测试方法".into()], primary_lab: Some("测试实验室".into()) }],
        };
        let tx = conn.transaction().expect("transaction");
        let counts = apply_import(&tx, &data, ImportMode::Upsert).expect("apply");
        assert_eq!(counts.created, 5);
        assert_eq!(counts.relation_sets, 1);
        tx.commit().expect("commit");

        let high_item: String = conn.query_row("SELECT high_item FROM projects WHERE name='测试项目'", [], |row| row.get(0)).expect("high item");
        assert_eq!(high_item, "高项A");
        let lab_links: i64 = conn.query_row("SELECT COUNT(*) FROM project_lab_links pll JOIN projects p ON p.id=pll.project_id WHERE p.name='测试项目'", [], |row| row.get(0)).expect("lab links");
        let method_links: i64 = conn.query_row("SELECT COUNT(*) FROM project_method_links pml JOIN projects p ON p.id=pml.project_id WHERE p.name='测试项目'", [], |row| row.get(0)).expect("method links");
        let type_links: i64 = conn.query_row("SELECT COUNT(*) FROM method_type_links mtl JOIN methods m ON m.id=mtl.method_id WHERE m.name='测试方法'", [], |row| row.get(0)).expect("type links");
        assert_eq!((lab_links, method_links, type_links), (1, 1, 1));
    }
}
