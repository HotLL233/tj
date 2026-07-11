# 样品信息登记 — 统计页完整复刻 & 列自定义管理方案

> 生成日期：2026-07-10
> 用途：
> 1. 将样品信息登记统计页「完整复刻」分析检测统计页（StatsPage.tsx）的模式
> 2. 评估录入表单列自定义编辑的可行性，给出设计方案
> **本次仅出方案，不修改代码**

---

## 一、代码检查结论

### 1.1 后端已有能力

| 能力 | 后端 API | 状态 |
|------|---------|------|
| 统计接口 | `GET /api/sample-info/stats` → `{ total, by_status, by_type, by_lab, by_project, by_user, by_month }` | ✅ 已存在 |
| 记录查询 | `GET /api/sample-info` → 分页列表，支持 type_key/status/user_name/lab_name/project_name/start/end 筛选 | ✅ 已存在 |
| 导出 Excel | `GET /api/sample-info/export` → 7 个 Sheet（明细/状态/类型/实验室/项目/送样人/月份） | ✅ 已存在 |
| 检测类型 CRUD | `GET /api/sample-info-types` → 支持 type_key/label/color/sort_order/is_active | ✅ 已存在 |
| 用户列表 | 可从记录列表去重获取 | ✅ 可复用 |

### 1.2 前端现有能力

| 组件 | 状态 |
|------|------|
| `SampleInfoHome.tsx` | 门户入口，仅有检测类型卡片 + 查看全部记录链接，**无统计入口** |
| `SampleInfoEntry.tsx` | Excel 模板式录入表单，列**硬编码**在 JSX 中 |
| `ManagePage.tsx` 的 `sampleinfo` tab | 已有检测类型 CRUD + 记录查询 + 统计概览 |
| `StatsPage.tsx` | 分析检测统计页，12 个 STAT_CARDS + DateRangePicker + 摘要卡片 + 详情表格 | ✅ 参考模板 |
| `SampleStatsPage.tsx` | 研发送样统计页，完整克隆 StatsPage 模式 | ✅ 克隆参考 |

### 1.3 列定制的可行性结论

**当前不可行**。原因：
- `SampleInfoEntry.tsx` 中列头（L249-L255）是硬编码的 `<TableCell>` 列表
- `RowData` 接口（L34-L43）是硬编码的 TypeScript 结构
- 数据提交（L129-L141）直接构造固定的 `createSampleInfo` payload
- 后端 `SampleInfoCreate` 结构体是固定的字段
- 没有「列配置」相关的 DB 表、API、管理 UI

**可以做**，但涉及前后端完整改动（详见第四章）。

---

## 二、统计页完整复刻分析检测模式

### 2.1 对照表：StatsPage vs 样品信息统计页

| 方面 | StatsPage（分析检测） | SampleInfoStatsPage（样品信息） |
|------|---------------------|-------------------------------|
| 页面标题 | 分析检测统计 | 样品信息登记统计 |
| 主题色 | 蓝色渐变 `#00897b → #43a047` | 绿色 `#2e7d32` |
| STAT_CARDS 数量 | 12 个 | **8 个**（复用 7 Sheet 维度 + 送样人记录 + 导出） |
| 摘要卡片 | `StatsCards` 组件，显示总数量/记录数/人员/项目/系数 | 直接显示：总记录数 + 4 种状态的数量 |
| DateRangePicker | ✅ | ✅ 同上 |
| 导出按钮 | 顶部绿色渐变按钮 | 同上 |
| 周/月统计 | 有（getStatsSummary + group_by） | ❌ 没有周/月维度的汇总统计（sample_info 没有「周」字段） |
| 用户记录明细 | 有，可编辑可删除 | ✅ 已有 getSampleInfoRecords 分页接口，可移入统计页 |
| Sheet 1-10 | 10 个 PreviewTable | **映射到 7 个统计维度**（用 PreviewTable 或定制化表格） |

### 2.2 样品信息 STAT_CARDS 设计

