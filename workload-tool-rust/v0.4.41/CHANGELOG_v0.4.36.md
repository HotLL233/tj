# v0.4.36 更新说明（尝试测试版）

> **版本**: 0.4.36 | **日期**: 2026-07-11 | **基线**: v0.4.35

## 可视化页面布局编辑器（MVP）

### 新增功能
- **可视化编辑器**：管理员可在页面上直接点击「✎ 编辑布局」进入编辑模式
- **三栏编辑器**：左侧字段库 → 中间页面预览 → 右侧属性面板
- **字段可编辑**：拖动排序、编辑标签、调整列宽、开关必填/显隐
- **7 种字段类型**：文本/下拉/多行文本/数字/日期/分隔线/标题
- **一键发布**：编辑完成后点「发布布局」保存，刷新页面生效

### MVP 范围
- 首期仅支持「研发送样录入」页的今日记录表格
- 管理员可用，普通用户看不到编辑按钮
- 布局配置存储在 `system_settings` 表中

## 技术细节
- **新建文件**（6 个）：`types/layout.ts`、`EditableField.tsx`、`FieldLibrary.tsx`、`FieldPropertyPanel.tsx`、`LayoutDesigner.tsx`、`EditablePageShell.tsx`
- **修改文件**（2 个）：`migrations.rs`（种子 layout_sample_entry）、`SampleEntryPage.tsx`（动态布局渲染）
- **基础设施复用**：`system_settings` 表 + `GET/PUT /api/settings/:key` API（来自 v0.4.35）

## 升级说明
- 安装后首次启动自动插入 `layout_sample_entry` 默认字段
- 管理员进入研发送样录入页面，右上角可见「✎ 编辑布局」
- 安装包可覆盖安装 v0.4.30~v0.4.35（AppId 固定）
