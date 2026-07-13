import React from 'react';
import { useNavigate } from 'react-router-dom';
import { Box, Typography, Paper, Grid, useMediaQuery, useTheme, IconButton } from '@mui/material';
import ArrowBackIcon from '@mui/icons-material/ArrowBack';
import ScienceIcon from '@mui/icons-material/Science';
import WhatshotIcon from '@mui/icons-material/Whatshot';
import BiotechIcon from '@mui/icons-material/Biotech';
import MoreHorizIcon from '@mui/icons-material/MoreHoriz';
import ListAltIcon from '@mui/icons-material/ListAlt';

const R = '2px';
const CATEGORIES = [
  { type: 'ICP', label: 'ICP', desc: '电感耦合等离子体检测', icon: <ScienceIcon sx={{ fontSize: 48, color: '#2e7d32' }} />, bg: 'linear-gradient(145deg,#e8f5e9,#c8e6c9)', border: '#2e7d32' },
  { type: '热稳定性', label: '热分析', desc: '热稳定性 · TGA · DSC 检测', icon: <WhatshotIcon sx={{ fontSize: 48, color: '#e65100' }} />, bg: 'linear-gradient(145deg,#fff3e0,#ffe0b2)', border: '#e65100' },
  { type: '质谱', label: '质谱', desc: '质谱分析检测', icon: <BiotechIcon sx={{ fontSize: 48, color: '#6a1b9a' }} />, bg: 'linear-gradient(145deg,#f3e5f5,#e1bee7)', border: '#6a1b9a' },
  { type: '其他', label: '其他', desc: '液相 · 气相 · 理化等', icon: <MoreHorizIcon sx={{ fontSize: 48, color: '#0277bd' }} />, bg: 'linear-gradient(145deg,#e1f5fe,#b3e5fc)', border: '#0277bd' },
];

const SampleInfoHome: React.FC = () => {
  const n = useNavigate();
  const theme = useTheme();
  const isMobile = useMediaQuery(theme.breakpoints.down('sm'));

  return (
    <Box sx={{ maxWidth: 900, mx: 'auto', mt: { xs: 1, md: 3 }, px: { xs: 1, md: 2 } }}>
      {/* 顶部标题栏 */}
      <Box sx={{ display: 'flex', alignItems: 'center', mb: 3, gap: 1 }}>
        <IconButton onClick={() => n('/')} size="small"><ArrowBackIcon /></IconButton>
        <Box>
          <Typography variant="h5" fontWeight={700} color="#2e7d32">样品信息登记</Typography>
          <Typography variant="body2" color="text.secondary">ICP、热分析、质谱等原样品基础信息填写</Typography>
        </Box>
      </Box>

      {/* 大类卡片 */}
      <Grid container spacing={3} sx={{ mb: 4 }}>
        {CATEGORIES.map((cat) => (
          <Grid item xs={12} sm={6} key={cat.type}>
            <Paper
              elevation={0}
              onClick={() => n(`/sample-info/entry?type=${encodeURIComponent(cat.type)}`)}
              sx={{
                p: { xs: 3, md: 4 }, borderRadius: R, cursor: 'pointer',
                background: cat.bg,
                border: `2px solid ${cat.border}`,
                boxShadow: `0 8px 32px ${cat.border}22`,
                transition: 'all 0.2s',
                '&:hover': { transform: 'translateY(-4px)', boxShadow: `0 12px 40px ${cat.border}33` },
                textAlign: 'center',
              }}
            >
              <Box sx={{ mb: 1.5 }}>{cat.icon}</Box>
              <Typography variant="h6" fontWeight={700} color={cat.border} gutterBottom>{cat.label}</Typography>
              <Typography variant="body2" color="text.secondary">{cat.desc}</Typography>
            </Paper>
          </Grid>
        ))}
      </Grid>

      {/* 底部链接 */}
      <Box sx={{ textAlign: 'center' }}>
        <Paper
          elevation={0}
          onClick={() => n('/sample-info/entry')}
          sx={{
            display: 'inline-flex', alignItems: 'center', gap: 1, px: 3, py: 1.5, borderRadius: R, cursor: 'pointer',
            border: '1px solid #2e7d32', color: '#2e7d32',
            transition: 'all 0.2s',
            '&:hover': { bgcolor: '#e8f5e9' },
          }}
        >
          <ListAltIcon sx={{ fontSize: 22 }} />
          <Typography variant="body2" fontWeight={600}>查看全部登记记录</Typography>
        </Paper>
      </Box>
    </Box>
  );
};
export default SampleInfoHome;
