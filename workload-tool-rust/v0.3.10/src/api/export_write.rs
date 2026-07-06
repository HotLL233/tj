/// 导出 Excel 写入层 - v0.3.6 重写版本
/// 支持 10 个 Sheet 的格式化写入

use rust_xlsxwriter::*;
use crate::error::{Result, AppError};
use super::export_data::*;

// ========== 列常量定义 ==========
pub const CA: u16 = 0;  // A列
pub const CB: u16 = 1;  // B列
pub const CC: u16 = 2;  // C列
pub const CD: u16 = 3;  // D列
pub const CE: u16 = 4;  // E列
pub const CF: u16 = 5;  // F列
pub const CG: u16 = 6;  // G列
pub const CH: u16 = 7;  // H列
pub const CI: u16 = 8;  // I列
pub const CJ: u16 = 9;  // J列
pub const CK: u16 = 10; // K列
pub const CL: u16 = 11; // L列
pub const CM: u16 = 12; // M列
pub const CN: u16 = 13; // N列

pub const HR: u32 = 0;  // 表头行（0-indexed，实际是第1行）

/// 列号转字母（0->A, 1->B, 25->Z, 26->AA）
pub fn col_letter(n: u16) -> String {
    let letters = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut n = n + 1;
    let mut result = String::new();
    while n > 0 {
        n -= 1;
        result.insert(0, letters.chars().nth((n % 26) as usize).unwrap());
        n /= 26;
    }
    result
}

// ========== 格式定义 ==========

pub struct Fmt {
    pub fh: Format,  // 表头格式
    pub fd: Format,  // 数据格式
    pub fb: Format,  // 加粗格式
}

impl Fmt {
    pub fn new() -> Self {
        Self {
            fh: Format::new()
                .set_bold()
                .set_font_name("仿宋")
                .set_font_size(14)
                .set_align(FormatAlign::Center)
                .set_align(FormatAlign::VerticalCenter)
                .set_border(FormatBorder::Thin),
            fd: Format::new()
                .set_font_name("仿宋")
                .set_font_size(14)
                .set_align(FormatAlign::Center)
                .set_align(FormatAlign::VerticalCenter)
                .set_border(FormatBorder::Thin),
            fb: Format::new()
                .set_bold()
                .set_font_name("仿宋")
                .set_font_size(14)
                .set_align(FormatAlign::Center)
                .set_align(FormatAlign::VerticalCenter)
                .set_border(FormatBorder::Thin),
        }
    }
}

// ========== Sheet 1: 各实验室项目方法对应表 ==========

