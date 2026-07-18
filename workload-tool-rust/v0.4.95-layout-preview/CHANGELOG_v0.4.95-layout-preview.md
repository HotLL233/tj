# v0.4.95-layout-preview 更新说明

## 本次更新

- 借鉴 New API 的布局思想，继续使用 React + MUI，扩大桌面端内容区，减少页面两侧无效留白。
- 统一记录表格的列宽计算规则：桌面端按列元数据约束宽度，长文本在单元格内自然换行；手机端保留横向滚动能力。
- 统计预览表在手机端自动切换为字段卡片视图，避免宽表格挤压字段；桌面端继续使用表格视图。
- 新增可复用的筛选工具栏和响应式编辑抽屉组件，为记录页和管理页提供统一交互基础。
- 实验室管理和检测方法管理改为紧凑列表，编辑入口使用右侧抽屉；项目进行中/已归档的业务逻辑保持不变。
- 统计柱状图按数据项使用主题色序列区分，分析检测和研发送样继续使用各自独立的颜色主题。
- 更新前端、Rust 和 Docker 构建版本号为 `0.4.95-layout-preview`。
- 新增相对路径 Inno Setup 脚本，保留历史 `AppId`，覆盖安装时保留用户数据目录。

## 验证结果

- `npm ci`：通过。
- `npm run build`：通过。
- `cargo check --features console`：通过，只有原有未使用代码警告。
- `cargo test --features console`：17 个测试全部通过。
- `cargo build --release --features console`：通过。
- 前台 staging 运行验证：`/api/health` 返回 `ok`，`/api/version` 返回 `0.4.95-layout-preview`，首页返回 HTTP 200。
- Inno Setup 6.7.2：安装包编译通过。
- Docker 镜像构建未在当前机器执行，原因是当前环境未安装 Docker CLI；Dockerfile 已改为先构建 `frontend` 并从 `backend/static` 复制运行时资源。

## 安装包

`installer/工作量统计工具_Rust_v0.4.95_布局改造预览版_Setup.exe`

安装包沿用旧版本 `AppId`，可覆盖上个版本安装；新脚本不删除 `{app}\data`，以保留现有数据库和用户数据。
