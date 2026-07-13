# v0.4.8 更新说明

## 发布日期
2026-07-08

## BugFix：导出 Excel 单价倍率与项目汇总公式修复

### 修复 1：项目/实验室汇总 Sheet 倍率分组问题
- **问题**：Sheet 3「项目-汇总」和 Sheet 4「实验室-汇总」中，同一项目+方法下有多条不同倍率记录时，GROUP BY 未包含倍率维度，导致不同倍率记录错误合并为一行，倍率值随机选取。
- **修复**：`export_data.rs` 中 `query_sheet3_data` 和 `query_sheet4_data` 的 GROUP BY 加入 `wr.multiplier`，不同倍率的记录各自独立成行。

### 修复 2：Sheet 8「项目总表」项目汇总列公式丢失
- **问题**：Sheet 8 写入时先为每行写了项目汇总公式，但后续合并同项目单元格时 `merge_range` 用空字符串覆写了公式，导致项目汇总无法计算。
- **修复**：合并后补写 SUM 公式，对液相/气相/理化三种类型的汇总列分别求和再汇总。

---

## 修改文件清单

### 后端（Rust · `workload-tool-rust/v0.4.8/src`）
| 文件 | 改动 |
|------|------|
| `api/export_data.rs` | Sheet 3/4 GROUP BY 加入 `wr.multiplier` |
| `api/export_write.rs` | Sheet 8 项目汇总合并后补写 SUM 公式 |

---

## 构建信息

- **Rust 编译**：待构建
- **前端**：无变更（前端代码不变）
- **安装包**：`installer/工作量统计工具_Rust_v0.4.8_Setup.exe`
- **覆盖安装**：✅ 支持（AppId 不变）

---

**版本隔离**：`v0.4.7 → v0.4.8` 完整复制后增量修改。
