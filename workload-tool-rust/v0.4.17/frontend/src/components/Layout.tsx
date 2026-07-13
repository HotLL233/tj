import React, { useState } from 'react';
import { Outlet, useNavigate, useLocation } from 'react-router-dom';
import { AppBar, Toolbar, Typography, IconButton, Button, BottomNavigation, BottomNavigationAction, Drawer, List, ListItem, ListItemIcon, ListItemText, Dialog, DialogTitle, DialogContent, DialogActions, TextField, Alert, useMediaQuery, useTheme, Box, Container } from '@mui/material';
import HomeIcon from '@mui/icons-material/Home'; import SettingsIcon from '@mui/icons-material/Settings'; import MenuBookIcon from '@mui/icons-material/MenuBook'; import MenuIcon from '@mui/icons-material/Menu'; import InfoIcon from '@mui/icons-material/Info';
import { adminLogin } from '../api/client';

const NAV_ITEMS = [
  { label: '主页', path: '/', icon: <HomeIcon /> },
  { label: '教程与帮助', path: '/help', icon: <MenuBookIcon /> },
  { label: '管理', path: '/manage', icon: <SettingsIcon /> },
];
const MOBILE_NAV = [
  { label: '主页', path: '/', icon: <HomeIcon /> },
  { label: '教程与帮助', path: '/help', icon: <MenuBookIcon /> },
  { label: '管理', path: '/manage', icon: <SettingsIcon /> },
];
const F_LIST = ['🔬 研发送样与工作量双入口','📊 按周/月/日多维度统计','🔬 液相/气相仪器分类统计','📦 研发送样记录与专属统计','📋 汇总模板格式导出','👤 用户日志与纠错编辑','🔧 分组/项目管理','📝 审计日志 / 🗑️ 回收站'];

