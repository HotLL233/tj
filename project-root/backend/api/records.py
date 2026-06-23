"""Work record CRUD endpoints with soft delete and audit logging."""
import json
from fastapi import APIRouter, Depends, Query
from database import get_db
from models import RecordCreate, RecordResponse, RecordUpdate, ApiResponse, PaginatedResponse

router = APIRouter(prefix="/records", tags=["records"])


def _write_audit(db, action: str, table_name: str, record_id: int, user_name: str, detail: str = ""):
    """Write an audit log entry."""
    db.execute(
        "INSERT INTO audit_log (action, table_name, record_id, user_name, detail) VALUES (?, ?, ?, ?, ?)",
        (action, table_name, record_id, user_name, detail),
    )
    db.commit()


def _row_to_response(row) -> RecordResponse:
    """Convert a database row to a RecordResponse."""
    return RecordResponse(
        id=row["id"],
        project_id=row["project_id"],
        project_name=row["project_name"] if "project_name" in row.keys() else "",
        group_name=row["group_name"] if "group_name" in row.keys() else "",
        user_name=row["user_name"],
        quantity=row["quantity"],
        recorded_at=row["recorded_at"],
        created_at=row["created_at"],
        deleted_at=row["deleted_at"],
    )


@router.get("")
def list_records(
    project_id: int = Query(None),
    user_name: str = Query(None),
    start: str = Query(None),
    end: str = Query(None),
    page: int = Query(1, ge=1),
    page_size: int = Query(50, ge=1, le=500),
    include_deleted: bool = Query(False),
    db=Depends(get_db),
):
    """List records with filtering and pagination."""
    conditions: list = []
    params: list = []

    if not include_deleted:
        conditions.append("wr.deleted_at IS NULL")

    if project_id is not None:
        conditions.append("wr.project_id = ?")
        params.append(project_id)

    if user_name is not None:
        conditions.append("wr.user_name = ?")
        params.append(user_name)

    if start is not None:
        conditions.append("wr.recorded_at >= ?")
        params.append(start)

    if end is not None:
        conditions.append("wr.recorded_at <= ?")
        params.append(end)

    where = " AND ".join(conditions) if conditions else "1=1"

    # Count total
    count_row = db.execute(
        f"""
        SELECT COUNT(*) AS cnt
        FROM work_records wr
        WHERE {where}
        """,
        params,
    ).fetchone()
    total = count_row["cnt"]

    # Fetch page
    offset = (page - 1) * page_size
    cursor = db.execute(
        f"""
        SELECT wr.*, p.name AS project_name, pg.name AS group_name
        FROM work_records wr
        LEFT JOIN projects p ON wr.project_id = p.id
        LEFT JOIN project_groups pg ON p.group_id = pg.id
        WHERE {where}
        ORDER BY wr.recorded_at DESC, wr.id DESC
        LIMIT ? OFFSET ?
        """,
        params + [page_size, offset],
    )
    rows = cursor.fetchall()

    records = [_row_to_response(row) for row in rows]
    return ApiResponse(
        data=PaginatedResponse(
            items=[r.model_dump() for r in records],
            total=total,
            page=page,
            page_size=page_size,
        ).model_dump()
    )


@router.post("")
def create_record(body: RecordCreate, db=Depends(get_db)):
    """Create a new work record with audit logging."""
    project = db.execute(
        "SELECT id FROM projects WHERE id = ?", (body.project_id,)
    ).fetchone()
    if not project:
        return ApiResponse(code=1, message="项目不存在")

    cursor = db.execute(
        "INSERT INTO work_records (project_id, user_name, quantity, recorded_at) VALUES (?, ?, ?, ?)",
        (body.project_id, body.user_name, body.quantity, body.recorded_at),
    )
    db.commit()
    record_id = cursor.lastrowid

    _write_audit(
        db,
        action="create",
        table_name="work_records",
        record_id=record_id,
        user_name=body.user_name,
        detail=json.dumps(
            {"project_id": body.project_id, "quantity": body.quantity},
            ensure_ascii=False,
        ),
    )

    row = db.execute(
        """
        SELECT wr.*, p.name AS project_name, pg.name AS group_name
        FROM work_records wr
        LEFT JOIN projects p ON wr.project_id = p.id
        LEFT JOIN project_groups pg ON p.group_id = pg.id
        WHERE wr.id = ?
        """,
        (record_id,),
    ).fetchone()

    return ApiResponse(
        data=_row_to_response(row).model_dump(),
        message="录入成功",
    )


@router.delete("/{record_id}")
def delete_record(record_id: int, db=Depends(get_db)):
    """Soft delete a work record."""
    row = db.execute(
        "SELECT * FROM work_records WHERE id = ?", (record_id,)
    ).fetchone()
    if not row:
        return ApiResponse(code=1, message="记录不存在")
    if row["deleted_at"] is not None:
        return ApiResponse(code=1, message="记录已被删除")

    db.execute(
        "UPDATE work_records SET deleted_at = datetime('now', 'localtime') WHERE id = ?",
        (record_id,),
    )

    _write_audit(
        db,
        action="delete",
        table_name="work_records",
        record_id=record_id,
        user_name=row["user_name"],
    )

    return ApiResponse(message="删除成功")


