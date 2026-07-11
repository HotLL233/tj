// 权限点白名单（与后端 src/models/role.rs PERMISSIONS 保持一致）
// 每项包含权限 key 与中文说明，用于角色权限矩阵展示。

export interface PermissionDef {
  key: string;
  label: string;
  group: string;
}

export const PERMISSIONS: PermissionDef[] = [
  // 门户入口
  { key: 'entry:sample', label: '研发送样', group: '门户入口' },
  { key: 'entry:workload', label: '分析检测', group: '门户入口' },
  { key: 'entry:sample-info', label: '样品信息登记', group: '门户入口' },
  // 系统管理
  { key: 'manage:projects', label: '研发项目管理', group: '系统管理' },
  { key: 'manage:groups', label: '实验室管理', group: '系统管理' },
  { key: 'manage:divisions', label: '部门管理', group: '系统管理' },
  { key: 'manage:methods', label: '检测方法管理', group: '系统管理' },
  { key: 'manage:trash', label: '回收站', group: '系统管理' },
  { key: 'manage:audit', label: '审计日志', group: '系统管理' },
  { key: 'manage:backup', label: '数据备份', group: '系统管理' },
  { key: 'manage:help', label: '教程与帮助', group: '系统管理' },
  { key: 'manage:sampleinfo', label: '样品信息登记管理', group: '系统管理' },
  { key: 'manage:users', label: '用户管理', group: '系统管理' },
  { key: 'manage:roles', label: '角色管理', group: '系统管理' },
];

// 权限分组顺序（用于矩阵分组展示）
export const PERMISSION_GROUPS: string[] = ['门户入口', '系统管理'];

// 通配权限（系统管理员使用）
export const ALL_PERMISSION = '*';

/** 给定权限点列表，是否包含某权限（支持 `*` 通配） */
export const hasPermission = (perms: string[], key: string): boolean =>
  perms.includes(ALL_PERMISSION) || perms.includes(key);

/** 是否拥有任一以指定前缀开头的权限（如 'manage:'） */
export const hasAnyPrefix = (perms: string[], prefix: string): boolean =>
  perms.includes(ALL_PERMISSION) || perms.some((p) => p.startsWith(prefix));
