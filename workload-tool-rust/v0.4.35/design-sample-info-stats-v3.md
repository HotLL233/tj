# 样品信息登记 — 统计页复刻 + 列纯自定义（同步导出/查询）方案

> 生成日期：2026-07-10
> 本次仅出方案，不动代码

---

## 一、需求理解

| 需求 | 详细说明 |
|------|---------|
| 统计页复刻 | 按分析检测 StatsPage.tsx 的完整模式（STAT_CARDS + 摘要卡片 + DateRangePicker + 详情表格），主题绿色 |
| 列纯自定义 | 不是简单的显隐/排序，而是 **完整 CRUD**：新增列/删除列/重命名/排序/改类型，所有操作从管理页面完成 |
| 同步导出 | 列自定义必须同步到 `/api/sample-info/export` 的 7 个 Sheet（列头/数据列一一对应） |
| 同步查询 | 列自定义必须同步到 **所有记录查询展示**（SampleInfoEntry 列表、ManagePage 查询、门户记录列表） |

---

## 二、代码检查结论

### 2.1 哪些是硬编码的（需要改）

| 层级 | 文件 | 硬编码内容 | 影响 |
|------|------|-----------|------|
| 数据库 | `migrations.rs` | `sample_info_records` 表结构固定 | 新增的列需用 `extra_fields` JSON 存 |
| 后端模型 | `models/sample_info.rs` | `SampleInfoRecord` / `Create / Update / Response` 结构体 | 需加 `extra_fields: Option<serde_json::Value>` |
| 后端查询 | `repo/sample_info_repo.rs` | `SELECT` 列列表、`build_where` 只写固定字段 | 需支持从 extra_fields 动态查询 |
| 后端导出 | `export_data.rs` | `SampleInfoExportRow` 结构体、`SELECT` SQL 固定 | 需动态读取 extra_fields |
| 后端导出 | `export_write.rs` | `write_sheet_detail` 列头和列宽硬编码 | 需动态读取列配置表写列 |
| 后端统计 | `handler.rs` | `stats()` SQL 仅对固定字段 GROUP BY | 自定义列不可统计 |
| 前端样本 | `SampleInfoEntry.tsx` | 列头/单元格/RowData/submit 全部硬编码 | 需动态渲染 |
| 前端管理 | `ManagePage.tsx` | 记录查询表格列硬编码 | 需动态渲染 |
| 前端类型 | `types/index.ts` | `SampleInfoRecord` 类型固定 | 需加 `extra_fields` |

### 2.2 改动范围总图

```
                           ┌───────────────┐
                           │ sample_info_   │
                           │ columns DB 表  │  ← 新建
                           └───────┬───────┘
                                   │
              ┌────────────────────┼────────────────────┐
              │                    │                    │
              ▼                    ▼                    ▼
      ┌──────────────┐    ┌──────────────┐    ┌──────────────┐
      │  录入表单      │    │  查询/列表    │    │  导出 Excel   │
      │SampleInfoEntry│    │ repo.list()  │    │export_data/  │
      │  动态渲染列    │    │ ManagePage   │    │ write       │
      └──────────────┘    └──────────────┘    └──────────────┘
              │                    │                    │
              ▼                    ▼                    ▼
      ┌──────────────┐    ┌──────────────┐    ┌──────────────┐
      │  extra_fields │    │  extra_fields │    │  extra_fields │
      │  JSON 写入    │    │  JSON 读取    │    │  JSON 读取+导出│
      └──────────────┘    └──────────────┘    └──────────────┘
                                   │
                                   ▼
                          ┌──────────────┐
                          │ sample_info_ │
                          │ records 表   │
                          │ (新增extra_  │
                          │ fields 列)   │
                          └──────────────┘
```

---

## 三、列纯自定义 — 完整设计

### 3.1 数据库设计

**新建表：`sample_info_columns`**

