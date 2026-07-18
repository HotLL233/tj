# v0.4.95.1 安装覆盖修复说明

## 修复内容

- 恢复与 v0.4.94 及以前版本一致的应用名：`样品管理系统`。
- 恢复旧默认安装目录：`{autopf}\\样品管理系统`。
- 保持历史 `AppId` 不变：`A1B2C3D4-E5F6-7890-ABCD-EF1234567890`。
- 增加 `UsePreviousAppDir=yes`，优先复用旧安装目录。
- 移除 x64 专用安装目录设置，避免旧版位于 `Program Files (x86)`、新版位于 `Program Files`。
- 保留 `{app}\\data`，升级覆盖不会删除用户数据库。
- 保留静态资源清理，避免旧前端资源残留。

## 验证结果

- `npm ci`：通过。
- `npm run build`：通过。
- `cargo build --release --features console`：通过。
- `cargo test --features console`：17 个测试全部通过。
- Inno Setup 6.7.2：安装包编译通过。

## 安装包

`installer/样品管理系统_v0.4.95.1_安装覆盖修复版_Setup.exe`

如果当前电脑旧版本没有卸载注册表记录，首次仍需在安装向导中选择旧程序实际目录；选择一次后，后续版本会按同一安装身份自动覆盖。
