"""Excel export — matches 汇总模板.xlsx format and styling."""
import re
from collections import defaultdict
from io import BytesIO
from urllib.parse import quote
from datetime import datetime, timedelta
from fastapi import APIRouter, Depends, Query
from fastapi.responses import StreamingResponse
from openpyxl import Workbook
from openpyxl.styles import Font, Alignment, PatternFill, Border, Side
from openpyxl.utils import get_column_letter
from database import get_db, get_method_full_name

router = APIRouter(prefix="/export", tags=["export"])


# ═══════════════════════════════════════════════════════════════════
#  Styles — matching 汇总模板.xlsx
# ═══════════════════════════════════════════════════════════════════

FONT_TITLE = Font(name="等线", size=14, bold=True, color="1976D2")
FONT_HEADER = Font(name="等线", size=11, bold=True, color="000000")
FONT_DAY_HEADER = Font(name="等线", size=9, bold=True)
FONT_DATA = Font(name="等线", size=11, color="000000")
FONT_DATA_EN = Font(name="Times New Roman", size=11, color="000000")
FONT_DATA_BOLD = Font(name="等线", size=11, bold=True, color="000000")
FONT_SUBTOTAL = Font(name="等线", size=11, bold=True)
FONT_GRAND = Font(name="等线", size=12, bold=True, color="B71C1C")
FONT_GRAND_DATA = Font(name="等线", size=11, bold=True, color="B71C1C")
FONT_DETAIL_HEADER = Font(name="等线", size=11, bold=True, color="FFFFFF")

ALIGN_CENTER = Alignment(horizontal="center", vertical="center")
ALIGN_WRAP = Alignment(horizontal="center", vertical="center", wrap_text=True)
ALIGN_LEFT = Alignment(horizontal="left", vertical="center")

THIN = Side(style="thin", color="000000")
MEDIUM = Side(style="medium", color="000000")

BORDER_THIN = Border(left=THIN, right=THIN, top=THIN, bottom=THIN)
BORDER_MEDIUM_LEFT = Border(left=MEDIUM, right=THIN, top=THIN, bottom=THIN)
BORDER_MEDIUM_RIGHT = Border(left=THIN, right=MEDIUM, top=THIN, bottom=THIN)
BORDER_MEDIUM_BOTTOM = Border(left=THIN, right=THIN, top=THIN, bottom=MEDIUM)
BORDER_MEDIUM_BOTTOM_LEFT = Border(left=MEDIUM, right=THIN, top=THIN, bottom=MEDIUM)
BORDER_MEDIUM_BOTTOM_RIGHT = Border(left=THIN, right=MEDIUM, top=THIN, bottom=MEDIUM)

FILL_HEADER = PatternFill(start_color="BBDEFB", end_color="BBDEFB", fill_type="solid")
FILL_TITLE = PatternFill(start_color="E3F2FD", end_color="E3F2FD", fill_type="solid")
FILL_GRAY = PatternFill(start_color="D9D9D9", end_color="D9D9D9", fill_type="solid")
FILL_RED = PatternFill(start_color="FFCDD2", end_color="FFCDD2", fill_type="solid")
FILL_LC = PatternFill(start_color="E3F2FD", end_color="E3F2FD", fill_type="solid")
FILL_GC = PatternFill(start_color="E8F5E9", end_color="E8F5E9", fill_type="solid")
FILL_TOTAL_COL = PatternFill(start_color="FFF3E0", end_color="FFF3E0", fill_type="solid")
FILL_GREEN_HEADER = PatternFill(start_color="43A047", end_color="43A047", fill_type="solid")
FILL_WHITE_HEADER = PatternFill(start_color="FFFFFF", end_color="FFFFFF", fill_type="solid")
FILL_BLUE_HEADER = PatternFill(start_color="1565C0", end_color="1565C0", fill_type="solid")


# ═══════════════════════════════════════════════════════════════════
#  Helpers
# ═══════════════════════════════════════════════════════════════════

def _col_letter(n: int) -> str:
    """Convert 1-based column index to Excel letter."""
    return get_column_letter(n)


def _parse_instrument(project_name: str) -> tuple[str, str, str]:
    """Parse project name into (method_base, instrument_code, instrument_type).

    Examples:
        'HYLY-LC-01(230106)'      → ('HYLY-(230106)', 'LC-01', '液相')
        'E003-GC-02甲乙醇'         → ('E003-甲乙醇', 'GC-02', '气相')
        '三氟苯硼酸-LC-05(251108)'  → ('三氟苯硼酸-(251108)', 'LC-05', '液相')
        '环糊精-LC-19(240610)'     → ('环糊精-(240610)', 'LC-19', '液相')
    """
    m = re.match(r'^(.+?)-((?:LC|GC)-\d+)(.*)$', project_name)
    if m:
        prefix = m.group(1)   # e.g. "HYLY", "E003", "三氟苯硼酸"
        code = m.group(2)      # e.g. "LC-01", "GC-02"
        suffix = m.group(3)    # e.g. "(230106)", "甲乙醇", ""
        itype = "液相" if code.upper().startswith("LC") else "气相"
        method_base = f"{prefix}-{suffix}" if suffix else prefix
        return (method_base, code, itype)
    return (project_name, "", "其他")


def _extract_project_code(project_name: str) -> str:
    """Extract short project code (prefix before first dash).

    Examples:
        'HYLY-LC-01(230106)' → 'HYLY'
        'E003-GC-02甲乙醇'    → 'E003'
        '三氟苯硼酸-LC-05(251108)' → '三氟苯硼酸'
    """
    return project_name.split("-", 1)[0] if "-" in project_name else project_name