pub fn write_sheet1(
    ws: &mut Worksheet,
    rows: &[FlatRow],
    fmt: &Fmt
) -> Result<()> {
    ws.set_name("各实验室项目方法对应表").map_err(|e| AppError::Internal(e.to_string()))?;
    ws.set_tab_color(Color::RGB(0x1976D2));

    // 设置列宽
    ws.set_column_width(CA, 14.0)?;  // 使用实验室
    ws.set_column_width(CB, 18.0)?;  // 项目代号
    ws.set_column_width(CC, 18.0)?;  // 液相仪器
    ws.set_column_width(CD, 30.0)?;  // 检测方法
    ws.set_column_width(CE, 12.0)?;  // 检测数量
    ws.set_column_width(CF, 12.0)?;  // 液相检测量
    ws.set_column_width(CG, 12.0)?;  // 气相检测量
    ws.set_column_width(CH, 15.0)?;  // 项目检测总量

    // 写表头
    let headers = ["使用实验室", "项目代号", "液相仪器", "检测方法", "检测数量", "液相检测量", "气相检测量", "项目检测总量"];
    for (i, h) in headers.iter().enumerate() {
        ws.write_with_format(HR, i as u16, *h, &fmt.fh)?;
    }

    // 检测分组（按实验室、项目代号分组）
    let mut groups: Vec<(u32, u32, u32, u32, String)> = vec![]; // (start, end, lc_count, gc_count, project_code)
    let mut i = 0usize;
    while i < rows.len() {
        let (ref_lab, ref_proj, _, _, _, _, _) = &rows[i];
        let start = HR + 1 + i as u32;
        let mut end = start;
        let mut lc = 0u32;
        let mut gc = 0u32;

        while i < rows.len() {
            let (lab, proj, _, _, _, is_gc, _) = &rows[i];
            if lab != ref_lab || proj != ref_proj {
                break;
            }
            if *is_gc {
                gc += 1;
            } else {
                lc += 1;
            }
            end = HR + 1 + i as u32;
            i += 1;
        }
        groups.push((start, end, lc, gc, ref_proj.clone()));
    }

    // 写数据
    let mut row_idx = HR + 1;
    for (lab, proj, inst, method, qty, _, _) in rows {
        ws.write_with_format(row_idx, CA, lab.as_str(), &fmt.fd)?;
        ws.write_with_format(row_idx, CB, proj.as_str(), &fmt.fd)?;
        ws.write_with_format(row_idx, CC, inst.as_str(), &fmt.fd)?;
        ws.write_with_format(row_idx, CD, method.as_str(), &fmt.fd)?;
        if *qty > 0 {
            ws.write_with_format(row_idx, CE, *qty as f64, &fmt.fd)?;
        }
        row_idx += 1;
    }

    // 合并单元格（实验室、项目代号、汇总列）
    // 注意：merge_range 会覆写左上角单元格，需传入正确的值
    for &(start, end, _, _, ref proj_code) in &groups {
        // 硬保护：start == end 时不 merge（Excel 不允许合并单个单元格）
        if start == end { continue; }
        // 合并项目代号列（保留项目代号值）
        ws.merge_range(start, CB, end, CB, proj_code.as_str(), &fmt.fd)?;
        // 合并汇总列（后续公式会覆写，传空串即可）
        for col in [CF, CG, CH] {
            ws.merge_range(start, col, end, col, "", &fmt.fd)?;
        }
    }

    // 合并实验室列
    let mut i = 0usize;
    while i < rows.len() {
        let ref_lab = &rows[i].0;
        let start = HR + 1 + i as u32;
        let mut end = start;
        while i < rows.len() && &rows[i].0 == ref_lab {
            end = HR + 1 + i as u32;
            i += 1;
        }
        // 硬保护：start == end 时不 merge（Excel 不允许合并单个单元格）
        if start == end { continue; }
        ws.merge_range(start, CA, end, CA, ref_lab.as_str(), &fmt.fd)?;
    }

    // 写公式
    for &(start, end, lc_cnt, gc_cnt, _) in &groups {
        let excel_start = start + 1; // Excel行号从1开始
        let excel_end = end + 1;

        // 液相检测量
        if lc_cnt > 0 {
            let lc_last = excel_start + lc_cnt - 1;
            ws.write_formula(start, CF, format!("=SUM({}{}:{}{})", col_letter(CE), excel_start, col_letter(CE), lc_last).as_str())?;
        }

        // 气相检测量
        if gc_cnt > 0 {
            let gc_start = excel_start + lc_cnt;
            ws.write_formula(start, CG, format!("=SUM({}{}:{}{})", col_letter(CE), gc_start, col_letter(CE), excel_end).as_str())?;
        }

        // 项目检测总量
        ws.write_formula(start, CH, format!("={}{}+{}{}", col_letter(CF), excel_start, col_letter(CG), excel_start).as_str())?;
    }

    // 总计行
    let total_row = row_idx;
    ws.write_with_format(total_row, CA, "总计", &fmt.fb)?;
    for col in [CB, CC, CD] {
        ws.write_with_format(total_row, col, "", &fmt.fb)?;
    }
    for col in [CE, CF, CG, CH] {
        let cl = col_letter(col);
        let last_data_row = if row_idx > HR + 1 { row_idx - 1 } else { HR + 1 };
        ws.write_formula(total_row, col, format!("=SUM({}{}:{}{})", cl, HR + 2, cl, last_data_row).as_str())?;
    }

    ws.set_freeze_panes(HR + 1, 2)?;
    Ok(())
}

// ========== Sheet 2: 仪器-汇总 ==========

pub fn write_sheet2(
    ws: &mut Worksheet,
    rows: &[InstrumentDailyRow],
    fmt: &Fmt
) -> Result<()> {
    ws.set_name("仪器-汇总").map_err(|e| AppError::Internal(e.to_string()))?;
    ws.set_tab_color(Color::RGB(0x43A047));

    ws.set_column_width(CA, 12.0)?;  // 日期
    ws.set_column_width(CB, 14.0)?;  // 仪器
    ws.set_column_width(CC, 14.0)?;  // 实验室
    ws.set_column_width(CD, 20.0)?;  // 项目
    ws.set_column_width(CE, 30.0)?;  // 方法
    ws.set_column_width(CF, 12.0)?;  // 数量
    ws.set_column_width(CG, 15.0)?;  // 按天数量总计

    let headers = ["日期", "仪器", "实验室", "项目", "方法", "数量", "按天数量总计"];
    for (i, h) in headers.iter().enumerate() {
        ws.write_with_format(HR, i as u16, *h, &fmt.fh)?;
    }

    let mut row_idx = HR + 1;
    let mut current_date = String::new();
    let mut date_start = HR + 1;

    for row in rows {
        ws.write_with_format(row_idx, CA, row.date.as_str(), &fmt.fd)?;
        ws.write_with_format(row_idx, CB, row.instrument.as_str(), &fmt.fd)?;
        ws.write_with_format(row_idx, CC, row.lab.as_str(), &fmt.fd)?;
        ws.write_with_format(row_idx, CD, row.project.as_str(), &fmt.fd)?;
        ws.write_with_format(row_idx, CE, row.method.as_str(), &fmt.fd)?;
        ws.write_with_format(row_idx, CF, row.quantity as f64, &fmt.fd)?;

        // 日期变化时写汇总公式
        if !current_date.is_empty() && current_date != row.date {
            // 写上一个日期的汇总
            let sum_row = row_idx - 1;
            ws.write_formula(date_start, CG, format!("=SUM({}{}:{}{})", col_letter(CF), date_start + 1, col_letter(CF), sum_row + 1).as_str())?;
            date_start = row_idx;
        }
        current_date = row.date.clone();
        row_idx += 1;
    }

    // 最后一个日期的汇总
    if row_idx > HR + 1 {
        ws.write_formula(date_start, CG, format!("=SUM({}{}:{}{})", col_letter(CF), date_start + 1, col_letter(CF), row_idx).as_str())?;
    }

    ws.set_freeze_panes(HR + 1, 2)?;
    Ok(())
}

