# v0.4.43 版本更新说明

## 更新日期
2026-07-13

## 更新内容
### 新增：管理页集中式「页面布局管理」
- **背景**：原有编辑功能分散在各页面中（PageEditToggle + PageSectionEditor），部分页面存在 fetch 无 token 导致 401 的问题，编辑体验不统一。
- **实现**：管理页新增「页面布局管理」标签，可统一编辑所有 12 个页面的布局：
  - 左侧页面列表（家页/统计/入口/录入/记录/管理页等）
  - 右侧 Section 编辑器：开关可见性、编辑显示文本、按钮文字、Chip 文字、操作按钮文字
  - 统一使用 Axios getSetting/updateSetting（自动附 token）
  - 支持「重置默认」和「保存布局」
- **改动文件**：
  - `frontend/src/components/PageLayoutAdmin.tsx` — 新增：集中式布局编辑器
  - `frontend/src/pages/ManagePage.tsx` — 新增 layouts tab + DashboardIcon

### 注
- 原有的页面内「编辑页面」按钮（PageEditToggle）暂时保留，不影响使用
- 后续版本将逐步移除分散式编辑

## 打包说明
- 无 cmd 窗口（生产模式 `cargo build --release`）
- 支持覆盖安装（AppId 固定）
