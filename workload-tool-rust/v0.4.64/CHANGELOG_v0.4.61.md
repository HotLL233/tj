# v0.4.61 版本更新说明

## 更新日期
2026-07-14

## 更新内容

### 修复：新建检测类型按钮无反应（P0）
- **根因**：`frontend/src/pages/ManagePage.tsx` 第 1227 行条件 `siTypeEdit !== null || siTypeForm.type_key !== '' || siTypeForm.label !== ''`，点击"新建类型"后三个条件全部为 false（siTypeEdit=null, type_key='', label=''）→ 表单块渲染 null → 用户看到"无反应"
- **修复**：增加独立 `siTypeNewOpen` 布尔状态追踪新建表单是否打开，条件改为 `siTypeEdit !== null || siTypeNewOpen`
- **MD5**：`747f6133…`（v0.4.60）→ `88c4c611…`（v0.4.61）

### 修复：提交登记后旧附件残留
- **根因**：`frontend/src/pages/SampleInfoEntry.tsx` doSubmit 成功后重置了表单行但没清空 `pendingFilesRef.current`，下次选文件时旧文件仍在 ref 里被一起上传
- **修复**：第 327 行后加 `pendingFilesRef.current = new Map()`
- **MD5**：`da787a3c…`（v0.4.60）→ `55d0f48f…`（v0.4.61）

### 修复：PDF 上传提示错误
- **根因**：`src/api/sample_info_attachment_handler.rs` 第 83-88 行的 MIME 类型白名单检查在浏览器发送非标准 MIME（如 `text/plain`）时拒绝合法文件
- **修复**：完全删除 MIME 类型验证，只保留文件扩展名（`.pdf`/`.doc`/`.docx`）验证
- **MD5**：`9d9280b8…`（v0.4.60）→ `3e09e713…`（v0.4.61）

## 改动文件清单

| 文件 | 改动 | MD5 变化 |
|---|---|---|
| `frontend/src/pages/ManagePage.tsx` | 加 `siTypeNewOpen` 状态 + 条件/按钮/save 适配（~6 行） | `747f6133…` → `88c4c611…` |
| `frontend/src/pages/SampleInfoEntry.tsx` | doSubmit 成功后 `pendingFilesRef.current = new Map()`（1 行） | `da787a3c…` → `55d0f48f…` |
| `src/api/sample_info_attachment_handler.rs` | 删除 MIME 类型检查块（约 -10 行） | `9d9280b8…` → `3e09e713…` |
| `Cargo.toml` / `frontend/package.json` / `build_installer.iss` | 版本号 0.4.60 → 0.4.61 | — |

**未改动文件**：所有其他源文件与 v0.4.60 完全一致。

## 打包说明
- 无 cmd 窗口（生产模式 `cargo build --release`）
- 支持覆盖安装（AppId 固定）
- 安装包：`D:\桌面\工作量统计工具项目\installer\样品管理系统_v0.4.61_Setup.exe`
- Docker：`ghcr.io/hotll233/tj/workload-tool:v0.4.61`
