import React, { useState } from 'react';
import { Outlet, useNavigate, useLocation } from 'react-router-dom';
import { AppBar, Toolbar, Typography, IconButton, Button, BottomNavigation, BottomNavigationAction, Drawer, List, ListItem, ListItemIcon, ListItemText, Dialog, DialogTitle, DialogContent, Alert, useMediaQuery, useTheme, Box, Container } from '@mui/material';
import HomeIcon from '@mui/icons-material/Home'; import SettingsIcon from '@mui/icons-material/Settings'; import MenuBookIcon from '@mui/icons-material/MenuBook'; import MenuIcon from '@mui/icons-material/Menu'; import InfoIcon from '@mui/icons-material/Info';
import { useUser } from '../UserContext';
import { hasAnyPrefix } from '../constants/permissions';
import UserMenu from './UserMenu';
import BackToTop from './BackToTop';

const NAV_ITEMS = [
  { label: '主页', path: '/', icon: <HomeIcon /> },
  { label: '教程与帮助', path: '/help', icon: <MenuBookIcon /> },
];
const MOBILE_NAV = [
  { label: '主页', path: '/', icon: <HomeIcon /> },
  { label: '教程与帮助', path: '/help', icon: <MenuBookIcon /> },
];
const F_LIST = ['🔬 研发送样与工作量双入口','📊 按周/月/日多维度统计','🔬 液相/气相仪器分类统计','📦 研发送样记录与专属统计','📋 汇总模板格式导出','👤 用户日志与纠错编辑','🔧 分组/项目管理','📝 审计日志 / 🗑️ 回收站'];

