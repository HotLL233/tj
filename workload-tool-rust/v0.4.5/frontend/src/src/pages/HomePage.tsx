import React from 'react';
import { useNavigate } from 'react-router-dom';
import { Box, Typography, Paper, useMediaQuery, useTheme } from '@mui/material';
import ScienceIcon from '@mui/icons-material/Science';
import BarChartIcon from '@mui/icons-material/BarChart';

const R = '2px';
const HomePage: React.FC = () => {
  const n = useNavigate();
  const theme = useTheme();
  const isMobile = useMediaQuery(theme.breakpoints.down('sm'));

  return (
    <Box sx={{ maxWidth: 900, mx: 'auto', mt: { xs: 2, md: 6 } }}>
      {/* Header */}
      <Box sx={{ textAlign: 'center', mb: 5 }}>
        <Typography variant="h3" fontWeight={800} sx={{ fontFamily: '"方正舒体", "FZSJ-KAFJT", "KaiTi", "STKaiti", "cursive", serif', color: '#667eea', mb: 1, fontSize: { xs: '2.8rem', md: '4rem' } }}>
          知微
        </Typography>
      </Box>

      {/* Two big cards */}
      <Box sx={{ display: 'flex', flexDirection: isMobile ? 'column' : 'row', gap: 3, flexWrap: 'wrap', justifyContent: 'center', alignItems: isMobile ? 'center' : undefined, mb: 4 }}>
        {/* 研发送样 */}
        <Paper
          elevation={0}
          onClick={() => n('/sample')}
          sx={{
            flex: isMobile ? '1 1 100%' : '1 1 240px', maxWidth: isMobile ? '100%' : 320, p: { xs: 3, md: 4 }, borderRadius: R, cursor: 'pointer',
            background: 'linear-gradient(145deg,#fff3e0,#ffe0b2)',
            border: '2px solid #e65100',
            boxShadow: '0 8px 32px rgba(230,81,0,0.12)',
            transition: 'all 0.2s',
            '&:hover': { transform: 'translateY(-4px)', boxShadow: '0 12px 40px rgba(230,81,0,0.2)' },
            textAlign: 'center', width: isMobile ? '100%' : undefined,
          }}
        >
          <ScienceIcon sx={{ fontSize: 56, color: '#e65100', mb: 1.5 }} />
          <Typography variant="h5" fontWeight={700} color="#e65100" gutterBottom>研发送样</Typography>
          <Typography variant="body2" color="text.secondary">送样录入 · 查看记录</Typography>
        </Paper>

        {/* 分析检测 */}
        <Paper
          elevation={0}
          onClick={() => n('/workload')}
          sx={{
            flex: isMobile ? '1 1 100%' : '1 1 240px', maxWidth: isMobile ? '100%' : 320, p: { xs: 3, md: 4 }, borderRadius: R, cursor: 'pointer',
            background: 'linear-gradient(145deg,#e8eaf6,#c5cae9)',
            border: '2px solid #283593',
            boxShadow: '0 8px 32px rgba(40,53,147,0.12)',
            transition: 'all 0.2s',
            '&:hover': { transform: 'translateY(-4px)', boxShadow: '0 12px 40px rgba(40,53,147,0.2)' },
            textAlign: 'center', width: isMobile ? '100%' : undefined,
          }}
        >
          <BarChartIcon sx={{ fontSize: 56, color: '#283593', mb: 1.5 }} />
          <Typography variant="h5" fontWeight={700} color="#283593" gutterBottom>分析检测</Typography>
          <Typography variant="body2" color="text.secondary">检测录入 · 统计 · 管理</Typography>
        </Paper>
      </Box>

      <Box sx={{ textAlign: 'center' }}>
        <Typography variant="body1" color="text.secondary">选择功能入口，开始操作</Typography>
      </Box>
    </Box>
  );
};
export default HomePage;
