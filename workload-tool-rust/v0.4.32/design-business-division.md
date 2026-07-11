# 事业部层级功能设计方案

## TL;DR

在现有「实验室 → 项目/方法 → 记录」结构之上新增一级 **事业部（Business Division）**：一个事业部包含多个实验室，实验室选择页新增事业部筛选 Chip 栏（复用现有方法类型筛选样式），分析检测与研发送样两条业务线在记录、统计、导出中同步支持事业部维度，并作为独立版本 **v0.4.24** 实现。

---

## 1. 用户故事与目标

### 1.1 为什么要加事业部？

当前 `project_groups` 表直接对应「实验室」（410/411/412 等），卡片数量会随实验室扩张而线性增长。用户希望按 **检测技术维度**（液相/气相/理化/ICP/热分析/质谱/红外/其他）把实验室归拢到更高层的事业部，从而减少录入入口的认知负担，并支持按事业部做统计和导出。

### 1.2 用户故事

| 编号 | 用户故事 |
|------|----------|
| US-1 | 作为检测人员，我希望在「工作量录入」首页按事业部筛选实验室，以便快速定位到目标实验室。 |
| US-2 | 作为研发送样人员，我希望在送样入口同样看到事业部筛选，保证两条业务线体验一致。 |
| US-3 | 作为管理员，我希望在管理页维护事业部及其下属实验室，便于后续组织结构调整。 |
| US-4 | 作为统计人员，我希望按事业部汇总工作量，方便向事业部负责人汇报。 |
| US-5 | 作为数据导出人员，我希望导出 Excel 中包含事业部维度，便于财务/运营按事业部核算。 |

### 1.3 产品目标

1. **减少选择成本**：实验室选择页通过事业部 Chip 过滤，将一屏 N 个实验室卡片收敛到若干事业部维度。
2. **数据一致性**：分析检测（`work_records`）与研发送样（`rd_work_records`）在记录中保留事业部上下文。
3. **可扩展性**：事业部作为独立主数据，可被统计、导出、样品信息登记等下游模块复用。

---

## 2. 需求清单（P0/P1/P2）

### P0 — 必须实现

- `divisions` 新表 + `project_groups.division_id` 外键关联。
- 实验室选择页（`WorkloadPortal`、`SamplePortal`）新增事业部筛选 Chip 栏，支持「全部 (n)」+ 各事业部 (n)。
- 事业部为空/未分配的实验室默认归入「未分配事业部」或「其他」，保证旧数据不丢失。
- 分析检测与研发送样在创建记录时保存 `division_id`（或可通过 `group_id` 稳定推导）。
- 管理页新增「事业部管理」入口（Tab/Card）。

### P1 — 应该实现

- `GroupResponse` / `ProjectGroup` 返回 `division_id`、`division_name`。
- 统计接口（`/stats/*`、`/rd-stats/*`）增加 `division_id` 查询参数与按事业部汇总。
- 导出 Excel 在相关 Sheet 中增加「事业部」列（Sheet 1/3/4/5/7 等）。
- 实验室编辑弹窗支持选择所属事业部。

### P2 —  nice to have

- 事业部支持颜色字段，Chip 与 GroupCard 可复用颜色。
- 导出新增独立「事业部汇总 Sheet」。
- 事业部支持软删除（`deleted_at`）。
- 事业部维度接入「样品信息登记」模块的筛选/统计。

---

## 3. 数据模型设计

### 3.1 新表：`divisions`（事业部）

| 字段 | 类型 | 约束 | 说明 |
|------|------|------|------|
| `id` | INTEGER | PK AUTOINCREMENT | 主键 |
| `name` | TEXT | NOT NULL UNIQUE | 事业部名称，如「液相」「气相」「理化」等 |
| `sort_order` | INTEGER | DEFAULT 0 | 排序 |
| `color` | TEXT | DEFAULT '#1976d2' | 颜色（P2） |
| `is_active` | INTEGER | DEFAULT 1 | 是否启用（P2） |
| `deleted_at` | TEXT | NULL | 软删除时间（P2） |
| `created_at` | TEXT | NOT NULL DEFAULT datetime('now') | 创建时间 |

### 3.2 现有表改动

| 表 | 改动 | 关系 | 说明 |
|----|------|------|------|
| `project_groups` | 新增 `division_id INTEGER REFERENCES divisions(id)` | **N:1** | 一个事业部包含多个实验室；一个实验室属于一个事业部 |
| `work_records` | 新增 `division_id INTEGER`（可选，见 3.3） | **N:1（推导/冗余）** | 记录录入时的事业部上下文 |
| `rd_work_records` | 新增 `division_id INTEGER`（可选，见 3.3） | **N:1（推导/冗余）** | 同上 |