```
┌──────────────────────────────────────────────────────────────┐
│  样品信息登记统计    [日期范围]  [导出 Excel]                    │
├──────────────────────────────────────────────────────────────┤
│  总记录数: 1,234  │  待检测: 500  │  待取样: 234  │  检测完成: 200  │
├──────────────────────────────────────────────────────────────┤
│  ┌──────────────────┐  ┌──────────────────┐                │
│  │ 📊 按状态        │  │ 🔬 按检测类型    │                │
│  │ 各状态记录数分布   │  │ ICP/热分析/质谱   │                │
│  └──────────────────┘  └──────────────────┘                │
│  ┌──────────────────┐  ┌──────────────────┐                │
│  │ 🏢 按实验室/车间  │  │ 📁 按所属项目    │                │
│  │ 8 个实验室        │  │ 12 个项目        │                │
│  └──────────────────┘  └──────────────────┘                │
│  ┌──────────────────┐  ┌──────────────────┐                │
│  │ 👤 按送样人      │  │ 📅 按月统计      │                │
│  │ 15 人            │  │ 12 个月           │                │
│  └──────────────────┘  └──────────────────┘                │
│  ┌──────────────────┐  ┌──────────────────┐                │
│  │ 📋 送样人记录    │  │ ⬇️ 导出 Excel    │                │
│  │ 逐条明细+分页     │  │ 7 Sheet 导出     │                │
│  └──────────────────┘  └──────────────────┘                │
├──────────────────────────────────────────────────────────────┤
│  点击卡片 → 展示详情区域                                       │
│                                                              │
│  「按检测类型」详情:                                          │
│  ┌──────┬──────┬──────┬────────────────────┐               │
│  │ 类型  │ 记录数│ 占比  │ 条形分布            │               │
│  ├──────┼──────┼──────┼────────────────────┤               │
│  │ ICP  │  420 │ 34%  │ ████████████░░░░    │               │
│  │ 热分析│  320 │ 26%  │ █████████░░░░░░░    │               │
│  │ ...  │      │      │                     │               │
│  └──────┴──────┴──────┴────────────────────┘               │
└──────────────────────────────────────────────────────────────┘
```

### 2.3 卡片定义（与 StatsPage 风格一致）

| key | label | 图标 | 颜色 | 数据来源 | 对应分析检测的什么 |
|------|-------|------|------|---------|-----------------|
| `status` | 按状态 | `ViewWeekIcon` | `#FF9800` | `by_status` | 类似 sheet2 |
| `by-type` | 按检测类型 | `ScienceIcon` | `#1B5E20` | `by_type` | 类似 sheet9 |
| `by-lab` | 按实验室/车间 | `BusinessIcon` | `#00796B` | `by_lab` | 类似 sheet4 |
| `by-project` | 按所属项目 | `FolderIcon` | `#FF9800` | `by_project` | 类似 sheet3 |
| `by-user` | 按送样人 | `PeopleIcon` | `#E91E63` | `by_user` | 类似 sheet5 |
| `by-month` | 按月统计 | `CalendarMonthIcon` | `#1976D2` | `by_month` | 类似 week/month |
| `user-log` | 送样人记录 | `HistoryIcon` | `#5D4037` | `getSampleInfoRecords()` 分页 | 同 user-log |
| `export` | 导出 Excel | `DownloadIcon` | `#4CAF50` | `exportSampleInfo()` | 同导出按钮 |

### 2.4 与 StatsPage 的结构对齐

