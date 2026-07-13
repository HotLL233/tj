# v0.4.40 版本更新说明

## 更新日期
2026-07-12

## 更新内容
### 修复：导出功能 500 错误（no such column: pg2.deleted_at）
- **根因**：`stats_service::by_division` 的子查询引用了 `project_groups.deleted_at`，但该表建表时未创建此列，导致 Sheet 11 事业部汇总 SQL 解析时直接报错，整份 Excel 生成失败返回 500。
- **修复 #1**：`stats_service.rs` 中移除 `pg2.deleted_at IS NULL` 条件（project_groups 未做软删除）
- **修复 #2**：`migrations.rs` 中为 `project_groups` 表补加 `deleted_at TEXT` 列（向后兼容）

## 改动文件
- `src/service/stats_service.rs` — 1 处 SQL 修改
- `src/db/migrations.rs` — 1 处 migration 新增

## 打包说明
- 无 cmd 窗口（生产模式 `cargo build --release`）
- 支持覆盖安装（AppId 固定）