// ========== Sheet 3: 项目-汇总 ==========

pub fn write_sheet3(
    ws: &mut Worksheet,
    rows: &[ProjectSummaryRow],
    fmt: &Fmt
) -> Result<()> {
    ws.set_name("项目-汇总").map_err(|e| AppError::Internal(e.to_string()))?;
    ws.set_tab_color(Color::RGB(0xFF9800));

    ws.set_column_width(CA, 20.0)?;  // 项目
    ws.set_column_width(CB, 14.0)?;  // 实验室
    ws.set_column_width(CC, 14.0)?;  // 仪器
    ws.set_column_width(CD, 30.0)?;  // 方法
    ws.set_column_width(CE, 12.0)?;  // 数量
    ws.set_column_width(CF, 12.0)?;  // 方法对应金额
    ws.set_column_width(CG, 15.0)?;  // 金额总计
    ws.set_column_width(CH, 15.0)?;  // 项目金额

    let headers = ["项目", "实验室", "仪器", "方法", "数量", "方法对应金额", "金额总计", "项目金额"];
    for (i, h) in headers.iter().enumerate() {
        ws.write_with_format(HR, i as u16, *h, &fmt.fh)?;
    }

    let mut row_idx = HR + 1;
    let mut project_groups: Vec<(u32, u32, String)> = vec![]; // (start, end, project_name)
    let mut current_project = String::new();
    let mut proj_start = HR + 1;

    for row in rows {
        ws.write_with_format(row_idx, CA, row.project.as_str(), &fmt.fd)?;
        ws.write_with_format(row_idx, CB, row.lab.as_str(), &fmt.fd)?;
        ws.write_with_format(row_idx, CC, row.instrument.as_str(), &fmt.fd)?;
        ws.write_with_format(row_idx, CD, row.method.as_str(), &fmt.fd)?;
        ws.write_with_format(row_idx, CE, row.quantity as f64, &fmt.fd)?;
        ws.write_with_format(row_idx, CF, row.amount, &fmt.fd)?;

        // 金额总计 = 数量 × 金额
        ws.write_formula(row_idx, CG, format!("={}{}*{}{}", col_letter(CE), row_idx + 1, col_letter(CF), row_idx + 1).as_str())?;

        if !current_project.is_empty() && current_project != row.project {
            project_groups.push((proj_start, row_idx - 1, current_project.clone()));
            proj_start = row_idx;
        }
        current_project = row.project.clone();
        row_idx += 1;
    }

    if row_idx > HR + 1 {
        project_groups.push((proj_start, row_idx - 1, current_project));
    }

    // 合并单元格和写项目汇总公式
    for &(start, end, ref proj_name) in &project_groups {
        // 硬保护：start == end 时不 merge（Excel 不允许合并单个单元格）
        if start == end { continue; }
        ws.merge_range(start, CA, end, CA, proj_name.as_str(), &fmt.fd)?;
        ws.merge_range(start, CH, end, CH, "", &fmt.fd)?;
        ws.write_formula(start, CH, format!("=SUM({}{}:{}{})", col_letter(CG), start + 1, col_letter(CG), end + 1).as_str())?;
    }

    ws.set_freeze_panes(HR + 1, 1)?;
    Ok(())
}

// ========== Sheet 4: 实验室-汇总 ==========

