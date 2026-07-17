# v0.4.60 版本更新说明

## 更新日期
2026-07-14

## 更新内容

### 修复：样品信息登记 — 附件删除按钮无效
- **根因**：`SampleInfoEntry.tsx` 第 411-415 行的删除按钮 onClick 错把文件数组写到了 React state (`updateRow(idx, '_pendingFiles', updated)`)，但 UI 渲染和提交上传都从 `pendingFilesRef.current` 读取。重新渲染时 ref 覆盖回去 → 删除按钮等于无效
- **修复**：onClick 改为直接修改 `pendingFilesRef.current.set(idx, arr)`，触发重渲染
- **改动文件**：`frontend/src/pages/SampleInfoEntry.tsx`（1 处，约 5 行）
- **MD5 验证**：`672e8f94…`（v0.4.59）→ `da787a3c…`（v0.4.60）

### 修复：样品信息登记 — 新建类型失败
- **根因**：`sample_info_type_handler::create` 第 56-57 行的 `init_for_type(&conn, &item.type_key)?` 在某些场景下失败时返回 Internal 错误，让整个 create 失败。但**类型本身已经在事务里成功创建**（commit 已发生）——前端看到"保存失败"但数据库里已有该类型
- **修复**：把 `init_for_type` 失败降级为 `tracing::error!` 警告日志，不让 create 失败。`list_active_by_type` 已有 LEFT JOIN + IS NULL 兜底（v0.4.56 修复），新类型即使没初始化 visibility 也能正常显示
- **改动文件**：`src/api/sample_info_type_handler.rs`（1 处，约 15 行）
- **MD5 验证**：`6c4c8e50…`（v0.4.59）→ `e9a26ad0…`（v0.4.60）

## 改动文件清单

| 文件 | 改动 | MD5（v0.4.59 → v0.4.60）|
|---|---|---|
| `frontend/src/pages/SampleInfoEntry.tsx` | 附件删除按钮 onClick | `672e8f94…` → `da787a3c…` |
| `src/api/sample_info_type_handler.rs` | create 端点降级 init_for_type 失败 | `6c4c8e50…` → `e9a26ad0…` |
| `Cargo.toml` | version 0.4.59 → 0.4.60 | — |
| `frontend/package.json` | version 0.4.59 → 0.4.60 | — |
| `build_installer.iss` | MyAppVersion / OutputBaseFilename / Source 路径 | — |

**未改动文件**：所有其他源文件（src/*、frontend/src/*）与 v0.4.59 完全一致。

## 打包说明
- 无 cmd 窗口（生产模式 `cargo build --release`）
- 支持覆盖安装（AppId 固定，MyAppName 不含版本号）
- 安装包：`D:\桌面\工作量统计工具项目\installer\样品管理系统_v0.4.60_Setup.exe` (6.6 MB)
- Docker 镜像：`ghcr.io/hotll233/workload-tool:v0.4.60`
