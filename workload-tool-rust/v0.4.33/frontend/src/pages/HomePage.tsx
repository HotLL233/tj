import React from 'react';
import { useNavigate } from 'react-router-dom';
import { Box, Typography, Paper, useMediaQuery, useTheme } from '@mui/material';
import ScienceIcon from '@mui/icons-material/Science';
import BarChartIcon from '@mui/icons-material/BarChart';
import AssignmentIcon from '@mui/icons-material/Assignment';
import { useUser } from '../UserContext';
import { Button } from '@mui/material';

const R = '2px';
const HomePage: React.FC = () => {
  const n = useNavigate();
  const theme = useTheme();
  const isMobile = useMediaQuery(theme.breakpoints.down('sm'));
  const { hasPermission, isLoggedIn } = useUser();

  // 三张入口卡片：按权限点门控，无权限且非管理员则隐藏
  const cards = [
    {
      perm: 'entry:sample',
      path: '/sample',
      icon: <ScienceIcon sx={{ fontSize: 56, color: '#e65100', mb: 1.5 }} />,
      title: '研发送样',
      subtitle: '送样录入 · 查看记录',
      grad: 'linear-gradient(145deg,#fff3e0,#ffe0b2)',
      border: '#e65100',
      titleColor: '#e65100',
    },
    {
      perm: 'entry:workload',
      path: '/workload',
      icon: <BarChartIcon sx={{ fontSize: 56, color: '#283593', mb: 1.5 }} />,
      title: '分析检测',
      subtitle: '检测录入 · 统计 · 管理',
      grad: 'linear-gradient(145deg,#e8eaf6,#c5cae9)',
      border: '#283593',
      titleColor: '#283593',
    },
    {
      perm: 'entry:sample-info',
      path: '/sample-info',
      icon: <AssignmentIcon sx={{ fontSize: 56, color: '#2e7d32', mb: 1.5 }} />,
      title: '样品信息登记',
      subtitle: 'ICP · 热分析 · 质谱等样品信息填写',
      grad: 'linear-gradient(145deg,#e8f5e9,#c8e6c9)',
      border: '#2e7d32',
      titleColor: '#2e7d32',
    },
  ].filter((c) => hasPermission(c.perm));

  return (
    <Box sx={{ maxWidth: 900, mx: 'auto', mt: { xs: 2, md: 6 } }}>
      {/* Header */}
      <Box sx={{ textAlign: 'center', mb: 5 }}>
        <Typography variant="h3" fontWeight={800} sx={{ fontFamily: '"方正舒体", "FZSJ-KAFJT", "KaiTi", "STKaiti", "cursive", serif', color: '#667eea', mb: 1, fontSize: { xs: '2.8rem', md: '4rem' } }}>
          知微
        </Typography>
      </Box>

      {cards.length === 0 ? (
        <Box sx={{ textAlign: 'center', mt: 6 }}>
          {isLoggedIn ? (
            <Typography variant="h6" color="text.secondary">您暂无可用功能入口，请联系管理员</Typography>
          ) : (
            <>
              <Typography variant="h6" color="text.secondary" sx={{ mb: 2 }}>请先登录系统</Typography>
              <Button variant="contained" onClick={() => n('/login')} sx={{ px: 4, py: 1.2, borderRadius: '2px' }}>
                去登录
              </Button>
            </>
          )}
        </Box>
      ) : (
        <Box sx={{ display: 'flex', flexDirection: isMobile ? 'column' : 'row', gap: 3, flexWrap: 'wrap', justifyContent: 'center', alignItems: isMobile ? 'center' : undefined, mb: 4 }}>
          {cards.map((c) => (
            <Paper
              key={c.path}
              elevation={0}
              onClick={() => n(c.path)}
              sx={{
                flex: isMobile ? '1 1 100%' : '1 1 240px', maxWidth: isMobile ? '100%' : 320, p: { xs: 3, md: 4 }, borderRadius: R, cursor: 'pointer',
                background: c.grad,
                border: `2px solid ${c.border}`,
                boxShadow: `0 8px 32px ${c.border}1f`,
                transition: 'all 0.2s',
                '&:hover': { transform: 'translateY(-4px)', boxShadow: `0 12px 40px ${c.border}33` },
                textAlign: 'center', width: isMobile ? '100%' : undefined,
              }}
            >
              {c.icon}
              <Typography variant="h5" fontWeight={700} color={c.titleColor} gutterBottom>{c.title}</Typography>
              <Typography variant="body2" color="text.secondary">{c.subtitle}</Typography>
            </Paper>
          ))}
        </Box>
      )}

      <Box sx={{ textAlign: 'center' }}>
        <Typography variant="body1" color="text.secondary">选择功能入口，开始操作</Typography>
      </Box>
    </Box>
  );
};
export default HomePage;
