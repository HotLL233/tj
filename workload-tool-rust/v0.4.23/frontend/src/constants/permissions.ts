// 权限点白名单（与后端 src/models/role.rs PERMISSIONS 保持一致）
// 每项包含权限 key 与中文说明，用于角色权限矩阵展示。

export interface PermissionDef {
  key: string;
  label: string;
  group: string;
}

export const PERMISSIONS: PermissionDef[] = [
  { key: 'user:manage', label: '用户管理', group: '系统' },
  { key: 'role:manage', label: '角色管理', group: '系统' },
  { key: 'audit:read', label: '审计查看', group: '系统' },
  { key: 'notification:read', label: '通知查看', group: '系统' },
  { key: 'notification:manage', label: '通知发送', group: '系统' },
  { key: 'approval_rule:read', label: '审批规则查看', group: '审批' },
  { key: 'approval_rule:manage', label: '审批规则管理', group: '审批' },
  { key: 'approval:read', label: '审批任务查看', group: '审批' },
  { key: 'approval:approve', label: '审批决策', group: '审批' },
  { key: 'inventory:read', label: '库存查看', group: '库存' },
  { key: 'inventory:write', label: '库存编辑', group: '库存' },
  { key: 'instrument:read', label: '仪器查看', group: '仪器' },
  { key: 'instrument:write', label: '仪器编辑', group: '仪器' },
  { key: 'instrument:approve', label: '仪器预约审批', group: '仪器' },
  { key: 'instrument:book', label: '仪器预约', group: '仪器' },
  { key: 'instrument:book_manage', label: '仪器预约管理', group: '仪器' },
  { key: 'purchase:read', label: '采购查看', group: '采购' },
  { key: 'purchase:write', label: '采购编辑', group: '采购' },
  { key: 'purchase:request', label: '采购申请', group: '采购' },
  { key: 'purchase:approve', label: '采购审批', group: '采购' },
  { key: 'supplier:read', label: '供应商查看', group: '采购' },
  { key: 'supplier:write', label: '供应商编辑', group: '采购' },
  { key: 'ops_stats:read', label: '运营统计查看', group: '统计' },
];

// 通配权限（系统管理员使用）
export const ALL_PERMISSION = '*';

/** 给定权限点列表，是否包含某权限（支持 `*` 通配） */
export const hasPermission = (perms: string[], key: string): boolean =>
  perms.includes(ALL_PERMISSION) || perms.includes(key);
