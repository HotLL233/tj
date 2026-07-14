# v0.4.49 版本更新说明

## 更新日期
2026-07-13

## 新增功能：统一录入表单配置器

### 「管理页 → 录入表单配置」tab

- 三个表单模板：研发送样录入 / 样品信息登记 / 分析检测录入
- 字段编辑：标签 · 宽度(px) · 类型(text/select/number/datetime/textarea) · 可见开关 · 拖拽排序
- 添加/删除字段 · 重置为默认
- 实时预览表格

## 修复

### P0 - 修复保存字段布局 key 不一致
- **根因**：`PageLayoutAdmin` 保存到 `sample_entry_fields`，但 `SampleEntryPage` 从 `layout_sample_entry_fields` 读取，DB 种子用的也是 `layout_sample_entry_fields`。保存成功但页面读的是旧数据。
- **修复**：`PageLayoutAdmin` key 改为 `layout_sample_entry_fields`（与 DB 种子 + 页面读取一致）
- **v0.4.49 新路径**：新增 `form_sample_entry` key，`SampleEntryPage` 优先读取（回退兼容旧 key）

### 样品信息登记字段可编辑
- 新增 `useEffect` 加载 `form_sample_info_entry`（ManageFormConfig 写入）
- 通过 `useMemo` 合并 FieldDef 覆盖 SampleInfoColumn 的宽度/标签/可见性
- 用户改字段宽度后，录入表单即时响应

### 分析检测录入字段定义
- 数据库种子 `form_entry`（7 个字段）
- 后续可通过 ManageFormConfig 编辑

## 技术改动

### 文件清单

| 文件 | 改动 |
|------|------|
| `components/ManageFormConfig.tsx` | **新增** 300 行，集中式表单配置器 |
| `pages/ManagePage.tsx` | 加 `forms` tab + ManageFormConfig 渲染 |
| `pages/SampleEntryPage.tsx` | 读取 key `form_sample_entry`（回退 `layout_sample_entry_fields`） |
| `pages/SampleInfoEntry.tsx` | 加 `formDefs` 加载 + `useMemo` 合并覆盖列宽/标签/可见性 |
| `components/PageLayoutAdmin.tsx` | key 修复 `sample_entry_fields` → `layout_sample_entry_fields` |
| `components/LayoutDesigner.tsx` | FieldDef 类型加 `datetime` |
| `types/layout.ts` | FieldDef 类型加 `datetime` |
| `db/migrations.rs` | 种子 `form_sample_entry` + `form_sample_info_entry` + `form_entry` |

### 存储架构

| key | 用途 | 状态 |
|-----|------|------|
| `form_sample_entry` | 研发送样录入字段 | ✅ EDITABLE |
| `form_sample_info_entry` | 样品信息登记字段 | ✅ EDITABLE |
| `form_entry` | 分析检测录入字段 | ✅ EDITABLE |
| `layout_sample_entry_fields` | 旧 key（v0.4.44-） | 保留兼容 |

## 用户操作路径

管理页 → 录入表单配置 → 选标签页 → 编辑字段 → 保存 → 导航到对应录入页面 → 即时生效

## 验证
- ✅ `tsc --noEmit` 0 errors
- ✅ `vite build` 2.83s
- ✅ `cargo build --release` 1m41s
- ✅ curl 实测 PUT form_sample_entry → 200 OK + GET 验证 → 正确读取
- ✅ 种子数据 form_sample_info_entry → 9 fields
- ✅ Inno Setup 打包成功
