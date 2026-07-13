# v0.4.47 版本更新说明

## 更新日期
2026-07-13

## 修复内容

### P0 - 修复保存字段布局后端 UTF-8 panic
- **根因**：`src/api/settings_handler.rs:78` 在 audit log 中 `&value_str[..200]` 字节切片，当 200 落在中文字符（如"取"）的 UTF-8 多字节边界内部时 panic，HTTP 连接断开，浏览器看到"网络错误"。
- **修复**：改用 `char_indices()` 寻找最近的字符边界，安全切片。

### P0 - 修复样品信息录入实验室列不显示
- **根因**：`SampleEntryPage` 录入表格的 9 个字段分支中，缺少 `lab_name` 渲染分支（其他 8 个都有），所以实验室列显示 `-`。
- **修复**：在 `user_name` 分支之前补 `lab_name` 分支（整页共用一个 lab，从 URL `group_id` 读取，禁用编辑，灰色背景）。

### P1 - 录入表格列宽弹性化
- **原状态**：表头/表体都用 `minWidth: field.width`，列被硬钉死。
- **修复**：
  - 表头 `width: ${(field.width || 100) / 10}%` 按比例分配
  - 表体 input `width: '100%'` 撑满 cell
  - 用户改 field.width 后列按比例伸缩，input 跟随

### P0 - 恢复内联编辑入口
- **场景**：v0.4.44 把所有页面的 PageEditToggle 删了，外部无编辑按钮。
- **修复**：
  - `Layout` 顶层用 `<PageEditProvider>` 包裹 `<Outlet />`（让所有 12 个页面拿到 edit context）
  - `SampleEntryPage` 加 `<PageEditToggle />` 按钮（用户截图最关心的页面）
  - 11 个其他页面走集中式编辑（管理页 → 页面布局管理）

### P2 - 添加浏览器 tab favicon
- **根因**：`index.html` 无 favicon，浏览器 tab 显示默认空图标。
- **修复**：`frontend/public/favicon.ico` 复制自 `icon.ico`，`index.html` 加 `<link rel="icon">`。
- **附带**：ISS 路径改为引用 v0.4.47 自己的 `icon.ico`。

## 改动文件

| 文件 | 改动 |
|------|------|
| `src/api/settings_handler.rs` | UTF-8 安全切片 |
| `frontend/src/components/Layout.tsx` | 全局包 PageEditProvider |
| `frontend/src/pages/SampleEntryPage.tsx` | 补 lab_name 分支 + 9 个 cell 改 width: 100% + 表头列宽按比例 + 加 PageEditToggle |
| `frontend/index.html` | 加 favicon link |
| `frontend/public/favicon.ico` | 新增（372KB） |
| `build_installer.iss` | icon path 改 v0.4.47 |

## 打包说明
- ISS 输出：`样品管理系统_v0.4.47_Setup.exe`
- 支持覆盖安装（AppId 固定）
- 无 cmd 窗口（release 模式）

## 验证
- ✅ `tsc --noEmit` 0 errors
- ✅ `vite build` 2.96s
- ✅ `cargo build --release` 1m48s
- ✅ curl 实测 PUT 9 字段（中文 label）→ 200 OK code:0
- ✅ Inno Setup 打包成功
