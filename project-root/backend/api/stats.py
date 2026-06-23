"""Statistics endpoints."""
import re
from collections import defaultdict
from fastapi import APIRouter, Depends, Query
from database import get_db
from models import StatsSummary, UserStats, ProjectStats, TypeStats, InstrumentStats, ApiResponse

router = APIRouter(prefix="/stats", tags=["stats"])


def _parse_instrument(project_name: str) -> tuple:
    """Extract (instrument_type, instrument_code) from project name.

    Examples:
        "HYLY-LC-01(230106)" → ("液相", "LC-01")
        "E003-GC-02甲乙醇"    → ("气相", "GC-02")
        "普通项目"            → ("其他", "")
    """
    m = re.search(r'(LC|GC)[-_](\d+)', project_name, re.IGNORECASE)
    if m:
        code = f"{m.group(1).upper()}-{m.group(2)}"
        itype = "液相" if m.group(1).upper() == "LC" else "气相"
        return (itype, code)
    return ("其他", "")


def _build_date_filter(start: str = None, end: str = None):
    """Build WHERE clause for date range filtering.
    Returns (conditions_list, params_list) where conditions_list is a list of SQL condition strings.
    """
    conditions = ["wr.deleted_at IS NULL"]
    params: list = []
    if start:
        conditions.append("wr.recorded_at >= ?")
        params.append(start)
    if end:
        conditions.append("wr.recorded_at <= ?")
        params.append(end)
    return conditions, params


@router.get("/summary")
def get_summary(
    start: str = Query(None),
    end: str = Query(None),
    group_by: str = Query("day", regex="^(week|month|day)$"),
    db=Depends(get_db),
):
    """Get comprehensive statistics summary."""
    conditions_base, params_base = _build_date_filter(start, end)
    where_base = " AND ".join(conditions_base)

    # Overall totals
    totals = db.execute(
        f"""
        SELECT
            COALESCE(SUM(wr.quantity), 0) AS total_quantity,
            COUNT(wr.id) AS total_records,
            COUNT(DISTINCT wr.user_name) AS user_count,
            COUNT(DISTINCT wr.project_id) AS project_count
        FROM work_records wr
        WHERE {where_base}
        """,
        params_base,
    ).fetchone()

    # Details by group_by
    if group_by == "week":
        label_expr = "strftime('%Y-%m', wr.recorded_at) || '第' || cast((strftime('%d', wr.recorded_at) - 1) / 7 + 1 as integer) || '周'"
        period_format = "%Y-%m%W"  # sortable key
    elif group_by == "month":
        label_expr = "strftime('%Y-%m', wr.recorded_at) || '月'"
        period_format = "%Y-%m"
    else:
        label_expr = "strftime('%Y-%m-%d', wr.recorded_at)"
        period_format = "%Y-%m-%d"

    if group_by == "week":
        period_extra = ", cast((strftime('%d', wr.recorded_at) - 1) / 7 + 1 as integer) as week_num"
    else:
        period_extra = ""

    details_cursor = db.execute(
        f"""
        SELECT
            {label_expr} AS period_label,
            SUM(wr.quantity) AS total_quantity,
            COUNT(wr.id) AS record_count
        FROM work_records wr
        WHERE {where_base}
        GROUP BY period_label
        ORDER BY period_label DESC
        """,
        params_base,
    )
    details = [
        {
            "period": row["period_label"],
            "total_quantity": row["total_quantity"],
            "record_count": row["record_count"],
        }
        for row in details_cursor.fetchall()
    ]

    return ApiResponse(
        data=StatsSummary(
            total_quantity=totals["total_quantity"],
            total_records=totals["total_records"],
            user_count=totals["user_count"],
            project_count=totals["project_count"],
            details=details,
        ).model_dump()
    )


@router.get("/by-user")
def get_stats_by_user(
    start: str = Query(None),
    end: str = Query(None),
    db=Depends(get_db),
):
    """Get statistics grouped by user."""
    conditions, params = _build_date_filter(start, end)
    where_clause = " AND ".join(conditions)
    cursor = db.execute(
        f"""
        SELECT
            wr.user_name,
            SUM(wr.quantity) AS total_quantity,
            COUNT(wr.id) AS record_count
        FROM work_records wr
        WHERE {where_clause}
        GROUP BY wr.user_name
        ORDER BY total_quantity DESC
        """,
        params,
    )
    rows = cursor.fetchall()
    stats = [
        UserStats(
            user_name=row["user_name"],
            total_quantity=row["total_quantity"],
            record_count=row["record_count"],
        )
        for row in rows
    ]
    return ApiResponse(data=[s.model_dump() for s in stats])


