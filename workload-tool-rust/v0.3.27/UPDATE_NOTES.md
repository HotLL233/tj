# v0.3.27 更新说明

## 发布日期
2026-07-06

## 问题修复

### 修复1：审计日志页面显示"共 0 条操作记录"

**现象**：系统管理 → 审计日志页面一直显示"共 0 条操作记录"，但用户已进行大量操作。

**根因**：前端 API 路径与后端路由不匹配。
- 前端 `getAuditLogs()` 请求：`GET /api/audit`
- 后端注册的路由：`GET /api/audit-logs`
- 路径不匹配导致请求返回 404，前端 `try/catch` 吞掉错误

**修复**：
- `project-root/frontend/src/api/client.ts` 第171行：`/audit` → `/audit-logs`

---

### 修复2：回收站显示"共 0 条已删除记录"（表中有数据）

**现象**：回收站表格中有已删除记录，但页面上方统计显示"共 0 条已删除记录"。

**根因**：`ManagePage.tsx` 回收站显示用了错误的状态变量 `rc`。
- `rc` 实际存储的是审计日志总数（`lt()` 函数中用 `getAuditLogs` 覆盖）
- 回收站列表 `tr` 有数据，但没用到它的长度

**修复**：
- `project-root/frontend/src/pages/ManagePage.tsx` 第541行：`{rc}` → `{tr.length}`

---

## 修改文件清单

| 文件 | 改动量 | 说明 |
|------|--------|------|
| `frontend/src/api/client.ts` | 1 行 | `/audit` → `/audit-logs` |
| `frontend/src/pages/ManagePage.tsx` | 1 行 | `{rc}` → `{tr.length}` |
| `Cargo.toml` | 1 行 | `0.3.26` → `0.3.27` |
| `build_installer.iss` | 6 处 | 全局版本号同步 |

## 构建信息

- **Rust 编译**：✅ 成功
- **前端构建**：✅ 成功
- **安装包**：`installer/工作量统计工具_Rust_v0.3.27_Setup.exe`
- **覆盖安装**：✅ 支持

---
**版本隔离说明**：仅修改前端 2 行代码，后端完全未动。