```sql
CREATE TABLE sample_info_columns (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  field_key TEXT NOT NULL UNIQUE,       -- 字段标识，内置字段固定、自定义字段用 'custom_xx' 格式
  label TEXT NOT NULL,                  -- 显示名称（用户可自定义）
  data_type TEXT NOT NULL DEFAULT 'text', -- 数据类型：text/number/select/date
  is_predefined INTEGER NOT NULL DEFAULT 0,  -- 1=内置字段不可删，0=自定义可删
  is_required INTEGER NOT NULL DEFAULT 0,    -- 是否必填
  is_active INTEGER NOT NULL DEFAULT 1,      -- 是否启用（不启用 = 隐藏）
  width INTEGER DEFAULT 100,                 -- 列宽（px）
  sort_order INTEGER DEFAULT 0,              -- 排序序号
  options TEXT,                              -- select 型的选项列表（JSON 数组 ["A","B"]）
  show_in_list INTEGER NOT NULL DEFAULT 1,   -- 在记录列表显示
  show_in_export INTEGER NOT NULL DEFAULT 1, -- 在导出中显示
  show_in_form INTEGER NOT NULL DEFAULT 1,   -- 在录入表单显示
  created_at TEXT NOT NULL DEFAULT (datetime('now','localtime')),
  updated_at TEXT DEFAULT (datetime('now','localtime'))
);
```

**预置数据**（`is_predefined=1` 的内置字段）：

| field_key | label | data_type | is_required | sort_order | 说明 |
|-----------|-------|-----------|-------------|-----------|------|
| seq_no | 序号 | number | 0 | 0 | 自动生成，只读 |
| user_name | 送样人 | text | 0 | 1 | 默认值「未知」 |
| division_id | 所属部门 | select | 0 | 2 | 从 divisions 表取 |
| lab_name | 实验室/车间 | text | 0 | 3 | ← 新增列 |
| project_name | 所属项目 | text | 0 | 4 | |
| quantity | 送样数量 | number | 0 | 5 | 默认 1 |
| batch_no | 样品批号 | text | 1 | 6 | 必填 |
| main_components | 样品主要成分 | text | 1 | 7 | 必填 |
| notes | 注意事项 | text | 0 | 8 | |
| submitted_at | 送样时间 | date | 0 | 9 | 整单公共 |
| detection_type | 检测类型 | text | 0 | 10 | 从 type_key 映射 |
| status | 状态 | text | 0 | 11 | 待检测/待取样/已取样/检测完成 |

**已有表改造：`sample_info_records` 加列**

```sql
ALTER TABLE sample_info_records ADD COLUMN extra_fields TEXT DEFAULT '{}';
```

### 3.2 后端改动

#### 3.2.1 新建模型：`models/sample_info_column.rs`

```rust
pub struct SampleInfoColumn { /* 所有 DB 列 */ }
pub struct ColumnCreate { field_key, label, data_type, options, ... }
pub struct ColumnUpdate { label, data_type, options, is_active, width, sort_order, show_in_list, show_in_export, show_in_form }
pub struct ColumnResponse { /* 返回给前端 */ }
```

#### 3.2.2 新建仓库：`repo/sample_info_column_repo.rs`

```rust
pub fn list_all(pool) -> Result<Vec<SampleInfoColumn>>  // 按 sort_order 排序
pub fn get_active(pool) -> Result<Vec<SampleInfoColumn>> // 仅 is_active=1
pub fn create(pool, data) -> Result<...>
pub fn update(pool, id, data) -> Result<...>
pub fn soft_delete(pool, id) -> Result<...>  // is_predefined=1 不可删
pub fn reorder(pool, ids: Vec<(id, sort_order)>) -> Result<...>
```

#### 3.2.3 新建处理层：`api/sample_info_column_handler.rs`

```rust
GET    /api/sample-info/columns         -> list (全部,含预置)
GET    /api/sample-info/columns/active  -> list (仅启用的)
POST   /api/sample-info/columns         -> create
PUT    /api/sample-info/columns/:id     -> update
DELETE /api/sample-info/columns/:id     -> delete
PUT    /api/sample-info/columns/sort    -> reorder
```

#### 3.2.4 现有模型改造：`models/sample_info.rs`

```rust
// SampleInfoRecord 加 extra_fields
pub extra_fields: Option<String>,  // JSON 字符串

// SampleInfoCreate 加 extra_fields
pub extra_fields: Option<HashMap<String, serde_json::Value>>,

// SampleInfoUpdate 加 extra_fields
pub extra_fields: Option<HashMap<String, serde_json::Value>>,
```

#### 3.2.5 现有仓库改造：`repo/sample_info_repo.rs`