def _apply_border_range(ws, row: int, col_start: int, col_end: int,
                        border: Border = BORDER_THIN) -> None:
    """Apply border to a range of cells in one row."""
    for c in range(col_start, col_end + 1):
        ws.cell(row=row, column=c).border = border


def _apply_fill_range(ws, row: int, col_start: int, col_end: int,
                      fill: PatternFill) -> None:
    """Apply fill to a range of cells in one row."""
    for c in range(col_start, col_end + 1):
        ws.cell(row=row, column=c).fill = fill


def _apply_font_range(ws, row: int, col_start: int, col_end: int,
                      font: Font) -> None:
    """Apply font to a range of cells in one row."""
    for c in range(col_start, col_end + 1):
        ws.cell(row=row, column=c).font = font


# ═══════════════════════════════════════════════════════════════════
#  Sheet builders
# ═══════════════════════════════════════════════════════════════════

def _build_monthly_summary(ws, db, start: str | None, end: str | None,
                           group_id: int | None) -> str:
    """Build the 月-汇总 sheet.

    Key changes from v1:
    - ALL projects from DB are shown (including those with 0 records)
    - E column shows 检测方法全称 (from METHOD_FULL_NAMES mapping)
    - Zero values are displayed as 0 instead of empty
    """

    # ── Determine date range ──
    if start and end:
        d_start = datetime.strptime(start[:10], "%Y-%m-%d")
        d_end = datetime.strptime(end[:10], "%Y-%m-%d")
    else:
        today = datetime.now().replace(day=1)
        d_start = today
        if today.month == 12:
            d_end = today.replace(year=today.year + 1, month=1, day=1) - timedelta(days=1)
        else:
            d_end = today.replace(month=today.month + 1, day=1) - timedelta(days=1)

    # Build day list
    days: list[str] = []
    cur = d_start
    while cur <= d_end:
        days.append(cur.strftime("%Y-%m-%d"))
        cur += timedelta(days=1)
    num_days = len(days)
    month_label = f"{d_start.year}年{d_start.month}月"

    # ── Step 1: Query ALL active projects (not just those with records) ──
    proj_conditions = ["p.is_active = 1"]
    proj_params: list = []
    if group_id is not None:
        proj_conditions.append("pg.id = ?")
        proj_params.append(group_id)
    proj_where = " AND ".join(proj_conditions)

    all_projects = db.execute(
        f"""
        SELECT p.id, p.name AS project_name, pg.name AS group_name,
               pg.sort_order AS group_sort, p.sort_order AS project_sort
        FROM projects p
        JOIN project_groups pg ON p.group_id = pg.id
        WHERE {proj_where}
        ORDER BY pg.sort_order, p.sort_order
        """,
        proj_params,
    ).fetchall()

    # ── Step 2: Query work records for the date range ──
    rec_conditions = ["wr.deleted_at IS NULL"]
    rec_params: list = []
    if start:
        rec_conditions.append("wr.recorded_at >= ?")
        rec_params.append(start)
    if end:
        rec_conditions.append("wr.recorded_at <= ?")
        rec_params.append(end[:10] + "T23:59:59")
    if group_id is not None:
        rec_conditions.append("pg.id = ?")
        rec_params.append(group_id)
    rec_where = " AND ".join(rec_conditions)

    records = db.execute(
        f"""
        SELECT p.id AS project_id, date(wr.recorded_at) AS work_day,
               SUM(wr.quantity) AS total_qty
        FROM work_records wr
        JOIN projects p ON wr.project_id = p.id
        JOIN project_groups pg ON p.group_id = pg.id
        WHERE {rec_where}
        GROUP BY p.id, date(wr.recorded_at)
        """,
        rec_params,
    ).fetchall()

    # Build record map: project_id → {day_str: qty}
    record_map: dict[int, dict[str, int]] = defaultdict(lambda: defaultdict(int))
    for row in records:
        record_map[row["project_id"]][row["work_day"]] = row["total_qty"]

    # ── Step 3: Organize data: lab → project_code → instruments ──
    lab_order: list[str] = []
    lab_data: dict[str, dict[str, dict[str, list]]] = {}  # lab → proj_code → {'lc': [...], 'gc': [...]}

    for proj in all_projects:
        group_name = proj["group_name"]
        proj_name = proj["project_name"]
        proj_code = _extract_project_code(proj_name)
        method_base, inst_code, inst_type = _parse_instrument(proj_name)
        full_name = get_method_full_name(group_name, proj_name)

        # Use full_name if available, otherwise fall back to method_base
        display_method = full_name if full_name else method_base

        day_map = record_map.get(proj["id"], {})

        if group_name not in lab_data:
            lab_data[group_name] = {}
            lab_order.append(group_name)
        if proj_code not in lab_data[group_name]:
            lab_data[group_name][proj_code] = {'lc': [], 'gc': []}

        entry = (inst_code, display_method, proj_name, dict(day_map))
        if inst_type == "气相":
            lab_data[group_name][proj_code]['gc'].append(entry)
        else:
            lab_data[group_name][proj_code]['lc'].append(entry)

    # ── Column layout ──
    # A: 序号         col 1
    # B: 实验室       col 2
    # C: 项目代码     col 3
    # D: 仪器         col 4
    # E: 检测方法     col 5
    # F..: daily      col 6..6+num_days-1
    # After daily: 检测数量, 液相检测量, 气相检测量, 项目检测总量, (空), 液相总数, 气相总数, 总数
    COL_A = 1
    COL_LAB = 2       # B
    COL_PROJ = 3      # C
    COL_INST = 4      # D
    COL_METHOD = 5    # E
    COL_DAY_START = 6  # F

    COL_MONTH_TOTAL = COL_DAY_START + num_days       # 检测数量 (月合计)
    COL_LC_QTY = COL_MONTH_TOTAL + 1                  # 液相检测量
    COL_GC_QTY = COL_LC_QTY + 1                       # 气相检测量
    COL_PROJ_TOTAL = COL_GC_QTY + 1                   # 项目检测总量
    COL_EMPTY = COL_PROJ_TOTAL + 1                     # 空列
    COL_LC_RUN = COL_EMPTY + 1                         # 液相总数
    COL_GC_RUN = COL_LC_RUN + 1                        # 气相总数
    COL_GRAND = COL_GC_RUN + 1                         # 总数

    LAST_COL = COL_GRAND
    HEADER_ROW = 2

    # ── Title row (row 1) ──
    ws.merge_cells(start_row=1, start_column=COL_LAB, end_row=1, end_column=LAST_COL)
    title_cell = ws.cell(row=1, column=COL_LAB, value=f"工作量统计 — {month_label}")
    title_cell.font = FONT_TITLE
    title_cell.alignment = ALIGN_CENTER
    title_cell.fill = FILL_TITLE
    ws.row_dimensions[1].height = 28

    # ── Header row 2 ──
    hdr_labels = {
        COL_LAB: "实验室",
        COL_PROJ: "项目代码",
        COL_INST: "仪器",
        COL_METHOD: "检测方法",
        COL_MONTH_TOTAL: "检测数量",
        COL_LC_QTY: "液相检测量",
        COL_GC_QTY: "气相检测量",
        COL_PROJ_TOTAL: "项目检测总量",
        COL_LC_RUN: "液相总数",
        COL_GC_RUN: "气相总数",
        COL_GRAND: "总数",
    }
    for ci, label in hdr_labels.items():
        c = ws.cell(row=HEADER_ROW, column=ci, value=label)
        c.font = FONT_HEADER
        c.alignment = ALIGN_CENTER
        c.fill = FILL_HEADER
        c.border = BORDER_THIN

    # Daily header: merge row 2 for date range label
    if num_days > 0:
        ws.merge_cells(start_row=HEADER_ROW, start_column=COL_DAY_START,
                       end_row=HEADER_ROW, end_column=COL_DAY_START + num_days - 1)
        day_header_cell = ws.cell(row=HEADER_ROW, column=COL_DAY_START,
                                  value=f"日期（{d_start.month}月）")
        day_header_cell.font = FONT_HEADER
        day_header_cell.alignment = ALIGN_CENTER
        day_header_cell.fill = FILL_HEADER
        for dc in range(COL_DAY_START, COL_DAY_START + num_days):
            ws.cell(row=HEADER_ROW, column=dc).border = BORDER_THIN
            ws.cell(row=HEADER_ROW, column=dc).fill = FILL_HEADER

    # Empty column header
    ws.cell(row=HEADER_ROW, column=COL_EMPTY).fill = FILL_HEADER
    ws.cell(row=HEADER_ROW, column=COL_EMPTY).border = BORDER_THIN

    # Borders: leftmost and rightmost get medium left/right
    for ci in [COL_LAB, COL_PROJ, COL_INST, COL_METHOD, COL_MONTH_TOTAL,
               COL_LC_QTY, COL_GC_QTY, COL_PROJ_TOTAL, COL_LC_RUN, COL_GC_RUN, COL_GRAND]:
        b = ws.cell(row=HEADER_ROW, column=ci).border
        left_side = MEDIUM if ci == COL_LAB else b.left
        right_side = MEDIUM if ci == COL_GRAND else b.right
        ws.cell(row=HEADER_ROW, column=ci).border = Border(
            left=left_side, right=right_side, top=MEDIUM, bottom=THIN)

    # LC/GC column fills on header
    ws.cell(row=HEADER_ROW, column=COL_LC_QTY).fill = FILL_LC
    ws.cell(row=HEADER_ROW, column=COL_GC_QTY).fill = FILL_GC
    ws.cell(row=HEADER_ROW, column=COL_LC_RUN).fill = FILL_LC
    ws.cell(row=HEADER_ROW, column=COL_GC_RUN).fill = FILL_GC

    # ── Row 3: day numbers ──
    ROW_DAY_NUM = HEADER_ROW + 1  # row 3
    for i, day_str in enumerate(days):
        dc = COL_DAY_START + i
        day_num = int(day_str[8:10])
        c = ws.cell(row=ROW_DAY_NUM, column=dc, value=day_num)
        c.font = FONT_DAY_HEADER
        c.alignment = ALIGN_CENTER
        c.fill = FILL_HEADER
        c.border = BORDER_THIN
    # Day number bottom borders
    for dc in range(COL_DAY_START, COL_DAY_START + num_days):
        b = ws.cell(row=ROW_DAY_NUM, column=dc).border
        right_side = MEDIUM if dc == COL_DAY_START + num_days - 1 else THIN
        ws.cell(row=ROW_DAY_NUM, column=dc).border = Border(
            left=b.left, right=right_side, top=THIN, bottom=MEDIUM)

    # Fill mapping for header cells (used after merge to restore row-3 fills)
    _header_fills = {
        COL_LAB: FILL_HEADER, COL_PROJ: FILL_HEADER, COL_INST: FILL_HEADER,
        COL_METHOD: FILL_HEADER, COL_MONTH_TOTAL: FILL_HEADER,
        COL_LC_QTY: FILL_LC, COL_GC_QTY: FILL_GC, COL_PROJ_TOTAL: FILL_HEADER,
        COL_EMPTY: FILL_HEADER, COL_LC_RUN: FILL_LC, COL_GC_RUN: FILL_GC,
        COL_GRAND: FILL_HEADER,
    }
    # Merge row 2-3 for non-daily header cells
    for ci in [COL_LAB, COL_PROJ, COL_INST, COL_METHOD, COL_MONTH_TOTAL,
               COL_LC_QTY, COL_GC_QTY, COL_PROJ_TOTAL, COL_EMPTY,
               COL_LC_RUN, COL_GC_RUN, COL_GRAND]:
        ws.merge_cells(start_row=HEADER_ROW, start_column=ci,
                       end_row=ROW_DAY_NUM, end_column=ci)
        # Restore bottom border and fill for merged area
        c = ws.cell(row=ROW_DAY_NUM, column=ci)
        c.border = Border(left=MEDIUM if ci == COL_LAB else THIN,
                          right=MEDIUM if ci == COL_GRAND else THIN,
                          top=THIN, bottom=MEDIUM)
        c.fill = _header_fills.get(ci, FILL_HEADER)

    # ── Column widths ──
    ws.column_dimensions[_col_letter(COL_A)].width = 4
    ws.column_dimensions[_col_letter(COL_LAB)].width = 14
    ws.column_dimensions[_col_letter(COL_PROJ)].width = 12
    ws.column_dimensions[_col_letter(COL_INST)].width = 10
    ws.column_dimensions[_col_letter(COL_METHOD)].width = 30
    for i in range(num_days):
        ws.column_dimensions[_col_letter(COL_DAY_START + i)].width = 5.5
    ws.column_dimensions[_col_letter(COL_MONTH_TOTAL)].width = 10
    ws.column_dimensions[_col_letter(COL_LC_QTY)].width = 12
    ws.column_dimensions[_col_letter(COL_GC_QTY)].width = 12
    ws.column_dimensions[_col_letter(COL_PROJ_TOTAL)].width = 12
    ws.column_dimensions[_col_letter(COL_EMPTY)].width = 3
    ws.column_dimensions[_col_letter(COL_LC_RUN)].width = 12
    ws.column_dimensions[_col_letter(COL_GC_RUN)].width = 12
    ws.column_dimensions[_col_letter(COL_GRAND)].width = 12

    # ── Data rows ──
    DATA_START = ROW_DAY_NUM + 1  # row 4
    r = DATA_START
    seq = 1
    running_lc = 0
    running_gc = 0

    for lab_name in lab_order:
        proj_data = lab_data[lab_name]
        lab_start_row = r

        for proj_code, instruments in proj_data.items():
            lc_list = instruments['lc']
            gc_list = instruments['gc']
            proj_start_row = r

            # Calculate project LC/GC totals
            proj_lc_total = 0
            proj_gc_total = 0

            # Write LC rows
            for inst_code, method_label, proj_name, day_map in lc_list:
                row_total = 0
                # Seq
                c = ws.cell(row=r, column=COL_A, value=seq)
                c.font = FONT_DATA; c.alignment = ALIGN_CENTER; c.border = BORDER_THIN
                # Lab
                c = ws.cell(row=r, column=COL_LAB, value=lab_name)
                c.font = FONT_DATA; c.alignment = ALIGN_CENTER; c.border = BORDER_THIN
                # Project code
                c = ws.cell(row=r, column=COL_PROJ, value=proj_code)
                c.font = FONT_DATA; c.alignment = ALIGN_CENTER; c.border = BORDER_THIN
                # Instrument
                c = ws.cell(row=r, column=COL_INST, value=inst_code)
                c.font = FONT_DATA_EN; c.alignment = ALIGN_CENTER; c.border = BORDER_THIN
                # Method (full name)
                c = ws.cell(row=r, column=COL_METHOD, value=method_label)
                c.font = FONT_DATA; c.alignment = ALIGN_WRAP; c.border = BORDER_THIN
                # Daily quantities — show 0 for all (including empty)
                for i, day_str in enumerate(days):
                    dc = COL_DAY_START + i
                    qty = day_map.get(day_str, 0)
                    row_total += qty
                    c = ws.cell(row=r, column=dc, value=qty if qty > 0 else 0)
                    c.font = FONT_DATA; c.alignment = ALIGN_CENTER; c.border = BORDER_THIN
                # Monthly total
                c = ws.cell(row=r, column=COL_MONTH_TOTAL, value=row_total if row_total > 0 else 0)
                c.font = FONT_DATA_BOLD; c.alignment = ALIGN_CENTER; c.border = BORDER_THIN
                c.fill = FILL_TOTAL_COL
                proj_lc_total += row_total
                r += 1
                seq += 1

            # Write GC rows
            for inst_code, method_label, proj_name, day_map in gc_list:
                row_total = 0
                c = ws.cell(row=r, column=COL_A, value=seq)
                c.font = FONT_DATA; c.alignment = ALIGN_CENTER; c.border = BORDER_THIN
                c = ws.cell(row=r, column=COL_LAB, value=lab_name)
                c.font = FONT_DATA; c.alignment = ALIGN_CENTER; c.border = BORDER_THIN
                c = ws.cell(row=r, column=COL_PROJ, value=proj_code)
                c.font = FONT_DATA; c.alignment = ALIGN_CENTER; c.border = BORDER_THIN
                c = ws.cell(row=r, column=COL_INST, value=inst_code)
                c.font = FONT_DATA_EN; c.alignment = ALIGN_CENTER; c.border = BORDER_THIN
                c = ws.cell(row=r, column=COL_METHOD, value=method_label)
                c.font = FONT_DATA; c.alignment = ALIGN_WRAP; c.border = BORDER_THIN
                for i, day_str in enumerate(days):
                    dc = COL_DAY_START + i
                    qty = day_map.get(day_str, 0)
                    row_total += qty
                    c = ws.cell(row=r, column=dc, value=qty if qty > 0 else 0)
                    c.font = FONT_DATA; c.alignment = ALIGN_CENTER; c.border = BORDER_THIN
                c = ws.cell(row=r, column=COL_MONTH_TOTAL, value=row_total if row_total > 0 else 0)
                c.font = FONT_DATA_BOLD; c.alignment = ALIGN_CENTER; c.border = BORDER_THIN
                c.fill = FILL_TOTAL_COL
                proj_gc_total += row_total
                r += 1
                seq += 1

            proj_end_row = r - 1

            # Skip empty projects — not applicable anymore since all are included
            if proj_start_row > proj_end_row:
                continue

            # Merge project code column
            if proj_end_row > proj_start_row:
                ws.merge_cells(start_row=proj_start_row, start_column=COL_PROJ,
                               end_row=proj_end_row, end_column=COL_PROJ)

            # Project LC/GC/Total — merge and fill
            proj_total = proj_lc_total + proj_gc_total
            if proj_end_row > proj_start_row:
                ws.merge_cells(start_row=proj_start_row, start_column=COL_LC_QTY,
                               end_row=proj_end_row, end_column=COL_LC_QTY)
                ws.merge_cells(start_row=proj_start_row, start_column=COL_GC_QTY,
                               end_row=proj_end_row, end_column=COL_GC_QTY)
                ws.merge_cells(start_row=proj_start_row, start_column=COL_PROJ_TOTAL,
                               end_row=proj_end_row, end_column=COL_PROJ_TOTAL)

            # Write project subtotals (show 0 instead of empty)
            c_lc = ws.cell(row=proj_start_row, column=COL_LC_QTY,
                           value=proj_lc_total if proj_lc_total > 0 else 0)
            c_lc.font = FONT_DATA_BOLD; c_lc.alignment = ALIGN_CENTER
            c_lc.fill = FILL_LC
            for br in range(proj_start_row, proj_end_row + 1):
                ws.cell(row=br, column=COL_LC_QTY).border = BORDER_THIN
                ws.cell(row=br, column=COL_LC_QTY).fill = FILL_LC

            c_gc = ws.cell(row=proj_start_row, column=COL_GC_QTY,
                           value=proj_gc_total if proj_gc_total > 0 else 0)
            c_gc.font = FONT_DATA_BOLD; c_gc.alignment = ALIGN_CENTER
            c_gc.fill = FILL_GC
            for br in range(proj_start_row, proj_end_row + 1):
                ws.cell(row=br, column=COL_GC_QTY).border = BORDER_THIN
                ws.cell(row=br, column=COL_GC_QTY).fill = FILL_GC

            c_pt = ws.cell(row=proj_start_row, column=COL_PROJ_TOTAL,
                           value=proj_total if proj_total > 0 else 0)
            c_pt.font = FONT_DATA_BOLD; c_pt.alignment = ALIGN_CENTER
            for br in range(proj_start_row, proj_end_row + 1):
                ws.cell(row=br, column=COL_PROJ_TOTAL).border = BORDER_THIN

            running_lc += proj_lc_total
            running_gc += proj_gc_total
            running_total = running_lc + running_gc

            # Write running totals (on every row of the project)
            for br in range(proj_start_row, proj_end_row + 1):
                c = ws.cell(row=br, column=COL_LC_RUN, value=running_lc if running_lc > 0 else 0)
                c.font = FONT_DATA_BOLD; c.alignment = ALIGN_CENTER; c.border = BORDER_THIN
                c.fill = FILL_LC

                c = ws.cell(row=br, column=COL_GC_RUN, value=running_gc if running_gc > 0 else 0)
                c.font = FONT_DATA_BOLD; c.alignment = ALIGN_CENTER; c.border = BORDER_THIN
                c.fill = FILL_GC

                c = ws.cell(row=br, column=COL_GRAND, value=running_total if running_total > 0 else 0)
                c.font = FONT_DATA_BOLD; c.alignment = ALIGN_CENTER; c.border = BORDER_THIN

        # ── Lab subtotal row ──
        lab_end_row = r - 1

        # Merge lab name column
        if lab_end_row > lab_start_row:
            ws.merge_cells(start_row=lab_start_row, start_column=COL_LAB,
                           end_row=lab_end_row, end_column=COL_LAB)

        # Calculate lab totals from daily columns
        lab_daily_totals = {day_str: 0 for day_str in days}
        lab_month_total = 0
        for br in range(lab_start_row, lab_end_row + 1):
            for i, day_str in enumerate(days):
                dc = COL_DAY_START + i
                v = ws.cell(row=br, column=dc).value
                if v and isinstance(v, (int, float)):
                    lab_daily_totals[day_str] += int(v)
            v2 = ws.cell(row=br, column=COL_MONTH_TOTAL).value
            if v2 and isinstance(v2, (int, float)):
                lab_month_total += int(v2)

        # Subtotal row
        ws.merge_cells(start_row=r, start_column=COL_LAB, end_row=r, end_column=COL_PROJ)
        c = ws.cell(row=r, column=COL_LAB, value=f"{lab_name} 小计")
        c.font = FONT_SUBTOTAL; c.alignment = ALIGN_CENTER; c.fill = FILL_GRAY
        for cc in range(COL_LAB, LAST_COL + 1):
            ws.cell(row=r, column=cc).fill = FILL_GRAY
            ws.cell(row=r, column=cc).font = FONT_SUBTOTAL
            ws.cell(row=r, column=cc).border = BORDER_THIN
            ws.cell(row=r, column=cc).alignment = ALIGN_CENTER

        # Fill daily totals in subtotal row (show 0)
        for i, day_str in enumerate(days):
            dc = COL_DAY_START + i
            v = lab_daily_totals[day_str]
            ws.cell(row=r, column=dc).value = v if v > 0 else 0

        # Monthly total
        ws.cell(row=r, column=COL_MONTH_TOTAL).value = lab_month_total if lab_month_total > 0 else 0

        # Running totals
        ws.cell(row=r, column=COL_LC_RUN).value = running_lc if running_lc > 0 else 0
        ws.cell(row=r, column=COL_GC_RUN).value = running_gc if running_gc > 0 else 0
        ws.cell(row=r, column=COL_GRAND).value = (running_lc + running_gc) if (running_lc + running_gc) > 0 else 0

        # Bottom medium border for subtotal row
        for cc in range(COL_LAB, LAST_COL + 1):
            b = ws.cell(row=r, column=cc).border
            ws.cell(row=r, column=cc).border = Border(
                left=b.left, right=b.right, top=b.top, bottom=MEDIUM)

        r += 1

    # ── Grand total row ──
    ws.merge_cells(start_row=r, start_column=COL_LAB, end_row=r, end_column=COL_PROJ)
    c = ws.cell(row=r, column=COL_LAB, value="总计")
    c.font = FONT_GRAND; c.alignment = ALIGN_CENTER; c.fill = FILL_RED

    # Apply grand-total row styling
    for cc in range(COL_LAB, LAST_COL + 1):
        ws.cell(row=r, column=cc).fill = FILL_RED
        ws.cell(row=r, column=cc).font = FONT_GRAND_DATA
        ws.cell(row=r, column=cc).alignment = ALIGN_CENTER
        ws.cell(row=r, column=cc).border = BORDER_THIN

    # Calculate grand totals from the data structure (including all projects)
    grand_daily = {day_str: 0 for day_str in days}
    grand_month = 0
    for lab_name in lab_order:
        for proj_code, instruments in lab_data[lab_name].items():
            for inst_code, method_label, proj_name, day_map in instruments['lc'] + instruments['gc']:
                for day_str, qty in day_map.items():
                    if day_str in grand_daily:
                        grand_daily[day_str] += qty
                        grand_month += qty

    for i, day_str in enumerate(days):
        dc = COL_DAY_START + i
        v = grand_daily[day_str]
        ws.cell(row=r, column=dc).value = v if v > 0 else 0

    ws.cell(row=r, column=COL_MONTH_TOTAL).value = grand_month if grand_month > 0 else 0
    ws.cell(row=r, column=COL_LC_RUN).value = running_lc if running_lc > 0 else 0
    ws.cell(row=r, column=COL_GC_RUN).value = running_gc if running_gc > 0 else 0
    ws.cell(row=r, column=COL_GRAND).value = (running_lc + running_gc) if (running_lc + running_gc) > 0 else 0

    # Medium bottom for grand total
    for cc in range(COL_LAB, LAST_COL + 1):
        b = ws.cell(row=r, column=cc).border
        ws.cell(row=r, column=cc).border = Border(
            left=b.left, right=b.right, top=b.top, bottom=MEDIUM)

    ws.row_dimensions[r].height = 22

    # ── Freeze panes: D4 (fix header rows 1-3 + lab/project columns B-C) ──
    ws.freeze_panes = f"{_col_letter(COL_INST)}{DATA_START}"

    return month_label


