# v0.3.28 更新说明

## 发布日期
2026-07-06

## 功能增强

### 审计日志详细化

所有操作日志的 `detail` 字段从简单描述改为包含完整操作上下文，方便后期溯源和异常分析。

**work_records（工作量记录）操作详情格式：**
- 创建：`在 {实验室} 录入项目「{项目名}」/ 方法「{方法名}」，数量 {数量}`
- 修改：`修改记录#{id}：项目「{项目名}」`
- 删除：`删除记录#{id}：项目「{项目名}」/ 方法「{方法名}」，数量 {数量}`
- 恢复：`恢复记录#{id}：项目「{项目名}」/ 方法「{方法名}」，数量 {数量}`

**其他模块操作详情格式：**
- 项目：`创建/编辑/删除项目「{项目名}」`
- 实验室：`创建/编辑/删除实验室「{实验室名}」`
- 方法：`创建/编辑/删除方法「{方法名}」`
- 方法类型：`创建/编辑/删除方法类型「{类型名}」`
- 导出：`导出Excel，时间范围 {start} ~ {end}`

**前端：** 审计日志表格新增"详情"列，显示操作上下文信息。

---

## 修改文件清单

| 文件 | 改动 |
|------|------|
| `src/repo/record_repo.rs` | create/update/delete/restore 的 audit detail 加入 实验室/项目/方法/数量 |
| `src/repo/project_repo.rs` | 项目 CRUD audit detail 加入项目名 |
| `src/repo/group_repo.rs` | 实验室 CRUD audit detail 加入实验室名 |
| `src/repo/method_repo.rs` | 方法/方法类型 CRUD audit detail 加入名称 |
| `src/api/export_handler.rs` | 导出 audit detail 加入时间范围 |
| `frontend/src/pages/ManagePage.tsx` | 审计日志表格新增"详情"列 |
| `Cargo.toml` | 版本号 → `0.3.28` |
| `build_installer.iss` | 版本号同步 |

---

## 构建信息

- **Rust 编译**：✅ 成功
- **前端构建**：✅ 成功
- **安装包**：`installer/工作量统计工具_Rust_v0.3.28_Setup.exe`
- **覆盖安装**：✅ 支持

---
**版本隔离说明**：5个后端文件 + 1个前端文件，无表结构变更。