**`build_where`** 函数需要：
- 从 `sample_info_columns` 表读取活跃列配置
- 对于自定义列：构建 `JSON_EXTRACT(extra_fields, '$.field_key')=?N` 条件
- 对于内置列：保持现有逻辑

**`list` 函数需要**：
- SELECT 加 `extra_fields` 列
- `SampleInfoRecord` 构造加 `extra_fields`

**`create` 函数需要**：
- INSERT 加 `extra_fields` 参数
- 读取列配置，验证必填列
- 自定义列的值写入 `extra_fields` JSON

**`update` 函数需要**：
- 读取列配置，对自定义字段更新 `extra_fields`
- `UPDATE` SQL 加 `extra_fields=?`

**`get_by_id_on_conn` 函数需要**：
- SELECT 加 `extra_fields`

#### 3.2.6 导出链路改造

**`export_data.rs`**：
- `query_detail`：需动态读取列配置，对自定义列 `JSON_EXTRACT(extra_fields, '$.key')`
- 不能再用固定的 `SampleInfoExportRow` 结构体，需改为动态 `Vec<HashMap<String, Value>>`

**`export_write.rs`**：
- `write_sheet_detail`：不能再用固定 header 数组，需从列配置读取列名列表
- 列宽从 `sample_info_columns.width` 读取

**核心设计**：导出函数接收列配置 `&[SampleInfoColumn]` 作为参数：

```rust
pub fn write_sheet_detail(
    ws: &mut Worksheet,
    rows: &[HashMap<String, serde_json::Value>],
    columns: &[SampleInfoColumn],
    fmt: &Fmt,
) -> Result<()> {
    // 只写 show_in_export=1 的列
    let export_cols: Vec<&SampleInfoColumn> = columns.iter()
        .filter(|c| c.show_in_export)
        .collect();
    // 写表头
    for (i, col) in export_cols.iter().enumerate() {
        ws.write_with_format(0, i as u16, &col.label, &fmt.fh)?;
        ws.set_column_width(i as u16, col.width as f64 / 7.5)?;  // px → 字符宽
    }
    // 写数据
    for (ri, row) in rows.iter().enumerate() {
        for (ci, col) in export_cols.iter().enumerate() {
            let val = row.get(&col.field_key);
            // 写入单元格
        }
    }
}
```

**`export_handler.rs`**：
- `export_excel` 函数需要加载列配置并传递给 `export_data` 和 `export_write`

### 3.3 前端改动

#### 3.3.1 类型系统

```typescript
// types/index.ts
export interface SampleInfoColumn {
  id: number;
  field_key: string;
  label: string;
  data_type: 'text' | 'number' | 'select' | 'date';
  is_predefined: boolean;
  is_required: boolean;
  is_active: boolean;
  width: number;
  sort_order: number;
  options?: string[];  // select 型
  show_in_list: boolean;
  show_in_export: boolean;
  show_in_form: boolean;
}

// SampleInfoRecord 加 extra_fields
export interface SampleInfoRecord {
  // ... 现有字段
  extra_fields?: Record<string, any>;
}
```

#### 3.3.2 API Client

```typescript
// api/client.ts 新增
export const getSampleInfoColumns = () => client.get('/sample-info/columns').then(r => r.data);
export const getActiveSampleInfoColumns = () => client.get('/sample-info/columns/active').then(r => r.data);
export const createSampleInfoColumn = (data) => client.post('/sample-info/columns', data).then(r => r.data);
export const updateSampleInfoColumn = (id, data) => client.put(`/sample-info/columns/${id}`, data).then(r => r.data);
export const deleteSampleInfoColumn = (id) => client.delete(`/sample-info/columns/${id}`).then(r => r.data);
export const reorderSampleInfoColumns = (ids: { id: number; sort_order: number }[]) =>
  client.put('/sample-info/columns/sort', { ids }).then(r => r.data);
```

#### 3.3.3 ManagePage — 列编辑器

在 `ManagePage.tsx` 的 `sampleinfo` tab 新增「④ 列管理」区块：

