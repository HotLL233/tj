use crate::db::DbPool;
use crate::error::{AppError, Result};
use crate::models::sample_info::{
    SampleInfoCreate, SampleInfoQuery, SampleInfoRecord, SampleInfoResponse, SampleInfoUpdate,
};
use crate::repo::audit_repo;

const STATUS_ORDER: &[&str] = &["待检测", "待取样", "已取样", "检测完成"];

/// 构建通用 WHERE 子句（不含分页），所有筛选维度：
/// 时间(submitted_at) / 送样人 / 实验室 / 项目 / 类型(type_key) / 状态
fn build_where(q: &SampleInfoQuery) -> (String, Vec<String>) {
    let mut clauses: Vec<String> = vec!["deleted_at IS NULL".to_string()];
    let mut params: Vec<String> = vec![];
    if let Some(dt) = &q.detection_type {
        if !dt.is_empty() {
            let i = params.len() + 1;
            clauses.push(format!("detection_type=?{}", i));
            params.push(dt.clone());
        }
    }
    if let Some(tk) = &q.type_key {
        if !tk.is_empty() {
            let i = params.len() + 1;
            clauses.push(format!("type_key=?{}", i));
            params.push(tk.clone());
        }
    }
    if let Some(s) = &q.status {
        if !s.is_empty() && s != "全部" {
            let i = params.len() + 1;
            clauses.push(format!("status=?{}", i));
            params.push(s.clone());
        }
    }
    if let Some(u) = &q.user_name {
        if !u.is_empty() {
            let i = params.len() + 1;
            clauses.push(format!("user_name=?{}", i));
            params.push(u.clone());
        }
    }
    if let Some(l) = &q.lab_name {
        if !l.is_empty() {
            let i = params.len() + 1;
            clauses.push(format!("lab_name=?{}", i));
            params.push(l.clone());
        }
    }
    if let Some(p) = &q.project_name {
        if !p.is_empty() {
            let i = params.len() + 1;
            clauses.push(format!("project_name=?{}", i));
            params.push(p.clone());
        }
    }
    if let Some(s) = &q.start {
        let i = params.len() + 1;
        clauses.push(format!("submitted_at>=?{}", i));
        params.push(s.clone());
    }
    if let Some(e) = &q.end {
        let i = params.len() + 1;
        clauses.push(format!("submitted_at<=?{}", i));
        params.push(format!("{}T23:59:59", e));
    }
    (clauses.join(" AND "), params)
}

/// 分页查询，支持全部维度筛选，未软删除
pub fn list(pool: &DbPool, q: &SampleInfoQuery) -> Result<(Vec<SampleInfoResponse>, i64)> {
    let conn = pool.get()?;
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(20).min(500);

    let (where_sql, params) = build_where(q);

    let sql = format!(
        "SELECT id, status, seq_no, batch_no, user_name, lab_name, project_name, \
         submitted_at, detection_date, main_components, detection_type, type_key, notes, \
         created_at, updated_at, deleted_at \
         FROM sample_info_records WHERE {} ORDER BY created_at DESC \
         LIMIT {} OFFSET {}",
        where_sql,
        page_size,
        (page - 1) * page_size
    );

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(
        rusqlite::params_from_iter(params.iter().map(|p| p as &dyn rusqlite::types::ToSql)),
        |row| {
            Ok(SampleInfoRecord {
                id: row.get(0)?,
                status: row.get(1)?,
                seq_no: row.get(2)?,
                batch_no: row.get(3)?,
                user_name: row.get(4)?,
                lab_name: row.get(5)?,
                project_name: row.get(6)?,
                submitted_at: row.get(7)?,
                detection_date: row.get(8)?,
                main_components: row.get(9)?,
                detection_type: row.get(10)?,
                type_key: row.get(11)?,
                notes: row.get::<_, String>(12).unwrap_or_default(),
                created_at: row.get(13)?,
                updated_at: row.get(14)?,
                deleted_at: row.get(15)?,
            })
        },
    )?;
    let items: Vec<SampleInfoResponse> = rows
        .collect::<std::result::Result<Vec<_>, _>>()?
        .into_iter()
        .map(SampleInfoResponse::from)
        .collect();

    // Count
    let count_sql = format!("SELECT COUNT(*) FROM sample_info_records WHERE {}", where_sql);
    let count: i64 = conn.query_row(
        &count_sql,
        rusqlite::params_from_iter(params.iter().map(|p| p as &dyn rusqlite::types::ToSql)),
        |r| r.get(0),
    )?;

    Ok((items, count))
}

