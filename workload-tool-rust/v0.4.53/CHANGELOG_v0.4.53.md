# v0.4.53 版本更新说明

## 更新日期
2026-07-13

## 修复
### P0 - 研发送样记录「部门」列不显示
- **根因**：`rd_work_records` 的 SELECT 列表没包含 `wr.division_id`，写入时写了但读不回
- **修复**：SELECT 加 `wr.division_id, wr.group_id`；`RdRecordResponse` 加对应字段；row.get 映射

### 实验室列改为可选下拉
- **之前**：实验室列只能整页固定一个 lab（按 URL group_id），显示为禁用文本框
- **之后**：每行可独立选择实验室（下拉列表），自动填入当前实验室
- **改动**：RowState 加 `group_id` 字段；submit 用 `row.group_id ?? gid`
- **今日记录**：使用记录的 `group_name` 显示每行的实验室名称

## 改动文件

| 文件 | 改动 |
|------|------|
| `src/repo/rd_record_repo.rs` | SELECT + row.get 加 `division_id, group_id` |
| `src/models/rd_record.rs` | `RdRecordResponse` 加 `division_id, group_id` |
| `frontend/src/pages/SampleEntryPage.tsx` | RowState + createEmptyRow + lab_name 下拉 + 提交 |

## 验证
- ✅ `tsc --noEmit` 0 errors
- ✅ `cargo check` 通过
- ✅ `vite build` 3.05s
- ✅ `cargo build --release` 1m25s
- ✅ Inno Setup 打包成功
