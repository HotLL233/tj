# v0.4.73 版本更新说明

## 更新日期
2026-07-15

## 新增：Excel 上传 → 可编辑表格

### Excel 解析
- 新增 `xlsx_parser.rs`：使用 calamine 库解析 xlsx 文件，每个 Sheet 生成 HTML `<table>` 表格
- 表头加粗、单元格带边框样式、空列补齐
- 上传 Word/PDF/Excel 后自动创建对应的帮助文章

### 可编辑表格
- 帮助页查看含有表格的文章时，有 `help:edit` 权限的用户可点击「编辑」按钮
- 编辑模式切换到 MUI `<Table>` 组件，每个单元格是 `<TextField>` 可输入框
- 支持添加行 / 删除行
- 保存后更新 `content_html`，切换回查看模式

### 权限管控
- 新增 `help:edit` 权限点，自动分配给系统管理员、分析检测员、样品登记员角色
- 前端 `HelpPage` 根据 `hasPermission('help:edit')` 控制编辑按钮显隐

## 改动文件
- `src/api/xlsx_parser.rs` — 新文件，Excel → HTML 表格解析
- `src/api/mod.rs` — +pub mod xlsx_parser
- `src/api/help_handler.rs` — upload 分支加 xlsx/xls 解析
- `src/db/migrations.rs` — +help:edit 种子权限
- `frontend/src/api/client.ts` — updateHelpArticle 加 content_html
- `frontend/src/constants/permissions.ts` — +help:edit 常量
- `frontend/src/pages/HelpPage.tsx` — 文章查看弹窗加编辑模式 + 可编辑 MUI Table