pub fn write_sheet4(
    ws: &mut Worksheet,
    rows: &[LabSummaryRow],
    fmt: &Fmt
) -> Result<()> {
    ws.set_name("实验室-汇总").map_err(|e| AppError::Internal(e.to_string()))?;
    ws.set_tab_color(Color::RGB(0x9C27B0));

    ws.set_column_width(CA, 14.0)?;  // 实验室
    ws.set_column_width(CB, 20.0)?;  // 项目
    ws.set_column_width(CC, 14.0)?;  // 仪器
    ws.set_column_width(CD, 30.0)?;  // 方法
    ws.set_column_width(CE, 12.0)?;  // 数量
    ws.set_column_width(CF, 12.0)?;  // 方法对应金额
    ws.set_column_width(CG, 12.0)?;  // 数量总计
    ws.set_column_width(CH, 15.0)?;  // 金额总计
    ws.set_column_width(CI, 15.0)?;  // 实验室汇总

    let headers = ["实验室", "项目", "仪器", "方法", "数量", "方法对应金额", "数量总计", "金额总计", "实验室汇总"];
    for (i, h) in headers.iter().enumerate() {
        ws.write_with_format(HR, i as u16, *h, &fmt.fh)?;
    }

    let mut row_idx = HR + 1;
    let mut lab_groups: Vec<(u32, u32, String)> = vec![];
    let mut current_lab = String::new();
    let mut lab_start = HR + 1;

    for row in rows {
        ws.write_with_format(row_idx, CA, row.lab.as_str(), &fmt.fd)?;
        ws.write_with_format(row_idx, CB, row.project.as_str(), &fmt.fd)?;
        ws.write_with_format(row_idx, CC, row.instrument.as_str(), &fmt.fd)?;
        ws.write_with_format(row_idx, CD, row.method.as_str(), &fmt.fd)?;
        ws.write_with_format(row_idx, CE, row.quantity as f64, &fmt.fd)?;
        ws.write_with_format(row_idx, CF, row.amount, &fmt.fd)?;

        // 数量总计 = 数量
        ws.write_formula(row_idx, CG, format!("={}{}", col_letter(CE), row_idx + 1).as_str())?;
        // 金额总计 = 数量总计 × 金额
        ws.write_formula(row_idx, CH, format!("={}{}*{}{}", col_letter(CG), row_idx + 1, col_letter(CF), row_idx + 1).as_str())?;

        if !current_lab.is_empty() && current_lab != row.lab {
            lab_groups.push((lab_start, row_idx - 1, current_lab.clone()));
            lab_start = row_idx;
        }
        current_lab = row.lab.clone();
        row_idx += 1;
    }

    if row_idx > HR + 1 {
        lab_groups.push((lab_start, row_idx - 1, current_lab));
    }

    // 合并单元格和写实验室汇总公式
    // 硬保护：start == end 时不 merge（Excel 不允许合并单个单元格）
    for &(start, end, ref lab_name) in &lab_groups {
        if start == end { continue; }
        if end > start {
            ws.merge_range(start, CA, end, CA, lab_name.as_str(), &fmt.fd)?;
            ws.merge_range(start, CI, end, CI, "", &fmt.fd)?;
            ws.write_formula(start, CI, format!("=SUM({}{}:{}{})", col_letter(CH), start + 1, col_letter(CH), end + 1).as_str())?;
        }
    }

    ws.set_freeze_panes(HR + 1, 1)?;
    Ok(())
}

// ========== Sheet 5: 人员-汇总（原始记录） ==========

pub fn write_sheet5(
    ws: &mut Worksheet,
    rows: &[PersonRecordRow],
    fmt: &Fmt
) -> Result<()> {
    ws.set_name("人员-汇总").map_err(|e| AppError::Internal(e.to_string()))?;
    ws.set_tab_color(Color::RGB(0xE91E63));

    ws.set_column_width(CA, 16.0)?;  // 录入时间
    ws.set_column_width(CB, 14.0)?;  // 实验室
    ws.set_column_width(CC, 20.0)?;  // 研发项目
    ws.set_column_width(CD, 30.0)?;  // 方法
    ws.set_column_width(CE, 12.0)?;  // 检测类型
    ws.set_column_width(CF, 10.0)?;  // 数量
    ws.set_column_width(CG, 12.0)?;  // 录入人

    let headers = ["录入时间", "实验室", "研发项目", "方法", "检测类型", "数量", "录入人"];
    for (i, h) in headers.iter().enumerate() {
        ws.write_with_format(HR, i as u16, *h, &fmt.fh)?;
    }

    let mut row_idx = HR + 1;
    for row in rows {
        ws.write_with_format(row_idx, CA, row.recorded_at.as_str(), &fmt.fd)?;
        ws.write_with_format(row_idx, CB, row.lab.as_str(), &fmt.fd)?;
        ws.write_with_format(row_idx, CC, row.project.as_str(), &fmt.fd)?;
        ws.write_with_format(row_idx, CD, row.method.as_str(), &fmt.fd)?;
        ws.write_with_format(row_idx, CE, row.method_type.as_str(), &fmt.fd)?;
        ws.write_with_format(row_idx, CF, row.quantity as f64, &fmt.fd)?;
        ws.write_with_format(row_idx, CG, row.user_name.as_str(), &fmt.fd)?;
        row_idx += 1;
    }

    ws.set_freeze_panes(HR + 1, 1)?;
    Ok(())
}

// ========== Sheet 6: 人员汇总表 ==========

