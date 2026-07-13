# v0.4.38 版本更新说明

## 更新日期
2026-07-11

## 更新内容
### 新增：细粒度 UI 元素可编辑
- **研发送样录入页（SampleEntryPage）**：送样时间、操作按钮组（添加行/删除选中/重置）、多行录入表格 现在均可在编辑模式下编辑文字和开关显隐
- **样品信息登记页（SampleInfoEntry）**：检测类型 Chip、序号自动生成提示、送样时间、操作按钮组、登记记录列表 现在均可在编辑模式下编辑
- 新增 `buttonLabel` / `chipLabel` / `actionButtons` 字段支持，编辑工具栏根据字段类型显示对应的编辑对话框
- 按钮组支持独立编辑三个按钮的文字内容

### 技术细节
- 基于 v0.4.37 版本隔离，新建 v0.4.38 独立文件夹
- 修改文件：
  - `frontend/src/components/PageSectionEditor.tsx` — SectionConfig 扩展 + 条件编辑
  - `frontend/src/pages/SampleEntryPage.tsx` — 3 个新 section 包裹
  - `frontend/src/pages/SampleInfoEntry.tsx` — 5 个新 section 包裹
  - `src/db/migrations.rs` — 种子数据追加新 section

## 打包说明
- 无 cmd 窗口（生产模式 `cargo build --release`）
- 支持覆盖安装（AppId 固定）
- Docker 镜像自动构建推送
