import React, { useState } from 'react';
import { Outlet, useNavigate, useLocation } from 'react-router-dom';
import {
  AppBar,
  Toolbar,
  Typography,
  IconButton,
  Button,
  BottomNavigation,
  BottomNavigationAction,
  Drawer,
  List,
  ListItem,
  ListItemIcon,
  ListItemText,
  Dialog,
  DialogTitle,
  DialogContent,
  useMediaQuery,
  useTheme,
  Box,
  Container,
} from '@mui/material';
import HomeIcon from '@mui/icons-material/Home';
import EditNoteIcon from '@mui/icons-material/EditNote';
import BarChartIcon from '@mui/icons-material/BarChart';
import SettingsIcon from '@mui/icons-material/Settings';
import MenuIcon from '@mui/icons-material/Menu';
import InfoIcon from '@mui/icons-material/Info';

const NAV_ITEMS = [
  { label: '主页', path: '/', icon: <HomeIcon /> },
  { label: '统计', path: '/stats', icon: <BarChartIcon /> },
  { label: '管理', path: '/manage', icon: <SettingsIcon /> },
];

const MOBILE_NAV_ITEMS = [
  { label: '主页', path: '/', icon: <HomeIcon /> },
  { label: '录入', path: '/', icon: <EditNoteIcon /> },
  { label: '统计', path: '/stats', icon: <BarChartIcon /> },
  { label: '管理', path: '/manage', icon: <SettingsIcon /> },
];

const FEATURE_LIST = [
  { emoji: '📊', text: '工作量录入与统计' },
  { emoji: '📅', text: '按周/月/日多维度统计' },
  { emoji: '🔬', text: '液相/气相仪器分类统计' },
  { emoji: '📋', text: '汇总模板格式导出' },
  { emoji: '👤', text: '用户日志与纠错编辑' },
  { emoji: '🔧', text: '分组/项目管理' },
  { emoji: '📝', text: '审计日志' },
  { emoji: '🗑️', text: '回收站' },
];

/**
 * Layout component with glassmorphism AppBar, pill navigation,
 * mobile bottom nav, and sidebar drawer.
 */
