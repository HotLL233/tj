# v0.4.64 版本更新说明

## 更新日期
2026-07-14

## 更新内容

### 修正 v0.4.63 的 3 个实施偏差

1. **撤销 high_items 独立表架构**（v0.4.63 错误实现）
   - 删除 3 个文件：`high_item.rs` / `high_item_repo.rs` / `high_item_handler.rs`
   - 高项改为纯文本：`projects.high_item TEXT`（自定义输入，不绑定表）

2. **批量导入对话框加「下载模板」按钮**
   - 三步流程：① 下载模板 → ② 上传文件 → ③ 解析结果
   - 模板为 CSV 格式，含 BOM 头确保 Excel 中文显示

3. **回收站重构**
   - 新增 3 个子 Tab：工作记录/检测类型/研发项目
   - 检测类型和研发项目的已删除记录（is_active=0）在对应 Tab 中展示
   - 支持「恢复」操作（设 is_active=1）

### 修复：检测类型唯一性检查不区分软删除
- `src/repo/sample_info_type_repo.rs:58` 加 `AND is_active=1`
- 软删除的类型可重新创建同名类型

## 改动文件清单

### 删除文件
- `src/models/high_item.rs`
- `src/repo/high_item_repo.rs`
- `src/api/high_item_handler.rs`

### 修改文件
- `src/db/migrations.rs` — v0.4.64 迁移（high_items DROP + projects/records 加 high_item TEXT）
- `src/models/project.rs` — high_item_id → high_item TEXT
- `src/models/mod.rs` — 移除 high_item
- `src/repo/project_repo.rs` — SQL 全改用 high_item 文本字段
- `src/repo/mod.rs` — 移除 high_item_repo
- `src/repo/sample_info_type_repo.rs` — 唯一性检查 `WHERE type_key=?1 AND is_active=1`
- `src/api/mod.rs` — 移除 high_item_handler 路由
- `frontend/src/types/index.ts` — high_item: string | null
- `frontend/src/api/client.ts` — 移除 6 个高项函数
- `frontend/src/pages/ManagePage.tsx` — 项目编辑 TextField + 下载模板 + 回收站 3 子 Tab

## 打包说明
- 覆盖安装（AppId 固定）
- 安装包：`样品管理系统_v0.4.64_Setup.exe`
