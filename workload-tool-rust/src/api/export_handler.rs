use axum::{extract::{Query, State}, Router, routing::get};
use axum::response::IntoResponse;
use axum::http::header;
use serde::Deserialize;
use crate::db::DbPool;
use crate::error::Result;

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

async fn export_excel(
    State(_pool): State<DbPool>,
    Query(q): Query<ExportQuery>,
) -> Result<impl IntoResponse> {
    use rust_xlsxwriter::*;
    use std::io::Cursor;

    let mut workbook = Workbook::new();
    let ws = workbook.add_worksheet();
    ws.set_name("月-汇总").map_err(|e| crate::error::AppError::Internal(e.to_string()))?;
    ws.set_tab_color(Color::RGB(0x1976D2));

    let font_h = Format::new()
        .set_bold()
        .set_font_size(16);
    let font_d = Format::new()
        .set_font_size(16);

    let headers = ["使用实验室", "项目代号", "液相仪器", "检测方法", "月检测数量", "液相检测量", "气相检测量", "项目检测总量"];
    for (i, h) in headers.iter().enumerate() {
        ws.write_with_format(1, (i + 1) as u16, *h, &font_h)?;
    }
    ws.set_column_width(0, 8.89)?;
    ws.set_column_width(1, 24.89)?;
    ws.set_column_width(2, 18.0)?;
    ws.set_column_width(3, 17.44)?;
    ws.set_column_width(4, 43.66)?;
    ws.set_column_width(5, 19.66)?;

    // 示例数据行
    ws.write_with_format(3, 1, "测试数据", &font_d)?;
    ws.write_with_format(3, 5, 42, &font_d)?;

    let mut buf = Cursor::new(Vec::new());
    workbook.save_to_writer(&mut buf).map_err(|e| crate::error::AppError::Internal(e.to_string()))?;
    let data = buf.into_inner();

    let headers = [
        (header::CONTENT_TYPE, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
        (header::CONTENT_DISPOSITION, "attachment; filename*=UTF-8''%E5%B7%A5%E4%BD%9C%E9%87%8F%E7%BB%9F%E8%AE%A1.xlsx"),
    ];
    Ok((headers, data))
}