def _build_daily_detail(ws, db, start: str | None, end: str | None,
                        group_id: int | None) -> None:
    """Build the 每日明细 sheet with daily aggregated records.

    Columns:
        A(日期) | B(实验室) | C(项目代码) | D(仪器) | E(全称) | F(类型) | G(数量) | H(录入人)
    """
    ws.sheet_properties.tabColor = "7B1FA2"

    rec_conditions = ["wr.deleted_at IS NULL"]
    rec_params: list = []
    if start:
        rec_conditions.append("wr.recorded_at >= ?")
        rec_params.append(start)
    if end:
        rec_conditions.append("wr.recorded_at <= ?")
        rec_params.append(end[:10] + "T23:59:59")
    if group_id is not None:
        rec_conditions.append("pg.id = ?")
        rec_params.append(group_id)
    rec_where = " AND ".join(rec_conditions)

    records = db.execute(
        f"""
        SELECT date(wr.recorded_at) AS work_day,
               pg.name AS group_name,
               p.name AS project_name,
               SUM(wr.quantity) AS total_qty,
               GROUP_CONCAT(DISTINCT wr.user_name) AS users
        FROM work_records wr
        JOIN projects p ON wr.project_id = p.id
        JOIN project_groups pg ON p.group_id = pg.id
        WHERE {rec_where}
        GROUP BY date(wr.recorded_at), pg.id, p.id
        ORDER BY work_day, pg.sort_order, p.sort_order
        """,
        rec_params,
    ).fetchall()

    # ── Headers ──
    headers = [
        ("日期", 12),
        ("实验室", 14),
        ("项目代码", 12),
        ("仪器", 10),
        ("全称", 30),
        ("类型", 8),
        ("数量", 8),
        ("录入人", 12),
    ]

    for ci, (label, _width) in enumerate(headers, 1):
        c = ws.cell(row=1, column=ci, value=label)
        c.font = FONT_DETAIL_HEADER
        c.alignment = ALIGN_CENTER
        c.fill = FILL_BLUE_HEADER
        c.border = BORDER_THIN

    # Column widths
    for ci, (_label, w) in enumerate(headers, 1):
        ws.column_dimensions[_col_letter(ci)].width = w

    # ── Data rows ──
    for ri, row in enumerate(records, 2):
        group_name = row["group_name"]
        proj_name = row["project_name"]
        proj_code = _extract_project_code(proj_name)
        method_base, inst_code, inst_type = _parse_instrument(proj_name)
        full_name = get_method_full_name(group_name, proj_name)
        display_method = full_name if full_name else method_base

        work_day = row["work_day"]
        if isinstance(work_day, str):
            work_day_display = work_day
        else:
            work_day_display = str(work_day)

        vals = [
            work_day_display,     # A: 日期
            group_name,           # B: 实验室
            proj_code,            # C: 项目代码
            inst_code,            # D: 仪器
            display_method,       # E: 全称
            inst_type,            # F: 类型
            row["total_qty"],     # G: 数量
            row["users"] or "",   # H: 录入人
        ]

        for ci, val in enumerate(vals, 1):
            c = ws.cell(row=ri, column=ci, value=val)
            if ci in (4, 7):  # instrument code & quantity use EN font
                c.font = FONT_DATA_EN
            else:
                c.font = FONT_DATA
            c.alignment = ALIGN_CENTER
            c.border = BORDER_THIN

        # Color LC/GC rows
        row_fill = FILL_LC if inst_type == "液相" else (FILL_GC if inst_type == "气相" else None)
        if row_fill:
            for ci in range(1, len(headers) + 1):
                ws.cell(row=ri, column=ci).fill = row_fill

    # Freeze header
    ws.freeze_panes = "A2"