const Layout: React.FC = () => {
  const navigate = useNavigate();
  const location = useLocation();
  const theme = useTheme();
  const isMobile = useMediaQuery(theme.breakpoints.down('md'));
  const [drawerOpen, setDrawerOpen] = useState(false);
  const [aboutOpen, setAboutOpen] = useState(false);

  const getMobileNavValue = (): number => {
    const path = location.pathname;
    if (path === '/') return 0;
    if (path.startsWith('/entry/')) return 1;
    if (path.startsWith('/stats')) return 2;
    if (path.startsWith('/manage')) return 3;
    return 0;
  };

  /** Determine per-page gradient accent for the active pill */
  const getActiveGradient = (): string => {
    const path = location.pathname;
    if (path === '/') return 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)';
    if (path.startsWith('/stats')) return 'linear-gradient(135deg, #00897b, #43a047)';
    if (path.startsWith('/manage')) return 'linear-gradient(135deg, #f4511e, #e53935)';
    if (path.startsWith('/entry/')) return 'linear-gradient(135deg, #1e88e5, #00acc1)';
    return 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)';
  };

  return (
    <Box sx={{ display: 'flex', flexDirection: 'column', minHeight: '100vh' }}>
      {/* Top AppBar — glassmorphism */}
      <AppBar
        position="sticky"
        elevation={0}
        className="glass-appbar"
        sx={{ zIndex: theme.zIndex.drawer + 1 }}
      >
        <Toolbar>
          {isMobile && (
            <IconButton
              edge="start"
              onClick={() => setDrawerOpen(true)}
              sx={{ mr: 1, color: '#333' }}
            >
              <MenuIcon />
            </IconButton>
          )}
          <Typography
            variant="h6"
            component="div"
            sx={{ flexGrow: 1, fontWeight: 700, color: '#333' }}
          >
            工作量统计
          </Typography>
          {!isMobile && (
            <Box sx={{ display: 'flex', gap: 0.5 }}>
              {NAV_ITEMS.map((item) => {
                const isActive = location.pathname === item.path;
                return (
                  <Button
                    key={item.path}
                    startIcon={item.icon}
                    onClick={() => navigate(item.path)}
                    className={isActive ? 'nav-pill-active' : 'nav-pill'}
                    sx={{
                      color: isActive ? '#fff' : '#555',
                      '&:hover': {
                        bgcolor: isActive ? undefined : 'rgba(102,126,234,0.08)',
                      },
                    }}
                  >
                    {item.label}
                  </Button>
                );
              })}
            </Box>
          )}
          <IconButton
            onClick={() => setAboutOpen(true)}
            title="关于"
            sx={{ ml: { xs: 0, md: 1 }, color: '#555' }}
          >
            <InfoIcon />
          </IconButton>
        </Toolbar>
      </AppBar>

      {/* Mobile Drawer */}
      <Drawer
        anchor="left"
        open={drawerOpen}
        onClose={() => setDrawerOpen(false)}
      >
        <Box sx={{ width: 250, pt: 2, bgcolor: '#f8fafc', height: '100%' }}>
          <Typography variant="h6" sx={{ px: 2, pb: 2, fontWeight: 700 }}>
            工作量统计
          </Typography>
          <List>
            {NAV_ITEMS.map((item) => {
              const isActive = location.pathname === item.path;
              return (
                <ListItem
                  key={item.path}
                  component="div"
                  onClick={() => {
                    navigate(item.path);
                    setDrawerOpen(false);
                  }}
                  sx={{
                    cursor: 'pointer',
                    mx: 1,
                    mb: 0.5,
                    borderRadius: 3,
                    bgcolor: isActive
                      ? 'rgba(102,126,234,0.12)'
                      : 'transparent',
                    '&:hover': {
                      bgcolor: 'rgba(102,126,234,0.06)',
                    },
                  }}
                >
                  <ListItemIcon sx={{ color: isActive ? '#667eea' : undefined }}>
                    {item.icon}
                  </ListItemIcon>
                  <ListItemText
                    primary={item.label}
                    primaryTypographyProps={{
                      fontWeight: isActive ? 700 : 400,
                      color: isActive ? '#667eea' : undefined,
                    }}
                  />
                </ListItem>
              );
            })}
          </List>
        </Box>
      </Drawer>

      {/* Main content */}
      <Box
        component="main"
        sx={{
          flexGrow: 1,
          pb: isMobile ? 7 : 2,
          pt: 2,
          px: { xs: 1, sm: 2, md: 3 },
        }}
      >
        <Container maxWidth="lg" disableGutters={isMobile}>
          <Outlet />
        </Container>
      </Box>

      {/* Mobile Bottom Navigation — glassmorphism */}
      {isMobile && (
        <BottomNavigation
          value={getMobileNavValue()}
          onChange={(_e, newValue: number) => {
            if (newValue === 0) navigate('/');
            else if (newValue === 1) navigate('/');
            else if (newValue === 2) navigate('/stats');
            else if (newValue === 3) navigate('/manage');
          }}
          className="glass-bottom-nav"
          sx={{
            position: 'fixed',
            bottom: 0,
            left: 0,
            right: 0,
            zIndex: theme.zIndex.appBar,
            borderTop: '1px solid',
            borderColor: 'rgba(0,0,0,0.06)',
            '& .MuiBottomNavigationAction-root': {
              minWidth: 'auto',
              py: 0.5,
              color: '#888',
            },
            '& .Mui-selected': {
              color: '#667eea',
            },
          }}
        >
          {MOBILE_NAV_ITEMS.map((item, idx) => (
            <BottomNavigationAction
              key={idx}
              label={item.label}
              icon={item.icon}
            />
          ))}
        </BottomNavigation>
      )}

      {/* About Dialog */}
      <Dialog
        open={aboutOpen}
        onClose={() => setAboutOpen(false)}
        maxWidth="xs"
        fullWidth
        PaperProps={{
          sx: { borderRadius: 4 },
        }}
      >
        <DialogTitle sx={{ fontWeight: 700, textAlign: 'center' }}>
          工作量统计工具
        </DialogTitle>
        <DialogContent
          sx={{
            textAlign: 'center',
          }}
        >
          <Typography
            variant="subtitle2"
            color="primary"
            sx={{ mb: 2, fontWeight: 600 }}
          >
            v1.7.0
          </Typography>
          <Box sx={{ textAlign: 'left', px: 2 }}>
            {FEATURE_LIST.map((item) => (
              <Typography
                key={item.text}
                variant="body2"
                sx={{ py: 0.5, display: 'flex', alignItems: 'center', gap: 1 }}
              >
                <Box component="span" sx={{ fontSize: '1.1rem' }}>
                  {item.emoji}
                </Box>
                {item.text}
              </Typography>
            ))}
          </Box>
          <Typography
            variant="caption"
            sx={{ mt: 3, display: 'block', color: 'text.disabled' }}
          >
            &copy; 2026 HotLL
          </Typography>
        </DialogContent>
      </Dialog>
    </Box>
  );
};

export default Layout;