const Layout: React.FC = () => {
  const navigate = useNavigate(); const location = useLocation(); const theme = useTheme(); const isMobile = useMediaQuery(theme.breakpoints.down('md'));
  const [drawerOpen, setDrawerOpen] = useState(false); const [aboutOpen, setAboutOpen] = useState(false);
  const [serverVer, setServerVer] = useState('');

  // 管理入口认证状态
  const [authOpen, setAuthOpen] = useState(false);
  const [authUser, setAuthUser] = useState('');
  const [authPass, setAuthPass] = useState('');
  const [authErr, setAuthErr] = useState('');
  const [authLoading, setAuthLoading] = useState(false);

  const getMobileNav = (): number => { const p = location.pathname; if (p === '/') return 0; if (p.startsWith('/help')) return 1; if (p.startsWith('/manage')) return 2; return 0; };
  React.useEffect(() => { fetch('/api/version').then(r => r.json()).then(d => setServerVer(d.version || '')).catch(() => {}); }, []);

  // 验证并进入管理页
  const handleAdminLogin = async () => {
    if (!authUser.trim()) { setAuthErr('请输入管理员账号'); return; }
    if (!authPass.trim()) { setAuthErr('请输入管理员密码'); return; }
    setAuthLoading(true); setAuthErr('');
    try {
      const r = await adminLogin({ username: authUser, password: authPass });
      if (r.code === 0) {
        sessionStorage.setItem('admin_token', r.data?.token || '');
        setAuthOpen(false);
        setAuthUser(''); setAuthPass('');
        navigate('/manage');
      } else {
        setAuthErr(r.message || '验证失败');
      }
    } catch {
      setAuthErr('网络错误，请重试');
    } finally {
      setAuthLoading(false);
    }
  };

  const handleManageClick = () => {
    // 已有有效token则直接进入
    const token = sessionStorage.getItem('admin_token');
    if (token) { navigate('/manage'); return; }
    // 弹出认证对话框
    setAuthErr('');
    setAuthUser('');
    setAuthPass('');
    setAuthOpen(true);
  };

  return (<Box sx={{ display: 'flex', flexDirection: 'column', minHeight: '100vh' }}>
    <AppBar position="sticky" elevation={0} className="glass-appbar" sx={{ zIndex: theme.zIndex.drawer + 1 }}><Toolbar>
      {isMobile && <IconButton edge="start" onClick={() => setDrawerOpen(true)} sx={{ mr: 1, color: '#333' }}><MenuIcon /></IconButton>}
      <Box sx={{ flexGrow: 1 }} />
      {!isMobile && <Box sx={{ display: 'flex', gap: 0.5 }}>{NAV_ITEMS.map(item => { const isActive = location.pathname === item.path; return <Button key={item.path} startIcon={item.icon} onClick={() => item.path === '/manage' ? handleManageClick() : navigate(item.path)} className={isActive ? 'nav-pill-active' : 'nav-pill'} sx={{ color: isActive ? '#fff' : '#555' }}>{item.label}</Button>; })}</Box>}
      <IconButton onClick={() => setAboutOpen(true)} title="关于" sx={{ ml: { xs: 0, md: 1 }, color: '#555' }}><InfoIcon /></IconButton>
    </Toolbar></AppBar>

    <Drawer anchor="left" open={drawerOpen} onClose={() => setDrawerOpen(false)}><Box sx={{ width: 250, pt: 2, bgcolor: '#f8fafc', height: '100%' }}>
      <Typography variant="h6" sx={{ px: 2, pb: 2, fontWeight: 700 }}>知微</Typography>
      <List>{NAV_ITEMS.map(item => { const isActive = location.pathname === item.path; return <ListItem key={item.path} component="div" onClick={() => { item.path === '/manage' ? handleManageClick() : navigate(item.path); setDrawerOpen(false); }} sx={{ cursor: 'pointer', mx: 1, mb: 0.5, borderRadius: '2px', bgcolor: isActive ? 'rgba(102,126,234,0.12)' : 'transparent' }}><ListItemIcon sx={{ color: isActive ? '#667eea' : undefined }}>{item.icon}</ListItemIcon><ListItemText primary={item.label} primaryTypographyProps={{ fontWeight: isActive ? 700 : 400, color: isActive ? '#667eea' : undefined }} /></ListItem>; })}</List>
    </Box></Drawer>

    <Box component="main" sx={{ flexGrow: 1, pb: isMobile ? 7 : 2, pt: 2, px: { xs: 1, sm: 2, md: 3 } }}><Container maxWidth="lg" disableGutters={isMobile}><Outlet /></Container></Box>

    {isMobile && <BottomNavigation value={getMobileNav()} onChange={(_e, v: number) => { if (v === 0) navigate('/'); else if (v === 1) navigate('/help'); else handleManageClick(); }} className="glass-bottom-nav" sx={{ position: 'fixed', bottom: 0, left: 0, right: 0, zIndex: theme.zIndex.appBar, borderTop: '1px solid rgba(0,0,0,0.06)' }}>{MOBILE_NAV.map((item, i) => <BottomNavigationAction key={i} label={item.label} icon={item.icon} />)}</BottomNavigation>}

    {/* 管理入口认证对话框 */}
    <Dialog open={authOpen} onClose={() => setAuthOpen(false)} maxWidth="xs" fullWidth PaperProps={{ sx: { borderRadius: '2px' } }}>
      <DialogTitle sx={{ fontWeight: 700 }}>管理员验证</DialogTitle>
      <DialogContent>
        {authErr && <Alert severity="error" sx={{ mb: 2, borderRadius: '2px' }}>{authErr}</Alert>}
        <TextField autoFocus label="管理员账号" fullWidth value={authUser} onChange={e => setAuthUser(e.target.value)} sx={{ mt: 1, '& .MuiOutlinedInput-root': { borderRadius: '2px' } }} onKeyDown={e => e.key === 'Enter' && handleAdminLogin()} />
        <TextField label="管理员密码" type="password" fullWidth value={authPass} onChange={e => setAuthPass(e.target.value)} sx={{ mt: 2, '& .MuiOutlinedInput-root': { borderRadius: '2px' } }} onKeyDown={e => e.key === 'Enter' && handleAdminLogin()} />
      </DialogContent>
      <DialogActions>
        <Button onClick={() => setAuthOpen(false)} sx={{ borderRadius: '2px' }}>取消</Button>
        <Button onClick={handleAdminLogin} variant="contained" disabled={authLoading} sx={{ borderRadius: '2px' }}>{authLoading ? '验证中...' : '确认'}</Button>
      </DialogActions>
    </Dialog>

    <Dialog open={aboutOpen} onClose={() => setAboutOpen(false)} maxWidth="sm" fullWidth PaperProps={{ sx: { borderRadius: '2px' } }}>
      <DialogTitle sx={{ fontWeight: 700, textAlign: 'center' }}>知微</DialogTitle>
      <DialogContent>
        <Typography variant="subtitle2" color="primary" sx={{ mb: 2, fontWeight: 600, textAlign: 'center' }}>v{serverVer || '...'}</Typography>
        <Alert severity="info" sx={{ mb: 2, borderRadius: '2px' }}>
          <Typography variant="subtitle2" sx={{ fontWeight: 600, mb: 0.5 }}>v0.4.0 更新内容</Typography>
          <Typography variant="body2">• 新增「研发送样」模块：与分析检测同级入口，录入数据独立存储、共用同一套主数据</Typography>
          <Typography variant="body2">• 研发送样统计页完整克隆（10 个 Sheet 预览），导出文件名「研发送样统计」</Typography>
          <Typography variant="body2">• 审计日志按模块隔离：分析检测 / 研发送样 / 共享主数据，回收站按模块展示</Typography>
          <Typography variant="body2">• 退役旧「送样」模块</Typography>
          <Typography variant="body2">• 修复回收站恢复功能：恢复按钮 URL 路径修正</Typography>
          <Typography variant="body2">• 修复审计日志路径错误：前端 /audit 改为 /audit-logs</Typography>
          <Typography variant="body2" sx={{ mt: 0.5, color: 'text.secondary' }}>v0.3.25: 修复导出/统计中实验室对应关系错误（work_records.group_id）</Typography>
          <Typography variant="body2" sx={{ color: 'text.secondary' }}>v0.3.23: 管理界面新增一览按钮 + 方法一览实时更新 + 删除导入功能 + 导出格式全局统一</Typography>
          <Typography variant="body2" sx={{ color: 'text.secondary' }}>v0.3.22: 修复导出格式、人员汇总公式、导出数量翻倍、number输入框spinner</Typography>
        </Alert>
        <Typography variant="subtitle2" sx={{ fontWeight: 600, mb: 1 }}>功能特性</Typography>
        <Box sx={{ textAlign: 'left', px: 1 }}>{F_LIST.map((text, i) => <Typography key={i} variant="body2" sx={{ py: 0.5 }}>{text}</Typography>)}</Box>
        <Typography variant="caption" sx={{ mt: 3, display: 'block', color: 'text.disabled', textAlign: 'center' }}>&copy; 2026 HotLL</Typography>
      </DialogContent>
    </Dialog>
  </Box>);
};
export default Layout;