pub fn write_sheet6(
    ws: &mut Worksheet,
    rows: &[PersonSummaryRow],
    fmt: &Fmt
) -> Result<()> {
    ws.set_name("人员汇总表").map_err(|e| AppError::Internal(e.to_string()))?;
    ws.set_tab_color(Color::RGB(0x00BCD4));

    // 列宽
    ws.set_column_width(CA, 12.0)?;  // 人员
    ws.set_column_width(CB, 10.0)?;  // 液相数量
    ws.set_column_width(CC, 8.0)?;   // 液相系数
    ws.set_column_width(CD, 12.0)?;  // 液相汇总
    ws.set_column_width(CE, 10.0)?;  // 气相数量
    ws.set_column_width(CF, 8.0)?;   // 气相系数
    ws.set_column_width(CG, 12.0)?;  // 气相汇总
    ws.set_column_width(CH, 10.0)?;  // 理化数量
    ws.set_column_width(CI, 8.0)?;   // 理化系数
    ws.set_column_width(CJ, 12.0)?;  // 理化汇总
    ws.set_column_width(CK, 15.0)?;  // 个人汇总
    ws.set_column_width(CL, 10.0)?;  // 工作占比
    ws.set_column_width(CM, 15.0)?;  // 总额
    ws.set_column_width(CN, 15.0)?;  // 平均值

    let headers = ["人员", "液相数量", "液相系数", "液相汇总", "气相数量", "气相系数", "气相汇总",
                   "理化数量", "理化系数", "理化汇总", "个人汇总", "工作占比", "总额", "平均值"];
    for (i, h) in headers.iter().enumerate() {
        ws.write_with_format(HR, i as u16, *h, &fmt.fh)?;
    }

    // 逐行输出（SQL 已正确按 (user_name, method_type, coefficient) 分组）
    let mut row_idx = HR + 1;
    let mut user_groups: Vec<(u32, u32, String)> = vec![];  // (start, end, user_name)
    let mut current_user = String::new();
    let mut user_start = HR + 1;

    for row in rows {
        // 检测用户变化（需要合并 A 列和 K 列）
        if !current_user.is_empty() && current_user != row.user_name {
            user_groups.push((user_start, row_idx - 1, current_user.clone()));
            user_start = row_idx;
        }
        current_user = row.user_name.clone();

        // 人员名：先填写（后续 merge 处理）
        ws.write_with_format(row_idx, CA, row.user_name.as_str(), &fmt.fd)?;

        // 根据 method_type 填充对应列
        if row.method_type.contains("液相") {
            ws.write_with_format(row_idx, CB, row.quantity as f64, &fmt.fd)?;
            ws.write_with_format(row_idx, CC, row.coefficient, &fmt.fd)?;
            ws.write_formula(row_idx, CD, format!("={}{}*{}{}", col_letter(CB), row_idx+1, col_letter(CC), row_idx+1).as_str())?;
        } else if row.method_type.contains("气相") {
            ws.write_with_format(row_idx, CE, row.quantity as f64, &fmt.fd)?;
            ws.write_with_format(row_idx, CF, row.coefficient, &fmt.fd)?;
            ws.write_formula(row_idx, CG, format!("={}{}*{}{}", col_letter(CE), row_idx+1, col_letter(CF), row_idx+1).as_str())?;
        } else {
            // 理化/质谱等
            ws.write_with_format(row_idx, CH, row.quantity as f64, &fmt.fd)?;
            ws.write_with_format(row_idx, CI, row.coefficient, &fmt.fd)?;
            ws.write_formula(row_idx, CJ, format!("={}{}*{}{}", col_letter(CH), row_idx+1, col_letter(CI), row_idx+1).as_str())?;
        }

        // 个人汇总：根据本行类型构建公式，未写列用 0 代替，避免引用空单元格导致 #VALUE!
        let mut parts: Vec<String> = Vec::new();
        if row.method_type.contains("液相") {
            parts.push(format!("{}{}*{}{}", col_letter(CB), row_idx+1, col_letter(CC), row_idx+1));
        } else {
            parts.push("0".to_string());
        }
        if row.method_type.contains("气相") {
            parts.push(format!("{}{}*{}{}", col_letter(CE), row_idx+1, col_letter(CF), row_idx+1));
        } else {
            parts.push("0".to_string());
        }
        if !row.method_type.contains("液相") && !row.method_type.contains("气相") {
            parts.push(format!("{}{}*{}{}", col_letter(CH), row_idx+1, col_letter(CI), row_idx+1));
        } else {
            parts.push("0".to_string());
        }
        ws.write_formula(row_idx, CK, format!("={}+{}+{}", parts[0], parts[1], parts[2]).as_str())?;

        row_idx += 1;
    }

    if row_idx > HR + 1 {
        user_groups.push((user_start, row_idx - 1, current_user));
    }

    // 合并用户列和 K 列
    for (start, end, user) in &user_groups {
        // 硬保护：start == end 时不 merge（Excel 不允许合并单个单元格）
        if *start == *end { continue; }
        ws.merge_range(*start, CA, *end, CA, user.as_str(), &fmt.fd)?;
        ws.merge_range(*start, CK, *end, CK, "", &fmt.fd)?;
    }

    // 总额和平均值
    if row_idx > HR + 1 {
        ws.write_formula(HR + 1, CM, format!("=SUM({}{}:{}{})", col_letter(CK), HR+2, col_letter(CK), row_idx).as_str())?;
        ws.write_formula(HR + 1, CN, format!("=AVERAGE({}{}:{}{})", col_letter(CK), HR+2, col_letter(CK), row_idx).as_str())?;

        // 工作占比 - 修正：引用第2行的平均值
        for r in (HR + 1)..row_idx {
            ws.write_formula(r, CL, format!("={}{}/{}{}", col_letter(CK), r+1, col_letter(CN), HR+2).as_str())?;
        }
    }

    ws.set_freeze_panes(HR + 1, 1)?;
    Ok(())
}

