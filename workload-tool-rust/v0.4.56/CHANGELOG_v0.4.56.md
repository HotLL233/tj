# v0.4.56 版本更新说明

## 更新日期
2026-07-13

## 更新内容

### 修复：研发送样记录编辑面板字段不全
- **根因**：编辑面板的 `setEditForm` 和 `handleSave` 只硬编码 6 个字段（送样人/项目/方法/数量/批号/注意事项），缺失部门、实验室、检测类型等字段
- **修复**：编辑面板改为基于 `columns` 动态渲染，`handleSave` 动态对比所有字段，新增行自动包含全部 show_in_list 字段
- **改动文件**：仅 `frontend/src/pages/RdRecordsPage.tsx`

### 修复：新增检测类型表格不显示列
- **根因**：`list_active_by_type` SQL 对预置列用了 `INNER JOIN sample_info_column_visibility`，新建的检测类型（如"热稳定性"）没有 visibility 行，导致返回 0 列
- **修复**：改为 `LEFT JOIN` + `v.is_visible = 1 OR v.is_visible IS NULL`（无 visibility 行时默认可见）
- **改动文件**：仅 `src/repo/sample_info_column_repo.rs`

### 改进：检测类型删除改为软删除
- **根因**：用户点击删除显示"删除成功"但实际只是软删除（is_active=0），UI 误导
- **修复**：文案改为"已停用（可在下方恢复）"；停用项新增 ♻ 恢复按钮
- **改动文件**：仅 `frontend/src/pages/ManagePage.tsx`

## 打包说明
- 无 cmd 窗口（生产模式 `cargo build --release`）
- 支持覆盖安装（AppId 固定）
- 同步更新 Docker 版
