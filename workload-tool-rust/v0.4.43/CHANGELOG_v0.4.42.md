# v0.4.42 版本更新说明

## 更新日期
2026-07-12

## 更新内容
### 修复：「编辑布局」保存报「未提供登录凭证」
- **根因**：`EditablePageShell.handleSave` 之前改成原生 `fetch` 但忘了手动附加 `Authorization` Bearer token（fetch 不会自动加），后端 401 返回「未提供登录凭证」。
- **修复**：改回 `updateSetting()`（Axios，自动走拦截器附 token）。
- **改动文件**：仅 `frontend/src/components/EditablePageShell.tsx`

## 打包说明
- 无 cmd 窗口（生产模式 `cargo build --release`）
- 支持覆盖安装（AppId 固定）