// ========== Sheet 7: 实验室总表 ==========

pub fn write_sheet7(
    ws: &mut Worksheet,
    rows: &[LabTotalRow],
    fmt: &Fmt
) -> Result<()> {
    ws.set_name("实验室总表").map_err(|e| AppError::Internal(e.to_string()))?;
    ws.set_tab_color(Color::RGB(0x4CAF50));

    ws.set_column_width(CA, 14.0)?;  // 实验室
    ws.set_column_width(CB, 20.0)?;  // 项目
    ws.set_column_width(CC, 10.0)?;  // 液相数量
    ws.set_column_width(CD, 10.0)?;  // 液相金额
    ws.set_column_width(CE, 12.0)?;  // 液相汇总
    ws.set_column_width(CF, 10.0)?;  // 气相数量
    ws.set_column_width(CG, 10.0)?;  // 气相金额
    ws.set_column_width(CH, 12.0)?;  // 气相汇总
    ws.set_column_width(CI, 10.0)?;  // 理化数量
    ws.set_column_width(CJ, 10.0)?;  // 理化金额
    ws.set_column_width(CK, 12.0)?;  // 理化汇总
    ws.set_column_width(CL, 15.0)?;  // 项目汇总
    ws.set_column_width(CM, 15.0)?;  // 实验室汇总

    let headers = ["实验室", "项目", "液相数量", "液相金额", "液相汇总", "气相数量", "气相金额", "气相汇总",
                   "理化数量", "理化金额", "理化汇总", "项目汇总", "实验室汇总"];
    for (i, h) in headers.iter().enumerate() {
        ws.write_with_format(HR, i as u16, *h, &fmt.fh)?;
    }

    // 逐行输出（SQL 已正确按 (lab, project, method_type, amount) 分组）
    let mut row_idx = HR + 1;
    let mut lab_groups: Vec<(u32, u32, String)> = vec![];
    let mut current_lab = String::new();
    let mut lab_start = HR + 1;

    for row in rows {
        // 检测实验室变化
        if !current_lab.is_empty() && current_lab != row.lab {
            lab_groups.push((lab_start, row_idx - 1, current_lab.clone()));
            lab_start = row_idx;
        }
        current_lab = row.lab.clone();

        ws.write_with_format(row_idx, CA, row.lab.as_str(), &fmt.fd)?;
        ws.write_with_format(row_idx, CB, row.project.as_str(), &fmt.fd)?;

        // 根据 method_type 填充
        if row.method_type.contains("液相") {
            ws.write_with_format(row_idx, CC, row.quantity as f64, &fmt.fd)?;
            ws.write_with_format(row_idx, CD, row.amount, &fmt.fd)?;
            ws.write_formula(row_idx, CE, format!("={}{}*{}{}", col_letter(CC), row_idx+1, col_letter(CD), row_idx+1).as_str())?;
        } else if row.method_type.contains("气相") {
            ws.write_with_format(row_idx, CF, row.quantity as f64, &fmt.fd)?;
            ws.write_with_format(row_idx, CG, row.amount, &fmt.fd)?;
            ws.write_formula(row_idx, CH, format!("={}{}*{}{}", col_letter(CF), row_idx+1, col_letter(CG), row_idx+1).as_str())?;
        } else {
            // 理化/质谱等
            ws.write_with_format(row_idx, CI, row.quantity as f64, &fmt.fd)?;
            ws.write_with_format(row_idx, CJ, row.amount, &fmt.fd)?;
            ws.write_formula(row_idx, CK, format!("={}{}*{}{}", col_letter(CI), row_idx+1, col_letter(CJ), row_idx+1).as_str())?;
        }

        // 项目汇总：根据本行类型构建公式，未写列用 0 代替，避免引用空单元格导致 #VALUE!
        let mut parts: Vec<String> = Vec::new();
        if row.method_type.contains("液相") {
            parts.push(format!("{}{}*{}{}", col_letter(CC), row_idx+1, col_letter(CD), row_idx+1));
        } else {
            parts.push("0".to_string());
        }
        if row.method_type.contains("气相") {
            parts.push(format!("{}{}*{}{}", col_letter(CF), row_idx+1, col_letter(CG), row_idx+1));
        } else {
            parts.push("0".to_string());
        }
        if !row.method_type.contains("液相") && !row.method_type.contains("气相") {
            parts.push(format!("{}{}*{}{}", col_letter(CI), row_idx+1, col_letter(CJ), row_idx+1));
        } else {
            parts.push("0".to_string());
        }
        ws.write_formula(row_idx, CL, format!("={}+{}+{}", parts[0], parts[1], parts[2]).as_str())?;

        row_idx += 1;
    }

    if row_idx > HR + 1 {
        lab_groups.push((lab_start, row_idx - 1, current_lab));
    }

    // 合并和写实验室汇总
    for &(start, end, ref lab_name) in &lab_groups {
        // 硬保护：start == end 时不 merge（Excel 不允许合并单个单元格）
        if start == end { continue; }
        ws.merge_range(start, CA, end, CA, lab_name.as_str(), &fmt.fd)?;
        ws.merge_range(start, CM, end, CM, "", &fmt.fd)?;
        ws.write_formula(start, CM, format!("=SUM({}{}:{}{})", col_letter(CL), start+1, col_letter(CL), end+1).as_str())?;
    }

    ws.set_freeze_panes(HR + 1, 2)?;
    Ok(())
}