def _build_instrument_summary(ws, db, start: str | None, end: str | None) -> None:
    """Build the 仪器汇总 sheet."""
    ws.sheet_properties.tabColor = "43A047"

    conditions = ["wr.deleted_at IS NULL"]
    params: list = []
    if start:
        conditions.append("wr.recorded_at >= ?")
        params.append(start)
    if end:
        conditions.append("wr.recorded_at <= ?")
        params.append(end)
    where = " AND ".join(conditions)

    cursor = db.execute(
        f"""
        SELECT
            p.name AS project_name,
            SUM(wr.quantity) AS total_quantity,
            COUNT(wr.id) AS record_count
        FROM work_records wr
        JOIN projects p ON wr.project_id = p.id
        WHERE {where}
        GROUP BY p.id
        """,
        params,
    )
    rows = cursor.fetchall()

    # Aggregate by instrument
    inst_data: dict[str, dict] = defaultdict(
        lambda: {"instrument_type": "", "total_quantity": 0, "record_count": 0, "methods": set()}
    )
    for row in rows:
        _, code, itype = _parse_instrument(row["project_name"])
        if code:
            inst_data[code]["instrument_type"] = itype
            inst_data[code]["total_quantity"] += row["total_quantity"]
            inst_data[code]["record_count"] += row["record_count"]
            inst_data[code]["methods"].add(row["project_name"])

    # Sort by type then code
    sorted_items = sorted(inst_data.items(),
                          key=lambda x: (0 if x[1]["instrument_type"] == "液相" else 1, x[0]))

    # Headers
    headers = ["仪器编号", "类型", "总数量", "记录数", "关联方法数"]
    header_font = Font(name="等线", size=11, bold=True, color="FFFFFF")
    header_fill = FILL_GREEN_HEADER

    for ci, h in enumerate(headers, 1):
        c = ws.cell(row=1, column=ci, value=h)
        c.font = header_font
        c.alignment = ALIGN_CENTER
        c.fill = header_fill
        c.border = BORDER_THIN

    # Data rows
    for ri, (code, d) in enumerate(sorted_items, 2):
        vals = [code, d["instrument_type"], d["total_quantity"],
                d["record_count"], len(d["methods"])]
        for ci, val in enumerate(vals, 1):
            c = ws.cell(row=ri, column=ci, value=val)
            if ci == 2:
                c.font = FONT_DATA
            elif ci >= 3:
                c.font = FONT_DATA_EN
            else:
                c.font = FONT_DATA_EN
            c.alignment = ALIGN_CENTER
            c.border = BORDER_THIN

        # Color code LC vs GC rows
        row_fill = FILL_LC if d["instrument_type"] == "液相" else FILL_GC
        for ci in range(1, len(headers) + 1):
            ws.cell(row=ri, column=ci).fill = row_fill

    # Column widths
    ws.column_dimensions["A"].width = 14
    ws.column_dimensions["B"].width = 10
    ws.column_dimensions["C"].width = 12
    ws.column_dimensions["D"].width = 10
    ws.column_dimensions["E"].width = 14

    # Freeze header
    ws.freeze_panes = "A2"


