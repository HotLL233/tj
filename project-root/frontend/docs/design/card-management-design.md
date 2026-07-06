# 卡片式数据管理方案 — 可落地设计

> 基于当前项目 `ManagePage.tsx` 渐进改进，不重写。
> 创建时间：2026-07-03

---

## 现状复用分析

| 现有能力 | 复用方式 |
|---------|---------|
| `Project.lab_ids` / `Project.method_ids` | 关联数据字段已就绪，只需改进前端交互 |
| `Method.type_ids` | 方法类型关联已支持 |
| 导入导出 API | 后端已支持 Excel 导入，可扩展为 JSON 配置导入导出 |
| MUI Card/Paper 组件 | 当前用 `Paper` 渲染列表行，可改为真正 Card 网格 |

---

## 阶段 1：就地编辑 + 关联可视化（1周）

### 目标
- 移除编辑弹窗（Dialog），改为点击卡片展开编辑区
- 卡片上直接显示关联标签，可点击查看

### 改动文件

#### `src/pages/ManagePage.tsx`
- **项目卡片**：移除 `Dialog`，改为 `expandedProjectId` 状态控制展开/折叠
- **方法卡片**：同上，新增 `expandedMethodId`
- **实验室卡片**：点击展开其下项目列表（只读，展示 `lab_names`）

#### 新增 `src/components/InlineEditCard.tsx`
通用就地编辑卡片组件：

```tsx
interface InlineEditCardProps<T> {
  item: T;
  isExpanded: boolean;
  onToggle: () => void;
  renderView: (item: T) => React.ReactNode;
  renderEdit: (item: T, onChange: (patch: Partial<T>) => void) => React.ReactNode;
  onSave: (item: T) => Promise<void>;
  onDelete: () => Promise<void>;
}
```

---

## 阶段 2：拖拽关联配置（2周）

### 新增依赖
```bash
npm install @dnd-kit/core @dnd-kit/sortable @dnd-kit/utilities
```

### 新增文件

#### `src/components/DraggableCard.tsx`
可拖拽卡片，使用 `useDraggable` hook。

#### `src/components/DropZone.tsx`
放置区域，使用 `useDroppable` hook。

#### `src/pages/AssociationPage.tsx`（新增）
独立的关联配置页面：
- 左侧：实验室列表（可拖拽）
- 右侧：项目列表（分区域显示，每个项目是一个 DropZone）
- 拖拽完成后调用 `updateProject(projectId, { lab_ids: [...existing, droppedLabId] })`

---

## 阶段 3：前端配置持久化（1周）

### 说明
实体数据仍走后端 API，**只持久化视图配置**（如：哪个卡片展开、当前筛选条件、排序方式）。

### 新增文件

#### `src/stores/uiConfigStore.ts`
```ts
import { create } from 'zustand';

interface UIConfigState {
  expandedProjectIds: number[];
  expandedMethodIds: number[];
  methodFilter: string;
  setExpandedProject: (id: number, expanded: boolean) => void;
  setMethodFilter: (filter: string) => void;
  exportConfig: () => UIConfig;
  importConfig: (config: UIConfig) => void;
}

export const useUIConfigStore = create<UIConfigState>((set, get) => ({
  expandedProjectIds: [],
  expandedMethodIds: [],
  methodFilter: '',
  setExpandedProject: (id, expanded) => { /* 更新 state + localStorage */ },
  setMethodFilter: (filter) => { /* 更新 state + localStorage */ },
  exportConfig: () => ({ expandedProjectIds, expandedMethodIds, methodFilter }),
  importConfig: (config) => set(config),
}));
```

#### `src/utils/exportViewConfig.ts`
导出当前视图配置为 JSON 文件（供用户备份偏好设置）。

#### `src/utils/importViewConfig.ts`
导入 JSON 配置并应用到 Store。

---

## 风险与注意事项

1. **dnd-kit 学习成本**：如果团队不熟悉，可用 simpler 方案（下拉多选 + 排序按钮）替代拖拽
2. **大数据量性能**：实验室/项目/方法各超过 100 条时，拖拽列表需虚拟滚动（`react-window`）
3. **后端 API 限制**：当前 `updateProject` 已支持 `lab_ids` 批量更新，无需改后端

---

## 开发优先级建议

1. **先做阶段 1**（就地编辑）—— 用户痛点最直接，改动相对小
2. **阶段 2**（拖拽关联）—— 如果用户对"关联配置"需求强烈
3. **阶段 3**（配置持久化）—— 优先级最低，可最后做

---

## 下一步

确认方案后，可直接开始阶段 1 实现：
1. 创建 `InlineEditCard.tsx` 组件
2. 改造 `ManagePage.tsx` 项目卡片区域
3. 改造方法卡片区域
4. 测试就地编辑流程
