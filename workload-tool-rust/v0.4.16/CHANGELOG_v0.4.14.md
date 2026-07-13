## v0.4.14 — 2026-07-08

### 新增
- **帮助文档弹窗内嵌查看**：点击卡片不再新标签页打开，改为页面内弹窗 iframe 预览
  - PDF/图片/TXT/CSV 直接弹窗内显示，用户无需离开 HelpPage
  - 使用 `fetch` + `blob` + `URL.createObjectURL` 方式加载文件，绕开 IDM 等下载工具拦截
  - 不支持在线预览的格式（如 docx/xlsx）提供「下载查看」按钮
  - 关闭弹窗自动释放 blob URL，避免内存泄漏

### 变更文件
- `frontend/src/pages/HelpPage.tsx` — 卡片「查看文档」按钮改为 fetch+blob+Dialog 内嵌 iframe 预览

### 安装说明
- 本次更新覆盖安装即可，无需卸载旧版本
- 安装包文件名：`工作量统计工具_Rust_v0.4.14_Setup.exe`
