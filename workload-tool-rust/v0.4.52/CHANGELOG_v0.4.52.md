# v0.4.52 版本更新说明

## 更新日期
2026-07-13

## 修复
### P0 - 录入表格列宽溢出（TableLayout: fixed）
- **根因**：`minWidth: 1280` 强制表宽 > 容器宽度 + 字段 `minWidth: field.width || 80` 防压缩 → 最后列被切
- **修复**：`tableLayout: 'fixed'` + `width: '100%'`；移除字段 `minWidth`；复选框表头加显式 width

## 阶段C：安全修复

### P0 - JWT_SECRET 环境变量化
- 从硬编码 `const` 改为 `fn jwt_secret()` 运行时检查环境变量 `JWT_SECRET`
- 未设置时回退内置密钥（向后兼容）
- Docker 用户可设置：`docker run -e JWT_SECRET=mysecret ...`
- **影响文件**：`src/service/auth_service.rs`

### P0 - 修复 audit_repo SQL 注入
- `format!("WHERE module='{}'", m.replace('\'', "''"))` 改为白名单校验 + 参数化查询
- 非 whitelist（work/rd/shared）直接返回 `AppError::Validation`
- **影响文件**：`src/repo/audit_repo.rs`

### P0 - ADMIN_PASSWORD 环境变量支持
- `config.rs` 的 `load()` 新增优先读取环境变量 `ADMIN_PASSWORD`
- Docker/Linux 用户可直接注入密码，无需 config.toml
- **影响文件**：`src/config.rs`

### P1 - 清理嵌套目录 + .bak 文件
- 删除 `frontend/src/src/`（最多 4 层嵌套的源代码副本）
- 删除所有 `*.bak` 备份文件

## 改动文件

| 文件 | 改动 |
|------|------|
| `frontend/src/pages/SampleEntryPage.tsx` | 列宽修复（3 处） |
| `src/service/auth_service.rs` | JWT_SECRET 环境变量化 |
| `src/repo/audit_repo.rs` | SQL 注入白名单修复 |
| `src/config.rs` | ADMIN_PASSWORD 环境变量 |
| `frontend/src/src/` | **删除**（嵌套目录） |

## 验证
- ✅ `cargo check` 通过
- ✅ `tsc --noEmit` 0 errors
- ✅ `vite build` 2.94s
- ✅ `cargo build --release` 1m46s
- ✅ Inno Setup 打包成功
