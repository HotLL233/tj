"""Project group CRUD endpoints."""
from fastapi import APIRouter, Depends
from database import get_db
from models import GroupCreate, GroupUpdate, GroupResponse, ApiResponse

router = APIRouter(prefix="/groups", tags=["groups"])


@router.get("")
def list_groups(db=Depends(get_db)):
    """List all groups with project count."""
    cursor = db.execute("""
        SELECT g.*, COUNT(p.id) AS project_count
        FROM project_groups g
        LEFT JOIN projects p ON g.id = p.group_id
        GROUP BY g.id
        ORDER BY g.sort_order ASC, g.id ASC
    """)
    rows = cursor.fetchall()
    groups = [
        GroupResponse(
            id=row["id"],
            name=row["name"],
            sort_order=row["sort_order"],
            project_count=row["project_count"],
            created_at=row["created_at"],
        )
        for row in rows
    ]
    return ApiResponse(data=[g.model_dump() for g in groups])


@router.post("")
def create_group(body: GroupCreate, db=Depends(get_db)):
    """Create a new group."""
    existing = db.execute(
        "SELECT id FROM project_groups WHERE name = ?", (body.name,)
    ).fetchone()
    if existing:
        return ApiResponse(code=1, message="分组名称已存在")

    cursor = db.execute(
        "INSERT INTO project_groups (name, sort_order) VALUES (?, ?)",
        (body.name, body.sort_order or 0),
    )
    db.commit()
    group_id = cursor.lastrowid

    row = db.execute(
        "SELECT * FROM project_groups WHERE id = ?", (group_id,)
    ).fetchone()
    return ApiResponse(
        data=GroupResponse(
            id=row["id"],
            name=row["name"],
            sort_order=row["sort_order"],
            project_count=0,
            created_at=row["created_at"],
        ).model_dump(),
        message="创建成功",
    )


@router.put("/{group_id}")
def update_group(group_id: int, body: GroupUpdate, db=Depends(get_db)):
    """Update a group."""
    row = db.execute(
        "SELECT * FROM project_groups WHERE id = ?", (group_id,)
    ).fetchone()
    if not row:
        return ApiResponse(code=1, message="分组不存在")

    updates = {}
    if body.name is not None:
        dup = db.execute(
            "SELECT id FROM project_groups WHERE name = ? AND id != ?",
            (body.name, group_id),
        ).fetchone()
        if dup:
            return ApiResponse(code=1, message="分组名称已存在")
        updates["name"] = body.name
    if body.sort_order is not None:
        updates["sort_order"] = body.sort_order

    if updates:
        set_clause = ", ".join(f"{k} = ?" for k in updates)
        values = list(updates.values()) + [group_id]
        db.execute(f"UPDATE project_groups SET {set_clause} WHERE id = ?", values)
        db.commit()

    row = db.execute(
        "SELECT * FROM project_groups WHERE id = ?", (group_id,)
    ).fetchone()
    project_count = db.execute(
        "SELECT COUNT(*) AS cnt FROM projects WHERE group_id = ?", (group_id,)
    ).fetchone()["cnt"]
    return ApiResponse(
        data=GroupResponse(
            id=row["id"],
            name=row["name"],
            sort_order=row["sort_order"],
            project_count=project_count,
            created_at=row["created_at"],
        ).model_dump(),
        message="更新成功",
    )


@router.delete("/{group_id}")
def delete_group(group_id: int, db=Depends(get_db)):
    """Delete a group (only if it has no projects)."""
    row = db.execute(
        "SELECT * FROM project_groups WHERE id = ?", (group_id,)
    ).fetchone()
    if not row:
        return ApiResponse(code=1, message="分组不存在")

    project_count = db.execute(
        "SELECT COUNT(*) AS cnt FROM projects WHERE group_id = ?", (group_id,)
    ).fetchone()["cnt"]
    if project_count > 0:
        return ApiResponse(code=1, message="该分组下还有项目，无法删除")

    db.execute("DELETE FROM project_groups WHERE id = ?", (group_id,))
    db.commit()
    return ApiResponse(message="删除成功")
