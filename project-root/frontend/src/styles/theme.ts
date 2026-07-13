import { createTheme } from '@mui/material/styles';

// 模块主题色（与后端约定一致）
export const MODULE_COLORS = {
  work: '#1976d2', // 分析检测（蓝）
  rd: '#e65100', // 研发送样（橙）
  ops: '#2e7d32', // 运营管理（绿）
  sys: '#546e7a', // 系统管理（灰）
};

const theme = createTheme({
  palette: {
    primary: { main: MODULE_COLORS.work },
    secondary: { main: MODULE_COLORS.ops },
    background: { default: '#f5f7fa' },
  },
  shape: { borderRadius: 2 },
  typography: {
    fontFamily: ['-apple-system', 'BlinkMacSystemFont', '"Segoe UI"', 'Roboto', '"Helvetica Neue"', 'Arial', '"Noto Sans SC"', 'sans-serif'].join(','),
    h4: { fontWeight: 700 },
    h5: { fontWeight: 700 },
    h6: { fontWeight: 700 },
  },
  components: {
    MuiPaper: {
      styleOverrides: {
        root: { backgroundImage: 'none' },
        elevation1: { boxShadow: '0 4px 20px rgba(0,0,0,0.06), 0 1px 3px rgba(0,0,0,0.04)' },
        elevation2: { boxShadow: '0 8px 30px rgba(0,0,0,0.08), 0 2px 6px rgba(0,0,0,0.04)' },
      },
    },
    MuiButton: { styleOverrides: { root: { borderRadius: 2, textTransform: 'none', fontWeight: 600 } } },
    MuiChip: { styleOverrides: { root: { borderRadius: 2 } } },
    MuiDialog: { styleOverrides: { paper: { borderRadius: 2 } } },
  },
  breakpoints: { values: { xs: 0, sm: 640, md: 768, lg: 1024, xl: 1280 } },
});

export default theme;