```
┌─── ④ 列管理 ────────────────────────────────────────────────┐
│  [+ 新增列]                                                  │
│                                                              │
│  ┌───┬──────┬───────┬────┬────┬──────┬────┬────┬────┬────┐  │
│  │ # │标识  │显示名 │类型│必填│启用  │宽度│列表│导出│表单│  │
│  ├───┼──────┼───────┼────┼────┼──────┼────┼────┼────┼────┤  │
│  │ 1 │user..│送样人 │文本│    │ ✓   │100 │ ✓  │ ✓  │ ✓  │  │
│  │ 2 │lab..│实验室  │文本│    │ ✓   │100 │ ✓  │ ✓  │ ✓  │  │
│  │ 3 │自定义1│测试项 │文本│    │ ✓   │100 │ ✓  │ ✓  │ ✓  │  │← 可删除
│  │ 4 │batch│批号   │文本│✓必填│ ✓   │130 │ ✓  │ ✓  │ ✓  │  │
│  └───┴──────┴───────┴────┴────┴──────┴────┴────┴────┴────┘  │
│  (预置字段不可删除，只有自定义列可删除)                          │
│  (点击操作列 ✏️ 弹出编辑框)                                    │
└──────────────────────────────────────────────────────────────┘
```

**编辑弹窗字段：**
- 字段标识（只读，新建时输入）
- 显示名称（可编辑）
- 数据类型（text/number/select/date — 新建时选定后不可改）
- 必填（toggle）
- 启用（toggle）
- 宽度（数字输入）
- 显示位置（复选框：录入表单/记录列表/导出Excel）
- 如果是 select 类型：选项列表（可增删的 Chip 输入）

**新增列流程：**
1. 用户点击「+ 新增列」
2. 输入字段标识（field_key，例如 `custom_test_item`）
3. 选择数据类型（text/number/select/date）
4. 输入显示名称
5. 点击保存 → POST /api/sample-info/columns
6. 后端创建记录（is_predefined=0）
7. 列出现在录入表单、列表、导出中（根据 show_in_* 设置）

#### 3.3.4 SampleInfoEntry — 动态渲染

**核心改造**：从 `getActiveSampleInfoColumns()` 获取列配置，动态渲染：

```typescript
// 组件加载时
const [columns, setColumns] = useState<SampleInfoColumn[]>([]);
useEffect(() => {
  getActiveSampleInfoColumns().then(r => {
    if (r.code === 0 && r.data) {
      // 按 sort_order 排序
      setColumns(r.data.sort((a,b) => a.sort_order - b.sort_order));
    }
  });
}, []);

// RowData 改为动态
type RowData = Record<string, any>;
const emptyRow = (): RowData => {
  const row: RowData = { checked: false, _id: Date.now() };
  columns.forEach(col => {
    if (col.is_predefined && col.field_key === 'division_id') {
      row[col.field_key] = '';
    } else if (col.data_type === 'number') {
      row[col.field_key] = 1;
    } else {
      row[col.field_key] = '';
    }
  });
  return row;
};

// 动态表头
<TableHead>
  <TableRow>
    <TableCell><Checkbox /></TableCell>
    <TableCell>序号</TableCell>
    {columns.filter(c => c.show_in_form).map(col => (
      <TableCell key={col.field_key} sx={{ fontWeight: 700, minWidth: col.width }}>
        {col.is_required ? `${col.label} *` : col.label}
      </TableCell>
    ))}
  </TableRow>
</TableHead>

// 动态行单元格
{rows.map((row, idx) => (
  <TableRow key={row._id}>
    <TableCell><Checkbox /></TableCell>
    <TableCell>{idx + 1}</TableCell>
    {columns.filter(c => c.show_in_form).map(col => (
      <TableCell key={col.field_key}>
        {col.data_type === 'select' ? (
          <Select value={row[col.field_key]} onChange={...}>
            <MenuItem value="">请选择</MenuItem>
            {col.options?.map(opt => <MenuItem key={opt} value={opt}>{opt}</MenuItem>)}
          </Select>
        ) : col.data_type === 'number' ? (
          <TextField type="number" value={row[col.field_key]} onChange={...} />
        ) : (
          <TextField value={row[col.field_key]} onChange={...} placeholder={col.label} />
        )}
      </TableCell>
    ))}
  </TableRow>
))}

// 动态提交：区分内置字段和自定义字段
const doSubmit = async () => {
  for (const row of rows) {
    const predefinedFields: Record<string, any> = {};
    const extraFields: Record<string, any> = {};
    
    for (const col of columns) {
      const val = row[col.field_key];
      if (col.is_predefined) {
        predefinedFields[col.field_key] = col.field_key === 'division_id' ? (val || null) : (val || '');
      } else {
        extraFields[col.field_key] = val;
      }
    }
    
    await createSampleInfo({
      ...predefinedFields,
      extra_fields: Object.keys(extraFields).length > 0 ? extraFields : undefined,
      type_key: dt,
      detection_type: typeLabel,
    });
  }
};
```