```tsx
// STAT_CARDS 定义（与 StatsPage L153-L245 一致的模式）
const STAT_CARDS: StatCardDef[] = [
  { key: "status", label: "按状态", icon: <ViewWeekIcon />, color: "#FF9800", desc: "各状态记录数分布" },
  { key: "by-type", label: "按检测类型", icon: <ScienceIcon />, color: "#1B5E20", desc: "ICP/热分析/质谱" },
  { key: "by-lab", label: "按实验室/车间", icon: <BusinessIcon />, color: "#00796B", desc: "色谱/质谱/物性" },
  { key: "by-project", label: "按所属项目", icon: <FolderIcon />, color: "#FF9800", desc: "项目维度汇总" },
  { key: "by-user", label: "按送样人", icon: <PeopleIcon />, color: "#E91E63", desc: "送样人维度汇总" },
  { key: "by-month", label: "按月统计", icon: <CalendarMonthIcon />, color: "#1976D2", desc: "月度趋势" },
  { key: "user-log", label: "送样人记录", icon: <HistoryIcon />, color: "#5D4037", desc: "逐条记录明细" },
  { key: "export", label: "导出 Excel", icon: <DownloadIcon />, color: "#4CAF50", desc: "7 Sheet 导出" },
];

// 页面布局（与 StatsPage L1862-L1967 一致）
<Box>
  <Typography variant="h5" fontWeight={700}>样品信息登记统计</Typography>
  {siStats && <StatsCards summary={siStats} themeColor="#2e7d32" onCardClick={setActiveCard} />}
  <DateRangePicker startDate={s} endDate={e} onStartChange={setS} onEndChange={setE} />
  <Button onClick={exportExcel}>导出 Excel</Button>
  {activeCard ? renderDetail(activeCard) : renderCardGrid(STAT_CARDS)}
</Box>
```

### 2.5 详情区域渲染

每个卡片点击后展示对应的表格数据，与 StatsPage 的 `cct()` 函数模式一致：

| 卡片 | 表格列 | 特殊处理 |
|------|--------|---------|
| status | 状态名称 / 记录数 / 占比 | 百分比 = 记录数 / total * 100 |
| by-type | 检测类型 / 记录数 / 占比 | 取 label 字段显示 |
| by-lab | 实验室/车间 / 记录数 / 占比 | 空 lab_name 显示为「未指定」 |
| by-project | 项目名称 / 记录数 / 占比 | 空 project_name 显示为「未指定」 |
| by-user | 送样人 / 记录数 / 占比 | 空 user_name 显示为「未知」 |
| by-month | 月份 / 记录数 / 趋势 | 按月份排序 ASC |
| user-log | 同当前 SampleInfoEntry 记录列表 | 支持状态筛选/类型筛选/送样人筛选 + 分页 + 内联编辑 |
| export | 直接触发文件下载 | 调用 exportSampleInfo() |

### 2.6 数据加载模式（与 StatsPage L355-L411 一致）

```tsx
const loadDetail = useCallback(async () => {
  if (!activeCard) return;
  setLoading(true);
  try {
    if (activeCard === "user-log") {
      await loadUserLog(); // 分页加载记录列表
    } else if (activeCard === "status") {
      // 使用 siStats.by_status
    } else if (activeCard === "by-type") {
      // 使用 siStats.by_type
    }
    // ... 其他卡片使用对应数据
  } catch {
    setError("加载失败");
  } finally {
    setLoading(false);
  }
}, [activeCard, siStats]);
```

### 2.7 门户入口

在 `SampleInfoHome.tsx` 顶部增加「查看统计」按钮，与 WorkloadPortal L36-L39 一致：

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

新增路由：
```tsx
<Route path="/sample-info/stats" element={<SampleInfoStatsPage />} />
```

---

## 三、统计页实现文件清单

| 文件 | 改动类型 | 说明 |
|------|---------|------|
| **新建** `frontend/src/pages/SampleInfoStatsPage.tsx` | 新建 | 完整复刻 StatsPage 模式，~800 行 |
| **修改** `frontend/src/pages/SampleInfoHome.tsx` | 编辑 | 增加「查看统计」按钮 |
| **修改** `frontend/src/App.tsx` | 编辑 | 注册 `/sample-info/stats` 路由 |
| 后端 | 无需改动 | 现有接口已满足 |

---

## 四、录入表单列自定义管理方案

### 4.1 需求分析

用户希望能在管理页面像 Excel 一样**自定义编辑**样品信息登记表单的列，包括：
- ✅ 新增列（添加新的字段）
- ✅ 删除列（隐藏不需要的字段）
- ✅ 重排列顺序（拖拽或排序号）
- ✅ 重命名列名（显示名称可自定义）
- ⚠️ 列类型（文本/数字/选择/日期）

### 4.2 架构设计

