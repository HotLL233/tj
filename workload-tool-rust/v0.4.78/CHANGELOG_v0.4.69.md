# v0.4.69 版本更新说明

## 更新日期
2026-07-15

## 修复
### Sheet 3（项目-汇总）导出报错修复
- 根因：write_sheet3 的 merge_range 调用缺少单行 guard
- 单项目单行时 start==end → merge_range(1,CA,1,CA) 合并单格 → Excel 报错
- 修复：merge_range 前加 if start != end，公式不受影响

### Sheet 4（实验室-汇总）单行公式修复
- 根因：write_sheet4 的公式在 if end > start 内部
- 单实验室单行时公式被跳过 → 实验室汇总列为空
- 修复：公式移出 if 块，merge 仍 guard

## 改动
- `src/api/export_write.rs` — write_sheet3 + write_sheet4 各 3 行
