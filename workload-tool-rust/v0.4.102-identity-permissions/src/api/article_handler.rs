use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    Json, Router,
};
use serde::Deserialize;
use crate::db::DbPool;
use crate::error::Result;
use crate::models::ApiResponse;
use crate::models::article::{HelpArticle, HelpArticleUpdate};
use crate::repo::article_repo;
use crate::service::authz_service;

fn require_help(pool: &DbPool, headers: &HeaderMap, edit: bool) -> Result<()> {
    let ctx = authz_service::authenticate(pool, headers)?;
    authz_service::require_permission(&ctx, if edit { "help:edit" } else { "manage:help" })
}

pub fn router(pool: DbPool) -> Router {
    Router::new()
        .route("/api/help-articles", axum::routing::get(list).post(create))
        .route("/api/help-articles/sort", axum::routing::put(reorder))
        .route("/api/help-articles/:id", axum::routing::get(get).put(update).delete(delete))
        .with_state(pool)
}

#[derive(Deserialize)]
struct ListQuery { visible_only: Option<bool> }

async fn list(State(pool): State<DbPool>, headers: HeaderMap, Query(q): Query<ListQuery>) -> Result<Json<ApiResponse<Vec<HelpArticle>>>> {
    require_help(&pool, &headers, false)?;
    Ok(Json(ApiResponse::ok(article_repo::list(&pool, q.visible_only.unwrap_or(false))?)))
}

async fn get(State(pool): State<DbPool>, headers: HeaderMap, Path(id): Path<i64>) -> Result<Json<ApiResponse<HelpArticle>>> {
    require_help(&pool, &headers, false)?;
    Ok(Json(ApiResponse::ok(article_repo::get_by_id(&pool, id)?)))
}

#[derive(Deserialize)]
struct CreateBody { title: String, content_html: String, toc_json: Option<String>, source_file: Option<String> }

async fn create(State(pool): State<DbPool>, headers: HeaderMap, Json(b): Json<CreateBody>) -> Result<Json<ApiResponse<HelpArticle>>> {
    require_help(&pool, &headers, true)?;
    Ok(Json(ApiResponse::ok(article_repo::create(&pool, &b.title, &b.content_html, b.toc_json.as_deref(), b.source_file.as_deref())?)))
}

async fn update(State(pool): State<DbPool>, headers: HeaderMap, Path(id): Path<i64>, Json(b): Json<HelpArticleUpdate>) -> Result<Json<ApiResponse<HelpArticle>>> {
    require_help(&pool, &headers, true)?;
    Ok(Json(ApiResponse::ok(article_repo::update(&pool, id, &b)?)))
}

async fn delete(State(pool): State<DbPool>, headers: HeaderMap, Path(id): Path<i64>) -> Result<Json<ApiResponse<()>>> {
    require_help(&pool, &headers, true)?;
    article_repo::delete(&pool, id)?;
    Ok(Json(ApiResponse::ok_msg("删除成功")))
}

#[derive(Deserialize)]
struct ArticleReorderItem { id: i64, sort_order: i64 }
#[derive(Deserialize)]
struct ArticleReorderBody { ids: Vec<ArticleReorderItem> }
async fn reorder(State(pool): State<DbPool>, headers: HeaderMap, Json(body): Json<ArticleReorderBody>) -> Result<Json<ApiResponse<()>>> {
    require_help(&pool, &headers, true)?;
    let items: Vec<(i64, i64)> = body.ids.iter().map(|x| (x.id, x.sort_order)).collect();
    article_repo::reorder(&pool, &items)?;
    Ok(Json(ApiResponse::ok(())))
}