/// Internal helper: query a single record on an existing connection
fn get_by_id_on_conn(conn: &rusqlite::Connection, id: i64) -> Result<SampleInfoRecord> {
    conn.query_row(
        "SELECT id, status, seq_no, batch_no, user_name, lab_name, project_name, \
         submitted_at, detection_date, main_components, detection_type, type_key, notes, \
         created_at, updated_at, deleted_at \
         FROM sample_info_records WHERE id=?1",
        [id],
        |row| {
            Ok(SampleInfoRecord {
                id: row.get(0)?,
                status: row.get(1)?,
                seq_no: row.get(2)?,
                batch_no: row.get(3)?,
                user_name: row.get(4)?,
                lab_name: row.get(5)?,
                project_name: row.get(6)?,
                submitted_at: row.get(7)?,
                detection_date: row.get(8)?,
                main_components: row.get(9)?,
                detection_type: row.get(10)?,
                type_key: row.get(11)?,
                notes: row.get::<_, String>(12).unwrap_or_default(),
                created_at: row.get(13)?,
                updated_at: row.get(14)?,
                deleted_at: row.get(15)?,
            })
        },
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => {
            crate::error::AppError::NotFound("样品信息记录不存在".into())
        }
        _ => e.into(),
    })
}

/// 创建记录，自动计算 seq_no
pub fn create(pool: &DbPool, data: &SampleInfoCreate) -> Result<SampleInfoResponse> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;

    // 如果未提供 submitted_at，使用当前时间
    let submitted_at = data
        .submitted_at
        .clone()
        .unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string());

    // 计算 seq_no：同 detection_type + 同天最大 seq_no + 1
    let seq_no: i64 = tx
        .query_row(
            "SELECT COALESCE(MAX(seq_no), 0) + 1 FROM sample_info_records \
             WHERE detection_type = ?1 AND date(submitted_at) = date(?2) AND deleted_at IS NULL",
            rusqlite::params![&data.detection_type, &submitted_at],
            |r| r.get(0),
        )
        .unwrap_or(1);

    let notes = data.notes.clone().unwrap_or_default();
    let detection_date = data.detection_date.clone().unwrap_or_default();

    tx.execute(
        "INSERT INTO sample_info_records \
         (status, seq_no, batch_no, user_name, lab_name, project_name, submitted_at, \
          detection_date, main_components, detection_type, type_key, notes) \
         VALUES ('待检测', ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        rusqlite::params![
            seq_no,
            &data.batch_no,
            &data.user_name,
            &data.lab_name,
            &data.project_name,
            &submitted_at,
            &detection_date,
            &data.main_components,
            &data.detection_type,
            &data.type_key,
            &notes,
        ],
    )?;

    let id = tx.last_insert_rowid();
    let detail = format!(
        "创建样品信息#{}：检测类型「{}」，批号「{}」，送样人「{}」",
        id, &data.detection_type, &data.batch_no, &data.user_name
    );
    audit_repo::log_on_conn_with_module(
        &tx, "create", "sample_info_records", Some(id),
        &data.user_name, &detail, "sample_info",
    )?;
    tx.commit()?;

    let record = get_by_id_on_conn(&conn, id)?;
    Ok(SampleInfoResponse::from(record))
}

