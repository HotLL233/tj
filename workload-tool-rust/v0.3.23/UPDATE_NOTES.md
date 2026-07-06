# v0.3.23 更新说明

## 发布日期
2026-07-04

## 更新内容

### 1. 导出Excel全局单元格格式统一（问题1）
- **修复**：为所有10个sheet函数添加 `set_column_format` 调用
- **效果**：导出Excel的所有单元格（含空白单元格）统一使用仿宋14号字体、水平垂直居中、细边框格式
- **影响文件**：`src/api/export_write.rs`

### 2. 管理界面所有卡片新增一览按钮（问题2）
- **新增功能**：在「研发项目管理」和「实验室管理」Tab 添加「项目一览」和「实验室一览」按钮
- **实现**：参考方法管理的「方法一览」功能，创建两个新的弹窗组件
  - 「项目一览」弹窗：支持内联编辑项目名称、全名、排序
  - 「实验室一览」弹窗：支持内联编辑实验室名称、排序
- **影响文件**：`project-root/frontend/src/pages/ManagePage.tsx`

### 3. 方法一览编辑表实时显示修复（问题3）
- **修复**：为方法一览弹窗中的所有编辑字段添加 `setMethodOverviewData` 立即更新
  - 方法名 `onBlur`：立即更新本地state
  - 类型 Chip `onClick`：立即更新本地state
  - 系数 `onBlur`：立即更新本地state
  - 单价 `onBlur`：立即更新本地state
- **效果**：编辑后实时显示编辑后的状态，无需手动刷新
- **影响文件**：`project-root/frontend/src/pages/ManagePage.tsx`

### 4. 删除统计页面导入Excel功能（问题4）
- **删除**：统计页面（StatsPage）的导入Excel按钮和对应功能代码
- **清理内容**：
  - 删除移动端和桌面端的导入按钮
  - 删除导入结果反馈区（Alert、Snackbar）
  - 删除 `importResult`、`importOpen`、`importErr` 状态变量
  - 删除 `hi` 函数和 `importing` 状态
  - 删除 `UploadFileIcon` 的 import 和 `ImportResult` 类型引用
- **影响文件**：`project-root/frontend/src/pages/StatsPage.tsx`

### 5. 修复统计页面实验室列显示多个名称（问题5）
- **根因**：`by_project` SQL 使用 `LEFT JOIN project_lab_links` 导致一个项目关联多个实验室时产生笛卡尔积，数量翻倍
- **修复**：将 SQL 从 `LEFT JOIN` 改为 `group_concat` 子查询
- **效果**：实验室显示列正确显示用逗号拼接的实验室名称（如"410,411"），但每个项目只显示一行
- **影响文件**：`src/api/stats_handler.rs`

## 技术细节

### 后端
- Rust Axum 0.7.9 + rusqlite 0.31 + r2d2
- 版本号：0.3.23
- 编译警告：6个（不影响功能）

### 前端
- React 18 + TypeScript + Vite + MUI
- 构建输出：静态文件位于 `static/` 目录
- 构建警告：某些chunk大于500kB（可忽略）

### 打包
- Inno Setup 6.7.2
- 安装包路径：`D:\桌面\工作量统计工具项目\installer\工作量统计工具_Rust_v0.3.23_Setup.exe`
- 覆盖安装：支持（AppId 保持不变）

## 已知问题
- 无新增已知问题

## 升级建议
- 直接安装新版本，覆盖旧版本（v0.3.14 或更高版本）
- 数据和配置会自动保留

---
**版本隔离说明**：本版本为独立版本目录 `v0.3.23`，不修改旧版本代码。
