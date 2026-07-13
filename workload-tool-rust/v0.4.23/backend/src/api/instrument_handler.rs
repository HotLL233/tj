//! 仪器管理 API：档案 / 预约 / 保养 / 二维码。

use axum::{extract::{Path, Query, State, Json}, Router, routing::{get, post, put, delete as delete_route}};
use serde::Deserialize;

use crate::db::DbPool;
use crate::error::Result;
use crate::models::ApiResponse;
use crate::models::instrument::{BookingCreate, InstrumentCreate, InstrumentUpdate, MaintenanceCreate};
use crate::middleware::auth::AuthedUser;
use crate::repo;
use crate::service::instrument_service;
use crate::utils::qrcode_util;

pub fn router(pool: DbPool) -> Router {
    Router::new()
        .route("/api/instruments", get(list).post(create))
        .route("/api/instruments/:id", get(get_one).put(update).delete(delete))
        .route("/api/instruments/:id/qrcode", post(generate_qr))
        .route("/api/instrument-bookings", get(list_bookings).post(submit_booking))
        .route("/api/instrument-maintenances", get(list_maintenances).post(add_maintenance))
        .with_state(pool)
}

#[derive(Deserialize)]
struct IdQuery { instrument_id: Option<i64>, status: Option<String>, applicant: Option<String> }

async fn list(State(pool): State<DbPool>, user: AuthedUser) -> Result<Json<ApiResponse<Vec<crate::models::instrument::InstrumentResponse>>>> {
    user.require("instrument:read")?;
    Ok(Json(ApiResponse::ok(repo::instrument_repo::list(&pool)?)))
}

async fn get_one(State(pool): State<DbPool>, user: AuthedUser, Path(id): Path<i64>) -> Result<Json<ApiResponse<crate::models::instrument::InstrumentResponse>>> {
    user.require("instrument:read")?;
    Ok(Json(ApiResponse::ok(repo::instrument_repo::get(&pool, id)?)))
}

async fn create(State(pool): State<DbPool>, user: AuthedUser, Json(body): Json<InstrumentCreate>) -> Result<Json<ApiResponse<crate::models::instrument::InstrumentResponse>>> {
    user.require("instrument:write")?;
    Ok(Json(ApiResponse::ok(instrument_service::create_instrument(&pool, &body, &user.username, &user.username)?)))
}

async fn update(State(pool): State<DbPool>, user: AuthedUser, Path(id): Path<i64>, Json(body): Json<InstrumentUpdate>) -> Result<Json<ApiResponse<crate::models::instrument::InstrumentResponse>>> {
    user.require("instrument:write")?;
    Ok(Json(ApiResponse::ok(instrument_service::update_instrument(&pool, id, &body, &user.username)?)))
}

async fn delete(State(pool): State<DbPool>, user: AuthedUser, Path(id): Path<i64>) -> Result<Json<ApiResponse<()>>> {
    user.require("instrument:write")?;
    instrument_service::delete_instrument(&pool, id, &user.username)?;
    Ok(Json(ApiResponse::ok_msg("仪器已删除")))
}

#[derive(Serialize)]
struct QrData { qr_data_url: String, qr_code_path: String }

async fn generate_qr(State(pool): State<DbPool>, user: AuthedUser, Path(id): Path<i64>) -> Result<Json<ApiResponse<QrData>>> {
    user.require("instrument:write")?;
    let inst = repo::instrument_repo::get(&pool, id)?;
    let content = format!("仪器：{}\n编号：{}", inst.name, id);
    let (abs_path, data_url) = qrcode_util::generate(&content, &format!("instrument_{}.png", id))?;
    repo::instrument_repo::set_qr_code(&pool, id, &abs_path)?;
    Ok(Json(ApiResponse::ok(QrData { qr_data_url: data_url, qr_code_path: abs_path })))
}

async fn list_bookings(State(pool): State<DbPool>, user: AuthedUser, Query(q): Query<IdQuery>) -> Result<Json<ApiResponse<Vec<crate::models::instrument::BookingResponse>>>> {
    user.require("instrument:read")?;
    let status = q.status.as_deref();
    let applicant = q.applicant.as_deref();
    Ok(Json(ApiResponse::ok(repo::instrument_repo::list_bookings(&pool, q.instrument_id, status, applicant)?)))
}

async fn submit_booking(State(pool): State<DbPool>, user: AuthedUser, Json(body): Json<BookingCreate>) -> Result<Json<ApiResponse<crate::models::instrument::BookingResponse>>> {
    user.require("instrument:book")?;
    Ok(Json(ApiResponse::ok(instrument_service::submit_booking(&pool, &body, &user.role, &user.username)?)))
}

async fn list_maintenances(State(pool): State<DbPool>, user: AuthedUser, Query(q): Query<IdQuery>) -> Result<Json<ApiResponse<Vec<crate::models::instrument::MaintenanceResponse>>>> {
    user.require("instrument:read")?;
    Ok(Json(ApiResponse::ok(repo::instrument_repo::list_maintenances(&pool, q.instrument_id)?)))
}

async fn add_maintenance(State(pool): State<DbPool>, user: AuthedUser, Json(body): Json<MaintenanceCreate>) -> Result<Json<ApiResponse<()>>> {
    user.require("instrument:write")?;
    instrument_service::add_maintenance(&pool, &body, &user.username)?;
    Ok(Json(ApiResponse::ok_msg("保养已登记")))
}

use serde::Serialize;
