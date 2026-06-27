# 项目完整记忆导出
> 生成时间：2026-06-26 20:14 | 来源：`.codebuddy/memory/` 全部5个日志 + MEMORY.md
> 格式：时间线正序 + 长期记忆附录，适合外部工具一次性读取

---

## 一、长期策略（MEMORY.md）

### 开发策略 (2026-06-25 起生效)

1. **Python 版已冻结**：不再进行任何更新，仅保留 v0.2.5 作为历史版本。
2. **全部精力投入 Rust 版**：后续所有开发只针对 `workload-tool-rust/`。
3. **版本隔离原则**：每次代码变更前，将当前最新版本完整复制到新的版本号文件夹（如 `v0.1.3/` → `v0.1.4/`），在新文件夹内修改，**绝不覆盖原有代码**。
4. **文件夹命名**：以清晰版本号命名，如 `workload-tool-rust/v0.1.3/`、`v0.1.4/`。
5. **当前最新版本**：v0.1.4，路径 `workload-tool-rust/v0.1.4/`。

### 关键路径

| 用途 | 路径 |
|------|------|
| Python 最终版 | `project-root/` |
| Rust 版本基线 | `workload-tool-rust/v0.1.4/` |
| 安装包输出 | `installer/` |
| Inno Setup | `D:\APP\Inno Setup 6\ISCC.exe` |

### 打包流程

1. 在新版本文件夹内 `cargo build --release`
2. 复制 exe 到 dist 文件夹
3. 复制前端 static 文件
4. 更新 `build_installer.iss`（文件源路径、版本号、输出文件名）
5. `ISCC.exe build_installer.iss`

### 技术约定

| 约定 | 说明 |
|------|------|
| 版本号来源 | `Cargo.toml` → `env!("CARGO_PKG_VERSION")` 自动传播 |
| 前端版本号 | `/api/version` 端点动态获取 |
| 错误响应 | 统一 HTTP 200 + JSON `{code, message, data}` |
| 数据库 | SQLite WAL 模式，连接池自动初始化 PRAGMA |
| 托盘 | tray-icon + winit，单实例保护，2秒打开节流 |

---

## 二、时间线日志

### 2026-06-22 — 编译打包安装包

- 修正 `build_installer.iss`：`MyAppExeName` 和 Source 路径从 `v1.1.0` 更新为 `v1.6.0`
- PyInstaller 按 `workload.spec` 成功打包 → `dist\工作量统计工具_v1.6.0\`
- Inno Setup 6 (ISCC.exe at `D:\APP\Inno Setup 6\ISCC.exe`) 编译成功
- 安装包输出：`installer\工作量统计工具_v1.6.0_Setup.exe`
- 项目技术栈：Python FastAPI 后端 + PyInstaller 打包 + Inno Setup 安装程序

---

### 2026-06-23 — v1.7.0 + 模板重构

- 版本号 1.6.0 → 1.7.0，更新 `main.py`、`workload.spec`、`build_installer.iss`
- `Layout.tsx` 关于对话框、`tray_app.py` 托盘标题同步
- **统计页圆角优化**：全部组件统一 `borderRadius: '2px'`（55+ 处）
- **Git 仓库创建**：`.gitignore`，关联 `origin → https://github.com/HotLL233/tj`，首次提交推送
- **导出模板重构**：分析 `导出模板\汇总模板.xlsx`，创建 `export_v3.py`
  - F列宽19.66、删除FONT_R/FONT_B、周汇总F-J合并表头+K/L/M汇总列
  - `__init__.py` 切换路由：`export` → `export_v3`
- 版本号 v0.2.0 → v0.2.1：`main.py`、`tray_app.py`、`Layout.tsx`、`build_installer.iss`、`workload.spec`
- 安装包：`installer\工作量统计工具_v0.2.1_Setup.exe` (44.5MB)

---

### 2026-06-24 — 导出模板对齐 + 多版本迭代

