/// 样品信息登记导出 — Excel 写入层（独立模块，复用 export_write::Fmt 样式）
use rust_xlsxwriter::*;
use crate::error::{Result, AppError};
use super::export_write::Fmt;
use super::sample_info_export_data::*;

/// Sheet 1: 全部记录明细
pub fn write_sheet_detail(ws: &mut Worksheet, rows: &[SampleInfoExportRow], fmt: &Fmt) -> Result<()> {
    ws.set_name("记录明细").map_err(|e| AppError::Internal(e.to_string()))?;
    ws.set_tab_color(Color::RGB(0x2E7D32));

    let widths = [10.0, 16.0, 12.0, 16.0, 18.0, 18.0, 14.0, 14.0, 12.0, 28.0, 30.0];
    for (i, w) in widths.iter().enumerate() {
        ws.set_column_width(i as u16, *w)?;
        ws.set_column_format(i as u16, &fmt.fd)?;
    }
    let headers = ["序号", "样品批号", "送样人", "实验室/车间", "所属项目", "送样时间", "检测时间", "检测类型", "状态", "样品主要成分", "注意事项"];
    for (i, h) in headers.iter().enumerate() {
        ws.write_with_format(0u32, i as u16, *h, &fmt.fh)?;
    }
    let mut r = 1u32;
    for row in rows {
        ws.write_with_format(r, 0, row.seq_no as f64, &fmt.fd)?;
        ws.write_with_format(r, 1, row.batch_no.as_str(), &fmt.fd)?;
        ws.write_with_format(r, 2, row.user_name.as_str(), &fmt.fd)?;
        ws.write_with_format(r, 3, row.lab_name.as_str(), &fmt.fd)?;
        ws.write_with_format(r, 4, row.project_name.as_str(), &fmt.fd)?;
        ws.write_with_format(r, 5, row.submitted_at.as_str(), &fmt.fd)?;
        ws.write_with_format(r, 6, row.detection_date.as_str(), &fmt.fd)?;
        ws.write_with_format(r, 7, row.detection_type.as_str(), &fmt.fd)?;
        ws.write_with_format(r, 8, row.status.as_str(), &fmt.fd)?;
        ws.write_with_format(r, 9, row.main_components.as_str(), &fmt.fd)?;
        ws.write_with_format(r, 10, row.notes.as_str(), &fmt.fd)?;
        r += 1;
    }
    Ok(())
}

/// 通用：写「名称-数量」型汇总 Sheet
fn write_name_count(ws: &mut Worksheet, title: &str, tab_color: u32, col_name: &str, rows: &[NameCountRow], fmt: &Fmt) -> Result<()> {
    ws.set_name(title).map_err(|e| AppError::Internal(e.to_string()))?;
    ws.set_tab_color(Color::RGB(tab_color));
    ws.set_column_width(0, 24.0)?;
    ws.set_column_width(1, 12.0)?;
    ws.set_column_format(0, &fmt.fd)?;
    ws.set_column_format(1, &fmt.fd)?;
    ws.write_with_format(0u32, 0, col_name, &fmt.fh)?;
    ws.write_with_format(0u32, 1, "数量", &fmt.fh)?;
    let mut r = 1u32;
    for row in rows {
        ws.write_with_format(r, 0, row.name.as_str(), &fmt.fd)?;
        ws.write_with_format(r, 1, row.count as f64, &fmt.fd)?;
        r += 1;
    }
    Ok(())
}

/// Sheet 2: 按状态
pub fn write_sheet_by_status(ws: &mut Worksheet, rows: &[NameCountRow], fmt: &Fmt) -> Result<()> {
    write_name_count(ws, "按状态", 0xE65100, "状态", rows, fmt)
}

/// Sheet 3: 按检测类型
pub fn write_sheet_by_type(ws: &mut Worksheet, rows: &[TypeCountRow], fmt: &Fmt) -> Result<()> {
    ws.set_name("按检测类型").map_err(|e| AppError::Internal(e.to_string()))?;
    ws.set_tab_color(Color::RGB(0x6A1B9A));
    ws.set_column_width(0, 24.0)?;
    ws.set_column_width(1, 18.0)?;
    ws.set_column_width(2, 12.0)?;
    ws.set_column_format(0, &fmt.fd)?;
    ws.set_column_format(1, &fmt.fd)?;
    ws.set_column_format(2, &fmt.fd)?;
    ws.write_with_format(0u32, 0, "检测类型", &fmt.fh)?;
    ws.write_with_format(0u32, 1, "类型标识", &fmt.fh)?;
    ws.write_with_format(0u32, 2, "数量", &fmt.fh)?;
    let mut r = 1u32;
    for row in rows {
        ws.write_with_format(r, 0, row.label.as_str(), &fmt.fd)?;
        ws.write_with_format(r, 1, row.type_key.as_str(), &fmt.fd)?;
        ws.write_with_format(r, 2, row.count as f64, &fmt.fd)?;
        r += 1;
    }
    Ok(())
}

/// Sheet 4: 按实验室
pub fn write_sheet_by_lab(ws: &mut Worksheet, rows: &[NameCountRow], fmt: &Fmt) -> Result<()> {
    write_name_count(ws, "按实验室", 0x0277BD, "实验室/车间", rows, fmt)
}

/// Sheet 5: 按项目
pub fn write_sheet_by_project(ws: &mut Worksheet, rows: &[NameCountRow], fmt: &Fmt) -> Result<()> {
    write_name_count(ws, "按项目", 0x00897B, "所属项目", rows, fmt)
}

/// Sheet 6: 按送样人
pub fn write_sheet_by_user(ws: &mut Worksheet, rows: &[NameCountRow], fmt: &Fmt) -> Result<()> {
    write_name_count(ws, "按送样人", 0x558B2F, "送样人", rows, fmt)
}

/// Sheet 7: 按月份
pub fn write_sheet_by_month(ws: &mut Worksheet, rows: &[MonthCountRow], fmt: &Fmt) -> Result<()> {
    ws.set_name("按月份").map_err(|e| AppError::Internal(e.to_string()))?;
    ws.set_tab_color(Color::RGB(0x5E35B1));
    ws.set_column_width(0, 16.0)?;
    ws.set_column_width(1, 12.0)?;
    ws.set_column_format(0, &fmt.fd)?;
    ws.set_column_format(1, &fmt.fd)?;
    ws.write_with_format(0u32, 0, "月份", &fmt.fh)?;
    ws.write_with_format(0u32, 1, "数量", &fmt.fh)?;
    let mut r = 1u32;
    for row in rows {
        ws.write_with_format(r, 0, row.month.as_str(), &fmt.fd)?;
        ws.write_with_format(r, 1, row.count as f64, &fmt.fd)?;
        r += 1;
    }
    Ok(())
}
