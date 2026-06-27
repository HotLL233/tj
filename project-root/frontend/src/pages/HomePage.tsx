import React from 'react';
import { useNavigate } from 'react-router-dom';
import { Box, Typography, Paper } from '@mui/material';
import ScienceIcon from '@mui/icons-material/Science';
import BarChartIcon from '@mui/icons-material/BarChart';

const R = '2px';
const HomePage: React.FC = () => {
  const n = useNavigate();

  return (
    <Box sx={{ maxWidth: 900, mx: 'auto', mt: { xs: 2, md: 6 } }}>
      {/* Header */}
      <Box sx={{ textAlign: 'center', mb: 5 }}>
        <Typography variant="h3" fontWeight={800} sx={{ background: 'linear-gradient(135deg,#667eea,#764ba2)', WebkitBackgroundClip: 'text', WebkitTextFillColor: 'transparent', mb: 1 }}>
          工作量统计工具
        </Typography>
        <Typography variant="body1" color="text.secondary">选择功能入口，开始操作</Typography>
      </Box>

      {/* Two big cards */}
      <Box sx={{ display: 'flex', flexDirection: { xs: 'column', sm: 'row' }, gap: 3, flexWrap: 'wrap', justifyContent: 'center' }}>
        {/* 实验室送样 */}
        <Paper
          elevation={0}
          onClick={() => n('/sample')}
          sx={{
            flex: '1 1 240px', maxWidth: 320, p: { xs: 3, md: 4 }, borderRadius: R, cursor: 'pointer',
            background: 'linear-gradient(145deg,#fff3e0,#ffe0b2)',
            border: '2px solid #e65100',
            boxShadow: '0 8px 32px rgba(230,81,0,0.12)',
            transition: 'all 0.2s',
            '&:hover': { transform: 'translateY(-4px)', boxShadow: '0 12px 40px rgba(230,81,0,0.2)' },
            textAlign: 'center',
          }}
        >
          <ScienceIcon sx={{ fontSize: 56, color: '#e65100', mb: 1.5 }} />
          <Typography variant="h5" fontWeight={700} color="#e65100" gutterBottom>实验室送样</Typography>
          <Typography variant="body2" color="text.secondary">送样录入 · 查看记录</Typography>
        </Paper>

        {/* 工作量录入 */}
        <Paper
          elevation={0}
          onClick={() => n('/workload')}
          sx={{
            flex: '1 1 240px', maxWidth: 320, p: { xs: 3, md: 4 }, borderRadius: R, cursor: 'pointer',
            background: 'linear-gradient(145deg,#e8eaf6,#c5cae9)',
            border: '2px solid #283593',
            boxShadow: '0 8px 32px rgba(40,53,147,0.12)',
            transition: 'all 0.2s',
            '&:hover': { transform: 'translateY(-4px)', boxShadow: '0 12px 40px rgba(40,53,147,0.2)' },
            textAlign: 'center',
          }}
        >
          <BarChartIcon sx={{ fontSize: 56, color: '#283593', mb: 1.5 }} />
          <Typography variant="h5" fontWeight={700} color="#283593" gutterBottom>工作量录入</Typography>
          <Typography variant="body2" color="text.secondary">检测录入 · 统计 · 管理</Typography>
        </Paper>
      </Box>
    </Box>
  );
};
export default HomePage;
