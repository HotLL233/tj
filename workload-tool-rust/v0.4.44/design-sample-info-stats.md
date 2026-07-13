# 样品信息登记 — 统计页 & 实验室/车间列 设计方案

> 生成日期：2026-07-09
> 用途：为样品信息登记模块添加独立统计页（入口在门户内），并在录入表单增加「实验室/车间」列
> **本次仅出方案，不修改代码**

---

## 一、修改概览

| 序号 | 修改项 | 影响文件 | 改动量 | 后端影响 |
|------|-------|---------|--------|---------|
| 1 | 样品信息登记门户增加「查看统计」按钮 | `SampleInfoHome.tsx` | 极小 | 无 |
| 2 | 新建独立统计页 `SampleInfoStatsPage.tsx` | `SampleInfoStatsPage.tsx` | 中等（~600 行） | 无（复用现有 `/sample-info/stats` 接口） |
| 3 | 录入表单增加「实验室/车间」列 | `SampleInfoEntry.tsx` | 极小 | 无（`lab_name` 已有） |
| 4 | 路由注册新页 | `App.tsx` | 1 行 | 无 |

---

## 二、1. 样品信息登记统计页

### 2.1 现状分析

**后端**已有 `/api/sample-info/stats` 接口，返回如下数据结构：

```json
{
  "total": 1234,
  "by_status": [{ "name": "待检测", "count": 500 }, ...],
  "by_type": [{ "type_key": "icp", "label": "ICP", "count": 300 }, ...],
  "by_lab": [{ "name": "色谱实验室", "count": 200 }, ...],
  "by_project": [{ "name": "A001项目", "count": 150 }, ...],
  "by_user": [{ "name": "张三", "count": 100 }, ...],
  "by_month": [{ "month": "2026-01", "count": 50 }, ...]
}
```

**前端**已有 `getSampleInfoStats()` API 调用和 `exportSampleInfo()` 导出接口（单 Sheet Excel）。

**缺少的**：
- 没有前端统计页展示这些数据
- `SampleInfoHome.tsx`（门户入口）缺少统计入口按钮
- 没有按分析检测 `StatsPage.tsx` 的样式来做

### 2.2 设计思路

与分析检测 `StatsPage.tsx` 对标的统计页设计，但由于样品信息登记的数据模型更简单（无方法/仪器/单价倍率等复杂字段），因此简化卡片结构：

**参考 `StatsPage.tsx` 的模板** → 简化适配样品信息的维度：

**StatsPage 有 10 个 Sheet 统计卡片：**
- 按周统计 / 按月统计 / 检测人记录
- Sheet 1-10（实验室-项目-方法、仪器、项目金额、实验室金额等）

**样品信息统计页简化为 8 个统计卡片：**

| 卡片 key | 标签 | 图标 | 颜色 | 数据来源 |
|---------|------|------|------|---------|
| `overview` | 总览 | `DashboardIcon` / `AssessmentIcon` | `#2e7d32` 绿色 | `total` + `by_status` |
| `by-type` | 按检测类型 | `ScienceIcon` / `BiotechIcon` | `#1b5e20` | `by_type` |
| `by-lab` | 按实验室/车间 | `BusinessIcon` / `PrecisionManufacturingIcon` | `#00796b` | `by_lab` |
| `by-project` | 按所属项目 | `FolderIcon` | `#FF9800` | `by_project` |
| `by-user` | 按送样人 | `PeopleIcon` | `#E91E63` | `by_user` |
| `by-month` | 按月统计 | `CalendarMonthIcon` | `#1976d2` | `by_month` |
| `user-log` | 送样人记录明细 | `HistoryIcon` | `#5d4037` | `getSampleInfoRecords()` + 分页 |
| `export` | 导出Excel | `DownloadIcon` | `#4CAF50` | `exportSampleInfo()` |

### 2.3 页面布局

