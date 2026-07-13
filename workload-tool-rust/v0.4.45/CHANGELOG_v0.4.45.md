# v0.4.45 版本更新说明

## 更新日期
2026-07-13

## 更新内容
### 修复：admin 用户 token 鉴权失败（P0）
- **根因**：`/api/auth/login` 对 admin 返回 `tok_admin_<ts>` 伪 token（非 JWT），所有需要 `extract_claims_from_headers` 鉴权的 PUT/POST/DELETE 操作均因 `verify_token` 解码失败返回「无效的登录凭证」
- **修复**：admin 登录改为走 `auth_service::generate_token` 生成真实 JWT，与普通用户统一鉴权
- **改动文件**：
  - `src/api/auth_handler.rs` — `login` 函数生成真实 JWT，AuthState 增加 pool
  - `src/api/mod.rs` — `auth_handler::router(config)` → `router(config, pool)`

## 打包说明
- 无 cmd 窗口（生产模式 `cargo build --release`）
- 支持覆盖安装（AppId 固定）