@router.get("/by-project")
def get_stats_by_project(
    start: str = Query(None),
    end: str = Query(None),
    group_id: int = Query(None),
    db=Depends(get_db),
):
    """Get statistics grouped by project."""
    conditions, params = _build_date_filter(start, end)
    if group_id is not None:
        conditions.append("p.group_id = ?")
        params.append(group_id)
    where_clause = " AND ".join(conditions)

    cursor = db.execute(
        f"""
        SELECT
            wr.project_id,
            p.name AS project_name,
            pg.name AS group_name,
            SUM(wr.quantity) AS total_quantity,
            COUNT(wr.id) AS record_count
        FROM work_records wr
        JOIN projects p ON wr.project_id = p.id
        JOIN project_groups pg ON p.group_id = pg.id
        WHERE {where_clause}
        GROUP BY wr.project_id
        ORDER BY total_quantity DESC
        """,
        params,
    )
    rows = cursor.fetchall()
    stats = [
        ProjectStats(
            project_id=row["project_id"],
            project_name=row["project_name"],
            group_name=row["group_name"] or "",
            total_quantity=row["total_quantity"],
            record_count=row["record_count"],
        )
        for row in rows
    ]
    return ApiResponse(data=[s.model_dump() for s in stats])


@router.get("/by-type")
def get_stats_by_type(
    start: str = Query(None),
    end: str = Query(None),
    db=Depends(get_db),
):
    """Get statistics grouped by instrument type (液相/气相/其他)."""
    conditions, params = _build_date_filter(start, end)
    where_clause = " AND ".join(conditions)

    cursor = db.execute(
        f"""
        SELECT
            p.name AS project_name,
            wr.quantity,
            wr.id AS record_id
        FROM work_records wr
        JOIN projects p ON wr.project_id = p.id
        WHERE {where_clause}
        """,
        params,
    )
    rows = cursor.fetchall()

    # Aggregate by instrument type in Python
    type_data: dict[str, dict] = defaultdict(lambda: {"total_quantity": 0, "record_count": 0})
    for row in rows:
        itype, _ = _parse_instrument(row["project_name"])
        type_data[itype]["total_quantity"] += row["quantity"]
        type_data[itype]["record_count"] += 1

    # Order: 液相, 气相, 其他
    ordered = ["液相", "气相", "其他"]
    stats = []
    for t in ordered:
        if t in type_data:
            stats.append(TypeStats(
                instrument_type=t,
                total_quantity=type_data[t]["total_quantity"],
                record_count=type_data[t]["record_count"],
            ))
    # Include any unexpected types
    for t, d in type_data.items():
        if t not in ordered:
            stats.append(TypeStats(
                instrument_type=t,
                total_quantity=d["total_quantity"],
                record_count=d["record_count"],
            ))

    return ApiResponse(data=[s.model_dump() for s in stats])


@router.get("/by-instrument")
def get_stats_by_instrument(
    start: str = Query(None),
    end: str = Query(None),
    db=Depends(get_db),
):
    """Get statistics grouped by individual instrument (LC-01, GC-02, etc.)."""
    conditions, params = _build_date_filter(start, end)
    where_clause = " AND ".join(conditions)

    cursor = db.execute(
        f"""
        SELECT
            p.name AS project_name,
            wr.quantity,
            wr.user_name,
            wr.id AS record_id
        FROM work_records wr
        JOIN projects p ON wr.project_id = p.id
        WHERE {where_clause}
        """,
        params,
    )
    rows = cursor.fetchall()

    # Aggregate by instrument code in Python
    instrument_data: dict[str, dict] = defaultdict(
        lambda: {"instrument_type": "", "total_quantity": 0, "record_count": 0, "users": set()}
    )
    for row in rows:
        itype, code = _parse_instrument(row["project_name"])
        if code:  # Only count records with a recognized instrument
            key = code
            instrument_data[key]["instrument_type"] = itype
            instrument_data[key]["total_quantity"] += row["quantity"]
            instrument_data[key]["record_count"] += 1
            instrument_data[key]["users"].add(row["user_name"])

    # Build stats sorted by total_quantity desc
    sorted_items = sorted(instrument_data.items(), key=lambda x: x[1]["total_quantity"], reverse=True)
    stats = [
        InstrumentStats(
            instrument=code,
            instrument_type=d["instrument_type"],
            total_quantity=d["total_quantity"],
            record_count=d["record_count"],
            user_count=len(d["users"]),
        )
        for code, d in sorted_items
    ]

    return ApiResponse(data=[s.model_dump() for s in stats])
