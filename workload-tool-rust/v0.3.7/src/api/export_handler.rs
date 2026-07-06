/// Excel 导出处理器 - v0.3.6 重写版本
/// 实现 10 个 Sheet 的导出功能

use axum::{extract::{Query, State}, Router, routing::get};
use axum::response::IntoResponse;
use axum::http::header;
use serde::Deserialize;
use crate::db::DbPool;
use crate::error::Result;
use super::{export_data, export_write};
use chrono::Datelike;

#[derive(Deserialize, utoipa::IntoParams)]
pub struct ExportQuery {
    /// 起始日期 (YYYY-MM-DD)，默认当月第一天
    pub start: Option<String>,
    /// 结束日期 (YYYY-MM-DD)，默认当月最后一天
    pub end: Option<String>,
    /// 筛选分组 ID（仅 Sheet 1 使用）
    pub group_id: Option<i64>,
}

pub fn router(pool: DbPool) -> Router {
    Router::new()
        .route("/api/export/excel", get(export_excel))
        .with_state(pool)
}

/// 主导出函数：生成包含 10 个 Sheet 的 Excel 文件
async fn export_excel(
    State(pool): State<DbPool>,
    Query(q): Query<ExportQuery>
) -> Result<impl IntoResponse> {
    use std::io::Cursor;

    // 确定日期范围
    let start_str: String;
    let end_str: String;
    let (start, end) = if let Some(ref s) = q.start {
        start_str = s.clone();
        end_str = q.end.as_ref().cloned().unwrap_or_else(|| s.clone());
        (start_str.as_str(), end_str.as_str())
    } else {
        // 默认当月
        let now = chrono::Local::now();
        start_str = format!("{}-{:02}-01", now.year(), now.month());

        // 计算月末
        let last_day = if now.month() == 12 {
            chrono::NaiveDate::from_ymd_opt(now.year() + 1, 1, 1)
                .and_then(|d| d.pred_opt())
        } else {
            chrono::NaiveDate::from_ymd_opt(now.year(), now.month() + 1, 1)
                .and_then(|d| d.pred_opt())
        };

        end_str = if let Some(d) = last_day {
            d.format("%Y-%m-%d").to_string()
        } else {
            format!("{}-{:02}-28", now.year(), now.month())
        };

        (start_str.as_str(), end_str.as_str())
    };

    let conn = pool.get()?;
    let fmt = export_write::Fmt::new();
    let mut wb = rust_xlsxwriter::Workbook::new();

    tracing::info!("开始导出 Excel: start={}, end={}, group_id={:?}", start, end, q.group_id);

    // ========== Sheet 1: 各实验室项目方法对应表 ==========
    match (|| -> Result<()> {
        let data = export_data::query_sheet1_data(&conn, start, end, q.group_id)?;
        tracing::info!("Sheet 1 查询完成: {} 行", data.len());
        let ws = wb.add_worksheet();
        export_write::write_sheet1(ws, &data, &fmt)?;
        tracing::info!("Sheet 1 写入完成");
        Ok(())
    })() {
        Ok(_) => {},
        Err(e) => {
            tracing::error!("Sheet 1 失败: {}", e);
            return Err(e);
        }
    }

    // ========== Sheet 2: 仪器-汇总 ==========
    match (|| -> Result<()> {
        let data = export_data::query_sheet2_data(&conn, start, end)?;
        tracing::info!("Sheet 2 查询完成: {} 行", data.len());
        let ws = wb.add_worksheet();
        export_write::write_sheet2(ws, &data, &fmt)?;
        tracing::info!("Sheet 2 写入完成");
        Ok(())
    })() {
        Ok(_) => {},
        Err(e) => {
            tracing::error!("Sheet 2 失败: {}", e);
            return Err(e);
        }
    }

    // ========== Sheet 3: 项目-汇总 ==========
    match (|| -> Result<()> {
        let data = export_data::query_sheet3_data(&conn, start, end)?;
        tracing::info!("Sheet 3 查询完成: {} 行", data.len());
        let ws = wb.add_worksheet();
        export_write::write_sheet3(ws, &data, &fmt)?;
        tracing::info!("Sheet 3 写入完成");
        Ok(())
    })() {
        Ok(_) => {},
        Err(e) => {
            tracing::error!("Sheet 3 失败: {}", e);
            return Err(e);
        }
    }

    // ========== Sheet 4: 实验室-汇总 ==========
    match (|| -> Result<()> {
        let data = export_data::query_sheet4_data(&conn, start, end)?;
        tracing::info!("Sheet 4 查询完成: {} 行", data.len());
        let ws = wb.add_worksheet();
        export_write::write_sheet4(ws, &data, &fmt)?;
        tracing::info!("Sheet 4 写入完成");
        Ok(())
    })() {
        Ok(_) => {},
        Err(e) => {
            tracing::error!("Sheet 4 失败: {}", e);
            return Err(e);
        }
    }

    // ========== Sheet 5: 人员-汇总（原始记录） ==========
    match (|| -> Result<()> {
        let data = export_data::query_sheet5_data(&conn, start, end)?;
        tracing::info!("Sheet 5 查询完成: {} 行", data.len());
        let ws = wb.add_worksheet();
        export_write::write_sheet5(ws, &data, &fmt)?;
        tracing::info!("Sheet 5 写入完成");
        Ok(())
    })() {
        Ok(_) => {},
        Err(e) => {
            tracing::error!("Sheet 5 失败: {}", e);
            return Err(e);
        }
    }

    // ========== Sheet 6: 人员汇总表 ==========
    match (|| -> Result<()> {
        let data = export_data::query_sheet6_data(&conn, start, end)?;
        tracing::info!("Sheet 6 查询完成: {} 行", data.len());
        let ws = wb.add_worksheet();
        export_write::write_sheet6(ws, &data, &fmt)?;
        tracing::info!("Sheet 6 写入完成");
        Ok(())
    })() {
        Ok(_) => {},
        Err(e) => {
            tracing::error!("Sheet 6 失败: {}", e);
            return Err(e);
        }
    }

    // ========== Sheet 7: 实验室总表 ==========
    match (|| -> Result<()> {
        let data = export_data::query_sheet7_data(&conn, start, end)?;
        tracing::info!("Sheet 7 查询完成: {} 行", data.len());
        let ws = wb.add_worksheet();
        export_write::write_sheet7(ws, &data, &fmt)?;
        tracing::info!("Sheet 7 写入完成");
        Ok(())
    })() {
        Ok(_) => {},
        Err(e) => {
            tracing::error!("Sheet 7 失败: {}", e);
            return Err(e);
        }
    }

    // ========== Sheet 8: 项目总表 ==========
    match (|| -> Result<()> {
        let data = export_data::query_sheet8_data(&conn, start, end)?;
        tracing::info!("Sheet 8 查询完成: {} 行", data.len());
        let ws = wb.add_worksheet();
        export_write::write_sheet8(ws, &data, &fmt)?;
        tracing::info!("Sheet 8 写入完成");
        Ok(())
    })() {
        Ok(_) => {},
        Err(e) => {
            tracing::error!("Sheet 8 失败: {}", e);
            return Err(e);
        }
    }

    // ========== Sheet 9: 仪器汇总表 ==========
    match (|| -> Result<()> {
        let data = export_data::query_sheet9_data(&conn, start, end)?;
        tracing::info!("Sheet 9 查询完成: {} 行", data.len());
        let ws = wb.add_worksheet();
        export_write::write_sheet9(ws, &data, &fmt)?;
        tracing::info!("Sheet 9 写入完成");
        Ok(())
    })() {
        Ok(_) => {},
        Err(e) => {
            tracing::error!("Sheet 9 失败: {}", e);
            return Err(e);
        }
    }

    // ========== Sheet 10: 理化汇总表 ==========
    match (|| -> Result<()> {
        let data = export_data::query_sheet10_data(&conn, start, end)?;
        tracing::info!("Sheet 10 查询完成: {} 行", data.len());
        let ws = wb.add_worksheet();
        export_write::write_sheet10(ws, &data, &fmt)?;
        tracing::info!("Sheet 10 写入完成");
        Ok(())
    })() {
        Ok(_) => {},
        Err(e) => {
            tracing::error!("Sheet 10 失败: {}", e);
            return Err(e);
        }
    }

    // 保存到内存
    let mut buf = Cursor::new(Vec::new());
    wb.save_to_writer(&mut buf).map_err(|e| crate::error::AppError::Internal(e.to_string()))?;
    let data = buf.into_inner();

    // 生成文件名
    let filename = format!("工作量统计_{}_{}.xlsx", start, end);
    let encoded_filename = format!("attachment; filename*=UTF-8''{}", url_escape::encode_component(&filename));

    tracing::info!("Excel 导出完成: {} bytes", data.len());

    Ok(axum::response::Response::builder()
        .header(header::CONTENT_TYPE, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
        .header(header::CONTENT_DISPOSITION, encoded_filename)
        .body(axum::body::Body::from(data))
        .unwrap())
}
