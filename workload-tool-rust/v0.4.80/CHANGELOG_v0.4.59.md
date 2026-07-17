# v0.4.59 版本更新说明

## 更新日期
2026-07-14

## 更新内容

### 修复：样品信息登记多文件上传只能上传一个
- **根因**：`pendingFiles` 存储在 React state（rows._pendingFiles），由于 React 闭包陈旧问题，提交时读取的文件列表不是最新状态，导致只有最后一个文件被上传
- **修复**：改用 `useRef<Map<number, File[]>>` 存储待上传文件，`useRef` 的 `.current` 始终返回最新值，彻底消除闭包陈旧
  - `onChange`：写入 `pendingFilesRef.current` + `setRows` 触发重渲染
  - 显示：从 `pendingFilesRef.current.get(idx)` 读取
  - 提交：从 `pendingFilesRef.current.get(i)` 读取
- **改动文件**：仅 `frontend/src/pages/SampleInfoEntry.tsx`

## 打包说明
- 无 cmd 窗口（生产模式 `cargo build --release`）
- 支持覆盖安装（AppId 固定）
- 同步更新 Docker 版
