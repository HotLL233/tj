# v0.4.25 变更日志

## 新增
- **样品信息登记：多行表格录入** — 将单条表单改为 Excel 模板式横向表格，支持批量添加/删除/重置/提交
- **样品信息登记：所属部门选择** — 每行增加「所属部门」下拉框（从 divisions API 加载）
- **样品信息登记：送样数量** — 每行增加「送样数量」数值输入，默认 1
- **管理页：样品记录统计卡片** — 记录查询区增加统计概览（总记录数/待检测/待取样/已取样/检测完成）
- **管理页：实验室一览增加关联部门列**

## 变更
- **后端：sample_info_records 表增加 `division_id` 和 `quantity` 字段**（数据库迁移）
- **后端：SampleInfoRecord/SampleInfoCreate/SampleInfoUpdate/SampleInfoResponse 模型增加对应字段**
- **后端：SQL 查询/插入/更新全部适配新字段**
- **前端：SampleInfoRecord 接口增加 `division_id`、`division_name`、`quantity`**
- **前端：createSampleInfo/updateSampleInfo API 增加 `division_id` 和 `quantity` 参数**
- **UI 文案：所有「事业部」统一改为「部门」**（管理页标题、按钮、对话框、提示文字）
- **UI：项目卡片/方法卡片的 Chip 删除按钮（×）移除**
- **统计页标题更名：分析检测 →「分析检测统计」；研发送样 →「研发送样统计」**
- **版本号：0.4.24 → 0.4.25**

## 修复
- 无

## 技术
- 数据库迁移：sample_info_records 新增 2 列（division_id, quantity）
- 前端类型定义和 API 客户端更新
- 打包配置和 Docker 镜像标签升级至 v0.4.25
