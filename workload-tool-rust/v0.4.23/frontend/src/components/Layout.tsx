import React, { useEffect, useState } from 'react';
import { Outlet, useNavigate, useLocation, NavLink } from 'react-router-dom';
import {
  AppBar, Toolbar, Drawer, List, ListItemButton, ListItemIcon, ListItemText, Box, Typography,
  Avatar, Chip, Badge, IconButton, Tooltip, Divider,
} from '@mui/material';
import DashboardIcon from '@mui/icons-material/Dashboard';
import ScienceIcon from '@mui/icons-material/Science';
import InventoryIcon from '@mui/icons-material/Inventory2';
import ShoppingCartIcon from '@mui/icons-material/ShoppingCart';
import ApprovalIcon from '@mui/icons-material/Approval';
import NotificationsIcon from '@mui/icons-material/Notifications';
import HistoryIcon from '@mui/icons-material/History';
import PeopleIcon from '@mui/icons-material/People';
import SecurityIcon from '@mui/icons-material/Security';
import RuleIcon from '@mui/icons-material/Rule';
import BarChartIcon from '@mui/icons-material/BarChart';
import LogoutIcon from '@mui/icons-material/Logout';
import PersonIcon from '@mui/icons-material/Person';
import { useAuth } from '../context/AuthContext';
import { getUnreadCount } from '../api/client';

interface NavItem { text: string; path: string; perm?: string; icon: React.ReactNode; group: string; }

const NAV: NavItem[] = [
  { text: '仪表盘', path: '/', perm: undefined, icon: <DashboardIcon />, group: '概览' },
  { text: '仪器管理', path: '/instruments', perm: 'instrument:read', icon: <ScienceIcon />, group: '运营管理' },
  { text: '库存管理', path: '/inventory', perm: 'inventory:read', icon: <InventoryIcon />, group: '运营管理' },
  { text: '采购管理', path: '/purchase', perm: 'purchase:read', icon: <ShoppingCartIcon />, group: '运营管理' },
  { text: '审批中心', path: '/approval', perm: 'approval:read', icon: <ApprovalIcon />, group: '运营管理' },
  { text: '通知', path: '/notifications', perm: 'notification:read', icon: <NotificationsIcon />, group: '个人' },
  { text: '审计日志', path: '/audit', perm: 'audit:read', icon: <HistoryIcon />, group: '系统' },
  { text: '用户管理', path: '/admin/users', perm: 'user:manage', icon: <PeopleIcon />, group: '系统' },
  { text: '角色权限', path: '/admin/roles', perm: 'role:manage', icon: <SecurityIcon />, group: '系统' },
  { text: '审批规则', path: '/admin/rules', perm: 'approval_rule:manage', icon: <RuleIcon />, group: '系统' },
  { text: '工作量统计', path: '/workload', perm: 'ops_stats:read', icon: <BarChartIcon />, group: '统计' },
];

const DRAWER_WIDTH = 232;

const Layout: React.FC = () => {
  const { user, hasPerm, logout } = useAuth();
  const navigate = useNavigate();
  const location = useLocation();
  const [unread, setUnread] = useState(0);

  const refreshUnread = () => { getUnreadCount().then((r) => { if (r.code === 0) setUnread(r.data ?? 0); }).catch(() => {}); };
  useEffect(() => { refreshUnread(); const t = setInterval(refreshUnread, 30000); return () => clearInterval(t); }, []);

  const groups = Array.from(new Set(NAV.map((n) => n.group)));

  return (
    <Box sx={{ display: 'flex', minHeight: '100vh', bgcolor: 'background.default' }}>
      <AppBar position="fixed" sx={{ zIndex: (t) => t.zIndex.drawer + 1, background: 'linear-gradient(90deg,#1976d2,#1565c0)' }}>
        <Toolbar>
          <Typography variant="h6" fontWeight={700} sx={{ flexGrow: 1 }}>本地化 LIMS</Typography>
          <Tooltip title="通知">
            <IconButton color="inherit" onClick={() => navigate('/notifications')}>
              <Badge badgeContent={unread} color="error"><NotificationsIcon /></Badge>
            </IconButton>
          </Tooltip>
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, ml: 2 }}>
            <Avatar sx={{ width: 32, height: 32, bgcolor: 'secondary.main' }}><PersonIcon fontSize="small" /></Avatar>
            <Box sx={{ display: { xs: 'none', sm: 'block' } }}>
              <Typography variant="body2" lineHeight={1.1}>{user?.display_name || user?.username}</Typography>
              <Chip size="small" label={user?.role || ''} sx={{ height: 18, fontSize: 11 }} />
            </Box>
            <Tooltip title="退出登录">
              <IconButton color="inherit" onClick={logout}><LogoutIcon /></IconButton>
            </Tooltip>
          </Box>
        </Toolbar>
      </AppBar>

      <Drawer variant="permanent" sx={{ width: DRAWER_WIDTH, flexShrink: 0, [`& .MuiDrawer-paper`]: { width: DRAWER_WIDTH, boxSizing: 'border-box' } }}>
        <Toolbar />
        <Box sx={{ overflow: 'auto', mt: 1 }}>
          {groups.map((g) => {
            const items = NAV.filter((n) => n.group === g && (!n.perm || hasPerm(n.perm)));
            if (items.length === 0) return null;
            return (
              <Box key={g} sx={{ px: 2, pt: 1.5, pb: 0.5 }}>
                <Typography variant="caption" color="text.secondary" sx={{ fontWeight: 700, letterSpacing: 1 }}>{g}</Typography>
                <List dense disablePadding>
                  {items.map((it) => (
                    <ListItemButton key={it.path} component={NavLink} to={it.path} end={it.path === '/'}
                      sx={{ borderRadius: 2, mb: 0.3 }}
                      selected={location.pathname === it.path}>
                      <ListItemIcon sx={{ minWidth: 36 }}>{it.icon}</ListItemIcon>
                      <ListItemText primary={it.text} primaryTypographyProps={{ fontSize: 14 }} />
                    </ListItemButton>
                  ))}
                </List>
                <Divider sx={{ mt: 1 }} />
              </Box>
            );
          })}
        </Box>
      </Drawer>

      <Box component="main" sx={{ flexGrow: 1, p: 3 }}>
        <Toolbar />
        <Outlet />
      </Box>
    </Box>
  );
};

export default Layout;
