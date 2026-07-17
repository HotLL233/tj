# v0.4.71 版本更新说明

## 更新日期
2026-07-15

## 新增：样品信息列按检测类型显示
- 新增 `sample_info_column_visibility` 表，支持列的 M:N 类型可见性
- 迁移回填：现有 type_key 不为空的列自动关联对应类型
- 管理页「自定义列配置」新增「显示类型」列，点击弹出 Popover 多选检测类型
- 未设置可见性的列默认所有类型可见（向后兼容）

## 后端改动
- `src/db/migrations.rs` — +sample_info_column_visibility 表 + 回填 SQL
- `src/models/sample_info_column.rs` — SampleInfoColumn 加 visible_types: Vec<String>
- `src/repo/sample_info_column_repo.rs` — list_all 加 JOIN 查询 visible_types + set_visibility

## 前端改动
- `frontend/src/types/index.ts` — SampleInfoColumn 加 visible_types
- `frontend/src/pages/ManagePage.tsx` — 新增「显示类型」列头 + Popover + 多选保存

## 兼容性
- 旧版本已有的列配置不受影响（type_key 逻辑保留）
- 未绑定的列默认全类型可见
