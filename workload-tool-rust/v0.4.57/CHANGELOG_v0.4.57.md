# v0.4.57 版本更新说明

## 更新日期
2026-07-13

## 更新内容

### 新增：长字符 Tooltip（TruncatedCell 组件）
所有列表页面的长文本列（项目名/方法名/注意事项等）鼠标悬停时展示完整内容，超长不截断

### 新增：一键回到顶部按钮（BackToTop 组件）
所有页面右下角悬浮 ⬆ 按钮，滚动超过 200px 自动出现

### 修复：检测类型删除 → 移入回收站
- 删除检测类型不再留在原列表，移至回收站（可恢复）
- 列表默认只显示启用的检测类型
- 支持彻底删除（从回收站永久删除）

### 改进：检测类型 API
- `DELETE /api/sample-info-types/:id` 软删除（移入回收站）
- `DELETE /api/sample-info-types/:id/permanent` 永久删除

### 改进：研发送样记录显示
- 所有长文本列改用 Tooltip 悬停展示
- 保留取样人/取样时间/状态显示
- 保留 inline 编辑功能

### 改动文件
- 新增：`frontend/src/components/TruncatedCell.tsx`
- 新增：`frontend/src/components/BackToTop.tsx`
- 修改：`frontend/src/components/Layout.tsx` — 集成 BackToTop
- 修改：`frontend/src/pages/RdRecordsPage.tsx` — TruncatedCell
- 修改：`frontend/src/pages/ManagePage.tsx` — 回收站、列表过滤
- 修改：`frontend/src/api/client.ts` — 永久删除 API
- 修改：`src/api/sample_info_type_handler.rs` — 永久删除路由
- 修改：`src/repo/sample_info_type_repo.rs` — 永久删除函数

## 打包说明
- 无 cmd 窗口（生产模式 `cargo build --release`）
- 支持覆盖安装（AppId 固定）
- 同步更新 Docker 版