#### 3.3.5 ManagePage 查询 — 动态列

```typescript
// 管理页记录查询表格改为动态列
<TableHead>
  <TableRow>
    <TableCell>序号</TableCell>
    {columns.filter(c => c.show_in_list).map(col => (
      <TableCell key={col.field_key}>{col.label}</TableCell>
    ))}
    <TableCell align="right">操作</TableCell>
  </TableRow>
</TableHead>
```

#### 3.3.6 SampleInfoHome/Entry 列表 — 动态列

SampleInfoEntry 底部的记录列表部分同样改为动态渲染，使用 `columns.filter(c => c.show_in_list)`。

---

## 四、统计页复刻 — 完整设计

### 4.1 页面布局

与第二版方案一致，8 个 STAT_CARDS：
- 按状态 / 按检测类型 / 按实验室/车间 / 按所属项目
- 按送样人 / 按月统计 / 送样人记录 / 导出 Excel

**特别注意**：自定义列的数据类型为 text/number/select/date，name-count 统计维度不适用于自定义列（GROUP BY 无意义）。因此 8 个统计维度的数据来源仍然是 `by_status / by_type / by_lab / by_project / by_user / by_month` 这 6 个预定义统计接口。

### 4.2 与列自定义的联动

| 统计维度 | 是否受列自定义影响 | 原因 |
|---------|------------------|------|
| 按状态 | ❌ 不受影响 | 状态是内置字段 |
| 按检测类型 | ❌ 不受影响 | detection_type 是内置字段 |
| 按实验室/车间 | ❌ 不受影响 | lab_name 是内置字段 |
| 按所属项目 | ❌ 不受影响 | project_name 是内置字段 |
| 按送样人 | ❌ 不受影响 | user_name 是内置字段 |
| 按月统计 | ❌ 不受影响 | submitted_at 是内置字段 |
| 送样人记录 | ✅ 受影响 | 列表列跟随 show_in_list |
| 导出 Excel | ✅ 受影响 | 列跟随 show_in_export |

### 4.3 实现文件

| 文件 | 改动类型 |
|------|---------|
| `frontend/src/pages/SampleInfoStatsPage.tsx` | **新建** — 完整复刻 StatsPage 模式 |
| `frontend/src/pages/SampleInfoHome.tsx` | 编辑 — 加「查看统计」按钮 |
| `frontend/src/App.tsx` | 编辑 — 加 `/sample-info/stats` 路由 |

**后端无改动** — 复用现有 `/api/sample-info/stats` 和 `/api/sample-info` 接口。

---

## 五、实现顺序与依赖图

```
一期：列自定义基础设施（~3-4 小时）
┌─────────────────────────────────────────────────────────────┐
│ 1. 后端 migration: sample_info_columns 建表 + 预置数据      │
│ 2. 后端 migration: sample_info_records 加 extra_fields 列  │
│ 3. 后端 model: SampleInfoColumn + SampleInfoRecord 改      │
│ 4. 后端 repo: sample_info_column_repo CRUD                 │
│ 5. 后端 repo: sample_info_repo 改造（list/create/update）  │
│ 6. 后端 api: column_handler + 路由注册                     │
│ 7. 前端 types + API client (列配置)                        │
│ 8. ManagePage: 列管理编辑器 UI                              │
└─────────────────────────────────────────────────────────────┘

二期：列自定义消费端（~3-4 小时）
┌─────────────────────────────────────────────────────────────┐
│ 9. SampleInfoEntry: 动态渲染录入表单                        │
│ 10. SampleInfoEntry: 动态提交（区分内置/自定义字段）        │
│ 11. SampleInfoEntry: 记录列表动态列                         │
│ 12. ManagePage: 记录查询动态列                              │
└─────────────────────────────────────────────────────────────┘

三期：列自定义导出同步（~2 小时）
┌─────────────────────────────────────────────────────────────┐
│ 13. export_data.rs: 动态列查询（JSON_EXTRACT）              │
│ 14. export_write.rs: 动态列头 + 动态列宽                    │
│ 15. export_handler.rs: 传递列配置                           │
│ 16. 前端 export 触发拉取列配置                              │
└─────────────────────────────────────────────────────────────┘

四期：统计页复刻（~1.5 小时）
┌─────────────────────────────────────────────────────────────┐
│ 17. SampleInfoStatsPage.tsx 新建（完整复刻 StatsPage）      │
│ 18. SampleInfoHome.tsx 加统计入口                           │
│ 19. App.tsx 加路由                                          │
└─────────────────────────────────────────────────────────────┘

总预估工时：~10 小时
```

