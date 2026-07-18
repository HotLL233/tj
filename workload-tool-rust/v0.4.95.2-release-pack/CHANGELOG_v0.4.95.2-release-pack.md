# v0.4.95.2 正式覆盖安装版

## 打包修复

- 按 v0.4.94 及以前的正式流程重新编译：使用 `cargo build --release`，未启用 `console` 特性。
- Windows PE 子系统已验证为 `2`（Windows GUI），不会弹出控制台窗口。
- 保持旧版 `AppId`、应用名“样品管理系统”和默认安装目录 `{autopf}\\样品管理系统`。
- 使用 `UsePreviousAppDir=yes`，覆盖安装时优先复用旧安装目录。
- 保留 `{app}\\data`，升级不会删除数据库和用户数据。
- Inno Setup 负责将 release exe、`backend\\static` 和图标打入安装包。

## 验证结果

- `npm ci`：通过。
- `npm run build`：通过。
- `cargo build --release`：通过。
- `cargo test`：17 个测试全部通过。
- release exe 启动验证：健康接口返回 `ok`，首页返回 HTTP 200。
- PE 子系统验证：`2 = Windows GUI`，无控制台。
- Inno Setup 6.7.2：安装包编译通过。

## 安装包

`installer/样品管理系统_v0.4.95.2_正式覆盖安装版_Setup.exe`
