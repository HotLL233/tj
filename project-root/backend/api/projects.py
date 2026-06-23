"""Project CRUD endpoints."""
from fastapi import APIRouter, Depends, Query
from database import get_db, get_method_full_name
from models import ProjectCreate, ProjectUpdate, ProjectResponse, ApiResponse

router = APIRouter(prefix="/projects", tags=["projects"])


def _build_project_response(row) -> dict:
    """Build a ProjectResponse dict from a DB row, including full_name and notes."""
    group_name = row["group_name"] or ""
    project_name = row["name"]
    # Use DB-stored full_name if available, otherwise compute from mapping
    full_name = row["full_name"] if "full_name" in row.keys() and row["full_name"] else get_method_full_name(group_name, project_name)
    notes = row["notes"] if "notes" in row.keys() else ""
    return ProjectResponse(
        id=row["id"],
        group_id=row["group_id"],
        group_name=group_name,
        name=project_name,
        full_name=full_name,
        notes=notes,
        sort_order=row["sort_order"],
        is_active=row["is_active"],
        created_at=row["created_at"],
    ).model_dump()


@router.get("")
def list_projects(
    group_id: int = Query(None),
    active_only: bool = Query(False),
    db=Depends(get_db),
):
    """List projects, optionally filtered by group and active status."""
    conditions = ["1=1"]
    params: list = []

    if group_id is not None:
        conditions.append("p.group_id = ?")
        params.append(group_id)

    if active_only:
        conditions.append("p.is_active = 1")

    where = " AND ".join(conditions)
    cursor = db.execute(
        f"""
        SELECT p.*, pg.name AS group_name
        FROM projects p
        LEFT JOIN project_groups pg ON p.group_id = pg.id
        WHERE {where}
        ORDER BY p.sort_order ASC, p.id ASC
        """,
        params,
    )
    rows = cursor.fetchall()
    projects = [_build_project_response(row) for row in rows]
    return ApiResponse(data=projects)


@router.post("")
def create_project(body: ProjectCreate, db=Depends(get_db)):
    """Create a new project."""
    group = db.execute(
        "SELECT id FROM project_groups WHERE id = ?", (body.group_id,)
    ).fetchone()
    if not group:
        return ApiResponse(code=1, message="所属分组不存在")

    cursor = db.execute(
        "INSERT INTO projects (group_id, name, sort_order) VALUES (?, ?, ?)",
        (body.group_id, body.name, body.sort_order or 0),
    )
    db.commit()
    project_id = cursor.lastrowid

    row = db.execute(
        """
        SELECT p.*, pg.name AS group_name
        FROM projects p
        LEFT JOIN project_groups pg ON p.group_id = pg.id
        WHERE p.id = ?
        """,
        (project_id,),
    ).fetchone()
    return ApiResponse(
        data=_build_project_response(row),
        message="创建成功",
    )


@router.put("/{project_id}")
def update_project(project_id: int, body: ProjectUpdate, db=Depends(get_db)):
    """Update a project."""
    row = db.execute("SELECT * FROM projects WHERE id = ?", (project_id,)).fetchone()
    if not row:
        return ApiResponse(code=1, message="项目不存在")

    updates = {}
    if body.name is not None:
        updates["name"] = body.name
    if body.full_name is not None:
        updates["full_name"] = body.full_name
    if body.notes is not None:
        updates["notes"] = body.notes
    if body.sort_order is not None:
        updates["sort_order"] = body.sort_order
    if body.is_active is not None:
        updates["is_active"] = body.is_active

    if updates:
        set_clause = ", ".join(f"{k} = ?" for k in updates)
        values = list(updates.values()) + [project_id]
        db.execute(f"UPDATE projects SET {set_clause} WHERE id = ?", values)
        db.commit()

    row = db.execute(
        """
        SELECT p.*, pg.name AS group_name
        FROM projects p
        LEFT JOIN project_groups pg ON p.group_id = pg.id
        WHERE p.id = ?
        """,
        (project_id,),
    ).fetchone()
    return ApiResponse(
        data=_build_project_response(row),
        message="更新成功",
    )


@router.delete("/{project_id}")
def delete_project(project_id: int, db=Depends(get_db)):
    """Delete a project (only if it has no records)."""
    row = db.execute("SELECT * FROM projects WHERE id = ?", (project_id,)).fetchone()
    if not row:
        return ApiResponse(code=1, message="项目不存在")

    record_count = db.execute(
        "SELECT COUNT(*) AS cnt FROM work_records WHERE project_id = ? AND deleted_at IS NULL",
        (project_id,),
    ).fetchone()["cnt"]
    if record_count > 0:
        return ApiResponse(code=1, message="该项目下还有记录，无法删除")

    db.execute("DELETE FROM projects WHERE id = ?", (project_id,))
    db.commit()
    return ApiResponse(message="删除成功")