@router.post("/{record_id}/restore")
def restore_record(record_id: int, db=Depends(get_db)):
    """Restore a soft-deleted work record."""
    row = db.execute(
        "SELECT * FROM work_records WHERE id = ?", (record_id,)
    ).fetchone()
    if not row:
        return ApiResponse(code=1, message="记录不存在")
    if row["deleted_at"] is None:
        return ApiResponse(code=1, message="记录未被删除，无需恢复")

    db.execute(
        "UPDATE work_records SET deleted_at = NULL WHERE id = ?",
        (record_id,),
    )

    _write_audit(
        db,
        action="restore",
        table_name="work_records",
        record_id=record_id,
        user_name=row["user_name"],
    )

    row = db.execute(
        """
        SELECT wr.*, p.name AS project_name, pg.name AS group_name
        FROM work_records wr
        LEFT JOIN projects p ON wr.project_id = p.id
        LEFT JOIN project_groups pg ON p.group_id = pg.id
        WHERE wr.id = ?
        """,
        (record_id,),
    ).fetchone()

    return ApiResponse(
        data=_row_to_response(row).model_dump(),
        message="恢复成功",
    )


@router.delete("/by-user/{user_name}")
def delete_records_by_user(
    user_name: str,
    start: str = Query(None),
    end: str = Query(None),
    db=Depends(get_db),
):
    """Soft-delete all non-deleted records for a given user in the given date range.

    Returns the number of records deleted and writes an audit log entry.
    """
    conditions: list = ["wr.user_name = ?", "wr.deleted_at IS NULL"]
    params: list = [user_name]

    if start is not None:
        conditions.append("wr.recorded_at >= ?")
        params.append(start)
    if end is not None:
        conditions.append("wr.recorded_at <= ?")
        params.append(end)

    where = " AND ".join(conditions)

    # Count how many records will be affected
    count_row = db.execute(
        f"SELECT COUNT(*) AS cnt FROM work_records wr WHERE {where}",
        params,
    ).fetchone()
    deleted_count = count_row["cnt"]

    if deleted_count == 0:
        return ApiResponse(
            code=1,
            message=f"用户「{user_name}」在指定日期范围内没有可删除的记录",
        )

    # Soft-delete all matching records
    db.execute(
        f"UPDATE work_records SET deleted_at = datetime('now', 'localtime') WHERE {where}",
        params,
    )
    db.commit()

    _write_audit(
        db,
        action="delete_user",
        table_name="work_records",
        record_id=0,
        user_name=user_name,
        detail=json.dumps(
            {"deleted_count": deleted_count, "start": start, "end": end},
            ensure_ascii=False,
        ),
    )

    return ApiResponse(
        data={"deleted_count": deleted_count},
        message=f"已删除用户「{user_name}」的 {deleted_count} 条记录",
    )


@router.put("/{record_id}")
def update_record(record_id: int, body: RecordUpdate, db=Depends(get_db)):
    """Update an existing work record (user correction).

    Allows modifying quantity, user_name, and recorded_at.
    Writes an audit log entry for the update.
    """
    row = db.execute(
        "SELECT * FROM work_records WHERE id = ?", (record_id,)
    ).fetchone()
    if not row:
        return ApiResponse(code=1, message="记录不存在")
    if row["deleted_at"] is not None:
        return ApiResponse(code=1, message="记录已被删除，无法编辑")

    updates: dict = {}
    old_values: dict = {}
    if body.user_name is not None:
        old_values["user_name"] = row["user_name"]
        updates["user_name"] = body.user_name
    if body.quantity is not None:
        old_values["quantity"] = row["quantity"]
        updates["quantity"] = body.quantity
    if body.recorded_at is not None:
        old_values["recorded_at"] = row["recorded_at"]
        updates["recorded_at"] = body.recorded_at

    if not updates:
        return ApiResponse(code=1, message="没有需要更新的字段")

    set_clause = ", ".join(f"{k} = ?" for k in updates)
    values = list(updates.values()) + [record_id]
    db.execute(f"UPDATE work_records SET {set_clause} WHERE id = ?", values)
    db.commit()

    _write_audit(
        db,
        action="update",
        table_name="work_records",
        record_id=record_id,
        user_name=body.user_name or row["user_name"],
        detail=json.dumps(
            {"old": old_values, "new": updates}, ensure_ascii=False
        ),
    )

    row = db.execute(
        """
        SELECT wr.*, p.name AS project_name, pg.name AS group_name
        FROM work_records wr
        LEFT JOIN projects p ON wr.project_id = p.id
        LEFT JOIN project_groups pg ON p.group_id = pg.id
        WHERE wr.id = ?
        """,
        (record_id,),
    ).fetchone()

    return ApiResponse(
        data=_row_to_response(row).model_dump(),
        message="更新成功",
    )
