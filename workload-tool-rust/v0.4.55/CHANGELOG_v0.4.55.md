# v0.4.55 版本更新说明

## 更新日期
2026-07-13

## 修复

### Q1 - 部门管理提示文案错误
- 旧：`按检测技术维度（液相/气相/...）` — 与组织部门无关
- 新：`按组织架构（部门/事业部）归拢实验室`

### Q2 - 研发送样记录部门显示数字 + 列宽不跟随配置
- **部门显示**：`getFieldValue('division_id')` 从直接输出 ID 改为 `divs.find(d => d.id === rec.division_id)?.name`
- **列宽**：`form_sample_entry` 配置不在时回退 `RdRecordColumn.width`，表格 `tableLayout: 'fixed'`

### Q3 - 切换检测类型后行数据丢失
- **旧**：`setRows([emptyRow(cols, user)])` — 清空所有行
- **新**：按 field_key 映射保留已有行数据，新表没有的字段留空

### Q4 - 附件按钮被禁用消失
- 保持禁用但恢复视觉样式（`AttachFileIcon` + `保存后上传`）

## 改动文件

| 文件 | 改动 |
|------|------|
| `frontend/src/pages/ManagePage.tsx` | 部门提示文案 |
| `frontend/src/pages/RdRecordsPage.tsx` | division_id 解析 + form 列宽 + tableLayout fixed |
| `frontend/src/pages/SampleInfoEntry.tsx` | 切换 dt 保留行 + 附件按钮样式 |

## 验证
- ✅ `tsc --noEmit` 0 errors
- ✅ `vite build` 3.07s
- ✅ `cargo build --release` 1m28s
- ✅ Inno Setup 打包成功
