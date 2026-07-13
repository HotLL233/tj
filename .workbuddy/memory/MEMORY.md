# 工作量统计工具项目 — 长期记忆

## 项目概况
- Rust (Axum 0.7.9 + rusqlite 0.31 + r2d2) 后端 + React 18 + TypeScript 前端
- 前端打包为静态文件由 Rust 后端 serve，SPA 模式
- 端口 8000，系统托盘运行（生产模式隐藏控制台）
- 数据库：SQLite，10+ 张表（project_groups, projects, work_records, rd_work_records(新增 v0.4.0), audit_log(新增 module 列 v0.4.0), methods, project_lab_links, project_method_links, method_type_links, import_mappings, sample_records(已退役保留)）

## ⚠️ 版本隔离规范（CRITICAL — 用户铁律）
- **核心原则**：每个版本 = 独立文件夹，**绝不原地修改已有版本文件夹的源码**。
- 用户项目目录里 `v0.1.0` ~ `v0.4.x` 每个版本都是独立文件夹，这就是"版本隔离"的真实含义。
- **新版本标准流程**：复制上一版本文件夹 → 新版本号文件夹 → **只在新的里面改** → 旧文件夹原封不动。
- ❌ 错误理解（2026-07-09 翻车）："文件夹名不变、只在里面把 version 字段从 0.4.21 改成 0.4.22" —— 这仍是原地改源码，违反版本隔离，用户明确反对。
- 当前文件夹分工：
  - `v0.4.21` = 纯净 v0.4.21 基线（git HEAD = `4b815a0`，跨平台 Linux 支持 + Docker 部署；**不含**样品信息模块）。曾被 v0.4.22 原地污染，已 `git reset --hard 4b815a0` 还原，今后**绝不碰**。
  - `v0.4.22` = 含样品信息登记模块的版本（从旧 v0.4.21 文件夹提取出的独立副本，version 0.4.22）。
  - `v0.4.23` = 当前发布版（从 v0.4.22 复制，增强：检测时间自动生成/管理页三块/门户卡片自定义/独立导出7Sheet，version 0.4.23，已打包+tag+推送+触发Docker构建）。

## 关键技术决策
- **axum 0.7 路由语法**：使用 `:id`（不是 `{id}`，后者是 axum 0.8 语法）
- **v0.3.0 导入系统**：列头路由由 import_mappings 表驱动（wildcard 匹配），替代硬编码 if-else
- **路由架构**：使用 `.merge()` + 完整路径前缀（不用 `.nest()`，避免嵌套路由参数匹配问题）
- **错误处理**：统一返回 HTTP 200 + 业务错误码（与 Python 版一致）
- **事务控制**：所有写操作用 `conn.transaction()` 包裹，审计日志用 `log_on_conn(&tx, ...)`
- **SPA fallback**：`.fallback(serve_index)` 服务 `static/index.html`，仅对非 API 路径生效
- **审计 module 隔离（v0.4.0）**：`audit_log` 加 `module` 列（work/rd/shared，默认 shared）；**新增** `log_with_module`/`log_on_conn_with_module`（不改旧 `log`/`log_on_conn` 签名）；`list` 按 module 过滤（work 见 work+shared，rd 见 rd+shared）
- **模块克隆模式（v0.4.0）**：研发送样完整克隆分析检测 — `cp` + `sed` 复制 record handler/service/repo + stats + export_data/handler/preview_handler，做 `work_records→rd_work_records`、路由 `/api/records→/api/rd-records` 等替换；主数据共用、数据独立
- **导出 person_label 参数化（v0.4.1）**：`export_write::write_sheet5/6` 加 `person_label: &str` 参数（work/rd 共用同一函数），分析检测传"检测人"、研发送样传"送样人"，避免克隆第二套写模块
- **实验室关联项目名（v0.4.1）**：`group_repo::list/get_by_id` 用关联子查询 `(SELECT GROUP_CONCAT(p.name) FROM project_lab_links pll JOIN projects p ON p.id=pll.project_id WHERE pll.group_id=g.id) AS project_names` 返回 `GroupResponse.project_names: Option<String>`；前端实验室一览展示

