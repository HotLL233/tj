# v0.4.34 更新说明

> **版本**: 0.4.34 | **日期**: 2026-07-11 | **基线**: v0.4.33

## Bug 修复

### 1. 研发送样录入页右上角状态标签修复
- 右上角 Chip 状态从硬编码"待检测"改为动态读取记录状态，默认为"待取样"

### 2. 附件上传功能
- 样品信息登记的列配置新增 `attachment_files`（附件）预置列
- 表单中附件列正常显示「上传附件」按钮

### 3. 登记记录表格排版优化
- 登记记录表头和数据行从 flexbox 布局改为 MUI `<Table>` 原生表格
- 解决列宽超容器导致换行错位的问题

## 功能新增

### 4. 取样权限 + 自动取样 + 记录锁定 + 行内编辑
- **新增权限点** `sample:collect`（研发送样-取样操作），分析员预设模板自动携带
- **JWT 自动取样**：取样 API 不再需要前端传入取样人姓名，后端直接从登录令牌提取当前用户名
- **后端鉴权**：取样操作需验证 `sample:collect` 权限
- **记录锁定**：已取样的记录不再可修改（update 返回 403 Forbidden）
- **行内编辑**：研发送样记录列表每行支持展开编辑，可修改送样人/项目/类型/方法/数量/批号/注意事项；已取样记录隐藏编辑按钮

### UI 修改
- 取样人列三种状态显示：
  - 已取样 → 绿色取样人文本
  - 有取样权限（分析员）→「取样」按钮，点击自动取样
  - 无取样权限（送样人）→ 灰色"待取样"标签

## 技术细节
- **版本隔离**：基于 v0.4.33 创建独立文件夹 v0.4.34
- **权限更新**：`constants/permissions.ts` + `models/role.rs` 新增 `sample:collect`
- **种子更新**：`migrations.rs` 新增 attachment 列种子 + 分析员权限更新
- **后端改动**（6 个）：`rd_record_handler.rs`、`rd_record_repo.rs`、`rd_record_service.rs`、`migrations.rs`、`models/role.rs`、`models/record.rs`
- **前端改动**（6 个）：`constants/permissions.ts`、`AdminRolesPage.tsx`、`SampleEntryPage.tsx`、`SampleInfoEntry.tsx`、`RdRecordsPage.tsx`、`api/client.ts`

## 升级说明
- 安装后首次启动会自动插入 attachment 列种子
- 分析员角色的权限自动升级（含 `sample:collect`）
- 安装包可覆盖安装 v0.4.30~v0.4.33（AppId 固定）
