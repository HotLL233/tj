# v0.2.15 — 方法导入列模式重构

## TL;DR
将方法导入从"三层级行模式"重构为"独立列模式"：每列独立解析，列头=分组名，列值=条目，含"方法"关键字的列聚合到"检测方法管理"卡片。

## 修改文件
| 文件 | 改动 |
|------|------|
| `workload-tool-rust/v0.2.15/src/api/project_handler.rs` | 重写 `method_import`（第33-115行），新增 `classify_method_type` |
| `workload-tool-rust/v0.2.15/src/repo/project_repo.rs` | `batch_import_3level` → `batch_import_by_column`（第56-120行） |

## 解析规则
- 列头含"方法" → `method_type = "检测方法"`
- 列头含"研发" → `method_type = "研发项目"`
- "实验室管理" → 只建分组，不建项目
- 去重：内存 + DB 双重保障

## 验证
- `cargo check`: ✅ 0 errors
- 6项逻辑审查: ✅ 全部 PASS