## 版本历史
- v1.7.0/v1.7.1：Python (FastAPI) 版，PyInstaller + Inno Setup 打包
- v0.4.10：实验室模块关联（show_in_work/show_in_rd）+ 方法一览类型列自适应换行 + Sheet 11 类型汇总（v0.4.8→v0.4.10，详见日志 2026-07-08）
- v0.4.22：样品信息登记模块（门户第三卡片 + 记录列表 + 四状态流转）；**v0.4.22 起改为每版本独立文件夹隔离**
- v0.4.23：样品信息登记增强（检测时间自动生成/管理页三块/门户卡片自定义/独立导出7Sheet），已实现+QA通过+已发布（installer EXE + Docker镜像 ghcr.io/hotll233/workload-tool:v0.4.23）
- v0.4.24：事业部层级（divisions表/实验室Chip筛选/管理页CRUD/双线同步/级联置空删除），已发布
- v0.4.25：样品登记改Excel模板式多行表格录入 + 事业部→部门文案统一 + Chip去× + 实验室一览加关联部门列 + 记录查询加统计卡片 + 统计页标题更名，已发布
- v0.4.9：修复倍率不能设 0（||→??）+ Sheet 11 类型汇总
- v0.4.8：修复导出 Group By 倍率分组 + Sheet8 项目汇总公式被覆写
- v0.4.7：修正导出 SQL 使用记录级单价倍率（COALESCE(wr.multiplier...)）+ 倍率→单价倍率改名 + Sheet 7/8 GROUP BY wr.multiplier
- v0.4.6：新增单价倍率功能（methods + work_records）、统计页筛选 stale closure 修复、前端工具标签颜色 #9c27b0→#00897b
- v0.4.5：修复研发送样跨实验室重复计数 + 系数标签→单价标签 + 取样人行内编辑
- v0.4.4：新增「研发送样记录」独立卡片+全量记录页；实验室卡片右上角待取样角标；送样时间改名；手机端布局对齐
- v0.4.3：研发送样记录新增状态列(待取样/已取样)+取样人列(弹窗输入+自动时间)+实验室卡片送样计数
- v0.4.2：修复新建项目 FK 错误 + 全模块「人员」→检测人/送样人 + 卡片颜色按模块统一(蓝/橙) + 修复研发送样返回标题回退
- v0.4.1：列头改名(检测人/送样人)+模块背景色(蓝/橙)区分+删关联配置卡片+备份页中文说明+项目/实验室/方法一览优化+数据状态表中文标签
- v0.4.0：研发送样模块完整克隆分析检测 — 独立 `rd_work_records` 存储、统计页 10 Sheet 预览克隆、审计 `module` 隔离（work/rd/shared）、退役旧 sample 送样模块
- v0.3.0：导入系统灵活化 — import_mappings 表驱动列头路由，项目-实验室 M:N 自动关联，方法级联选择
- v0.2.19：项目编辑对话框恢复字段、方法编辑增加金额+类型复选、方法类型筛选、实验室删除Bug修复
- v0.2.18：卡片独立分离版本 — methods 表独立，三卡片各自管理
- v0.2.17：卡片独立分离 — methods 表独立，三卡片各自管理（项目/方法/实验室），3张关联表
- v0.2.16：卡片合并（4合1检测方法管理）+ 项目编辑关联实验室/检测方法多选
- v0.2.15：方法导入从行模式重构为列模式（每列独立，列头=分组，含"方法"→检测方法管理卡片）

## 已知问题与修复记录
- 2026-06-24：修复路由 405/HTML bug（根因：`{id}` 语法错误 → `:id`）
- 2026-06-24：修复 record_repo.rs 事务原子性（跨连接审计 → 同连接事务）
- 2026-06-24：修复 serve_index 编译错误（tuple 顺序 `(StatusCode, String)` 非 `(String, StatusCode)`）
- 2026-07-08：v0.4.8 — 修复 Sheet 3/4 GROUP BY 缺少 wr.multiplier 导致不同倍率记录塌缩 + Sheet 8 项目汇总公式被 merge_range 覆写
- 2026-07-01：v0.2.19 — methods 表增加 amount 字段；项目编辑恢复 full_name/sort_order/is_active；方法卡片增加类型筛选；实验室删除改用 project_lab_links

## 方法表结构（v0.2.19）
- id, name, full_name, coefficient, amount(新增), notes, is_active, created_at
- 关联表: method_type_links(method_id → method_types.id)
- **⚠️ name vs full_name 区别（CRITICAL）**:
  - `name` = 原始方法名，含 `@[仪器]` 格式（如 `HYLY-230106-1-DAD@[LC-01]`）— **提取仪器必须用这个字段**
  - `full_name` = 导入时自动生成（如 `410实验室/A001`），**不含 `@[]` 格式**
  - 统计查询中 `extract_instrument()` 必须传 `m.name` 而非 `m.full_name`，否则返回"未知"