```
┌──────────────────────────────────────────┐
│           前端层                           │
│                                           │
│  ManagePage                               │
│  ┌───────────────────────────────┐        │
│  │ 列管理编辑器                    │        │
│  │ ┌────┬──────┬────┬────┬────┐  │        │
│  │ │序号│字段名 │显示名│类型│启用│  │        │
│  │ │ 1 │user_nm│送样人│文本│ ✓ │  │        │
│  │ │ 2 │div_id│部门  │选择│ ✓ │  │        │
│  │ │ 3 │lab_nm│实验室│文本│ ✓ │  │        │
│  │ │(可拖拽排序)      │    │   │  │        │
│  │ └────┴──────┴────┴────┴────┘  │        │
│  │ [+ 新增列]                     │        │
│  └───────────────────────────────┘        │
│                                           │
│  SampleInfoEntry                           │
│  ┌───────────────────────────────┐        │
│  │ 动态渲染列头 + 单元格           │        │
│  │ 根据列配置表生成 TableHeader    │        │
│  │ 动态绑定 row[key] 到 TextField │        │
│  └───────────────────────────────┘        │
│                                           │
│  API Client                               │
│  ┌───────────────────────────────┐        │
│  │ GET /sample-info/columns     │ 获取配置 │
│  │ POST/PUT/DELETE              │ CRUD   │
│  │ PUT /sample-info/columns/sort│ 排序    │
│  └───────────────────────────────┘        │
└──────────────────────────────────────────┘
         │ API
         ▼
┌──────────────────────────────────────────┐
│           后端层                           │
│                                           │
│  DB: sample_info_columns 表               │
│  ┌──────────┬─────────┬─────────┬─────┐  │
│  │ field_key│ label   │ col_type│sort │  │
│  ├──────────┼─────────┼─────────┼─────┤  │
│  │user_name │ 送样人   │ text    │  1  │  │
│  │lab_name  │实验室/..│ text    │  3  │  │
│  │project_nm│所属项目  │ text    │  4  │  │
│  │quantity  │送样数量  │ number  │  5  │  │
│  │extra_col1│自定义1   │ text    │ 10  │  │
│  └──────────┴─────────┴─────────┴─────┘  │
│                                           │
│  Handler: /api/sample-info/columns        │
│  - GET: 返回所有列配置（按 sort_order 排序）│
│  - POST: 添加新列                          │
│  - PUT /:id: 更新列配置                    │
│  - DELETE /:id: 删除列                    │
│  - PUT /sort: 批量更新排序                 │
└──────────────────────────────────────────┘
```

### 4.3 数据库设计

```sql
CREATE TABLE sample_info_columns (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  field_key TEXT NOT NULL UNIQUE,      -- 字段标识（用于代码引用）
  label TEXT NOT NULL,                 -- 显示名称（可自定义）
  col_type TEXT NOT NULL DEFAULT 'text', -- 列类型：text/select/date/number
  is_required INTEGER DEFAULT 0,       -- 是否必填
  is_active INTEGER DEFAULT 1,         -- 是否启用
  width INTEGER DEFAULT 100,           -- 列宽（px）
  sort_order INTEGER DEFAULT 0,        -- 排序序号
  options TEXT,                        -- 选择型列的可选项（JSON 数组）
  created_at TEXT DEFAULT (datetime('now','localtime')),
  updated_at TEXT DEFAULT (datetime('now','localtime'))
);
```

**预置数据**（与当前硬编码结构对应）：

| field_key | label | col_type | is_required | sort_order | 对应当前列 |
|-----------|-------|---------|-------------|-----------|-----------|
| user_name | 送样人 | text | 0 | 1 | L248 |
| division_id | 所属部门 | select | 0 | 2 | L249 |
| project_name | 所属项目 | text | 0 | 3 | L250 |
| lab_name | 实验室/车间 | text | 0 | 4 | ← 新增 |
| quantity | 送样数量 | number | 0 | 5 | L251 |
| batch_no | 样品批号 | text | 1 | 6 | L252 |
| main_components | 样品主要成分 | text | 1 | 7 | L253 |
| notes | 注意事项 | text | 0 | 8 | L254 |

### 4.4 后端改动

