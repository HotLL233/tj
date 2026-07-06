# v0.2.19 项目编辑 & 方法编辑 优化方案

> 基于 v0.2.18，2026-07-01

## 需求概述

1. 项目编辑对话框：恢复全称/排序/启用开关，**移除管理系数**
2. 方法编辑对话框：增加金额字段 + 方法类型复选 + 系数/备注
3. 方法管理卡片：增加类型筛选下拉框
4. 修复实验室删除 Bug：检查 project_lab_links 而非 projects.group_id

---

## 一、对话框 UI 设计

### 1.1 项目编辑对话框

```
┌─ 编辑项目 ──────────────────────┐
│ 项目名称    [_______________]    │
│ 全称        [_______________]    │
│ 关联实验室  [◢ Autocomplete多选] │
│ 关联方法    [◢ Autocomplete多选] │
│ 排序        [0            ▴▾]    │
│ 启用        [▓▓▓▓▓▓▓▓░]         │
│ 备注        [_______________]    │
│             [_______________]    │
│           [取消]  [保存]         │
└────────────────────────────────┘
```

### 1.2 方法编辑对话框

```
┌─ 编辑方法 ──────────────────────┐
│ 方法名称    [_______________]    │
│ 系数        [1.0          ▴▾]    │
│ 金额        [0.00         ▴▾]    │
│ 方法类型    [√]液相 [√]气相 [ ]理化│
│             [ ]ICP  [√]热分析 [ ]质谱 │
│             [ ]其他              │
│ 备注        [_______________]    │
│             [_______________]    │
│           [取消]  [保存]         │
└────────────────────────────────┘
```

---

## 二、数据库变更

### methods 表增加 amount 字段

```sql
ALTER TABLE methods ADD COLUMN amount REAL NOT NULL DEFAULT 0.0;
```

### projects 表字段确认（已存在）

- full_name, sort_order, is_active, coefficient 均已存在（v0.2.2/0.2.16 迁移添加）

---

## 三、后端改动清单

| 文件 | 改动 |
|------|------|
| `src/db/migrations.rs` | 添加 `ALTER TABLE methods ADD COLUMN amount` |
| `src/models/method.rs` | MethodResponse/MethodCreate/MethodUpdate 增加 amount |
| `src/models/project.rs` | ProjectResponse 增加 full_name/sort_order/is_active；ProjectUpdate 去掉 coefficient |
| `src/repo/method_repo.rs` | SQL 查询/创建/更新增加 amount 字段 |
| `src/repo/project_repo.rs` | PROJ_SQL 增加字段；update() 处理 full_name/sort_order/is_active |
| `src/repo/group_repo.rs` | delete() 改用 project_lab_links 检查 |

---

## 四、前端改动清单

| 文件 | 改动 |
|------|------|
| `src/types/index.ts` | Project + full_name/sort_order/is_active；Method + amount |
| `src/pages/ManagePage.tsx` | 项目对话框(+全称/排序/启用)，方法对话框(+金额)，方法列表(+类型筛选) |
