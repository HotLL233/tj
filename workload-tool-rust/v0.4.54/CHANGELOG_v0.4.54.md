# v0.4.54 版本更新说明

## 更新日期
2026-07-13

## 修复

### P0 - 切换检测类型后列配置不刷新
- **根因**：`useEffect` 依赖数组为 `[]`，切换 `dt` 后 columns 不重新加载
- **修复**：改为 `[dt]`，切换检测类型时自动重新加载对应列配置

### P0 - 录入表单内附件按钮无操作
- **根因**：`renderCellInput` 中的 `case 'attachment'` 只渲染了无 onClick 的装饰按钮
- **修复**：改为禁用按钮显示"保存后上传"（录入行无 recordId，无法上传附件）

### P0 - 检测类型软删除后状态不明显
- **根因**：管理页 SiType 表 Chip 使用 `color="default"` 灰色不明显
- **修复**：Chip 改为 `color="error"` 红色 + label "已停用"（更醒目）

## 改动文件

| 文件 | 改动 |
|------|------|
| `frontend/src/pages/SampleInfoEntry.tsx` | useEffect 依赖 `[dt]`；附件按钮改为禁用 |
| `frontend/src/pages/ManagePage.tsx` | SiType 表格停用 Chip 改为红色 |

## 验证
- ✅ `tsc --noEmit` 0 errors
- ✅ `vite build` 2.97s
- ✅ `cargo build --release` 1m25s
- ✅ Inno Setup 打包成功
