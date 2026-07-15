# v0.4.68 版本更新说明

## 更新日期
2026-07-15

## 修复

### 检测人记录高项编辑框无法输入
- 根因：TextField 受控但缺少 onChange，输入被覆盖
- 修复：加 onChange 局部状态 + onBlur 同步后端（与 multiplier 编辑逻辑一致）

### 部分表高项显示为 -
- 根因：v0.4.67 全改用 wr.high_item，未编辑的记录为 NULL 显示为 -
- 修复：Sheet 1-4 SQL 改为 COALESCE(wr.high_item, p.high_item)
  - 记录编辑过 → 用记录值
  - 记录未编辑 → 回退到项目当前值

## 改动文件
- `src/api/export_data.rs` — wr.high_item → COALESCE(wr.high_item, p.high_item)
- `frontend/src/pages/StatsPage.tsx` — 高项 TextField 加 onChange + 局部状态
