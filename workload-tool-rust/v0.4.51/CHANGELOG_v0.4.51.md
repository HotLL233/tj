# v0.4.51 版本更新说明

## 更新日期
2026-07-13

## 修复
### P0 - 录入表格列宽不自洽
- **根因**：表体复选框/序号列的 TableCell 未设 `width`，与表头的按比例压缩布局不匹配，导致最后一列被切到屏幕外
- **修复**：4 处加 `width: tableConfig.seq_column_width` / `tableConfig.checkbox_column_width`

## 新增：导出模板配置器（阶段B）
### 管理页 → 导出模板配置 tab
- 3 个模板：分析检测统计 / 研发送样统计 / 样品信息登记
- 编辑内容：
  - 文件名模板（支持 `{s}` `{e}` 占位符）
  - 每张 Sheet 名称、标签颜色、启用/禁用
  - 每列标签、列宽
- 实时预览导出 Sheet 清单

### 后端集成（分析检测统计导出）
- 导出时自动读取 `export_template_workload` 配置
- Sheet 名称、标签颜色、列宽按配置覆盖（硬编码为回退默认值）

## 改动文件

| 文件 | 改动 |
|------|------|
| `components/ManageExportConfig.tsx` | **新增** 330 行导出模板编辑器 |
| `pages/ManagePage.tsx` | 加 `exports` tab |
| `pages/SampleEntryPage.tsx` | 4 处表体 cell 加 width（列宽 Bug） |
| `src/api/export_handler.rs` | 加 `apply_sheet_cfg()` 函数 + 11 个 sheet 读取模板 |
| `db/migrations.rs` | 种子 3 条 `export_template_*` |

## 验证
- ✅ `tsc --noEmit` 0 errors
- ✅ `vite build` 3.10s
- ✅ `cargo check` 通过
- ✅ `cargo build --release` 1m36s
- ✅ Inno Setup 打包成功