```
┌─────────────────────────────────────────────────────────────────┐
│  ← 返回 样品信息登记统计                                         │
│  [起始日期] ~ [结束日期]                              [导出Excel]  │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐          │
│  │ 总记录数  │ │ 待检测    │ │ 已取样    │ │ 检测完成  │          │
│  │  1,234   │ │   500    │ │   300    │ │   200    │          │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘          │
├─────────────────────────────────────────────────────────────────┤
│  stats_card_grid (2行 × 4列):                                    │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐          │
│  │按检测类型 │ │按实验室   │ │按所属项目 │ │按送样人   │          │
│  │  📊      │ │  🏢      │ │  📁      │ │  👥      │          │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘          │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐          │
│  │按月统计   │ │送样人记录 │ │导出Excel  │ │          │          │
│  │  📅      │ │  📋      │ │  ⬇️      │ │          │          │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘          │
├─────────────────────────────────────────────────────────────────┤
│  选中卡片后的内容展示区域：                                        │
│  - 按检测类型 → 表格：类型 | 记录数 | 占比（横向柱状图）            │
│  - 按实验室   → 表格：实验室/车间 | 记录数 | 占比                  │
│  - 按所属项目 → 表格：项目名称 | 记录数 | 占比                     │
│  - 按送样人   → 表格：送样人 | 记录数 | 占比                      │
│  - 按月统计   → 表格：月份 | 记录数 | 趋势（折线/柱状）            │
│  - 总览       → 状态分布卡片列表 + 概要数字                       │
│  - 送样人记录 → 带分页的明细列表（含筛选：类型、状态、送样人）      │
└─────────────────────────────────────────────────────────────────┘
```

### 2.4 页面结构实现思路

**新建 `SampleInfoStatsPage.tsx`**，参考 `SampleStatsPage.tsx` 的结构：

```
SampleInfoStatsPage
  ├── 头部：返回按钮 + 标题 + 日期范围选择器 + 导出按钮
  ├── 统计摘要卡片行（4个数字卡片：总记录数、待检测、已取样、检测完成）
  ├── STAT_CARDS 网格（8个统计入口卡片，与 StatsPage 风格一致）
  └── 内容详情区（点击某个卡片后展示对应的表格/图表数据）
      ├── overview: 展示 by_status 的分布（各种状态 Chip 排列）
      ├── by-type: 表格展示检测类型分布
      ├── by-lab: 表格展示实验室分布
      ├── by-project: 表格展示项目分布
      ├── by-user: 表格展示送样人分布
      ├── by-month: 表格展示月度趋势
      ├── user-log: 分页明细列表（含状态筛选、类型筛选、送样人筛选）
      └── export: 调用 exportSampleInfo() 下载
```

### 2.5 与 `StatsPage.tsx` 的关键差异

| 方面 | StatsPage（分析检测） | SampleInfoStatsPage（样品信息登记） |
|------|---------------------|-----------------------------------|
| 维度 | 10 Sheet + 周/月/人 | 6 维度 + 人记录明细 |
| 金额 | 有（单价倍率/金额） | 无 |
| 仪器 | 有（仪器汇总、仪器类型） | 无 |
| 方法 | 有（方法类型） | 无 |
| 实验室 | 关联 project_lab_links | 直接来自记录的 `lab_name` |
| 导出 | 7 Sheet Excel | 单 Sheet Excel |
| 筛选 | 各 Sheet 有独立筛选 | 顶部日期范围 + 类型下拉 |
| 背景色 | 蓝色 `#1976d2` | 绿色 `#2e7d32` |

---

## 三、2. 门户入口添加「查看统计」按钮

### 3.1 现状

`SampleInfoHome.tsx` 当前只有检测类型卡片和底部「查看全部登记记录」链接：
```
[← 样品信息登记]
[检测类型卡片网格]
      [查看全部登记记录]
```

### 3.2 改为

在顶部区域增加「查看统计」按钮，与分析检测 `WorkloadPortal.tsx` 的风格一致：

```
[← 样品信息登记]                     [📊 查看统计]
ICP · 热分析 · 质谱等样品基础信息填写
     |
     [检测类型卡片网格]
          [查看全部登记记录]
```

**具体改动**：在 `SampleInfoHome.tsx` 的顶部 `Box` 中增加：
```tsx
<Button
  variant="outlined"
  startIcon={<BarChartIcon />}
  onClick={() => n('/sample-info/stats')}
  sx={{ borderRadius: R, borderColor: '#2e7d32', color: '#2e7d32' }}
>
  查看统计
</Button>
```

### 3.3 路由注册

在 `App.tsx` 中增加一行路由：
```tsx
<Route path="/sample-info/stats" element={<SampleInfoStatsPage />} />
```

