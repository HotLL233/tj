# v0.4.33 方案（仅方案，不动代码）

## 用户原始需求汇总
1. **用户管理-岗位分级**（如"分析员""实验员"），岗位决定可见入口（如实验员分组只能看研发送样+样品信息登记）
2. **研发送样录入布局**：列宽不合理（送样人80、数量65、批号80、部门90 太窄），全部列要能显示
3. **研发送样-今日记录**：增加取样人填写框（或者像样品信息登记那样支持列自定义）
4. **样品信息登记-自动填充**：送样人/部门/实验室根据当前登录用户自动填写
5. **样品信息登记-取样人列太窄**：所有列要能显示
6. **部门管理-删除后重现**：上个版本删除了一些部门，安装新版本后又出现
7. **部门管理-说明文案**：优化描述

## 代码现状（v0.4.32）

### 角色权限（已有，v0.4.32 实现）
- 权限点：`*`、`entry:sample`、`entry:workload`、`entry:sample-info`、11 个 `manage:*`
- 5 个种子角色：系统管理员/分析检测员/研发送样员/样品登记员/查看者
- JWT 携带 permissions，前端 `hasPermission` 门控

### 研发送样录入（SampleEntryPage.tsx）
- 列宽（px）：复选框40/序号40/送样人80/部门90/实验室auto/项目120/类型100/方法140/数量65/批号80/注意事项minWidth 100
- minWidth: 1100
- 缺陷：送样人/部门/数量/批号 过窄；方法名长会截断

### 研发送样今日记录（取样人）
- SampleEntryPage.tsx 第 510-527 行、RdRecordsPage.tsx 第 124-156 行 — 写死列
- 样品信息登记有 `sample_info_columns` + `sample_info_column_visibility` 架构（seed 12 条内置列）
- 复用成本：6-8 个文件新增/改动

### 样品信息登记自动填充
- SampleInfoEntry.tsx **没有** useUser，user_name 默认为空 → 提交时 fallback "未知"
- 对比 SampleEntryPage.tsx 第 92-96 行已有 useUser + division_id 自动填充模式
- User 模型字段：id, username, password, division_id, division_name, group_id, group_name, is_admin, is_active, role_id, permissions, created_at, updated_at

### 样品信息登记列宽
- 表单：minWidth 100/130（attachment）
- 列表：minWidth 80
- 缺固定 width

### 部门删除后重现（根因）
- divisions 表字段：id, name UNIQUE, sort_order, color, is_active DEFAULT 1, deleted_at, created_at
- `division_handler.rs` 第 47-56 行 → 物理删除 `DELETE FROM divisions WHERE id=?1`
- seed 用 `INSERT OR IGNORE`（按 name 唯一） — 删除同名部门不会被 seed 重现
- **真实重现原因猜测**：用户说的是"上个版本删除的又出现"，可能 v0.4.32 migrations 重新跑了 seed，或者 v0.4.32 seed 中 names 完全没变（液相/气相/...）；但 INSERT OR IGNORE 应该不会重现
- 实际可能：用户记错了，或者 v0.4.32 之前的版本部门是"软删除"后来被改回物理删除
- **修复方向**：从物理删除改为软删除（UPDATE is_active=0, deleted_at），list 过滤 is_active=1

### 部门管理说明文案
- 文件：ManagePage.tsx 第 708 行
- 当前：「按检测技术维度（液相/气相/理化/ICP/热分析/质谱/红外/其他）归拢实验室；删除部门仅解除实验室归属，不删除实验室。」
- 不准确：实际是物理删除，不是"仅解除"

---

## 方案设计（v0.4.33）

### 需求 1：岗位分级 + 入口可见
**决策**：不新建独立"岗位"表，复用现有 `roles` 表。原因：
- 用户语义上的"分析员/实验员"其实就是一种角色
- roles 已携带 permissions，足够控制入口可见
- 新建岗位表会导致 role + position 双层概念，徒增复杂度
- 实施：在管理页「角色管理」中提供 2 个新预设模板（"分析员"/"实验员"），每个绑定特定的入口权限
- 预设权限：
  - **分析员**：entry:workload（只看分析检测入口，不含 manage:*）
  - **实验员**：entry:sample + entry:sample-info（只看研发送样和样品信息登记，不含 manage:*）
- 后续：管理员手动创建/编辑角色时，可以基于预设创建
- 改 ManagePage 的角色编辑对话框：增加"基于模板"下拉，选项 = 系统管理员/分析检测员/研发送样员/样品登记员/查看者/分析员/实验员
- **UI 改动**：AdminRolesPage（已在 v0.4.32 引入）
- **新建/改动**：1 个文件改动（AdminRolesPage.tsx 加模板下拉）
- **方案 B**（如果用户坚持独立岗位概念）：新建 `positions` 表 + 关联 `user.position_id`，但这会显著增加复杂度