---

## 六、文件清单（完整）

### 新建文件（6 个）

| 路径 | 说明 |
|------|------|
| `src/models/sample_info_column.rs` | 列配置模型 |
| `src/repo/sample_info_column_repo.rs` | 列配置 DAO |
| `src/api/sample_info_column_handler.rs` | 列配置 API |
| `frontend/src/pages/SampleInfoStatsPage.tsx` | 统计页 |
| 前端*列编辑弹窗组件* | 可内嵌到 ManagePage 或独立组件 |

### 修改文件（17 个）

| 路径 | 改动说明 |
|------|---------|
| `src/db/migrations.rs` | 新建 `sample_info_columns` 表 + 预置数据 + `extra_fields` 列 |
| `src/models/sample_info.rs` | `Record/Create/Update/Response` 加 `extra_fields` |
| `src/repo/sample_info_repo.rs` | `build_where/list/create/update/get_by_id_on_conn` 改 |
| `src/api/sample_info_handler.rs` | `stats_where` 不改（统计维度固定），`list` handler 传递 extra_fields |
| `src/api/sample_info_export_data.rs` | 动态列查询 |
| `src/api/sample_info_export_write.rs` | 动态列头+列宽 |
| `src/api/sample_info_export_handler.rs` | 传递列配置参数 |
| `src/api/mod.rs` | 注册新 route |
| `frontend/src/types/index.ts` | 加 `SampleInfoColumn` 类型，`SampleInfoRecord` 加 `extra_fields` |
| `frontend/src/api/client.ts` | 加列配置 API + 改 createSampleInfo payload |
| `frontend/src/pages/SampleInfoEntry.tsx` | 动态渲染录入表单 + 动态提交 + 动态列表 |
| `frontend/src/pages/ManagePage.tsx` | 加「列管理」区块 + 记录查询动态列 |
| `frontend/src/pages/SampleInfoHome.tsx` | 加统计入口按钮 |
| `frontend/src/App.tsx` | 加统计路由 |
| `frontend/src/components/` | 可选：列编辑弹窗组件 |

**总计：6 新建 + 17 修改 = 23 个文件**

---

## 七、风险与注意事项

### 7.1 数据一致性风险

- **自定义列被删除后，已有数据的 extra_fields 中该 key 残留** → 查询时忽略即可，不影响功能
- **列类型不可改**（新建时选定后不可再改）→ 避免数据格式冲突
- **select 型列的 options 修改后，已有数据值不变** → 可显示旧值，Options 只影响新增行

### 7.2 性能风险

- `JSON_EXTRACT` 在 SQLite 中性能较差，大表（10万+行）自定义列筛选会变慢
- **解决方案**：自定义列不做 WHERE 条件（仅在 `show_in_list` 展示），可由用户决定是否允许自定义列筛选

### 7.3 前端复杂度

- 动态渲染的列头/单元格 + 动态提交逻辑较复杂
- 编辑弹窗的编辑模式（只读/编辑行）需要适配动态字段
- 状态流转按钮（待检测→待取样→已取样→检测完成）不受列自定义影响

---

## 八、待确认事项

1. **自定义列是否允许在统计页作为筛选条件？** 还是仅展示？
2. **列类型**：text / number / select / date 四种已够用，还需要其他类型吗？
3. **预置字段中**：`seq_no`（序号）和 `status`（状态）是否允许隐藏（show_in_form=0）？
4. **自定义列的查询筛选**：是否需要在记录列表的顶部筛选区动态显示自定义列的筛选框？
5. **统计页**：8 个维度够了吗？需要额外增加什么维度吗？
