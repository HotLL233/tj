# 本地化 LIMS（路线 C：混合本地 LIMS）部署说明

> 适用版本：**v0.4.21**（基于既有 Rust + Axum 工作量统计工具扩展）
> 适用范围：科研实验室本地/内网单机或局域网部署，包含仪器、库存、采购、审批、通知、审计与 RBAC 鉴权。

---

## 1. 架构总览

- **后端**：Rust + Axum（SQLite + r2d2 连接池 + WAL）。源码位于 `workload-tool-rust/v0.4.21/`，编译产物为 `workload-tool.exe`（Windows）/ `workload-tool`（Linux）。
- **前端**：React 18 + Vite + MUI + Tailwind，源码位于 `project-root/frontend/`，构建产物位于 `project-root/frontend/dist/`。
- **托管方式**：后端以 SPA 方式同时托管 API（`/api/*`）与前端静态资源（`static/index.html`、`static/assets`），统一端口访问，无需独立前端服务器。
- **鉴权**：JWT（HS256，12 小时有效期）+ RBAC。密钥首启自动生成于 `data/.jwt_secret`（Unix 下权限 600），可被 `config.toml` 的 `jwt_secret` 覆盖。
- **审计**：WORM（一次写多次读）设计，历史表仅允许 INSERT，UPDATE/DELETE 由触发器 `RAISE(ABORT)` 拒绝。

---

## 2. 环境要求

| 组件 | 版本要求 | 说明 |
| --- | --- | --- |
| Rust 工具链 | 1.80+（`cargo`/`rustc`） | 用于编译后端 |
| Node.js | 18+（`npm`） | 用于构建前端 |
| 操作系统 | Windows 10+ / Linux（x64） | SQLite 为本地文件，无需额外数据库服务 |
| 内存 | ≥ 2 GB | 常规实验室规模足够 |
| 端口 | 默认 `8000`（可配置） | 需允许局域网访问时开放防火墙 |

---

## 3. 从源码构建

### 3.1 后端

```bash
cd workload-tool-rust/v0.4.21
cargo build --release
# 产物：target/release/workload-tool(.exe)
```

### 3.2 前端

```bash
cd project-root/frontend
npm install
npm run build
# 产物：project-root/frontend/dist/{index.html, assets/...}
```

### 3.3 目录组装（部署包）

将以下内容放在一起即可运行：

```
<部署目录>/
├── workload-tool(.exe)        # 后端可执行文件
├── static/                     # 后端静态资源 + 前端产物
│   ├── index.html              # 来自 frontend/dist/index.html
│   └── assets/                 # 来自 frontend/dist/assets/
├── config.toml                 # 可选，参数配置
└── data/                       # 运行时自动生成（数据库、密钥、审计、二维码）
```

> 注意：`static/` 目录需要同时包含后端既有静态资源与前端构建产物。若使用 Inno Setup 安装包（见第 7 节），打包脚本已自动完成此合并。

---

## 4. 配置（config.toml）

配置文件位于可执行文件同级目录。所有字段均可省略，使用默认值。

```toml
# 服务端口（默认 8000）
server_port = 8000

# 数据库目录（相对 exe 同级，或绝对路径；支持环境变量 WORKLOAD_DATA_DIR 覆盖）
db_dir = "data"

# 日志级别：trace/debug/info/warn/error
log_level = "info"

# JWT 签名密钥（可选）。留空则首启自动生成于 data/.jwt_secret。
# 重置该值会使所有已签发 Token 失效，已登录用户需重新登录。
jwt_secret = ""

# 自动备份
backup_enabled = false
backup_interval_hours = 24
max_backup_count = 10
```

> 说明：旧版本的 `admin_user` / `admin_pass` 明文字段已废弃，仅保留以兼容旧配置，不再作为鉴权依据。
> 首次启动管理员账号为 `admin` / `admin123`，**首次登录会被强制要求修改密码**。

---

## 5. 首次启动

1. 首次运行可执行文件，自动完成以下初始化：
   - 生成 `data/.jwt_secret`（随机 UUID，作为 JWT 签名密钥）。
   - 执行数据库迁移，创建全部业务表与审计 WORM 触发器。
   - 种子数据：6 个内置角色（系统管理员 / 主管 / 库管员 / 采购员 / 实验员 / 仪器管理员）及其默认权限。
   - 创建管理员账号 `admin`（密码 `admin123`，`must_change_password = 1`）。
