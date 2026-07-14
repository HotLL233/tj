# v0.4.58 版本更新说明

## 更新日期
2026-07-14

## 更新内容

### 修复：样品信息登记附件上传（填写时可上传）
- **根因**：`case 'attachment'` 渲染 disabled 按钮（v0.4.55 遗留）
- **修复**：改为真实 `<input type="file">` 多文件选择按钮，选定文件后立即显示文件名/大小/删除按钮，提交时一并上传
- **改动文件**：`frontend/src/pages/SampleInfoEntry.tsx`

### 修复：悬停显示全部信息
- `SampleInfoEntry` 列表行所有字段改用 `TruncatedCell`（鼠标悬停显示完整内容）
- `TruncatedCell` 支持 ReactNode 类型
- **改动文件**：`frontend/src/components/TruncatedCell.tsx`、`frontend/src/pages/SampleInfoEntry.tsx`

### 修复：导出 Sheet 11 名称与数据一致
- **根因**：`ManageExportConfig` sheet11 标签为"类型汇总表"，但后端实际写入的是事业部汇总数据（`by_division`），`apply_sheet_cfg` 用错误名称覆盖了 `set_name`
- **修复**：`ManageExportConfig` sheet11 改为"事业部汇总"，列定义匹配实际数据（事业部/实验室数/检测数量/记录数/系数分）
- **改动文件**：`frontend/src/components/ManageExportConfig.tsx`

## 打包说明
- 无 cmd 窗口（生产模式 `cargo build --release`）
- 支持覆盖安装（AppId 固定）
- 同步更新 Docker 版