- **export_v4 模板对齐 + v0.2.2 打包**
  - 创建 `export_v4.py`：日/周/用户三个Sheet全部对齐月-汇总B-I模板格式
  - 每日工作量：B=日期→C=实验室→D=代号→E=方法→F=日检测数量→G/H/I汇总
  - 每周工作量：B-I同月汇总，J起为每周数值列，F=所有周合计
  - 用户统计：B=用户名→C=实验室→D=代号→E=方法→F=月检测数量→G/H/I汇总
  - `__init__.py` 切换路由：`export_v3` → `export_v4`
  - 版本号 v0.2.1 → v0.2.2

- **v0.2.2 → v0.2.3 迭代**
  - 录入页时间选择器常驻显示
  - 用户名 localStorage → sessionStorage（Tab 级记忆）
  - SPA 路由刷新 404 修复（Python 中间件）
  - 托盘退出修复 + 双击防重复（2秒节流）
  - 版本号同步 v0.2.2 → v0.2.3

- **v0.2.3 → v0.2.4**
  - Rust 错误响应 HTTP 真实状态码→统一 200（避免 axios 抛异常）
  - WAL 连接池初始化修复
  - 统计字段名 breakdown→details 对齐前端
  - 用户日志→用户录入记录（单条删除+修改）

- **Rust 版创建 (v0.1.0 → v0.1.2)**
  - Axum + rusqlite, 安装包 3.7MB（vs Python 44MB）
  - 托盘图标从项目图标目录加载
  - 绑定 0.0.0.0 支持局域网访问
  - `/api/version` 端点，前端动态获取版本

---

### 2026-06-25 — 导出修复 + 策略更新

- Python v0.2.5 最终版打包
- **Rust导出文件损坏修复 (v0.1.3)**
  - `_cl()` 改为 0-based (0→"A", 5→"F")
  - `merge_range` 先合后写，公式行跳过 `write_with_format` 覆盖
  - 总计行修正到 F/G/H/I
- **Python 版正式冻结**
- **版本隔离策略生效**：v0.1.3 快照到 `workload-tool-rust/v0.1.3/`

---

### 2026-06-26 — v0.1.4: 导出5Sheet完整实现

- 从 v0.1.3 复制到 v0.1.4，严格版本隔离
- **全部5个Sheet完整实现**（对齐Python export_v4.py）：

| Sheet | 结构 | 新增内容 |
|-------|------|----------|
| 1. 月-汇总 | lab→code→instrument 三级树形 | B-I模板+SUM公式+总计行 |
| 2. 每日工作量 | date→lab/code→instrument 四级树形 | B-I模板+合并+公式（原为简化列表） |
| 3. 每周工作量 | 月汇总+周数据列(J起) | chrono Datelike周范围计算 |
| 4. 原始记录 | 序号/日期/实验室/项目/仪器/方法/类型/数量/录入人 | 9列明细（原为stub） |
| 5. 用户统计 | user→lab/code→instrument 四级树形 | B-I模板（原为stub） |

- 公共函数：`detect_groups`/`write_tree_sheet` 供 Sheet1/2/5 复用
- 工具函数：`week_ranges`(chrono Datelike)、`build_lab_hierarchy`、`flatten_lab`

---

## 三、版本演进速查

| 日期 | Python 版 | Rust 版 | 关键变更 |
|------|-----------|---------|----------|
| 06-22 | v1.6.0 | — | 初始打包流程 |
| 06-23 | v1.7.0 / v0.2.1 | — | 圆角优化 + 导出模板v3 |
| 06-24 | v0.2.2 → v0.2.4 | v0.1.0 → v0.1.2 | export_v4, SPA修复, Rust创建 |
| 06-25 | **v0.2.5(终)** | v0.1.3 | 导出修复, Python冻结, 版本隔离 |
| 06-26 | 冻结 | **v0.1.4** | 5Sheet完整导出实现 |

---

## 四、导出模块文件对应

| Python | Rust | 状态 |
|--------|------|------|
| `api/export_v4.py` (571行) | `v0.1.4/src/api/export_handler.rs` (584行) | 功能已对齐 |
| `database.py` 方法全称95条 | `export_handler.rs` `get_method_full_name` 12条 | 待补全(下次迭代) |
| `export_v0.py` (旧版) | — | 已淘汰 |
