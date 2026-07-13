## v0.4.15 — 2026-07-08

### 修复
- **彻底解决 IDM 下载拦截**：PDF 查看改用 Canvas 逐页渲染
  - 机制：`fetch` → `ArrayBuffer` → `pdfjsLib.getDocument()` → Canvas 逐页自绘
  - IDM 无法拦截 Canvas 绘制的内容（没有 PDF 资源的 HTTP 暴露）
  - 非 PDF 格式（图片/TXT/CSV）仍用 blob URL + iframe
  - 新增页面翻页控件（上一页/下一页 + 页码显示）
  - 关闭弹窗时调用 `pdfDoc.destroy()` 释放资源

### 变更文件
- `frontend/src/pages/HelpPage.tsx` — PDF 查看改用 pdfjs-dist Canvas 渲染，添加翻页功能
- `frontend/package.json` — 新增依赖 `pdfjs-dist`

### 安装说明
- 本次更新覆盖安装即可，无需卸载旧版本
- 安装包文件名：`工作量统计工具_Rust_v0.4.15_Setup.exe`
