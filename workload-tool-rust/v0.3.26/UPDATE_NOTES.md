# v0.3.26 更新说明

## 发布日期
2026-07-06

## 问题修复

### 修复1：回收站恢复功能无效

**现象**：回收站中点击"恢复"按钮，前端没有任何反应，记录无法恢复。

**根因**：前端 `client.ts` 中 `restoreRecord` 函数 URL 错误：
- 错误：`POST /api/records/${id}/restore`
- 正确：`POST /api/records/restore/${id}`（后端路由注册顺序：restore 在 `:id` 之前）
- 因为 RESTful 风格路由 `/:id` 会先匹配 `/123/restore`，把 `restore` 当成 ID 参数传入

**修复**：
- `project-root/frontend/src/api/client.ts`：URL 从 `/records/${id}/restore` 改为 `/records/restore/${id}`
- 返回类型从 `ApiResponse<null>` 改为 `ApiResponse<WorkRecord>`

---

### 修复2：审计日志缺失部分操作记录

**现象**：审计日志页面显示"共 0 条操作记录"，无法追踪操作历史。

**诊断**：
- 审计日志基础设施正常（`audit_repo.rs`、数据库表、查询接口）
- 大部分操作已有记录：record CRUD ✅、project CRUD ✅、group CRUD ✅、method CRUD ✅
- **缺失记录的操作**：
  1. **方法类型（MethodType）CRUD**：创建/修改/删除方法类型未记录
  2. **数据导出**：导出 Excel 未记录

**修复**：
1. `src/repo/method_repo.rs`：为 `create_method_type`、`update_method_type`、`delete_method_type` 添加审计日志
2. `src/api/export_handler.rs`：导出成功后记录审计日志（使用 `.ok()` 静默失败，不影响导出功能）
3. `src/repo/project_repo.rs`：移除 `create()` 中的重复审计日志（原有两处记录同一操作）

---

## 修改文件清单

| 文件 | 修改内容 |
|------|----------|
| `project-root/frontend/src/api/client.ts` | 修复 `restoreRecord` URL 和返回类型 |
| `src/api/export_handler.rs` | 新增导出审计日志 |
| `src/repo/method_repo.rs` | 新增方法类型 CRUD 审计日志 |
| `src/repo/project_repo.rs` | 移除重复审计日志 |
| `Cargo.toml` | 版本号 → `0.3.26` |
| `build_installer.iss` | 版本号同步更新 |

---

## 构建信息

- **Rust 编译**：✅ 成功（6个 warning，无 error）
- **前端构建**：✅ 成功（3.18s）
- **安装包**：`installer/工作量统计工具_Rust_v0.3.26_Setup.exe`
- **覆盖安装**：✅ 支持（AppId 保持不变）

---

## 升级建议

1. 直接安装 `工作量统计工具_Rust_v0.3.26_Setup.exe`
2. 覆盖安装（v0.3.25 或更高版本），数据和配置自动保留
3. 安装后操作回收站恢复功能和审计日志页面，确认修复生效

---
**版本隔离说明**：本版本为独立版本目录 `v0.3.26`，不修改旧版本代码。