def _build_raw_records(ws, db, start: str | None, end: str | None,
                       group_id: int | None = None) -> None:
    """Build the 原始记录 sheet with individual work records."""
    ws.sheet_properties.tabColor = "FF9800"

    conditions = ["wr.deleted_at IS NULL"]
    params: list = []
    if start:
        conditions.append("wr.recorded_at >= ?")
        params.append(start)
    if end:
        conditions.append("wr.recorded_at <= ?")
        params.append(end[:10] + "T23:59:59")
    if group_id is not None:
        conditions.append("pg.id = ?")
        params.append(group_id)
    where = " AND ".join(conditions)

    cursor = db.execute(
        f"""
        SELECT
            pg.name AS group_name,
            p.name AS project_name,
            wr.user_name,
            wr.quantity,
            wr.recorded_at
        FROM work_records wr
        JOIN projects p ON wr.project_id = p.id
        JOIN project_groups pg ON p.group_id = pg.id
        WHERE {where}
        ORDER BY wr.recorded_at, pg.sort_order, p.sort_order
        """,
        params,
    )
    rows = cursor.fetchall()

    # Headers
    headers = ["序号", "日期", "实验室", "项目名称", "仪器", "检测方法",
               "仪器类型", "数量", "录入人"]
    header_font = Font(name="等线", size=11, bold=True, color="FFFFFF")
    header_fill = PatternFill(start_color="FF9800", end_color="FF9800", fill_type="solid")

    for ci, h in enumerate(headers, 1):
        c = ws.cell(row=1, column=ci, value=h)
        c.font = header_font
        c.alignment = ALIGN_CENTER
        c.fill = header_fill
        c.border = BORDER_THIN

    # Data rows
    for ri, row in enumerate(rows, 2):
        group_name = row["group_name"]
        proj_name = row["project_name"]
        method_base, inst_code, inst_type = _parse_instrument(proj_name)
        recorded = row["recorded_at"]
        if isinstance(recorded, str):
            recorded_date = recorded[:10]
        else:
            recorded_date = str(recorded)[:10]

        vals = [
            ri - 1,               # 序号
            recorded_date,        # 日期
            group_name,           # 实验室
            proj_name,            # 项目名称
            inst_code,            # 仪器
            method_base,          # 检测方法
            inst_type,            # 仪器类型
            row["quantity"],      # 数量
            row["user_name"],     # 录入人
        ]
        for ci, val in enumerate(vals, 1):
            c = ws.cell(row=ri, column=ci, value=val)
            if ci in (5, 8):  # instrument code & quantity use EN font
                c.font = FONT_DATA_EN
            else:
                c.font = FONT_DATA
            c.alignment = ALIGN_CENTER
            c.border = BORDER_THIN

        # Color LC/GC rows
        if inst_type == "液相":
            for ci in range(1, len(headers) + 1):
                ws.cell(row=ri, column=ci).fill = FILL_LC
        elif inst_type == "气相":
            for ci in range(1, len(headers) + 1):
                ws.cell(row=ri, column=ci).fill = FILL_GC

    # Column widths
    widths = [6, 12, 14, 30, 10, 30, 8, 8, 12]
    for ci, w in enumerate(widths, 1):
        ws.column_dimensions[_col_letter(ci)].width = w

    # Freeze header
    ws.freeze_panes = "A2"