### 3.3 关系设计说明

- **事业部 : 实验室 = 1 : N**。
  - 原因：用户需求明确「一个事业部包含多个实验室」，且当前实验室（`project_groups`）已经是叶子节点，没有多事业部归属的必要。
  - 实现：在 `project_groups` 上加 `division_id` 外键，软关联即可（SQLite 外键约束可开，但删除事业部时建议置空或阻止删除）。
- **事业部 : 项目/方法 = 间接关联**。
  - 通过 `project_lab_links` → `project_groups` → `divisions` 推导，不新增冗余关联表。
- **记录中是否保存 `division_id`？**
  - 推荐 **保存冗余列** `division_id`。
  - 理由：
    1. 已有 `group_id` 保存录入时实验室上下文，事业部同理需要保存录入时上下文。
    2. 如果后续实验室调整所属事业部，历史记录仍可按原事业部统计，避免数据漂移。
    3. 导出、统计查询不需要每次 JOIN 三层表，性能更好。
  - 回退方案：不保存，查询时通过 `wr.group_id = pg.id` 推导 `pg.division_id`。

---

## 4. UI/UX 设计方案

### 4.1 实验室选择页（`WorkloadPortal` / `SamplePortal`）

#### 布局

在现有搜索框与卡片网格之间插入事业部筛选 Chip 栏：

```
┌─────────────────────────────────────────────┐
│ 工作量录入                        [查看统计] │
│ 选择实验室，开始录入检测数据                  │
├─────────────────────────────────────────────┤
│ 🔍 搜索实验室...                              │
├─────────────────────────────────────────────┤
│ [全部 (8)] [液相 (3)] [气相 (1)] [理化 (1)] … │  ← 新增
├─────────────────────────────────────────────┤
│ ┌────────┐ ┌────────┐ ┌────────┐            │
│ │研发送样│ │  410   │ │  411   │ ...        │
│ └────────┘ └────────┘ └────────┘            │
└─────────────────────────────────────────────┘
```

#### 样式

- 复用 `EntryPage` 中方法类型筛选的 `Chip` 组件：
  - 选中：`variant="filled"`、`color="primary"`
  - 未选中：`variant="outlined"`、`color="default"`
  - `borderRadius: '2px'`、字号与现有保持一致。
- 计数逻辑：每个事业部 Chip 右侧显示当前筛选条件下属于该事业部的实验室数量，例如「液相 (3)」。
- 搜索框与事业部筛选 **叠加生效**：先按事业部过滤，再按搜索关键字过滤。

#### 计数逻辑

```ts
const divisions = useMemo(() => [
  { id: 0, name: '全部' },
  ...fetchedDivisions.filter(d => d.is_active !== false),
], [fetchedDivisions]);

const counts = useMemo(() => {
  const map: Record<number, number> = {};
  divisions.forEach(d => {
    if (d.id === 0) return;
    map[d.id] = groups.filter(g => g.division_id === d.id).length;
  });
  return map;
}, [divisions, groups]);
```

#### 空状态处理

- 如果系统中没有任何事业部：
  - 方案 A：隐藏事业部 Chip 栏，保持现有体验（推荐，避免空栏干扰）。
  - 方案 B：显示「全部 (n)」一个 Chip。
- 如果某事业部下没有实验室：该 Chip 仍显示「事业部名 (0)」，但点击后卡片区域显示「该事业部暂无实验室」。
- 如果搜索+事业部筛选后无结果：显示「未找到」。

### 4.2 事业部管理（`ManagePage`）

- 在 `TC` 数组中新增一个 Tab：`{ key: 'divisions', label: '事业部管理', icon: <BusinessIcon />, desc: '管理事业部及下属实验室' }`。
- 事业部管理 UI 复用现有「检测方法类型管理」或「实验室管理」的 InlineEditCard / Dialog 模式：
  - 列表展示：事业部名称、排序、下属实验室数量、颜色（P2）。
  - 操作：新建、编辑、删除。
  - 删除限制：若该事业部下仍有实验室，阻止删除并提示「请先迁移或删除下属实验室」。
- 在「实验室管理」编辑弹窗中，在「排序」下方新增「所属事业部」下拉选择（`Select` + `MenuItem`）。

---

## 5. 页面/路由/API 改动清单

### 5.1 后端改动（按实现顺序）