// ========== Sheet 8: 项目总表 ==========

pub fn write_sheet8(
    ws: &mut Worksheet,
    rows: &[ProjectTotalRow],
    fmt: &Fmt
) -> Result<()> {
    ws.set_name("项目总表").map_err(|e| AppError::Internal(e.to_string()))?;
    ws.set_tab_color(Color::RGB(0xFFC107));

    ws.set_column_width(CA, 20.0)?;  // 项目
    ws.set_column_width(CB, 10.0)?;  // 液相数量
    ws.set_column_width(CC, 10.0)?;  // 液相金额
    ws.set_column_width(CD, 12.0)?;  // 液相汇总
    ws.set_column_width(CE, 10.0)?;  // 气相数量
    ws.set_column_width(CF, 10.0)?;  // 气相金额
    ws.set_column_width(CG, 12.0)?;  // 气相汇总
    ws.set_column_width(CH, 10.0)?;  // 理化数量
    ws.set_column_width(CI, 10.0)?;  // 理化金额
    ws.set_column_width(CJ, 12.0)?;  // 理化汇总
    ws.set_column_width(CK, 15.0)?;  // 项目汇总

    let headers = ["项目", "液相数量", "液相金额", "液相汇总", "气相数量", "气相金额", "气相汇总",
                   "理化数量", "理化金额", "理化汇总", "项目汇总"];
    for (i, h) in headers.iter().enumerate() {
        ws.write_with_format(HR, i as u16, *h, &fmt.fh)?;
    }

    // 逐行输出（SQL 已正确按 (project, method_type, amount) 分组）
    let mut row_idx = HR + 1;
    let mut project_groups: Vec<(u32, u32, String)> = vec![];
    let mut current_project = String::new();
    let mut project_start = HR + 1;

    for row in rows {
        // 检测项目变化
        if !current_project.is_empty() && current_project != row.project {
            project_groups.push((project_start, row_idx - 1, current_project.clone()));
            project_start = row_idx;
        }
        current_project = row.project.clone();

        ws.write_with_format(row_idx, CA, row.project.as_str(), &fmt.fd)?;

        // 根据 method_type 填充
        if row.method_type.contains("液相") {
            ws.write_with_format(row_idx, CB, row.quantity as f64, &fmt.fd)?;
            ws.write_with_format(row_idx, CC, row.amount, &fmt.fd)?;
            ws.write_formula(row_idx, CD, format!("={}{}*{}{}", col_letter(CB), row_idx+1, col_letter(CC), row_idx+1).as_str())?;
        } else if row.method_type.contains("气相") {
            ws.write_with_format(row_idx, CE, row.quantity as f64, &fmt.fd)?;
            ws.write_with_format(row_idx, CF, row.amount, &fmt.fd)?;
            ws.write_formula(row_idx, CG, format!("={}{}*{}{}", col_letter(CE), row_idx+1, col_letter(CF), row_idx+1).as_str())?;
        } else {
            // 理化/质谱等
            ws.write_with_format(row_idx, CH, row.quantity as f64, &fmt.fd)?;
            ws.write_with_format(row_idx, CI, row.amount, &fmt.fd)?;
            ws.write_formula(row_idx, CJ, format!("={}{}*{}{}", col_letter(CH), row_idx+1, col_letter(CI), row_idx+1).as_str())?;
        }

        // 项目汇总：根据本行类型构建公式，未写列用 0 代替，避免引用空单元格导致 #VALUE!
        let mut parts: Vec<String> = Vec::new();
        if row.method_type.contains("液相") {
            parts.push(format!("{}{}*{}{}", col_letter(CB), row_idx+1, col_letter(CC), row_idx+1));
        } else {
            parts.push("0".to_string());
        }
        if row.method_type.contains("气相") {
            parts.push(format!("{}{}*{}{}", col_letter(CE), row_idx+1, col_letter(CF), row_idx+1));
        } else {
            parts.push("0".to_string());
        }
        if !row.method_type.contains("液相") && !row.method_type.contains("气相") {
            parts.push(format!("{}{}*{}{}", col_letter(CH), row_idx+1, col_letter(CI), row_idx+1));
        } else {
            parts.push("0".to_string());
        }
        ws.write_formula(row_idx, CK, format!("={}+{}+{}", parts[0], parts[1], parts[2]).as_str())?;

        row_idx += 1;
    }

    if row_idx > HR + 1 {
        project_groups.push((project_start, row_idx - 1, current_project));
    }

    // 合并项目列
    for (start, end, project) in &project_groups {
        // 硬保护：start == end 时不 merge（Excel 不允许合并单个单元格）
        if *start == *end { continue; }
        ws.merge_range(*start, CA, *end, CA, project.as_str(), &fmt.fd)?;
        ws.merge_range(*start, CK, *end, CK, "", &fmt.fd)?;
    }

    ws.set_freeze_panes(HR + 1, 1)?;
    Ok(())
}