| 文件 | 改动 |
|------|------|
| `src/models/sample_info_column.rs` | **新建** — 定义 ColumnCreate/ColumnUpdate/ColumnResponse |
| `src/repo/sample_info_column_repo.rs` | **新建** — DAO 层 CRUD |
| `src/api/sample_info_column_handler.rs` | **新建** — 5 个 API 端点 |
| `src/db/migrations.rs` | 增加 `sample_info_columns` 建表 + 预置数据 |
| `src/api/api.rs` | 注册 `/api/sample-info/columns` 路由 |
| `src/models/sample_info.rs` | 新增 `extra_fields: Option<serde_json::Value>` 存储动态字段值 |

### 4.5 前端改动

| 文件 | 改动 |
|------|------|
| `frontend/src/api/client.ts` | 新增 `getSampleInfoColumns()` / `createColumn()` / `updateColumn()` / `deleteColumn()` / `reorderColumns()` |
| `frontend/src/types/index.ts` | 新增 `SampleInfoColumn` 类型 |
| `frontend/src/pages/ManagePage.tsx` | `sampleinfo` tab 增加「④ 列管理」区块 |
| `frontend/src/pages/SampleInfoEntry.tsx` | 从硬编码改动态渲染 |

### 4.6 前端动态渲染方案

`SampleInfoEntry.tsx` 的改动核心：

```tsx
// 状态：从 API 获取列配置
const [columns, setColumns] = useState<SampleInfoColumn[]>([]);

// 加载列配置
useEffect(() => {
  getSampleInfoColumns().then(r => {
    if (r.code === 0 && r.data) setColumns(r.data.filter(c => c.is_active).sort((a,b) => a.sort_order - b.sort_order));
  });
}, []);

// RowData 改为动态对象（不再硬编码接口）
type RowData = Record<string, any>;
const emptyRow = (): RowData => {
  const row: RowData = { checked: false };
  columns.forEach(col => {
    row[col.field_key] = col.col_type === 'number' ? 1 : '';
  });
  return row;
};

// 动态渲染表格
<TableHead>
  <TableRow>
    <TableCell><Checkbox /></TableCell>
    <TableCell>序号</TableCell>
    {columns.map(col => (
      <TableCell key={col.field_key} sx={{ fontWeight: 700, minWidth: col.width }}>
        {col.is_required ? `${col.label} *` : col.label}
      </TableCell>
    ))}
  </TableRow>
</TableHead>

// 动态提交
const doSubmit = async () => {
  for (const row of rows) {
    const payload: Record<string, any> = {};
    columns.forEach(col => {
      if (col.field_key === 'division_id') {
        payload[col.field_key] = row[col.field_key] || null;
      } else if (col.col_type === 'number') {
        payload[col.field_key] = row[col.field_key] || 1;
      } else {
        payload[col.field_key] = row[col.field_key] || '';
      }
    });
    // 固定字段
    payload.type_key = dt;
    payload.detection_type = typeLabel;
    
    // 收集动态字段到 extra_fields
    const extraFields: Record<string, any> = {};
    const predefinedFields = ['batch_no', 'user_name', 'lab_name', 'project_name', 'submitted_at', 'detection_date', 'main_components', 'type_key', 'division_id', 'quantity', 'notes'];
    Object.keys(payload).forEach(key => {
      if (!predefinedFields.includes(key)) {
        extraFields[key] = payload[key];
        delete payload[key];
      }
    });
    if (Object.keys(extraFields).length > 0) {
      payload.extra_fields = JSON.stringify(extraFields);
    }
    
    await createSampleInfo(payload);
  }
};
```

### 4.7 管理页列编辑 UI

```
┌─── ④ 列管理 ──────────────────────────────────────────┐
│                                                        │
│  [+ 新增列]                                             │
│                                                        │
│  ┌───┬──────┬─────────┬──────┬────┬────┬───────────┐  │
│  │ # │字段名 │ 显示名称 │ 类型  │宽(px)│启用│ 操作      │  │
│  ├───┼──────┼─────────┼──────┼────┼────┼───────────┤  │
│  │ 1 │user..│ 送样人   │文本   │100 │ ✓  │ [✏️][🗑️] │  │
│  │ 2 │lab_..│实验室/..│文本   │100 │ ✓  │ [✏️][🗑️] │  │
│  │ 3 │proj..│所属项目  │文本   │100 │ ✓  │ [✏️][🗑️] │  │
│  │ 4 │....  │        │      │    │    │           │  │
│  └───┴──────┴─────────┴──────┴────┴────┴───────────┘  │
│                                                        │
│  (点击 ✏️ 弹出编辑框，可修改显示名称/类型/宽度/必填)      │
│  (点击 🗑️ 删除列，会影响已有数据)                        │
│  (拖拽或上下箭头改变排序)                                │
└────────────────────────────────────────────────────────┘
```

