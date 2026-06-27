// ─── Excel structure definitions & write helpers ──────────────────
// Tests: cargo test -- export_write

pub const CB: u16 = 1; pub const CC: u16 = 2; pub const CD: u16 = 3; pub const CE: u16 = 4;
pub const CF: u16 = 5; pub const CG: u16 = 6; pub const CH: u16 = 7; pub const CI: u16 = 8;
pub const HR: u32 = 2;

/// 0-based column index → Excel column letter (0=A, 1=B, ...)
pub fn _cl(n: u16) -> String {
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

pub struct Fmt { pub fh: rust_xlsxwriter::Format, pub fd: rust_xlsxwriter::Format }

impl Fmt {
    pub fn new() -> Self {
        use rust_xlsxwriter::*;
        Self { fh: Format::new().set_bold().set_font_size(16), fd: Format::new().set_font_size(16) }
    }
}

pub fn set_col_widths(ws: &mut rust_xlsxwriter::Worksheet) -> crate::error::Result<()> {
    let w = [8.89, 24.89, 18.0, 17.44, 43.66, 19.66, 12.0, 12.0, 22.11];
    for (i, v) in w.iter().enumerate() { ws.set_column_width(i as u16, *v)?; }
    Ok(())
}

/// Detect group boundaries for merge: (start, end, lc_count, gc_count)
pub fn detect_groups(rows: &[super::export_data::FlatRow]) -> Vec<(u32, u32, u32, u32)> {
    let mut groups: Vec<(u32, u32, u32, u32)> = vec![];
    let mut i = 0usize;
    while i < rows.len() {
        let (ref_l1, ref_l2, _, _, _, _) = &rows[i];
        let gs = (HR + 1) + i as u32;
        let mut ge = gs;
        let mut lc = 0u32; let mut gc = 0u32;
        while i < rows.len() {
            let (l1, l2, _, _, _, is_gc) = &rows[i];
            if l1 != ref_l1 || l2 != ref_l2 { break; }
            if *is_gc { gc += 1; } else { lc += 1; }
            ge = (HR + 1) + i as u32;
            i += 1;
        }
        groups.push((gs, ge, lc, gc));
    }
    groups
}

/// Write a full tree sheet (used by monthly/daily/user stats)
pub fn write_tree_sheet(
    ws: &mut rust_xlsxwriter::Worksheet, rows: &[super::export_data::FlatRow],
    l1_hdr: &str, l2_hdr: &str, qty_hdr: &str, fmt: &Fmt,
) -> crate::error::Result<()> {
    let hdrs = [l1_hdr, l2_hdr, "液相仪器", "检测方法", qty_hdr, "液相检测量", "气相检测量", "项目检测总量"];
    for (i, h) in hdrs.iter().enumerate() {
        ws.write_with_format(HR, (i + 1) as u16, *h, &fmt.fh)?;
    }
    let groups = detect_groups(rows);

    // Step 1: write data first
    let mut ri = HR + 1;
    for (l1, l2, ic, ml, qty, _) in rows {
        ws.write_with_format(ri, CB, l1.as_str(), &fmt.fd)?;
        ws.write_with_format(ri, CC, l2.as_str(), &fmt.fd)?;
        ws.write_with_format(ri, CD, ic.as_str(), &fmt.fd)?;
        ws.write_with_format(ri, CE, ml.as_str(), &fmt.fd)?;
        if *qty > 0 { ws.write_with_format(ri, CF, *qty as f64, &fmt.fd)?; }
        ri += 1;
    }

    // Step 2: merge G/H/I
    for &(gs, ge, _, _) in &groups {
        if ge > gs { for col in [CG, CH, CI] { ws.merge_range(gs, col, ge, col, "", &fmt.fd)?; } }
    }

    // Step 3: formulas to G/H/I
    for &(gs, ge, lc_cnt, gc_cnt) in &groups {
        if lc_cnt > 0 {
            let lc_last = gs + lc_cnt - 1;
            ws.write_formula(gs, CG, format!("=SUM({}{}:{}{})", _cl(CF), gs, _cl(CF), lc_last).as_str())?;
        }
        if gc_cnt > 0 {
            let gc_start = gs + lc_cnt;
            ws.write_formula(gs, CH, format!("=SUM({}{}:{}{})", _cl(CF), gc_start, _cl(CF), ge).as_str())?;
        }
        ws.write_formula(gs, CI, format!("=SUM({}{}:{}{})", _cl(CG), gs, _cl(CH), gs).as_str())?;
    }

    // Step 4: merge B(l1) + C(l2) with actual values
    let mut i = 0usize;
    while i < rows.len() {
        let ref_l1 = &rows[i].0;
        let bs = (HR + 1) + i as u32;
        let mut be = bs;
        while i < rows.len() && &rows[i].0 == ref_l1 { be = (HR + 1) + i as u32; i += 1; }
        if be > bs { ws.merge_range(bs, CB, be, CB, ref_l1.as_str(), &fmt.fd)?; }
    }
    for &(gs, ge, _, _) in &groups {
        if ge > gs {
            let row_idx = gs as usize - (HR + 1) as usize;
            if row_idx < rows.len() {
                ws.merge_range(gs, CC, ge, CC, rows[row_idx].1.as_str(), &fmt.fd)?;
            }
        }
    }

    // Step 5: total row
    let tr = ri;
    ws.write_with_format(tr, CB, "总计", &fmt.fd)?;
    for col in [CC, CD, CE] { ws.write_with_format(tr, col, "", &fmt.fd)?; }
    for col in [CF, CG, CH, CI] {
        let cl = _cl(col);
        ws.write_formula(tr, col, format!("=SUM({}{}:{}{})", cl, HR + 1, cl, tr - 1).as_str())?;
    }
    ws.set_freeze_panes(HR + 1, 4)?;
    Ok(())
}
