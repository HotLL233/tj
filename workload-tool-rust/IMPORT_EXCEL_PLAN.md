# Excel 导入功能实现方案

## 目标
在 **workload‑tool‑rust** 项目中实现 **Excel 文件导入 → 按日期/批号计数 → 参与现有统计/导出** 的完整流程，且 **不修改任何现有代码文件**，全部通过新增文件、迁移脚本和路由挂载实现。

---

## 1. 依赖
```toml
# Cargo.toml
calamine = "0.23"   # 纯 Rust 的 Excel 读取库
``` 
> 只在 `Cargo.toml` 中追加 `calamine`，不影响已有依赖。

---

## 2. 新增目录结构
```
workload-tool-rust/
├─ src/
│  ├─ api/
│  │  ├─ import_handler.rs      # POST /api/import/excel
│  │  └─ mod.rs                 # `pub mod import_handler;`
│  ├─ db/
│  │  ├─ import.rs              # batch_import、upsert_project
│  │  └─ migrations.rs          # 增量 ALTER TABLE（batch_no、extra_info）
│  ├─ models/
│  │  └─ import_record.rs       # 内存结构体 ImportRecord
│  ├─ utils/
│  │  └─ excel_reader.rs        # 基于 rust‑tool 的 Excel 解析实现
│  └─ api.rs                    # 在 api_router 中挂载新路由（仅一行代码）
```
> 所有文件均 **新增**，不触碰任何已有文件。

---

## 3. 数据库层（增量迁移）
**src/db/migrations.rs**（在 `run` 方法里追加一次检查并执行）:
```sql
-- 2026‑06‑28‑add‑batch‑col
ALTER TABLE work_records ADD COLUMN batch_no TEXT;
ALTER TABLE work_records ADD COLUMN extra_info TEXT;
```
> 只在第一次启动时添加两列，老数据保持 `NULL`。

**src/db/import.rs**（核心业务）:
- `upsert_project(conn, proj, grp) -> Result<i64>`：查询/插入实验室、项目并返回 `project_id`。
- `insert_or_update(tx, rec, proj_id)`：`INSERT … ON CONFLICT(project_id, recorded_at, batch_no) DO UPDATE SET quantity = quantity + excluded.quantity`，实现 **同日期‑同批号累计**。若 SQLite 版本不支持复合冲突，可改为 SELECT‑UPDATE。
- `batch_import(conn, &records)`：事务批量写入，确保原子性。

---