---

## 四、3. 录入表单增加「实验室/车间」列

### 4.1 现状

`SampleInfoEntry.tsx` 的 Excel 模板式多行录入表格当前列：

| 序号 | 送样人 | 所属部门 | 所属项目 | 送样数量 | 样品批号 * | 样品主要成分 * | 注意事项 |

但后端模型 `SampleInfoRecord` 已有 `lab_name: String` 字段：
- `createSampleInfo()` 已接受 `lab_name` 参数
- 详情展开和编辑模式已正确显示了「实验室/车间」
- 只是录入表格本身的列头中**缺少了这一列**

### 4.2 改动点

在当前表格的「所属项目」和「送样数量」之间插入「实验室/车间」列：

| 序号 | 送样人 | 所属部门 | 所属项目 | **实验室/车间** | 送样数量 | 样品批号 * | 样品主要成分 * | 注意事项 |

**涉及2处改动：**

**① 表头（L249-L254）**：在「所属项目」和「送样数量」之间插入：
```tsx
<TableCell sx={{ fontWeight: 700, p: 1, minWidth: 100 }}>实验室/车间</TableCell>
```

**② 表体（L290-L296）**：在「所属项目」和「送样数量」之间插入一列单元格：
```tsx
<TableCell sx={{ p: 0.5 }}>
  <TextField
    size="small" fullWidth
    value={row.lab_name}
    onChange={e => updateRow(idx, 'lab_name', e.target.value)}
    placeholder="实验室/车间"
    sx={{ '& .MuiOutlinedInput-root': { fontSize: '0.8rem' } }}
  />
</TableCell>
```

**③ RowData 接口（L34-L43）**：增加 `lab_name` 字段：
```ts
interface RowData {
  user_name: string;
  division_id: number | '';
  project_name: string;
  lab_name: string;  // 新增
  quantity: number;
  batch_no: string;
  main_components: string;
  notes: string;
  checked: boolean;
}
```

**④ emptyRow()（L45-L48）**：增加 `lab_name: ''` 默认值。

### 4.3 提交逻辑

`doSubmit()` 中构造 `createSampleInfo` 参数时，`lab_name` 已包含在 `RowData` 中，只需从行数据取值：
```ts
await createSampleInfo({
  ...
  lab_name: row.lab_name || '',  // 从行数据取
  ...
});
```

### 4.4 无需后端改动

后端 `SampleInfoRecord` 模型已有 `lab_name` 字段，`SampleInfoCreate` 也接受 `lab_name`，`INSERT` SQL 也已包含 `lab_name` 列。**纯前端改动。**

---

## 五、UI 设计图

### 5.1 样品信息统计页

见随附的 HTML 设计图 `design-sample-info-stats-page-ui.html`。

### 5.2 录入表单增加实验室/车间列

见随附的 HTML 设计图 `design-sample-info-entry-form-ui.html`。

---

## 六、实现顺序建议

| 步骤 | 内容 | 预估工时 | 备注 |
|------|------|---------|------|
| 1 | `SampleInfoEntry.tsx` — 增加实验室/车间列 | ~15 分钟 | 只加列头 + 单元格 + RowData 接口 |
| 2 | 新建 `SampleInfoStatsPage.tsx` | ~60 分钟 | 参考 `SampleStatsPage.tsx` 结构，简化 |
| 3 | `SampleInfoHome.tsx` — 增加「查看统计」按钮 | ~5 分钟 | 加一个 Button |
| 4 | `App.tsx` — 注册 `/sample-info/stats` 路由 | ~2 分钟 | 1 行代码 |
| 5 | `cargo check + npm run build` 验证 | ~5 分钟 | |
| **合计** | | **~1.5 小时** | |

---

## 七、待确认事项

- ❓ 统计页状态卡片展示：需要展示「待检测/待取样/已取样/检测完成」四个状态的数量，还是只展示「总记录数」+「按状态分布」？
- ❓ 统计页是否需要在按检测类型/实验室/项目等图表中添加**柱状图/饼图**可视化（参考 StatsPage 那样纯表格）？
- ❓ 导出 Excel 是否需要在现有单 Sheet 基础上增加更多 Sheet（类似分析检测的 7 个 Sheet）？