### 需求 2：研发送样录入列宽优化
- 改 SampleEntryPage.tsx 第 313-333 行
- 列宽改为：复选框40/序号40/送样人**120**/部门**140**/实验室auto/项目**150**/类型**110**/方法**180**/数量**80**/批号**100**/注意事项minWidth 130
- 整体 minWidth: 1100 → **1280**
- 启用水平滚动兜底（overflow-x: auto）
- **新建/改动**：1 个文件改动

### 需求 3：研发送样今日记录取样人
**方案 A（轻量）**：仅在 RdRecordsPage.tsx 给"取样人"列从写死列改为可编辑的 TextField（与 v0.4.32 一样）
- 改 1 个文件
- **方案 B（完整）**：让研发送样记录支持列自定义（复用 sample_info_columns 架构，新建 rd_record_columns 表）
- 6-8 文件改动
- **建议**：先做方案 A，若用户后续需要再加方案 B

### 需求 4：样品信息登记自动填充
- 改 SampleInfoEntry.tsx
- 引入 useUser，在 emptyRow() 初始化时填入 user.username
- 每行新增行的 user_name 字段默认 = 当前登录用户
- division_id 自动从 user.division_id 填入
- 部门/实验室字段通过 useEffect 初始化时填入
- 改 1 个文件
- **代码示例**：
  ```typescript
  const { user } = useUser();
  const defaultUserName = user?.username ?? '';
  const defaultDivisionId = user?.division_id ?? null;
  // emptyRow({ userName: defaultUserName, divisionId: defaultDivisionId })
  ```

### 需求 5：样品信息登记列宽
- 在 column 编辑弹窗里给所有列加 width 字段（如果还没有）
- 默认 width 调整为更合理的值：seq_no=60/user_name=100/division_id=100/lab_name=100/project_name=120/quantity=80/batch_no=120/main_components=180/notes=180/submitted_at=140/detection_type=100/status=80
- 或者前端写死渲染宽度（更稳定）

### 需求 6：部门物理删除 → 软删除
- `division_repo.rs` delete 函数：DELETE → UPDATE is_active=0, deleted_at=datetime('now')
- list 函数：加 WHERE is_active=1
- `division_handler.rs` 第 47-56 行：调用逻辑不变（前端不用改）
- 增加 DELETE 确认：删除前检查"是否有活跃的实验室引用"，如果有提示"先解除关联"
- 改 1-2 个文件
- **重要**：用户的"安装新版本后又出现"问题，最直接的修复其实是 seed 不写死名字，而是把硬编码的 8 个部门改成"如果表为空才 seed"（避免每次重装覆盖用户改动）。但 INSERT OR IGNORE 应该已实现这点，需要进一步排查是哪个版本引入了 seed 行为变化

### 需求 7：部门说明文案
- ManagePage.tsx 第 708 行
- 改后：「按检测技术维度（液相/气相/理化/ICP/热分析/质谱/红外/其他）归拢实验室；删除部门采用软删除，关联实验室的归属将自动置空，部门可在回收站恢复。」
- 改 1 个文件

---

## 实施计划（v0.4.33 范围）

### 版本隔离
- 从 v0.4.32 复制创建 v0.4.33 文件夹
- 不动 v0.4.21 ~ v0.4.32

### 文件改动清单
| # | 文件 | 类型 | 估时 |
|---|------|------|------|
| 1 | AdminRolesPage.tsx | 加 2 个预设角色模板 | 0.5h |
| 2 | SampleEntryPage.tsx | 列宽 + minWidth | 0.3h |
| 3 | RdRecordsPage.tsx | 取样人列可编辑 | 0.3h |
| 4 | SampleInfoEntry.tsx | useUser 自动填充 | 0.5h |
| 5 | sample_info_columns seed | width 字段 | 0.3h |
| 6 | division_repo.rs + handler | 软删除 | 0.5h |
| 7 | ManagePage.tsx | 文案 | 0.1h |
| **总计** | | | **2.5h** |

### 待确认问题
1. **需求 1 路径 A vs B**：是用现有 roles 加 2 个模板（分析员/实验员），还是新建独立 positions 表？
2. **需求 3 路径 A vs B**：仅改取样人可编辑，还是完整列自定义？
3. **需求 6 seed 策略**：是否需要把 seed 的 8 个部门改为「仅表为空时 seed」？（已 INSERT OR IGNORE，若仍是"删除后又出现"问题，需进一步排查旧版本）
4. **需求 5 column width**：宽度写在 seed 写死，还是做成可配置（在 AdminColumns 弹窗维护）？