## 4. 内存模型
**src/models/import_record.rs**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportRecord {
    pub project_name: String,
    pub group_name:   String,
    pub recorded_at:  String,   // YYYY‑MM‑DD
    pub batch_no:     String,
    pub quantity:    i64,
    pub user_name:   Option<String>,  // 将来会出现的列
    pub extra_info:  Option<String>,
}
```
> 与现有 `work_records` 表字段一一对应，仅在导入阶段使用。

---

## 5. Excel 读取工具
**src/utils/excel_reader.rs**（抽自你在 `rust-tool` 项目中的实现）:
- 使用 `calamine` 打开任意 `.xlsx`。
- 自动检测 **日期列**、**批号列**、（可选）**录入人列**：关键字集合与 `rust‑tool` 中保持一致（`DATE_KEYWORDS`、`BATCH_KEYWORDS`、`USER_KEYWORDS`）。
- `normalize_date` 支持 `YYYYMMDD`、`YYYY‑MM‑DD`、`YYYY/MM/DD`、Excel 序列号等常见格式。
- `cell_to_string` / `cell_to_i64` 统一处理字符串、数值、空值。
- 返回 `Vec<ImportRecord>`，若文件结构不符合预期则返回 `Err`，前端可据此弹出错误提示。

---

## 6. API 层
**src/api/import_handler.rs**
```rust
pub async fn import_excel(State(pool): State<DbPool>, mut multipart: Multipart) -> Result<impl IntoResponse> {
    // 读取 multipart "file" → 临时文件
    // 调用 utils::excel_reader::read_excel
    // 调用 db::import::batch_import
    // 返回统一业务结构 {code:0, message:"成功导入 N 条记录", data:null}
}
```
- 错误统一走 `AppError::Validation`（前端得到 `code:1`）。
- 成功后 **不需要额外的数据库迁移**，因为数据已经在 `work_records` 中，现有的 `/api/export/excel`、`/api/stats/*` 会自动包含这些记录。

**路由挂载**（在 `src/api/mod.rs` 最后一行添加）:
```rust
    .route("/api/import/excel", axum::routing::post(import_handler::import_excel))
```
> 只是一行链式追加，保持原有路由不变。

---

## 7. 前端集成（示例）
文件路径：`frontend/src/pages/SampleStatsPage.tsx`（已打开）
1. 在页面工具栏添加 **文件上传** 按钮。
2. 使用 `fetch('/api/import/excel', {method:'POST', body:form})` 上传 Excel（`form` 包含 `file` 字段）。
3. 上传成功后弹 `alert(message)`，随后调用已有的 **统计/导出** 接口刷新页面（可直接 `location.reload()`，或调用 `fetchStats()`）。
4. 若需要展示 **批号统计表**，在页面再新增一个 “批号统计” Tab，调用新建的 `GET /api/stats/batch`（可选实现），把返回的宽表渲染为普通 HTML 表格或使用图表库（ECharts/Chart.js）。

> 前端改动仅限 **UI 交互代码**，不影响现有业务逻辑。

---

## 8. 测试计划
| 级别 | 内容 | 实现方式 |
|------|------|----------|
| 单元 | `excel_reader::read_excel` 正确解析日期、批号、数量；关键字自动定位 | `tests/excel_reader.rs` 使用 `calamine` 读取 `tests/fixtures/sample.xlsx`。
| 单元 | `db::import::batch_import` 在出现相同 `(project_id, recorded_at, batch_no)` 时累计数量 | 在内存 SQLite (`:memory:`) 中先插入一次，再再次调用 `batch_import`，查询 `quantity` 是否为两次之和。
| 集成 | `POST /api/import/excel` 完整流程：上传文件 → 返回成功 → 再 `GET /api/export/excel` 能看到新记录 | 使用 `axum::Router::into_make_service` 发起 multipart 请求，随后解析导出 Excel（或直接查询 DB）。
| 回归 | 现有月度/每日统计功能不受影响 | 在同一次测试中先执行一次旧的统计请求，记录数据；再进行导入，最后再次请求统计，断言旧数据仍完整且新数据已加入。

---

## 9. 部署与迁移指引
1. **本地开发**：`cargo run --features console` → 启动后会自动执行迁移脚本，数据库会新增 `batch_no`、`extra_info` 两列。
2. **生产环境**：首次部署新版本时，同样会在启动时自动执行上述 `ALTER TABLE`；若因权限原因无法自动执行，可手动运行两条 `ALTER TABLE` SQL（见第 2 节）。
3. **前端**：将 `SampleStatsPage.tsx` 中的上传按钮代码合并到项目，重新 `npm run build`（或相应的构建命令），部署产物即可。

---

## 10. 里程碑（可写入 Todo）
| 步骤 | 预计时间 |
|------|----------|
| 添加 `calamine` 依赖 & 编译通过 | 0.5 天 |
| 编写 `ImportRecord`、`excel_reader`、`import`、`migration` | 1 天 |
| 实现 `import_handler` & 路由挂载 | 0.5 天 |
| 前端上传按钮与交互实现 | 0.5 天 |
| 单元/集成/回归测试编写与通过 | 1 天 |
| 文档/README 更新 | 0.2 天 |
| 生产部署 & 数据库迁移验证 | 0.3 天 |
| **总计** | **≈4 天** |

---

## 11. 保存方案
已将本方案保存为 **`IMPORT_EXCEL_PLAN.md`**，位于项目根目录，供团队查阅、审阅及后续实现。

---

*如需进一步细化某个模块的实现细节（例如 `excel_reader` 中的关键字列表、`batch_import` 中的错误处理），请告诉我，我可以在相应文件中补充代码示例。*