// ========== Sheet 9: 仪器汇总表 ==========

pub fn write_sheet9(
    ws: &mut Worksheet,
    rows: &[InstrumentSummaryRow],
    fmt: &Fmt
) -> Result<()> {
    ws.set_name("仪器汇总表").map_err(|e| AppError::Internal(e.to_string()))?;
    ws.set_tab_color(Color::RGB(0x9E9E9E));

    ws.set_column_width(CA, 16.0)?;  // 仪器编号
    ws.set_column_width(CB, 12.0)?;  // 检测量
    ws.set_column_width(CC, 12.0)?;  // 类型
    ws.set_column_width(CD, 15.0)?;  // 按类型汇总

    let headers = ["仪器编号", "检测量", "类型", "按类型汇总"];
    for (i, h) in headers.iter().enumerate() {
        ws.write_with_format(HR, i as u16, *h, &fmt.fh)?;
    }

    // 按类型分组计算汇总
    use std::collections::HashMap;
    let mut type_totals: HashMap<String, i64> = HashMap::new();
    for row in rows {
        *type_totals.entry(row.instrument_type.clone()).or_insert(0) += row.quantity;
    }

    let mut row_idx = HR + 1;
    let mut type_groups: Vec<(u32, u32, String)> = vec![];
    let mut current_type = String::new();
    let mut type_start = HR + 1;

    for row in rows {
        ws.write_with_format(row_idx, CA, row.instrument.as_str(), &fmt.fd)?;
        ws.write_with_format(row_idx, CB, row.quantity as f64, &fmt.fd)?;
        ws.write_with_format(row_idx, CC, row.instrument_type.as_str(), &fmt.fd)?;

        if !current_type.is_empty() && current_type != row.instrument_type {
            type_groups.push((type_start, row_idx - 1, current_type.clone()));
            type_start = row_idx;
        }
        current_type = row.instrument_type.clone();
        row_idx += 1;
    }

    if row_idx > HR + 1 {
        type_groups.push((type_start, row_idx - 1, current_type));
    }

    // 写按类型汇总
    for &(start, _, ref itype) in &type_groups {
        if let Some(&total) = type_totals.get(itype) {
            ws.write_with_format(start, CD, total as f64, &fmt.fd)?;
        }
    }

    ws.set_freeze_panes(HR + 1, 1)?;
    Ok(())
}

// ========== Sheet 10: 理化汇总表 ==========

pub fn write_sheet10(
    ws: &mut Worksheet,
    rows: &[PhysChemRow],
    fmt: &Fmt
) -> Result<()> {
    ws.set_name("理化汇总表").map_err(|e| AppError::Internal(e.to_string()))?;
    ws.set_tab_color(Color::RGB(0x795548));

    ws.set_column_width(CA, 40.0)?;  // 方法名
    ws.set_column_width(CB, 12.0)?;  // 数量

    let headers = ["方法名", "数量"];
    for (i, h) in headers.iter().enumerate() {
        ws.write_with_format(HR, i as u16, *h, &fmt.fh)?;
    }

    let mut row_idx = HR + 1;
    for row in rows {
        ws.write_with_format(row_idx, CA, row.method.as_str(), &fmt.fd)?;
        ws.write_with_format(row_idx, CB, row.quantity as f64, &fmt.fd)?;
        row_idx += 1;
    }

    // 总计行
    if row_idx > HR + 1 {
        ws.write_with_format(row_idx, CA, "总计", &fmt.fb)?;
        ws.write_formula(row_idx, CB, format!("=SUM({}{}:{}{})", col_letter(CB), HR+2, col_letter(CB), row_idx).as_str())?;
    }

    ws.set_freeze_panes(HR + 1, 1)?;
    Ok(())
}
