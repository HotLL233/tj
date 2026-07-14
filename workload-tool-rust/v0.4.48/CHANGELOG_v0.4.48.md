# v0.4.48 版本更新说明

## 更新日期
2026-07-13

## 修复内容

### P0 - 修复保存字段布局后端 UTF-8 panic
- **根因**：`src/api/settings_handler.rs` 写 audit log 时 `&value_str[..200]` 字节切片，当 200 落在中文字符（如"取"）的 UTF-8 多字节边界内部时 panic，HTTP 连接断开，浏览器看到"网络错误"。
- **修复**：改用 `char_indices()` 寻找最近的字符边界，安全切片。

### P0 - 修复样品信息录入实验室列不显示
- **根因**：`SampleEntryPage` 录入表格的 9 个字段分支中，缺少 `lab_name` 渲染分支。
- **修复**：在 `user_name` 分支之前补 `lab_name` 分支（整页共用一个 lab，从 URL `group_id` 读取，禁用编辑，灰色背景）。

### P1 - 录入表格列宽弹性化
- 表头 `width: ${(field.width || 100) / 10}%` 按比例分配
- 表体 input `width: '100%'` 撑满 cell

### P2 - 浏览器 tab favicon
- `frontend/public/favicon.ico`（372KB） + `index.html` 加 `<link rel="icon">`
- ISS 路径改引用 v0.4.48 的 `icon.ico`

## v0.4.48 回退（恢复 v0.4.44 设计）
- ❌ **移除** `Layout` 全局 `<PageEditProvider>` 包裹（v0.4.47 错误添加）
- ❌ **移除** `SampleEntryPage` 的 `<PageEditToggle />` 按钮（v0.4.47 错误添加）
- ✅ **保留** 编辑功能只在管理页「页面布局管理」tab（v0.4.44 设计）

## 改动文件

| 文件 | 改动 |
|------|------|
| `src/api/settings_handler.rs` | UTF-8 安全切片（1 处） |
| `frontend/src/pages/SampleEntryPage.tsx` | 补 lab_name 分支 + 9 个 cell 改 width: 100% 弹性 + 表头按比例 |
| `frontend/index.html` | 加 favicon link |
| `frontend/public/favicon.ico` | 新增（372KB） |
| `build_installer.iss` | icon path 改 v0.4.48 |

## 版本隔离清理
v0.4.48 是从 v0.4.47 复制后：
- 删除了 v0.4.47 引入的旧版本文件混入（CHANGELOG v0.4.22-v0.4.47、design docs）
- 删除了 v0.4.47 错误添加的外部编辑入口（恢复 v0.4.44 设计）
- 删除了构建产物（dist/static/backend/）让 build 重新生成

## 验证
- ✅ `tsc --noEmit` 0 errors
- ✅ `vite build` 2.75s
- ✅ `cargo build --release` 1m27s
- ✅ Inno Setup 打包成功（6.6MB）
- ✅ ISS 路径正确（v0.4.48/icon.ico + v0.4.48/dist/workload-tool.exe + v0.4.48/backend/static/）

## 推送
- ✅ **workload-tool** 仓库（v0.4.48 推送）
- ❌ **不再推 tj 仓库**（tj 是个人工作仓库，不应推工具代码）
