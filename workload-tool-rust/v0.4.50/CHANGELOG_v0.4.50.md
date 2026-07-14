# v0.4.50 版本更新说明

## 更新日期
2026-07-13

## 新增功能：表格全局配置

### 管理页 → 录入表单配置 — 新增「表格全局设置」
- **行高**（默认 48px）— 控制录入表格每行的高度
- **序号列宽**（默认 50px）— 控制每张表格序号列的宽度
- **复选框列宽**（默认 36px）— 控制复选框列的宽度

### 向后兼容
- v0.4.49 旧格式（纯 `FieldDef[]` 数组）仍可正常读取
- 保存后自动升级为 `FormLayout` 新格式（`{table_config, fields}`）

## 技术改动

### 文件清单

| 文件 | 改动 |
|------|------|
| `types/layout.ts` | 新增 `TableConfig`、`FormLayout` 接口 + `DEFAULT_TABLE_CONFIG` 常量 |
| `components/ManageFormConfig.tsx` | 加表格全局设置编辑区；load/save 兼容新旧格式；预览使用 tableConfig |
| `pages/SampleEntryPage.tsx` | 加载 tableConfig；`width:40` → `seq_column_width`；`height:48` → `row_height` |
| `pages/SampleInfoEntry.tsx` | 加载 tableConfig；3 处硬编码宽度改为配置值 |
| `db/migrations.rs` | 3 条 form_* 种子改为 `FormLayout` 格式 |

## 验证
- ✅ `tsc --noEmit` 0 errors
- ✅ `vite build` 3.31s
- ✅ `cargo build --release` 2m02s
- ✅ Inno Setup 打包成功
