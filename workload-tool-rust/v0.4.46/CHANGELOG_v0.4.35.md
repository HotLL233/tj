# v0.4.35 更新说明（尝试测试版）

> **版本**: 0.4.35 | **日期**: 2026-07-11 | **基线**: v0.4.34

## 全 UI 自定义系统（一期）

### 新增：系统设置中心
- **新建 `system_settings` 表**，存储所有 UI 配置（key-value JSON 格式）
- 后台 API：`GET/PUT /api/settings/:key`，JWT 管理员鉴权 + 审计日志
- 预置 5 套默认配置：主题 / 首页卡片 / 门户样式 / 管理页菜单 / 统计卡片

### P1 - 主题自定义
- 管理页新增「主题设置」子 Tab
- 可自定义：主色、辅助色、背景色、卡片圆角、登录页背景色、登录按钮色、品牌文字
- 前端 `main.tsx` 异步加载主题配置，MUI ThemeProvider 实时响应

### P2 - 首页 + 门户配置
- 管理页新增「首页配置」子 Tab
- 首页三张入口卡片可从管理页自定义：标题、图标、颜色、渐变、路径、权限点
- 门户页（研发送样/分析检测）颜色和品牌名从设置读取

### P3 - 管理页 Tab 配置
- 管理页 11 个子 Tab 的显隐和排序可从设置中控制

### P4 - 统计卡片配置
- 统计页的统计卡片可从管理页自定义：标题、颜色、渐变

## 技术细节
- **版本隔离**：基于 v0.4.34 创建独立文件夹 v0.4.35
- **新建文件**（3 个）：`models/settings.rs`、`repo/settings_repo.rs`、`api/settings_handler.rs`
- **新建表**：`system_settings`（5 条种子数据）
- **修改文件**（12 个）：`migrations.rs`、`mod.rs`×3、`types/index.ts`、`api/client.ts`、`main.tsx`、`HomePage.tsx`、`ManagePage.tsx`、`StatsCards.tsx`、`SamplePortal.tsx`、`WorkloadPortal.tsx`

## 升级说明
- 安装后首次启动自动创建 `system_settings` 表并插入 5 条默认配置
- 已有管理员可进入「系统管理→主题设置/首页配置/统计卡片」自定义 UI
- 安装包可覆盖安装 v0.4.30~v0.4.34（AppId 固定）
