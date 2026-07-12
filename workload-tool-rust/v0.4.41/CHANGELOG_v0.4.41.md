# v0.4.41 版本更新说明

## 更新日期
2026-07-12

## 更新内容
### 修复：研发送样录入表格「编辑布局」真正生效
- **根因**：录入表格列宽读 `rd_record_columns.width`，但「编辑布局」按钮改的是 `layout_sample_entry_fields.width`（FieldDef），两个独立数据源，编辑不生效。
- **修复**：录入表格改为直接读 `layoutFields`（EditablePageShell 管理的 FieldDef 数组），「编辑布局」改列宽/标签 → 录入表格即时生效。
- **改动文件**：仅 `frontend/src/pages/SampleEntryPage.tsx`
  - 移除 `rdColumns` 状态和 `getRdRecordColumns()` API 调用
  - 表头/表体 `formColumns.map(col => ...)` → `visibleLayoutFields.map(field => ...)`
  - `col.name` → `field.key`，`col.width` → `field.width`

## 打包说明
- 无 cmd 窗口（生产模式 `cargo build --release`）
- 支持覆盖安装（AppId 固定）