| 顺序 | 文件路径 | 改动要点 |
|------|----------|----------|
| 1 | `src/db/migrations.rs` | 新增 `divisions` 表；`project_groups` 增加 `division_id`；`work_records` / `rd_work_records` 增加 `division_id`（若决定冗余）。 |
| 2 | `src/models/division.rs`（新建） | `DivisionResponse`、`DivisionCreate`、`DivisionUpdate` 结构体。 |
| 3 | `src/models/group.rs` | `GroupResponse` / `GroupCreate` / `GroupUpdate` 增加 `division_id`。 |
| 4 | `src/repo/division_repo.rs`（新建） | 事业部 CRUD；删除时检查是否有实验室引用。 |
| 5 | `src/repo/group_repo.rs` | 列表/详情查询 JOIN `divisions` 返回 `division_id`、`division_name`；create/update 写入 `division_id`。 |
| 6 | `src/api/division_handler.rs`（新建） | `/api/divisions` GET/POST/PUT/DELETE。 |
| 7 | `src/api/group_handler.rs` | 无需大改，路由已注册。 |
| 8 | `src/repo/record_repo.rs` / `src/service/record_service.rs` | create 时接收 `division_id`，写入 `work_records.division_id`；list 时支持 `division_id` 查询参数。 |
| 9 | `src/repo/rd_record_repo.rs` / `src/service/rd_record_service.rs` | 与 record 对称修改。 |
| 10 | `src/api/record_handler.rs` / `src/api/rd_record_handler.rs` | `RecordQuery` 增加 `division_id`；create 透传 `division_id`。 |
| 11 | `src/api/stats_handler.rs` / `src/api/rd_stats_handler.rs` | `StatsQuery` 增加 `division_id`；新增 `/stats/by-division` 与 `/rd-stats/by-division` 接口。 |
| 12 | `src/api/export_data.rs` | 各 Sheet 查询增加 `division_id` 过滤与事业部列；新增 `query_sheet_division_data`。 |
| 13 | `src/api/export_write.rs` | 相关 Sheet 写入事业部列；新增事业部汇总 Sheet（P2）。 |
| 14 | `src/api/rd_export_data.rs` / `src/api/rd_export_handler.rs` / `src/api/rd_export_preview_handler.rs` | 复用 export_data 类型与逻辑，底层表换为 `rd_work_records`。 |
| 15 | `src/main.rs` | 注册 `division_handler::router`。 |

### 5.2 前端改动（按实现顺序）

| 顺序 | 文件路径 | 改动要点 |
|------|----------|----------|
| 1 | `frontend/src/types/index.ts` | 新增 `Division` 接口；`ProjectGroup` 增加 `division_id`、`division_name`。 |
| 2 | `frontend/src/api/client.ts` | 新增 `getDivisions`、`createDivision`、`updateDivision`、`deleteDivision`；`getGroups` 响应结构已包含事业部字段。 |
| 3 | `frontend/src/pages/WorkloadPortal.tsx` | 新增事业部 Chip 筛选栏；搜索与事业部过滤叠加。 |
| 4 | `frontend/src/pages/SamplePortal.tsx` | 与 WorkloadPortal 同步复用相同筛选逻辑。 |
| 5 | `frontend/src/pages/EntryPage.tsx` / `frontend/src/pages/SampleEntryPage.tsx` | 创建记录时传入 `division_id`（从 `group.division_id` 取）。 |
| 6 | `frontend/src/pages/ManagePage.tsx` | 新增 `divisions` Tab；事业部列表/新建/编辑/删除；实验室编辑弹窗增加事业部下拉。 |
| 7 | `frontend/src/pages/StatsPage.tsx` / `frontend/src/pages/SampleStatsPage.tsx` | 增加事业部筛选与事业部汇总图表（P1/P2）。 |

---

## 6. 分析检测 & 研发送样同步策略

### 6.1 克隆式同步点

当前代码库中分析检测与研发送样已经是 **镜像结构**：

| 分析检测 | 研发送样 | 同步方式 |
|----------|----------|----------|
| `work_records` 表 | `rd_work_records` 表 | 同结构加 `division_id` |
| `src/api/record_handler.rs` | `src/api/rd_record_handler.rs` | 同逻辑透传 `division_id` |
| `src/repo/record_repo.rs` / `service` | `src/repo/rd_record_repo.rs` / `service` | 同逻辑写入 `division_id` |
| `src/api/stats_handler.rs` | `src/api/rd_stats_handler.rs` | 同逻辑新增 by-division |
| `src/api/export_data.rs` | `src/api/rd_export_data.rs`（`pub use` + 换表） | 事业部列与过滤同步 |
| `src/api/export_handler.rs` | `src/api/rd_export_handler.rs` | 同逻辑 |
| `frontend/src/pages/EntryPage.tsx` | `frontend/src/pages/SampleEntryPage.tsx` | 同逻辑传入 `division_id` |
| `frontend/src/pages/WorkloadPortal.tsx` | `frontend/src/pages/SamplePortal.tsx` | 同逻辑事业部筛选 |

