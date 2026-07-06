# Excel 导出错误修复方案

## 问题分析

根据错误提示"Excel 无法打开文件，因为文件格式或文件扩展名无效"，可能的原因：

1. **Sheet 写入异常** - 某个 Sheet 在写入时出错导致文件损坏
2. **公式错误** - Excel 公式格式不正确
3. **单元格合并冲突** - 合并范围重叠
4. **数据类型错误** - 写入了不支持的数据类型

## 快速诊断步骤

### 步骤 1：添加详细日志

在 `src/api/export_handler.rs` 中，每个 Sheet 生成后添加错误捕获：

```rust
// ========== Sheet 1: 各实验室项目方法对应表 ==========
{
    let data = export_data::query_sheet1_data(&conn, start, end, q.group_id)?;
    let ws = wb.add_worksheet();
    match export_write::write_sheet1(ws, &data, &fmt) {
        Ok(_) => tracing::info!("Sheet 1 完成: {} 行", data.len()),
        Err(e) => {
            tracing::error!("Sheet 1 写入失败: {}", e);
            return Err(e);
        }
    }
}
```

### 步骤 2：逐个 Sheet 测试

临时修改 `export_handler.rs`，注释掉大部分 Sheet，只保留 Sheet 1：

```rust
// ========== Sheet 1: 各实验室项目方法对应表 ==========
{
    let data = export_data::query_sheet1_data(&conn, start, end, q.group_id)?;
    let ws = wb.add_worksheet();
    export_write::write_sheet1(ws, &data, &fmt)?;
    tracing::info!("Sheet 1 完成: {} 行", data.len());
}

// 注释掉 Sheet 2-10
/*
// ========== Sheet 2: 仪器-汇总 ==========
{
    let data = export_data::query_sheet2_data(&conn, start, end)?;
    let ws = wb.add_worksheet();
    export_write::write_sheet2(ws, &data, &fmt)?;
    tracing::info!("Sheet 2 完成: {} 行", data.len());
}
... (其他 Sheet)
*/
```

逐个取消注释测试，找出有问题的 Sheet。

## 常见问题修复

### 问题 1：公式引用错误

**症状**：公式中行号或列号越界

**修复**：在 `export_write.rs` 中检查公式生成：

```rust
// 错误示例
ws.write_formula(row_idx, CF, format!("=SUM({}{}:{}{})", 
    col_letter(CE), row_idx+1, col_letter(CE), row_idx))?;
// ↑ 结束行小于开始行

// 正确示例
if end_row >= start_row {
    ws.write_formula(start_row, CF, format!("=SUM({}{}:{}{})", 
        col_letter(CE), start_row+1, col_letter(CE), end_row+1))?;
}
```

### 问题 2：单元格合并冲突

**症状**：合并范围重叠

**修复**：确保合并前检查范围：

```rust
// 添加检查
if end > start {
    ws.merge_range(start, CA, end, CA, &lab, &fmt.fd)?;
}
```

### 问题 3：空数据导致的问题

**症状**：某些 Sheet 数据为空时出错

**修复**：添加空数据检查：

```rust
pub fn write_sheet1(
    ws: &mut Worksheet,
    rows: &[FlatRow],
    fmt: &Fmt
) -> Result<()> {
    ws.set_name("各实验室项目方法对应表")
        .map_err(|e| AppError::Internal(e.to_string()))?;
    
    // 添加空数据检查
    if rows.is_empty() {
        tracing::warn!("Sheet 1 数据为空");
        // 仍然写入表头
        let headers = ["使用实验室", "项目代号", "液相仪器", "检测方法", "检测数量", "液相检测量", "气相检测量", "项目检测总量"];
        for (i, h) in headers.iter().enumerate() {
            ws.write_with_format(HR, i as u16, *h, &fmt.fh)?;
        }
        return Ok(());
    }
    
    // 继续正常逻辑...
}
```

### 问题 4：日期范围错误

**症状**：查询返回空结果或错误数据

**修复**：在 `export_handler.rs` 中修正日期处理：

