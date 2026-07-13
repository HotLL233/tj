# v0.4.44 版本更新说明

## 更新日期
2026-07-13

## 更新内容
### 全面集中化编辑：管理页统一编辑 + 外部去除编辑UI
- **外部页面**：去除所有 12 个页面的 PageEditProvider/PageEditToggle/PageSectionEditor/EditablePageShell 包裹，页面恢复正常展示（无编辑工具栏）
- **管理页**：「页面布局管理」增强，新增字段布局编辑模块：
  - Section 编辑（同 v0.4.43）
  - 字段布局编辑（录入表格字段）：排序、改列宽、改标签、开关可见、添加/删除字段
  - 针对「研发送样录入」的 sample_entry_fields 生效

## 改动文件
- `components/PageLayoutAdmin.tsx` — 增强：加 FieldDef 编辑模块
- `pages/HomePage.tsx` — 去编辑UI
- `pages/StatsPage.tsx` — 去编辑UI
- `pages/SamplePortal.tsx` — 去编辑UI
- `pages/WorkloadPortal.tsx` — 去编辑UI
- `pages/SampleInfoHome.tsx` — 去编辑UI
- `pages/RdRecordsPage.tsx` — 去编辑UI
- `pages/SampleStatsPage.tsx` — 去编辑UI
- `pages/SampleInfoStatsPage.tsx` — 去编辑UI
- `pages/EntryPage.tsx` — 去编辑UI
- `pages/SampleInfoEntry.tsx` — 去编辑UI
- `pages/SampleEntryPage.tsx` — 去编辑UI + EditablePageShell → getSetting
- `pages/ManagePage.tsx` — 去编辑UI

## 打包说明
- 无 cmd 窗口（生产模式 `cargo build --release`）
- 支持覆盖安装（AppId 固定）
