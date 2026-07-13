//! 审批中心 API：审批规则管理 + 审批任务决策。

use axum::{extract::{Path, Query, State, Json}, Router, routing::{get, post, put, delete}};

use crate::db::DbPool;
use crate::error::Result;
use crate::models::ApiResponse;
use crate::models::approval::{ApprovalAction, ApprovalRuleCreate, ApprovalRuleUpdate};
use crate::middleware::auth::AuthedUser;
use crate::repo;
use crate::service::approval_service;

pub fn router(pool: DbPool) -> Router {
    Router::new()
        .route("/api/approval/rules", get(list_rules).post(create_rule))
        .route("/api/approval/rules/:id", put(update_rule).delete(delete_rule))
        .route("/api/approval/tasks", get(list_tasks))
        .route("/api/approval/tasks/:id/decide", post(decide))
        .with_state(pool)
}

#[derive(serde::Deserialize)]
struct RuleQuery { biz_type: Option<String> }
#[derive(serde::Deserialize)]
struct TaskQuery { view: Option<String> }

async fn list_rules(State(pool): State<DbPool>, user: AuthedUser, Query(q): Query<RuleQuery>) -> Result<Json<ApiResponse<Vec<crate::models::approval::ApprovalRule>>>> {
    user.require("approval_rule:read")?;
    Ok(Json(ApiResponse::ok(repo::approval_repo::list_rules(&pool, q.biz_type.as_deref())?)))
}

async fn create_rule(State(pool): State<DbPool>, user: AuthedUser, Json(body): Json<ApprovalRuleCreate>) -> Result<Json<ApiResponse<crate::models::approval::ApprovalRule>>> {
    user.require("approval_rule:manage")?;
    Ok(Json(ApiResponse::ok(repo::approval_repo::create_rule(&pool, &body, &user.username)?)))
}

async fn update_rule(State(pool): State<DbPool>, user: AuthedUser, Path(id): Path<i64>, Json(body): Json<ApprovalRuleUpdate>) -> Result<Json<ApiResponse<crate::models::approval::ApprovalRule>>> {
    user.require("approval_rule:manage")?;
    Ok(Json(ApiResponse::ok(repo::approval_repo::update_rule(&pool, id, &body, &user.username)?)))
}

async fn delete_rule(State(pool): State<DbPool>, user: AuthedUser, Path(id): Path<i64>) -> Result<Json<ApiResponse<()>>> {
    user.require("approval_rule:manage")?;
    repo::approval_repo::delete_rule(&pool, id, &user.username)?;
    Ok(Json(ApiResponse::ok_msg("审批规则已删除")))
}

async fn list_tasks(State(pool): State<DbPool>, user: AuthedUser, Query(q): Query<TaskQuery>) -> Result<Json<ApiResponse<Vec<crate::models::approval::ApprovalTask>>>> {
    user.require("approval:read")?;
    let items = match q.view.as_deref() {
        Some("mine") => repo::approval_repo::list_tasks(&pool, None, None, None, None, Some(&user.username))?,
        Some("all") => repo::approval_repo::list_tasks(&pool, None, None, None, None, None)?,
        _ => {
            // 待我审批：审批人=我 或 审批角色=我的角色
            let mut todo = repo::approval_repo::list_tasks(&pool, None, Some("待审批"), Some(&user.username), None, None)?;
            let by_role = repo::approval_repo::list_tasks(&pool, None, Some("待审批"), None, Some(&user.role), None)?;
            todo.extend(by_role);
            todo
        }
    };
    Ok(Json(ApiResponse::ok(items)))
}

async fn decide(State(pool): State<DbPool>, user: AuthedUser, Path(id): Path<i64>, Json(body): Json<ApprovalAction>) -> Result<Json<ApiResponse<()>>> {
    // 须具备审批权限；且为任务指派的审批人 / 审批角色
    user.require("approval:approve")?;
    let task = repo::approval_repo::get_task(&pool, id)?;
    let is_assignee = task.approver.as_deref() == Some(&user.username)
        || task.approver_role.as_deref() == Some(&user.role);
    if !is_assignee {
        return Err(crate::error::AppError::Forbidden("您不是该任务的指定审批人".into()));
    }
    if body.decision != "approve" && body.decision != "reject" {
        return Err(crate::error::AppError::Validation("decision 必须为 approve 或 reject".into()));
    }
    approval_service::decide(&pool, id, &body.decision, &user.username, &body.note)?;
    Ok(Json(ApiResponse::ok_msg("审批已完成")))
}
