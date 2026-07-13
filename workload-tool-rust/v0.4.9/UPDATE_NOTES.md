# v0.4.9 更新说明

## 发布日期
2026-07-08

## BugFix

### 单价倍率支持设为 0
- **问题**：前端输入框使用 `||` 判空，JavaScript 将 `0` 视为 falsy，导致输入 0 后自动改为 1.0
- **修复**：`StatsPage.tsx` 和 `ManagePage.tsx` 中 `||` 改为 `??`，空字符串单独判回退 1.0
- **影响范围**：检测人记录页单价倍率列 + 方法管理编辑弹窗

## 功能增强

### 新增 Sheet 11「类型汇总表」
- 按检测类型（液相/气相/理化/其他）汇总数量、单价、单价倍率、金额
- 不同单价分行显示（同一类型下不同单价各自独立成行）
- 每种类型小计 + 全部合计行
- 同时覆盖分析检测和研发送样两个模块的导出

**Excel 列结构**：

| 检测类型 | 数量 | 单价 | 单价倍率 | 金额总计 | 类型金额 |
|----------|------|------|----------|----------|----------|
| 液相     | 120  | 200  | 1.0      | 24000    | 48000    |
| 液相     | 50   | 300  | 1.5      | 22500    |          |
| 气相     | 80   | 150  | 1.0      | 12000    | 18000    |
| 气相     | 30   | 200  | 1.0      | 6000     |          |
| 理化     | ...  | ...  | ...      | ...      | ...      |
| **合计** |      |      |          |          |          |

---

## 修改文件清单

### 前端（React · `workload-tool-rust/v0.4.9/frontend/src`）
| 文件 | 改动 |
|------|------|
| `pages/StatsPage.tsx` | 单价倍率 `\|\|` → `??`，空字符串回退 1.0 |
| `pages/ManagePage.tsx` | 单价倍率 `\|\|` → `??`，空字符串回退 1.0 |

### 后端（Rust · `workload-tool-rust/v0.4.9/src`）
| 文件 | 改动 |
|------|------|
| `api/export_data.rs` | 新增 `TypeSummaryRow` + `query_sheet11_data` |
| `api/export_write.rs` | 新增 `write_sheet11` |
| `api/export_handler.rs` | 新增加 Sheet 11 导出块 |
| `api/export_preview_handler.rs` | 新增 sheet11 预览路由 |
| `api/rd_export_data.rs` | 新增 `TypeSummaryRow` + `query_sheet11_data`（RD 模块） |
| `api/rd_export_handler.rs` | 新增加 Sheet 11 导出块（RD 模块） |
| `api/rd_export_preview_handler.rs` | 新增 sheet11 预览路由（RD 模块） |

---

## 构建信息

- **Rust 编译**：待构建
- **前端**：需重新构建（StatsPage.tsx / ManagePage.tsx 有改动）
- **安装包**：`installer/工作量统计工具_Rust_v0.4.9_Setup.exe`
- **覆盖安装**：✅ 支持（AppId 不变）

---

**版本隔离**：`v0.4.8 → v0.4.9` 完整复制后增量修改。