const Layout: React.FC = () => {
  const navigate = useNavigate(); const location = useLocation(); const theme = useTheme(); const isMobile = useMediaQuery(theme.breakpoints.down('md'));
  const [drawerOpen, setDrawerOpen] = useState(false); const [aboutOpen, setAboutOpen] = useState(false);
  const [serverVer, setServerVer] = useState('');
  const { user, isLoggedIn } = useUser();
  // 顶部「管理」入口：管理员 或 拥有任一 manage:* 权限的用户可见
  const showManage = isLoggedIn && !!(user?.is_admin || hasAnyPrefix(user?.permissions || [], 'manage:'));

  const getMobileNav = (): number => { const p = location.pathname; if (p === '/') return 0; if (p.startsWith('/help')) return 1; return 0; };
  React.useEffect(() => { fetch('/api/version').then(r => r.json()).then(d => setServerVer(d.version || '')).catch(() => {}); }, []);

  return (<Box sx={{ display: 'flex', flexDirection: 'column', minHeight: '100vh' }}>
    <AppBar position="sticky" elevation={0} className="glass-appbar" sx={{ zIndex: theme.zIndex.drawer + 1 }}><Toolbar>
      {isMobile && <IconButton edge="start" onClick={() => setDrawerOpen(true)} sx={{ mr: 1, color: '#333' }}><MenuIcon /></IconButton>}
      <Box sx={{ flexGrow: 1 }} />
      {!isMobile && <Box sx={{ display: 'flex', gap: 0.5, alignItems: 'center' }}>
        {NAV_ITEMS.map(item => { const isActive = location.pathname === item.path; return <Button key={item.path} startIcon={item.icon} onClick={() => navigate(item.path)} className={isActive ? 'nav-pill-active' : 'nav-pill'} sx={{ color: isActive ? '#fff' : '#555' }}>{item.label}</Button>; })}
        {showManage && (
          <Button startIcon={<SettingsIcon />} onClick={() => navigate('/manage')} className={location.pathname === '/manage' ? 'nav-pill-active' : 'nav-pill'} sx={{ color: location.pathname === '/manage' ? '#fff' : '#555' }}>
            管理
          </Button>
        )}
      </Box>}
      {isLoggedIn ? <UserMenu /> : (
        <Button onClick={() => navigate('/login')} sx={{ color: '#1976d2', ml: 1 }}>
          登录
        </Button>
      )}
      <IconButton onClick={() => setAboutOpen(true)} title="关于" sx={{ ml: { xs: 0, md: 1 }, color: '#555' }}><InfoIcon /></IconButton>
    </Toolbar></AppBar>

    <Drawer anchor="left" open={drawerOpen} onClose={() => setDrawerOpen(false)}><Box sx={{ width: 250, pt: 2, bgcolor: '#f8fafc', height: '100%' }}>
      <Typography variant="h6" sx={{ px: 2, pb: 2, fontWeight: 700 }}>知微</Typography>
      <List>
        {NAV_ITEMS.map(item => { const isActive = location.pathname === item.path; return <ListItem key={item.path} component="div" onClick={() => { navigate(item.path); setDrawerOpen(false); }} sx={{ cursor: 'pointer', mx: 1, mb: 0.5, borderRadius: '2px', bgcolor: isActive ? 'rgba(102,126,234,0.12)' : 'transparent' }}><ListItemIcon sx={{ color: isActive ? '#667eea' : undefined }}>{item.icon}</ListItemIcon><ListItemText primary={item.label} primaryTypographyProps={{ fontWeight: isActive ? 700 : 400, color: isActive ? '#667eea' : undefined }} /></ListItem>; })}
        {showManage && (
          <ListItem component="div" onClick={() => { navigate('/manage'); setDrawerOpen(false); }} sx={{ cursor: 'pointer', mx: 1, mb: 0.5, borderRadius: '2px', bgcolor: location.pathname === '/manage' ? 'rgba(102,126,234,0.12)' : 'transparent' }}>
            <ListItemIcon sx={{ color: location.pathname === '/manage' ? '#667eea' : undefined }}><SettingsIcon /></ListItemIcon>
            <ListItemText primary="管理" primaryTypographyProps={{ fontWeight: location.pathname === '/manage' ? 700 : 400, color: location.pathname === '/manage' ? '#667eea' : undefined }} />
          </ListItem>
        )}
      </List>
    </Box></Drawer>

    <Box component="main" sx={{ flexGrow: 1, pb: isMobile ? 7 : 2, pt: 2, px: { xs: 1, sm: 2, md: 3 } }}><Container maxWidth="lg" disableGutters={isMobile}><Outlet /></Container></Box>

    {isMobile && <BottomNavigation value={getMobileNav()} onChange={(_e, v: number) => { if (v === 0) navigate('/'); else if (v === 1) navigate('/help'); }} className="glass-bottom-nav" sx={{ position: 'fixed', bottom: 0, left: 0, right: 0, zIndex: theme.zIndex.appBar, borderTop: '1px solid rgba(0,0,0,0.06)' }}>{MOBILE_NAV.map((item, i) => <BottomNavigationAction key={i} label={item.label} icon={item.icon} />)}</BottomNavigation>}

    <Dialog open={aboutOpen} onClose={() => setAboutOpen(false)} maxWidth="sm" fullWidth PaperProps={{ sx: { borderRadius: '2px' } }}>
      <DialogTitle sx={{ fontWeight: 700, textAlign: 'center' }}>知微</DialogTitle>
      <DialogContent>
        <Typography variant="subtitle2" color="primary" sx={{ mb: 2, fontWeight: 600, textAlign: 'center' }}>v{serverVer || '...'}</Typography>
        <Alert severity="info" sx={{ mb: 2, borderRadius: '2px' }}>
          <Typography variant="subtitle2" sx={{ fontWeight: 600, mb: 0.5 }}>v0.4.28 更新内容</Typography>
          <Typography variant="body2">• 登录页改版：深色渐变背景 + 知微品牌</Typography>
          <Typography variant="body2">• 用户中心：个人信息 / 修改密码 / 退出登录</Typography>
          <Typography variant="body2">• 权限控制：管理页仅管理员可见，后端 API 鉴权</Typography>
          <Typography variant="body2">• 工作量录入自动填充用户名</Typography>
          <Typography variant="body2">• 研发送样表格优化：级联选择 + 列宽优化</Typography>
          <Typography variant="body2">• 事业部统计 + 导出新增事业部 Sheet</Typography>
          <Typography variant="body2">• 管理页样品登记卡片化</Typography>
          <Typography variant="body2">• 自定义列统一模板 + 附件类型支持</Typography>
        </Alert>
        <Typography variant="subtitle2" sx={{ fontWeight: 600, mb: 1 }}>功能特性</Typography>
        <Box sx={{ textAlign: 'left', px: 1 }}>{F_LIST.map((text, i) => <Typography key={i} variant="body2" sx={{ py: 0.5 }}>{text}</Typography>)}</Box>
        <Typography variant="caption" sx={{ mt: 3, display: 'block', color: 'text.disabled', textAlign: 'center' }}>&copy; 2026 HotLL</Typography>
      </DialogContent>
      <BackToTop />
    </Dialog>
  </Box>);
};
export default Layout;