```rust
// 确定日期范围
let (start, end) = if let Some(ref s) = q.start {
    let e = q.end.as_ref().map(|e| e.as_str()).unwrap_or(s);
    (s.clone(), e.to_string())
} else {
    // 默认当月
    let now = chrono::Local::now();
    let start = format!("{}-{:02}-01", now.year(), now.month());
    
    // 计算月末
    let next_month = if now.month() == 12 {
        chrono::NaiveDate::from_ymd_opt(now.year() + 1, 1, 1)
    } else {
        chrono::NaiveDate::from_ymd_opt(now.year(), now.month() + 1, 1)
    };
    
    let end = if let Some(nm) = next_month {
        nm.pred_opt().map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| format!("{}-{:02}-28", now.year(), now.month()))
    } else {
        format!("{}-{:02}-28", now.year(), now.month())
    };
    
    (start, end)
};

tracing::info!("导出日期范围: {} 到 {}", start, end);
```

## 临时简化版本（紧急修复）

如果时间紧急，可以先恢复到 v0.3.5 的 5 Sheet 版本：

1. 复制 `v0.3.5/src/api/export_*.rs` 的备份
2. 或者从 git 回滚：`git checkout v0.3.5 -- src/api/export_*.rs`
3. 重新编译测试

## 推荐的调试流程

1. **查看服务器日志**
   ```bash
   # 运行带控制台版本
   cargo run --features console
   
   # 或查看日志文件
   type app.log | findstr /C:"Sheet" /C:"export" /C:"ERROR"
   ```

2. **测试单个 Sheet**
   - 修改代码只导出 Sheet 1
   - 测试能否打开
   - 逐个添加 Sheet 2、3、4...

3. **使用浏览器开发者工具**
   - F12 打开开发者工具
   - 查看 Network 标签
   - 点击导出，查看响应状态和大小

4. **文件完整性检查**
   ```bash
   # 检查文件大小
   dir 工作量统计_*.xlsx
   
   # 如果文件只有几 KB，说明写入不完整
   # 正常应该有 50KB 以上
   ```

## 快速修复代码片段

创建一个简化的测试版本 `export_handler_simple.rs`：

```rust
/// 简化版导出（仅 Sheet 1-5，稳定版本）
async fn export_excel_simple(
    State(pool): State<DbPool>,
    Query(q): Query<ExportQuery>
) -> Result<impl IntoResponse> {
    use std::io::Cursor;

    let (start, end) = determine_date_range(&q);
    let conn = pool.get()?;
    let fmt = export_write::Fmt::new();
    let mut wb = rust_xlsxwriter::Workbook::new();

    tracing::info!("开始导出 Excel (简化版): start={}, end={}", start, end);

    // 只导出前 5 个最稳定的 Sheet
    // Sheet 1
    {
        let data = export_data::query_sheet1_data(&conn, &start, &end, q.group_id)?;
        if !data.is_empty() {
            let ws = wb.add_worksheet();
            export_write::write_sheet1(ws, &data, &fmt)?;
            tracing::info!("Sheet 1 完成: {} 行", data.len());
        }
    }

    // Sheet 2
    {
        let data = export_data::query_sheet2_data(&conn, &start, &end)?;
        if !data.is_empty() {
            let ws = wb.add_worksheet();
            export_write::write_sheet2(ws, &data, &fmt)?;
            tracing::info!("Sheet 2 完成: {} 行", data.len());
        }
    }

    // ... 继续 Sheet 3-5

    // 保存
    let mut buf = Cursor::new(Vec::new());
    wb.save_to_writer(&mut buf)
        .map_err(|e| {
            tracing::error!("Excel 保存失败: {}", e);
            AppError::Internal(format!("Excel 保存失败: {}", e))
        })?;
    
    let data = buf.into_inner();
    tracing::info!("Excel 生成成功: {} bytes", data.len());

    // 返回响应
    let filename = format!("工作量统计_{}_{}.xlsx", start, end);
    Ok(axum::response::Response::builder()
        .header(header::CONTENT_TYPE, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
        .header(header::CONTENT_DISPOSITION, 
            format!("attachment; filename*=UTF-8''{}", url_escape::encode_component(&filename)))
        .body(axum::body::Body::from(data))
        .unwrap())
}
```

## 需要您提供的信息

为了更准确地定位问题，请提供：

1. **服务器日志**：运行时的控制台输出或 app.log
2. **文件大小**：导出的 xlsx 文件有多大？
3. **测试数据**：数据库中有多少条记录？
4. **浏览器信息**：F12 开发者工具 Network 标签中的响应信息

## 下一步

我建议您：

1. **运行带控制台的版本**查看详细日志
2. **从最简单的版本开始**（只导出 1-2 个 Sheet）
3. **逐步添加 Sheet**找出问题所在

我可以根据您提供的日志信息进一步定位问题并提供精确的修复方案。
