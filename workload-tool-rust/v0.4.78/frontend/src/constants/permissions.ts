export interface PermissionDef {
  key: string;
  label: string;
  group: string;
}

export const PERMISSIONS: PermissionDef[] = [
  { key: 'entry:sample', label: '研发送样', group: '门户入口' },
  { key: 'entry:workload', label: '分析检测', group: '门户入口' },
  { key: 'entry:sample-info', label: '样品信息登记', group: '门户入口' },
  { key: 'sample:collect', label: '研发送样-取样操作', group: '门户入口' },

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

  { key: 'stats:workload:access', label: '分析检测统计入口', group: '统计管理' },
  { key: 'stats:workload:week', label: '按周统计', group: '统计管理' },
  { key: 'stats:workload:month', label: '按月统计', group: '统计管理' },
  { key: 'stats:workload:user-log', label: '检测人记录', group: '统计管理' },
  { key: 'stats:workload:division', label: '事业部统计', group: '统计管理' },
  { key: 'stats:workload:sheet1', label: '实验室-项目-方法', group: '统计管理' },
  { key: 'stats:workload:sheet2', label: '仪器汇总', group: '统计管理' },
  { key: 'stats:workload:sheet3', label: '项目汇总（含金额）', group: '统计管理' },
  { key: 'stats:workload:sheet4', label: '实验室汇总（含金额）', group: '统计管理' },
  { key: 'stats:workload:sheet5', label: '检测人汇总（原始记录）', group: '统计管理' },
  { key: 'stats:workload:sheet6', label: '检测人汇总表（含系数）', group: '统计管理' },
  { key: 'stats:workload:sheet7', label: '实验室总表', group: '统计管理' },
  { key: 'stats:workload:sheet8', label: '项目总表', group: '统计管理' },
  { key: 'stats:workload:sheet9', label: '仪器类型汇总', group: '统计管理' },
  { key: 'stats:workload:sheet10', label: '理化汇总', group: '统计管理' },
  { key: 'stats:workload:view-all', label: '查看全部统计数据', group: '统计管理' },

  { key: 'help:edit', label: '编辑帮助文档', group: '内容管理' },
];

export const PERMISSION_GROUPS: string[] = ['门户入口', '系统管理', '统计管理'];

export const ALL_PERMISSION = '*';

export const hasPermission = (perms: string[], key: string): boolean =>
  perms.includes(ALL_PERMISSION) || perms.includes(key);

export const hasAnyPrefix = (perms: string[], prefix: string): boolean =>
  perms.includes(ALL_PERMISSION) || perms.some((p) => p.startsWith(prefix));