# ═══════════════════════════════════════════════════════════════════
#  Export endpoint
# ═══════════════════════════════════════════════════════════════════

@router.get("/excel")
def export_excel(
    start: str = Query(None),
    end: str = Query(None),
    group_id: int = Query(None),
    db=Depends(get_db),
):
    """Export work records in cross-tab format matching 汇总模板.xlsx.

    Generates 4 sheets:
      1. 月-汇总   — cross-tab with ALL projects (including zero-data), full method names
      2. 每日明细   — daily aggregated records per project
      3. 仪器汇总   — instrument-level aggregation
      4. 原始记录   — raw individual records
    """
    wb = Workbook()

    # ── Sheet 1: 月-汇总 ──
    ws1 = wb.active
    ws1.title = "月-汇总"
    ws1.sheet_properties.tabColor = "1976D2"
    month_label = _build_monthly_summary(ws1, db, start, end, group_id)

    # ── Sheet 2: 每日明细 ──
    ws2 = wb.create_sheet(title="每日明细")
    _build_daily_detail(ws2, db, start, end, group_id)

    # ── Sheet 3: 仪器汇总 ──
    ws3 = wb.create_sheet(title="仪器汇总")
    _build_instrument_summary(ws3, db, start, end)

    # ── Sheet 4: 原始记录 ──
    ws4 = wb.create_sheet(title="原始记录")
    _build_raw_records(ws4, db, start, end, group_id)

    # ── Output ──
    output = BytesIO()
    wb.save(output)
    output.seek(0)

    filename = quote(f"工作量统计_{month_label}.xlsx")
    return StreamingResponse(
        output,
        media_type="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        headers={"Content-Disposition": f"attachment; filename*=UTF-8''{filename}"},
    )