**建议**：保持现有「克隆」模式，不要强行抽象。两份代码修改点高度对称，可由同一次 PR 统一完成，并通过 diff 自检保证一致。

### 6.2 统计与导出最小重复

- **统计**：在 `stats_handler.rs` 与 `rd_stats_handler.rs` 中分别新增 `by_division` 端点。由于两个 handler 已是独立文件，直接复制 `by_project` 模式修改 GROUP BY 字段即可，重复度可控。
- **导出**：
  - `export_data.rs` 中各 Sheet 查询统一使用 `LEFT JOIN divisions d ON pg.division_id = d.id` 获取事业部名，增加 `division_id` 过滤参数。
  - `rd_export_data.rs` 已通过 `pub use super::export_data::*` 复用类型，只需把 SQL 中 `work_records` 替换为 `rd_work_records`，事业部列同步生效。
  - 建议新增一个内部辅助函数 `division_filter_sql(division_id: Option<i64>) -> String` 减少重复 SQL 拼接。

---

## 7. 待确认问题

以下问题需要用户/业务方拍板，建议在进入开发前明确：

1. **事业部名称与分类是否固定？**
   - 是否就是截图 2 中的「液相/气相/理化/ICP/热分析/质谱/红外/其他」这 8 个？还是允许管理员自定义？
2. **是否需要在记录中冗余保存 `division_id`？**
   - 推荐保存，可保证历史记录按原事业部统计；否则通过 `group_id` 实时推导，实现更简单但数据可能随事业部调整而漂移。
3. **事业部是否支持软删除？**
   - 推荐 P2 实现，先硬删除但要求事业部下无实验室；若需保留历史，加 `deleted_at`。
4. **事业部是否支持颜色字段？**
   - 若支持，Chip 与卡片可带颜色；否则统一使用主色。
5. **导出是否需要独立「事业部汇总 Sheet」？**
   - P2 建议新增；P1 仅在现有 Sheet 中增加事业部列。
6. **旧数据（已有实验室）如何归属？**
   - 默认全部分到「未分配事业部」？还是业务方先批量划分？
7. **事业部维度是否接入「样品信息登记」模块？**
   - 当前 `sample_info_records` 通过 `lab_name` 软关联，加事业部筛选需要额外 JOIN。

---

## 8. 版本隔离建议

本功能涉及：

- 数据库 Schema 变更（新增表 + 多表加列）
- 后端 API 新增与改造
- 前端多处页面同步改动
- 统计、导出等核心数据链路改动

**强烈建议作为独立版本 `v0.4.24` 实现**：

1. 复制 `D:\桌面\工作量统计工具项目\workload-tool-rust\v0.4.23\` 到同级目录 `v0.4.24\`。
2. 在 `v0.4.24` 内完成全部开发、自测、构建。
3. 保留 `v0.4.23` 原目录不动，作为可回滚基线。
4. 升级 `Cargo.toml` 与 `frontend/package.json` 中的版本号为 `0.4.24`。
5. 新增 `CHANGELOG_v0.4.24.md` 记录本次变更。

**禁止直接在 `v0.4.23` 目录原地修改**，避免破坏可稳定发布的基线版本。

---

## 9. 推荐下一步

**建议顺序：用户先确认待确认问题 → 架构师补充技术设计 → 工程实现。**

具体而言：

1. **产品确认（本轮）**：请用户确认第 7 节中的 7 个问题，尤其是事业部分类、是否冗余保存 `division_id`、旧数据归属策略。
2. **架构师/技术负责人补充**：
   - 细化 `divisions` 表与 `project_groups` 的级联/置空策略；
   - 确认统计/导出中事业部列的具体位置与公式影响；
   - 确认 SQLite 迁移脚本对旧数据的回填逻辑。
3. **工程实现**：按第 5 节清单在 `v0.4.24` 目录中开发，完成后进行端到端测试（重点验证分析检测与研发送样导出一致性）。

---

*文档版本：v0.1*  
*基于代码版本：workload-tool-rust v0.4.23*  
*输出时间：2026-07-09*
