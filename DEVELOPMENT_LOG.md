# 工作量统计工具 — 开发历程文档

> 生成时间: 2026-06-23 08:00
> 当前版本: v1.7.0
> 项目路径: `D:\桌面\工作量统计工具项目\`

---

## 一、项目起源 (2026-06-19)

基于原 Rust Tauri 工具 `D:\桌面\py\rust-tool\` (检测数据统计工具 v5.0.0) 改造而来。

### 原工具
- 架构: Rust + Tauri 原生桌面应用 (~3MB)
- 功能: 单机读取 Excel 统计，无数据库、无多用户

### 第一版工作量统计工具 (Rust/Tauri)
- 项目: `D:\桌面\workload-tool\`
- 改造: 统计算法从去重→求和，列名从批号→个数，Sheet 过滤增加模板页
- 编译: `cargo check` 零 Error 零 Warning
- 问题: 后来需求变更为多用户 Web 应用，Rust 版废弃

---

## 二、Web 版完整开发 (2026-06-21) — v1.0.0

采用标准 SOP 流程（TeamCreate → PRD → 架构设计 → 代码实现 → QA 验证）。

### 团队协作
- **产品经理 许清楚**: 产出 PRD — 5 目标 + 5 用户故事 + P0/P1/P2 需求池
- **架构师 高见远**: FastAPI + SQLite + React + MUI + Tailwind，4 表 DDL，17 API 端点
- **工程师 寇豆码**: 32 个文件（backend 10 + frontend 22）
- **QA 严过关**: 两轮测试，发现 audit-logs 端点缺失 → 修复通过

### 技术栈
| 层级 | 技术 |
|------|------|
| 后端 | Python 3.10, FastAPI, SQLite (WAL), openpyxl |
| 前端 | Vite, React 18, TypeScript, MUI 5, Tailwind CSS |
| 打包 | PyInstaller + Inno Setup |
| 数据库 | SQLite (data/workload.db), 4 表 |

### 数据库 (4 表)
```sql
project_groups  -- 实验室分组 (8个: 410/415/417/418/生物合成/车间/801厂区/707)
projects        -- 项目/方法 (44个, 含仪器信息)
work_records    -- 工作量记录 (软删除)
audit_log       -- 审计日志
```

### 前端页面
- 主页: 实验室分组卡片 → 点击进入录入
- 录入页: 项目列表 + 行内提交 (数量/用户名)
- 统计页: 4 维度 (按周/按月/按用户/按项目)
- 管理页: 3 Tab (分组管理/项目管理/回收站)

### 内置种子数据
- seed_data.py → 后改为 database.py 内置硬编码
- 8 分组 44 项目，首次启动自动创建

### 启动命令
```bash
cd project-root/backend && pip install -r requirements.txt
cd project-root/frontend && npm install && npm run build
cd ../backend && python main.py
# 浏览器 http://localhost:8000
```

---

## 三、迭代升级历程 (2026-06-22)

### v1.0.0 — 初始发布
- PyInstaller 打包 → Inno Setup 安装程序 (45MB)
- console=True 控制台模式

### v1.1.0 — 导出格式 + SPA 路由 + 托盘
- 导出 Excel 匹配汇总模板格式（按月交叉表+分组小计）
- SPA 路由修复: `/{full_path:path}` 回退到 index.html 解决刷新 404
- 种子数据内联到 database.py（解决 PyInstaller 模块导入问题）
- reload=True 改为 uvicorn.run(app)（解决 exe 死循环重启）

### v1.2.0 — 托盘图标 + 统计分类
- 系统托盘图标: pystray + PIL, 右键打开页面/退出
- 统计分类: LC→液相, GC→气相, 自动解析仪器编号
- 按类型统计 + 按仪器统计两个新维度
- stdout/stderr 重定向（GUI 模式 None 崩溃修复）
- 单实例保护

### v1.3.0 — 导出三 Sheet
- 月-汇总: 实验室/项目合并单元格 + LC/GC 分列 + 每日数据
- 仪器汇总: 按仪器编号统计
- 原始记录: 逐条明细
- 样式: 等线 11pt, 液相蓝底/气相绿底, 小计灰底/总计红底

### v1.4.0 — 全称映射 + 全项目导出
- 95 条方法简称→全称映射 (database.py METHOD_FULL_NAMES)
- 录入页: 简称大字 + 全称小字
- 导出: 所有 44 项目都显示（含 0 数据）, 每日明细独立 sheet

### v1.5.0 — 自定义信息 + 用户日志 + 记录纠错
- projects 表增加 full_name / notes 列（DB 迁移兼容旧库）
- 管理页: 项目编辑增加全称/备注输入
- 统计页 用户日志 Tab: 逐条显示，分页，编辑按钮
- PUT /records/{id}: 修改数量/用户名/时间，写入审计日志

### v1.5.1 — 交互优化
- 统计卡片可点击跳转对应 Tab
- 用户日志: 添加用户筛选下拉
- 周统计: 从 ISO 年周 → 每月第几周 ("2026-06第4周")

### v1.6.0 — UI 卡片化 + 关于 + 删除用户
- 右上角"关于"按钮: 版本号 + 8 项功能列表 + © 2026 HotLL
- 统计页完全卡片化: 7 张功能卡片（uiverse.io 风格），点击展开
- 删除用户: `DELETE /api/records/by-user/{name}` → 批量进回收站

### v1.7.0 — 全局 uiverse.io UI 改版
- MUI Theme 全局覆盖: 圆角 12px, 阴影减轻, 主色 #667eea, 背景 #f8fafc
- 新建组件: GroupCard.tsx (彩色 Avatar + Chip)
- 主页: Hero 蓝紫渐变 + 3 列卡片网格 + 搜索
- 录入页: 卡片式项目行（左侧 4px 液相蓝/气相绿竖线）
- 统计页: 渐变边框卡片 + 图标框 + 彩色数值
- 管理页: 2×2 功能卡片 + 圆角 Paper 列表
- Layout: 毛玻璃 AppBar (backdrop-filter: blur), Pill 导航, 毛玻璃底部导航
- 颜色系统: 蓝紫/蓝青/青绿/橙红四主题

---

## 四、当前状态总览

### 文件结构
```
project-root/
├── backend/
│   ├── main.py              # FastAPI 入口, SPA fallback, tray 模式
│   ├── database.py           # SQLite 连接, 4 表 DDL, 种子数据, 全称映射
│   ├── models.py             # Pydantic 模型 (16 个)
│   ├── tray_app.py           # 系统托盘 (pystray + PIL)
│   ├── workload.spec         # PyInstaller 配置
│   ├── seed_data.py          # 种子数据 (已被 database.py 内置取代)
│   ├── api/
│   │   ├── __init__.py       # APIRouter 聚合 (7 子路由)
│   │   ├── groups.py         # 分组 CRUD
│   │   ├── projects.py       # 项目 CRUD + full_name/notes
│   │   ├── records.py        # 记录 CRUD + 软删除/恢复/更新/批量删除用户
│   │   ├── stats.py          # 多维度统计 (摘要/week/month/user/project/type/instrument)
│   │   ├── export.py         # Excel 导出 (4 sheets: 月汇总/每日明细/仪器汇总/原始记录)
│   │   └── audit_logs.py    # 审计日志查询
│   ├── static/               # 前端构建产物
│   └── data/workload.db      # SQLite 数据库（运行时生成）
├── frontend/
│   ├── src/
│   │   ├── main.tsx          # MUI Theme 全局配置
│   │   ├── App.tsx           # 路由 + ErrorBoundary + 404
│   │   ├── types/index.ts    # TypeScript 接口 (12 个)
│   │   ├── api/client.ts     # API 方法 (18 个)
│   │   ├── components/
│   │   │   ├── Layout.tsx    # 毛玻璃导航 + 关于对话框
│   │   │   ├── GroupCard.tsx  # 主页实验室卡片
│   │   │   ├── ProjectRow.tsx # 录入页项目卡片行
│   │   │   ├── StatsCards.tsx # 统计摘要卡片
│   │   │   └── DateRangePicker.tsx
│   │   └── pages/
│   │       ├── HomePage.tsx   # 主页 (Hero + 搜索 + 卡片网格)
│   │       ├── EntryPage.tsx  # 录入页 (项目卡片列表)
│   │       ├── StatsPage.tsx  # 统计页 (7 功能卡片 + 用户日志编辑)
│   │       └── ManagePage.tsx # 管理页 (4 功能卡片 + CRUD)
│   └── package.json
├── build_installer.iss       # Inno Setup 安装脚本
├── installer/                # 安装包输出目录
└── .workbuddy/memory/        # 开发记忆日志
```

### API 端点 (22 个)
| 分组 | 端点 | 方法 |
|------|------|------|
| groups | /api/groups | GET/POST |
| groups | /api/groups/{id} | PUT/DELETE |
| projects | /api/projects | GET/POST |
| projects | /api/projects/{id} | PUT/DELETE |
| records | /api/records | GET/POST |
| records | /api/records/{id} | PUT/DELETE |
| records | /api/records/{id}/restore | POST |
| records | /api/records/by-user/{name} | DELETE |
| stats | /api/stats/summary | GET |
| stats | /api/stats/by-user | GET |
| stats | /api/stats/by-project | GET |
| stats | /api/stats/by-type | GET |
| stats | /api/stats/by-instrument | GET |
| export | /api/export/excel | GET |
| audit | /api/audit-logs | GET |

### 功能特性
- ✅ 8 实验室 44 项目内置
- ✅ 多用户录入（用户名+数量）
- ✅ 7 维度统计（周/月/日/用户/项目/类型/仪器）
- ✅ 导出 4 Sheet Excel（对齐汇总模板格式）
- ✅ 软删除+回收站+恢复
- ✅ 审计日志（创建/删除/恢复/修改/删除用户）
- ✅ 记录纠错编辑
- ✅ 95 条方法全称映射
- ✅ 自定义全称+备注
- ✅ 系统托盘运行（无窗口）
- ✅ 单实例保护
- ✅ SPA 路由（刷新不 404）
- ✅ 移动端适配
- ✅ 毛玻璃 UI + uiverse.io 卡片风格
- ✅ Inno Setup 安装程序

---

## 五、版本安装包

| 版本 | 安装包 | 日期 |
|------|--------|------|
| v1.0.0 | 工作量统计工具_v1.0.0_Setup.exe | 06-22 |
| v1.1.0 | 工作量统计工具_v1.1.0_Setup.exe | 06-22 |
| v1.2.0 | 工作量统计工具_v1.2.0_Setup.exe | 06-22 |
| v1.3.0 | 工作量统计工具_v1.3.0_Setup.exe | 06-22 |
| v1.4.0 | 工作量统计工具_v1.4.0_Setup.exe | 06-22 |
| v1.5.0 | 工作量统计工具_v1.5.0_Setup.exe | 06-22 |
| v1.5.1 | 工作量统计工具_v1.5.1_Setup.exe | 06-22 |
| v1.6.0 | 工作量统计工具_v1.6.0_Setup.exe | 06-22 |
| v1.7.0 | 工作量统计工具_v1.7.0_Setup.exe | 06-23 |
