# v0.4.39 版本更新说明

## 更新日期
2026-07-12

## 更新内容
### 修复：导出功能异常
- **根因**：v0.4.38 StatsPage 的导出 hx() 函数从 Axios 改成了原生 fetch，但 Axios 的请求拦截器自动附加 JWT token，fetch 版本需手动附加。同时导出失败的错误信息只在选中卡片后才能看到，主页面无显示。
- **修复**：hx() 恢复为 Axios 调用（与 v0.4.28 一致），保留全局错误提示 Alert，确保主页面即可看到导出失败的具体原因。
- **改动文件**：仅 `frontend/src/pages/StatsPage.tsx`

## 打包说明
- 无 cmd 窗口（生产模式 `cargo build --release`）
- 支持覆盖安装（AppId 固定）
