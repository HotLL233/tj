/// 导出数据预览 API - v0.3.7
/// 提供 10 个预览端点，对应 10 个 Sheet 的数据查询

use axum::{extract::{Query, State}, Router, response::Json, routing::get};
use chrono::Datelike;
use crate::db::DbPool;
use crate::error::Result;
use crate::models::ApiResponse;
use super::rd_export_data;
use super::rd_export_handler::ExportQuery;

/// 解析日期范围（复用 export_handler 逻辑）
fn resolve_date_range(q: &ExportQuery) -> (String, String) {
    if let Some(ref s) = q.start {
        let end = q.end.as_ref().cloned().unwrap_or_else(|| s.clone());
        (s.clone(), end)
    } else {
        let now = chrono::Local::now();
        let start = format!("{}-{:02}-01", now.year(), now.month());

        let last_day = if now.month() == 12 {
            chrono::NaiveDate::from_ymd_opt(now.year() + 1, 1, 1)
                .and_then(|d| d.pred_opt())
        } else {
            chrono::NaiveDate::from_ymd_opt(now.year(), now.month() + 1, 1)
                .and_then(|d| d.pred_opt())
        };

        let end = if let Some(d) = last_day {
            d.format("%Y-%m-%d").to_string()
        } else {
            format!("{}-{:02}-28", now.year(), now.month())
        };

        (start, end)
    }
}

pub fn router(pool: DbPool) -> Router {
    Router::new()
        .route("/api/rd-export/preview/sheet1", get(preview_sheet1))
        .route("/api/rd-export/preview/sheet2", get(preview_sheet2))
        .route("/api/rd-export/preview/sheet3", get(preview_sheet3))
        .route("/api/rd-export/preview/sheet4", get(preview_sheet4))
        .route("/api/rd-export/preview/sheet5", get(preview_sheet5))
        .route("/api/rd-export/preview/sheet6", get(preview_sheet6))
        .route("/api/rd-export/preview/sheet7", get(preview_sheet7))
        .route("/api/rd-export/preview/sheet8", get(preview_sheet8))
        .route("/api/rd-export/preview/sheet9", get(preview_sheet9))
        .route("/api/rd-export/preview/sheet10", get(preview_sheet10))
        .with_state(pool)
}

// ========== Sheet 1: 各实验室项目方法对应表 ==========

async fn preview_sheet1(
    State(pool): State<DbPool>,
    Query(q): Query<ExportQuery>,
) -> Result<Json<ApiResponse<Vec<rd_export_data::FlatRow>>>> {
    let (start, end) = resolve_date_range(&q);
    let conn = pool.get()?;
    let data = rd_export_data::query_sheet1_data(&conn, &start, &end, q.group_id)?;
    Ok(Json(ApiResponse::ok(data)))
}

// ========== Sheet 2: 仪器-汇总 ==========

async fn preview_sheet2(
    State(pool): State<DbPool>,
    Query(q): Query<ExportQuery>,
) -> Result<Json<ApiResponse<Vec<rd_export_data::InstrumentDailyRow>>>> {
    let (start, end) = resolve_date_range(&q);
    let conn = pool.get()?;
    let data = rd_export_data::query_sheet2_data(&conn, &start, &end)?;
    Ok(Json(ApiResponse::ok(data)))
}

// ========== Sheet 3: 项目-汇总 ==========

async fn preview_sheet3(
    State(pool): State<DbPool>,
    Query(q): Query<ExportQuery>,
) -> Result<Json<ApiResponse<Vec<rd_export_data::ProjectSummaryRow>>>> {
    let (start, end) = resolve_date_range(&q);
    let conn = pool.get()?;
    let data = rd_export_data::query_sheet3_data(&conn, &start, &end)?;
    Ok(Json(ApiResponse::ok(data)))
}

// ========== Sheet 4: 实验室-汇总 ==========

async fn preview_sheet4(
    State(pool): State<DbPool>,
    Query(q): Query<ExportQuery>,
) -> Result<Json<ApiResponse<Vec<rd_export_data::LabSummaryRow>>>> {
    let (start, end) = resolve_date_range(&q);
    let conn = pool.get()?;
    let data = rd_export_data::query_sheet4_data(&conn, &start, &end)?;
    Ok(Json(ApiResponse::ok(data)))
}

// ========== Sheet 5: 人员-汇总（原始记录） ==========

async fn preview_sheet5(
    State(pool): State<DbPool>,
    Query(q): Query<ExportQuery>,
) -> Result<Json<ApiResponse<Vec<rd_export_data::PersonRecordRow>>>> {
    let (start, end) = resolve_date_range(&q);
    let conn = pool.get()?;
    let data = rd_export_data::query_sheet5_data(&conn, &start, &end)?;
    Ok(Json(ApiResponse::ok(data)))
}

// ========== Sheet 6: 人员汇总表 ==========

async fn preview_sheet6(
    State(pool): State<DbPool>,
    Query(q): Query<ExportQuery>,
) -> Result<Json<ApiResponse<Vec<rd_export_data::PersonSummaryRow>>>> {
    let (start, end) = resolve_date_range(&q);
    let conn = pool.get()?;
    let data = rd_export_data::query_sheet6_data(&conn, &start, &end)?;
    Ok(Json(ApiResponse::ok(data)))
}

// ========== Sheet 7: 实验室总表 ==========

async fn preview_sheet7(
    State(pool): State<DbPool>,
    Query(q): Query<ExportQuery>,
) -> Result<Json<ApiResponse<Vec<rd_export_data::LabTotalRow>>>> {
    let (start, end) = resolve_date_range(&q);
    let conn = pool.get()?;
    let data = rd_export_data::query_sheet7_data(&conn, &start, &end)?;
    Ok(Json(ApiResponse::ok(data)))
}

// ========== Sheet 8: 项目总表 ==========

async fn preview_sheet8(
    State(pool): State<DbPool>,
    Query(q): Query<ExportQuery>,
) -> Result<Json<ApiResponse<Vec<rd_export_data::ProjectTotalRow>>>> {
    let (start, end) = resolve_date_range(&q);
    let conn = pool.get()?;
    let data = rd_export_data::query_sheet8_data(&conn, &start, &end)?;
    Ok(Json(ApiResponse::ok(data)))
}

// ========== Sheet 9: 仪器汇总表 ==========

async fn preview_sheet9(
    State(pool): State<DbPool>,
    Query(q): Query<ExportQuery>,
) -> Result<Json<ApiResponse<Vec<rd_export_data::InstrumentSummaryRow>>>> {
    let (start, end) = resolve_date_range(&q);
    let conn = pool.get()?;
    let data = rd_export_data::query_sheet9_data(&conn, &start, &end)?;
    Ok(Json(ApiResponse::ok(data)))
}

// ========== Sheet 10: 理化汇总表 ==========

async fn preview_sheet10(
    State(pool): State<DbPool>,
    Query(q): Query<ExportQuery>,
) -> Result<Json<ApiResponse<Vec<rd_export_data::PhysChemRow>>>> {
    let (start, end) = resolve_date_range(&q);
    let conn = pool.get()?;
    let data = rd_export_data::query_sheet10_data(&conn, &start, &end)?;
    Ok(Json(ApiResponse::ok(data)))
}