### 4.8 动态字段存储方案

**关键决策**：新增的列数据存储在哪？

| 方案 | 描述 | 优缺点 |
|------|------|--------|
| **方案 A: JSON 扩展字段** ✅ | `sample_info_records` 表加 `extra_fields TEXT` 列，存储 `{"custom_col1": "value1", ...}` | ✅ 灵活，新增列无需改表结构；⚠️ 查询效率较低 |
| **方案 B: EAV (Entity-Attribute-Value)** | 新建 `sample_info_record_values` 表，每行一个字段值 | ❌ 查询极其复杂，不适合出统计 |
| **方案 C: 动态加列** | 新增列时执行 `ALTER TABLE` 加列 | ❌ SQLite ALTER TABLE 限制多 |
| **方案 D: 仅前端表名列配置** | 只配置列的显隐/顺序/显示名，不新增额外字段 | ✅ 最简单；⚠️ 无法新增「不存在」的字段 |

**推荐方案 D**（初期）+ **方案 A**（长期）：
- **一期（推荐）**：只做列的显隐、排序、显示名自定义，不新增额外数据字段。现有字段已覆盖用户需求（送样人/部门/项目/实验室/数量/批号/成分/注意事项等 8 个字段）。
- **二期（未来）**：如果需要新增额外自定义字段，再加 `extra_fields` JSON 列。

---

## 五、实现顺序建议

### 第一期（统计页复刻）

| 步骤 | 内容 | 预估工时 |
|------|------|---------|
| 1 | 新建 `SampleInfoStatsPage.tsx` | ~60 分钟 |
| 2 | `SampleInfoHome.tsx` 增加「查看统计」按钮 | ~5 分钟 |
| 3 | `App.tsx` 注册 `/sample-info/stats` 路由 | ~2 分钟 |
| 4 | 编译验证 + 测试 | ~10 分钟 |
| **合计** | | **~1.3 小时** |

### 第二期（列自定义管理）

| 步骤 | 内容 | 依赖 | 预估工时 |
|------|------|------|---------|
| 1 | 数据库 migration + 预置数据 | 无 | ~15 分钟 |
| 2 | 后端 model + repo + handler (CRUD) | 1 | ~40 分钟 |
| 3 | 注册路由 + api.rs | 2 | ~5 分钟 |
| 4 | 前端 types + API client | 2 | ~10 分钟 |
| 5 | ManagePage 增加列管理 UI | 3,4 | ~40 分钟 |
| 6 | SampleInfoEntry 改为动态渲染 | 1-5 | ~60 分钟 |
| 7 | 编译验证 + 测试 | 6 | ~15 分钟 |
| **合计** | | | **~3 小时** |

---

## 六、收益与风险

### 统计页复刻
- ✅ **收益**：用户获得与分析检测一致的统计体验
- ⚠️ **风险**：低——现有后端接口完全满足，纯前端改动

### 列自定义管理
- ✅ **收益**：极大增强灵活性，用户可自定义录入表单
- ⚠️ **风险**：
  - 动态列渲染 + 动态提交较复杂
  - 新增「extra_fields」字段后，统计/导出需相应处理
  - 与现有管理页的记录查询/MUI 编辑区的列名也需要同步

---

## 七、待确认事项

- ❓ **统计页**：是否要显示「按检测类型/按实验室等维度的条形图/柱状图」，还是纯表格即可？
- ❓ **列自定义**：
  - 第一期只做列显隐/排序/重命名，不做新增额外字段，OK吗？
  - 列自定义是否也影响到管理页记录查询表格的列？
  - 是否影响导出 Excel 的列？
