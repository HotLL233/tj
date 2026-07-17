# v0.4.66 版本更新说明

## 更新日期
2026-07-15

## 更新内容

### 导出 Excel 全 10 Sheet + 统计页全 5 分类卡加"高项"列
- **export_data.rs**: 5 个 Row 结构体加 high_item 字段 + 5 个 SQL 查询加 p.high_item/wr.high_item
- **export_write.rs**: 5 个 Sheet write 函数各加高项列（调整列宽/表头/公式引用）
- **rd_export_data.rs**: 对应研发送样导出加 high_item 字段
- **StatsPage.tsx**: 5 个 PreviewTable 各加高项列定义

### 修复：汇总求和公式单行跳过 bug
- export_write.rs Sheet 1/3/4 的单行聚合跳过 `if start == end { continue; }` 导致汇总列为空
- 修复：单行也写公式（直接引用值，可扩展为多行 SUM）

## 改动文件
- `src/api/export_data.rs`
- `src/api/export_write.rs`
- `src/api/rd_export_data.rs`
- `frontend/src/pages/StatsPage.tsx`