2. 用浏览器访问 `http://<服务器IP>:8000/`，使用 `admin / admin123` 登录。
3. 系统将强制跳转到「修改密码」流程，完成后即可正常使用。
4. （推荐）在「系统管理 → 角色管理 / 用户管理」中按需创建业务账号并分配角色。

---

## 6. 日常运行与访问

- 直接双击运行 `workload-tool(.exe)`（托盘程序），或命令行 `./workload-tool` 前台运行。
- 访问地址：`http://localhost:8000/`（本机）或 `http://<局域网IP>:8000/`（同网段其他终端）。
- 前端为单页应用：除 `/login` 与 `/api/*` 外，所有路由由前端路由接管；未登录访问受保护页面会自动跳转登录页。

### 角色与权限速查

| 角色 | 主要权限 |
| --- | --- |
| 系统管理员 | 全部（`*` 通配） |
| 主管 | 库存/仪器/采购查看、审批决策、运营统计、审计、通知查看 |
| 库管员 | 库存读写、仪器查看、采购/审批/通知查看 |
| 采购员 | 采购读写与审批、供应商读写、库存查看、审批/通知查看 |
| 实验员 | 库存/仪器查看与预约、采购申请、审批/通知查看 |
| 仪器管理员 | 仪器读写与预约审批、库存/审批/通知查看 |

权限点常量集中定义于后端 `src/models/role.rs` 的 `PERMISSIONS`，前端镜像于 `src/constants/permissions.ts`。

---

## 7. 制作安装包（Inno Setup）

仓库根目录提供 `build_installer.iss`（已更新至 v0.4.21）：

1. 先完成第 3 节的后端与前端构建。
2. 用 Inno Setup Compiler 打开 `build_installer.iss`，编译生成 `installer/工作量统计工具_Rust_v0.4.21_Setup.exe`。
3. 安装包已包含：后端 exe、后端 `static/`、以及 `project-root/frontend/dist/*`（合并进 `static/`），安装后即可双击运行。

> 安装脚本中的源路径为绝对路径，CI/打包机需保持目录结构与仓库一致，或相应调整路径。

---

## 8. 升级与数据迁移

- **小版本升级（v0.4.x → v0.4.21）**：直接覆盖 `workload-tool(.exe)` 与 `static/` 即可。迁移脚本（`db/migrations.rs`）为增量式，启动时会自动补齐新增表/列/触发器，**已有 `data/workload.db` 与审计数据不会丢失**。
- **JWT 密钥**：升级不会重置 `.jwt_secret`，已登录会话保持有效。
- **回滚**：保留旧版 exe 与 `data/` 备份即可回退；由于迁移为单向增量，回滚到旧版前建议先备份数据库。

---

## 9. 常见问题（FAQ）

| 现象 | 排查 |
| --- | --- |
| 启动后无法访问页面 | 确认端口未被占用；局域网访问需放行防火墙 8000 端口；检查 `data/` 目录有写入权限 |
| 登录提示 401 | Token 失效或密钥被重置；清除浏览器 `localStorage` 的 `limsc_token` 重新登录 |
| 强制改密无法跳过 | 属预期安全行为；管理员 `admin` 首登必改密 |
| 审计记录无法修改/删除 | WORM 设计，历史表禁止 UPDATE/DELETE，属正常保护 |
| 二维码不显示 | 确认 `data/qr/` 目录可写；仪器二维码在「仪器」页点击生成后展示 |
| 审批无对应规则时 | 默认自动通过，并生成通知与审计记录（策略：无规则即放行） |

---

## 10. 目录与文件索引

| 路径 | 说明 |
| --- | --- |
| `workload-tool-rust/v0.4.21/src/` | 后端源码（鉴权、RBAC、仪器、库存、采购、审批、通知、审计、二维码） |
| `workload-tool-rust/v0.4.21/build_installer.iss` | 版本内安装脚本 |
| `build_installer.iss` | 根目录安装脚本（v0.4.21，含前端） |
| `project-root/frontend/src/` | 前端源码（鉴权上下文、布局导航、各业务页面） |
| `docs/本地化LIMS_PRD.md` | 产品需求文档 |
| `docs/本地化LIMS_架构设计.md` | 架构设计文档 |