/// 更新记录，记录审计
pub fn update(
    pool: &DbPool,
    id: i64,
    data: &SampleInfoUpdate,
    user_name: &str,
) -> Result<SampleInfoResponse> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;

    let existing = get_by_id_on_conn(&tx, id)?;
    if existing.deleted_at.is_some() {
        return Err(AppError::Validation("记录已被删除，无法编辑".into()));
    }

    let mut changed = false;
    let mut changes: Vec<String> = vec![];

    if let Some(ref s) = data.status {
        if s != &existing.status {
            changes.push(format!("状态 {} → {}", existing.status, s));
            tx.execute(
                "UPDATE sample_info_records SET status=?1, updated_at=datetime('now','localtime') WHERE id=?2",
                rusqlite::params![s, id],
            )?;
            changed = true;
        }
    }
    if let Some(ref b) = data.batch_no {
        if b != &existing.batch_no {
            changes.push(format!("批号 {} → {}", existing.batch_no, b));
            tx.execute(
                "UPDATE sample_info_records SET batch_no=?1, updated_at=datetime('now','localtime') WHERE id=?2",
                rusqlite::params![b, id],
            )?;
            changed = true;
        }
    }
    if let Some(ref u) = data.user_name {
        if u != &existing.user_name {
            changes.push(format!("送样人 {} → {}", existing.user_name, u));
            tx.execute(
                "UPDATE sample_info_records SET user_name=?1, updated_at=datetime('now','localtime') WHERE id=?2",
                rusqlite::params![u, id],
            )?;
            changed = true;
        }
    }
    if let Some(ref l) = data.lab_name {
        if l != &existing.lab_name {
            changes.push(format!("实验室 {} → {}", existing.lab_name, l));
            tx.execute(
                "UPDATE sample_info_records SET lab_name=?1, updated_at=datetime('now','localtime') WHERE id=?2",
                rusqlite::params![l, id],
            )?;
            changed = true;
        }
    }
    if let Some(ref p) = data.project_name {
        if p != &existing.project_name {
            changes.push(format!("项目 {} → {}", existing.project_name, p));
            tx.execute(
                "UPDATE sample_info_records SET project_name=?1, updated_at=datetime('now','localtime') WHERE id=?2",
                rusqlite::params![p, id],
            )?;
            changed = true;
        }
    }
    if let Some(ref sa) = data.submitted_at {
        if sa != &existing.submitted_at {
            changes.push(format!("送样时间 {} → {}", existing.submitted_at, sa));
            tx.execute(
                "UPDATE sample_info_records SET submitted_at=?1, updated_at=datetime('now','localtime') WHERE id=?2",
                rusqlite::params![sa, id],
            )?;
            changed = true;
        }
    }
    if let Some(ref dd) = data.detection_date {
        if dd != &existing.detection_date {
            changes.push(format!("检测时间 {} → {}", existing.detection_date, dd));
            tx.execute(
                "UPDATE sample_info_records SET detection_date=?1, updated_at=datetime('now','localtime') WHERE id=?2",
                rusqlite::params![dd, id],
            )?;
            changed = true;
        }
    }
    if let Some(ref mc) = data.main_components {
        if mc != &existing.main_components {
            changes.push(format!("主要成分 {} → {}", existing.main_components, mc));
            tx.execute(
                "UPDATE sample_info_records SET main_components=?1, updated_at=datetime('now','localtime') WHERE id=?2",
                rusqlite::params![mc, id],
            )?;
            changed = true;
        }
    }
    if let Some(ref n) = data.notes {
        if n != &existing.notes {
            changes.push(format!("注意事项 {} → {}", existing.notes, n));
            tx.execute(
                "UPDATE sample_info_records SET notes=?1, updated_at=datetime('now','localtime') WHERE id=?2",
                rusqlite::params![n, id],
            )?;
            changed = true;
        }
    }

    if !changed {
        return Err(AppError::Validation("没有需要更新的字段".into()));
    }
    if changes.is_empty() {
        changes.push("无变化".into());
    }
    let detail = format!("修改样品信息#{}：{}", id, changes.join("，"));
    audit_repo::log_on_conn_with_module(
        &tx, "update", "sample_info_records", Some(id),
        user_name, &detail, "sample_info",
    )?;
    tx.commit()?;

    let record = get_by_id_on_conn(&conn, id)?;
    Ok(SampleInfoResponse::from(record))
}

