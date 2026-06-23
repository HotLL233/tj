"""Audit log query endpoint."""
from fastapi import APIRouter, Depends, Query
from database import get_db
from models import ApiResponse, PaginatedResponse

router = APIRouter(tags=["audit-logs"])


@router.get("")
def list_audit_logs(
    page: int = Query(1, ge=1),
    page_size: int = Query(50, ge=1, le=500),
    db=Depends(get_db),
):
    """List audit logs with pagination, ordered by most recent first."""
    # Count total
    count_row = db.execute("SELECT COUNT(*) AS cnt FROM audit_log").fetchone()
    total = count_row["cnt"]

    # Fetch page
    offset = (page - 1) * page_size
    cursor = db.execute(
        "SELECT * FROM audit_log ORDER BY created_at DESC LIMIT ? OFFSET ?",
        (page_size, offset),
    )
    rows = cursor.fetchall()

    items = [
        {
            "id": row["id"],
            "action": row["action"],
            "table_name": row["table_name"],
            "record_id": row["record_id"],
            "user_name": row["user_name"],
            "detail": row["detail"],
            "created_at": row["created_at"],
        }
        for row in rows
    ]

    return ApiResponse(
        data=PaginatedResponse(
            items=items,
            total=total,
            page=page,
            page_size=page_size,
        ).model_dump()
    )