## 项目管理表结构（v0.2.19）
- projects: id, name, full_name, notes, sort_order, is_active, coefficient(保留不用)
- 关联表: project_lab_links, project_method_links

## 文件结构
```
workload-tool-rust/
  src/
    main.rs          — 入口，路由组装，SPA fallback
    api/
      mod.rs         — API 路由汇总（.merge()）
      record_handler.rs  — 记录 CRUD + restore + by-user
      rd_record_handler.rs — 研发送样记录 CRUD（v0.4.0，克隆 record_handler）
      stats_handler.rs   — 统计（summary/by-user/by-project/by-type/by-instrument）
      rd_stats_handler.rs — 研发送样统计（v0.4.0）
      export_handler.rs  — Excel 导出
      rd_export_handler.rs / rd_export_data.rs / rd_export_preview_handler.rs — 研发送样导出（v0.4.0）
      audit_handler.rs   — 审计日志（支持 module 过滤）
    repo/
      record_repo.rs — 记录 DAO（事务原子性已修复）
      rd_record_repo.rs — 研发送样 DAO（v0.4.0，写 rd_work_records，审计 module=rd）
      group_repo.rs  — 分组 DAO（v0.2.19 修复 delete 检查 project_lab_links）
      project_repo.rs— 项目 DAO（v0.2.19 增加 full_name/sort_order/is_active）
      method_repo.rs — 方法 DAO（v0.2.19 增加 amount）
      audit_repo.rs  — 审计 DAO（log/log_on_conn + log_with_module/log_on_conn_with_module）
    models/          — 数据模型
    error.rs         — 统一错误处理
    db/              — 数据库初始化 + 迁移 + 种子数据
    tray.rs          — 系统托盘
project-root/
  frontend/src/api/client.ts — 前端 API 客户端（baseURL: /api）

## 打包工具路径
- **Inno Setup 6.7.2**: `D:/APP/Inno Setup 6/ISCC.exe`
- 编译命令: `"D:/APP/Inno Setup 6/ISCC.exe" build_installer.iss`

## ⚠️ 打包流程（CRITICAL — 必须严格遵守）
1. `cargo build --release` → 输出到 `target/release/workload-tool.exe`
2. **必须复制**: `cp target/release/workload-tool.exe dist/workload-tool.exe`（ISS 打包的是 dist/ 目录！）
3. 前端: `npm run build` → 输出到 `v0.4.x/backend/static/`
4. ISCC 打包: 引用 `dist/workload-tool.exe` 和 `static/*`
- **忘记 cp exe 到 dist/** → 安装包永远是旧版本！（2026-07-04 血的教训）
- **ISS 静态文件打包路径（CRITICAL — 2026-07-10 血的教训）**：
  - `vite.config.ts` 的 `build.outDir: '../backend/static'` → `npm run build` 输出到 `backend/static/`
  - 但 `build_installer.iss` 的 `Source` 路径写的是 `static\*`（根目录），不是 `backend\static\*`
  - 后果：安装包里打包的是根目录 `static/` 下的旧版静态文件，exe 是新版但页面是旧的
  - **修复**：ISS 的静态文件 Source 必须从 `static\*` 改为 `backend\static\*`（与 Dockerfile 的 COPY 路径对齐）
  - Dockerfile 的 `COPY --from=builder /app/backend/static/ /app/static/` 一直正确，Docker 版不受影响
  - 每次打 Windows 安装包前确认 ISS 的 Source 路径指向正确的构建输出目录
- **MyAppName 不要含版本号**（2026-07-09 血的教训）：`#define MyAppName "工作量统计工具 (Rust)"`（不用加 v0.4.x），否则 `DefaultDirName={autopf}\{#MyAppName}` 每版不同目录，无法覆盖安装

## 操作偏好
- **前台执行**：所有编译、打包、构建操作必须在**前台执行**（不带 run_in_background），确保用户能看到实时进度
- **分批输出**：长时间操作（如 cargo build）可接受分批 wait，但绝不能后台隐藏进度
- **只做被要求的活**：严格按用户字面指令执行，不自行添加额外功能/优化/猜测意图。用户没说的不做
