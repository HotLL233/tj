# v0.4.70 版本更新说明

## 更新日期
2026-07-15

## 修复
### Sheet 1 实验室-项目-方法高项不显示
- 根因1: Sheet1Data 类型只有 7 元组，FlatRow 是 9 元组（缺 high_item + coefficient）
- 根因2: renderCell switch 没有 case "high_item"
- 修复: Sheet1Data 扩展为 9 元组 + renderCell 加 case + 索引调整

### 高项编辑失败无提示
- 根因: StatsPage onBlur catch 是空的 { /* ignored */ }
- 修复: 加 console.error 日志

## 改动
- `frontend/src/types/index.ts` — Sheet1Data 从 7-tuple 扩展到 9-tuple
- `frontend/src/pages/StatsPage.tsx` — renderCell 加 high_item case + 索引修正 + catch 日志