/// 软删除
pub fn soft_delete(pool: &DbPool, id: i64, user_name: &str) -> Result<()> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;

    let deleted: Option<String> = tx
        .query_row(
            "SELECT deleted_at FROM sample_info_records WHERE id=?1",
            [id],
            |r| r.get(0),
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                AppError::NotFound("样品信息记录不存在".into())
            }
            _ => e.into(),
        })?;

    if deleted.is_some() {
        return Err(AppError::Validation("记录已被删除".into()));
    }

    let rows = tx.execute(
        "UPDATE sample_info_records SET deleted_at=datetime('now','localtime') WHERE id=?1",
        [id],
    )?;
    if rows == 0 {
        return Err(AppError::NotFound("样品信息记录不存在".into()));
    }

    let detail = format!("删除样品信息#{}", id);
    audit_repo::log_on_conn_with_module(
        &tx, "delete", "sample_info_records", Some(id),
        user_name, &detail, "sample_info",
    )?;
    tx.commit()?;
    Ok(())
}

/// 状态流转
pub fn update_status(
    pool: &DbPool,
    id: i64,
    new_status: &str,
    user_name: &str,
) -> Result<SampleInfoResponse> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;

    let existing = get_by_id_on_conn(&tx, id)?;
    if existing.deleted_at.is_some() {
        return Err(AppError::Validation("记录已被删除".into()));
    }

    // 验证状态值有效性
    if !STATUS_ORDER.contains(&new_status) {
        return Err(AppError::Validation(format!(
            "无效状态: {}，允许的状态: {}",
            new_status,
            STATUS_ORDER.join(", ")
        )));
    }

    // 检测完成不可再流转
    if existing.status == "检测完成" {
        return Err(AppError::Validation("检测完成状态不可再流转".into()));
    }

    // 只能按顺序流转，不可跳转
    let current_idx = STATUS_ORDER
        .iter()
        .position(|&s| s == existing.status)
        .unwrap_or(0);
    let target_idx = STATUS_ORDER
        .iter()
        .position(|&s| s == new_status)
        .unwrap_or(0);

    if target_idx != current_idx + 1 {
        return Err(AppError::Validation(format!(
            "状态只能按顺序流转：{} → {}，不能直接流转到 {}",
            existing.status,
            STATUS_ORDER.get(current_idx + 1).unwrap_or(&"检测完成"),
            new_status
        )));
    }

    // 检测完成：自动写入检测时间；其他状态只更新 status
    if new_status == "检测完成" {
        tx.execute(
            "UPDATE sample_info_records SET status=?1, detection_date=date('now','localtime'), updated_at=datetime('now','localtime') WHERE id=?2",
            rusqlite::params![new_status, id],
        )?;
    } else {
        tx.execute(
            "UPDATE sample_info_records SET status=?1, updated_at=datetime('now','localtime') WHERE id=?2",
            rusqlite::params![new_status, id],
        )?;
    }

    let detail = format!(
        "样品信息#{} 状态流转：{} → {}",
        id, existing.status, new_status
    );
    audit_repo::log_on_conn_with_module(
        &tx, "status_change", "sample_info_records", Some(id),
        user_name, &detail, "sample_info",
    )?;
    tx.commit()?;

    let record = get_by_id_on_conn(&conn, id)?;
    Ok(SampleInfoResponse::from(record))
}
