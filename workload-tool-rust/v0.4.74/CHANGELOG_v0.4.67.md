# v0.4.67 版本更新说明

## 更新日期
2026-07-15

## 更新内容

### 高项改为可编辑（类比单价倍率行为）
- 创建记录时：默认从项目高项复制到 work_records.high_item，支持手动覆盖
- 显示/统计/导出：全部使用 `wr.high_item`（记录创建时的快照），与项目当前值解耦
- 检测人记录（StatsPage）：高项列支持双击编辑（行内编辑 + onBlur 保存）
- 编辑后单独统计，不联动项目

### 历史 NULL 回填
- 启动时自动将 work_records / rd_work_records 中 high_item IS NULL 的记录
  回填为对应 project 的当前 high_item

### 改动文件
- `src/api/export_data.rs` — Sheet 1-4 SQL 从 p.high_item 改为 wr.high_item
- `src/db/migrations.rs` — 历史 NULL 回填 SQL
- `src/models/record.rs` — RecordCreate/RecordUpdate 加 high_item
- `src/repo/record_repo.rs` — 创建/更新逻辑处理 high_item
- `src/api/record_handler.rs` / `rd_record_handler.rs` — 传递 high_item
- `frontend/src/api/client.ts` — updateRecord 加 high_item 类型
- `frontend/src/pages/StatsPage.tsx` — 高项列可编辑 TextField
