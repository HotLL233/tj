import React from 'react';
import { useNavigate } from 'react-router-dom';
import { Box, Grid, Paper, Typography, CardActionArea } from '@mui/material';
import ScienceIcon from '@mui/icons-material/Science';
import InventoryIcon from '@mui/icons-material/Inventory2';
import ShoppingCartIcon from '@mui/icons-material/ShoppingCart';
import ApprovalIcon from '@mui/icons-material/Approval';
import NotificationsIcon from '@mui/icons-material/Notifications';
import BarChartIcon from '@mui/icons-material/BarChart';
import { useAuth } from '../context/AuthContext';
import { MODULE_COLORS } from '../styles/theme';

interface Card { text: string; desc: string; path: string; perm?: string; icon: React.ReactNode; color: string; }

const CARDS: Card[] = [
  { text: '仪器管理', desc: '档案 / 预约 / 保养 / 二维码', path: '/instruments', perm: 'instrument:read', icon: <ScienceIcon />, color: MODULE_COLORS.ops },
  { text: '库存管理', desc: '物料 / 批次 / 出入库流水', path: '/inventory', perm: 'inventory:read', icon: <InventoryIcon />, color: MODULE_COLORS.ops },
  { text: '采购管理', desc: '申请 / 采购单 / 供应商', path: '/purchase', perm: 'purchase:read', icon: <ShoppingCartIcon />, color: MODULE_COLORS.ops },
  { text: '审批中心', desc: '待我审批 / 审批规则', path: '/approval', perm: 'approval:read', icon: <ApprovalIcon />, color: MODULE_COLORS.ops },
  { text: '通知', desc: '站内消息与待办提醒', path: '/notifications', perm: 'notification:read', icon: <NotificationsIcon />, color: MODULE_COLORS.sys },
  { text: '工作量统计', desc: '分析检测 / 研发送样', path: '/workload', perm: 'ops_stats:read', icon: <BarChartIcon />, color: MODULE_COLORS.work },
];

const DashboardPage: React.FC = () => {
  const { hasPerm } = useAuth();
  const navigate = useNavigate();
  const visible = CARDS.filter((c) => !c.perm || hasPerm(c.perm));

  return (
    <Box>
      <Typography variant="h4" fontWeight={700} gutterBottom>仪表盘</Typography>
      <Typography color="text.secondary" sx={{ mb: 3 }}>欢迎使用本地化 LIMS，点击下方模块开始操作。</Typography>
      <Grid container spacing={2}>
        {visible.map((c) => (
          <Grid item xs={12} sm={6} md={4} key={c.path}>
            <Paper elevation={1} sx={{ overflow: 'hidden' }}>
              <CardActionArea onClick={() => navigate(c.path)} sx={{ p: 2.5 }}>
                <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
                  <Box sx={{ width: 48, height: 48, borderRadius: 2, display: 'flex', alignItems: 'center', justifyContent: 'center', color: '#fff', bgcolor: c.color }}>{c.icon}</Box>
                  <Box>
                    <Typography variant="h6" fontWeight={700}>{c.text}</Typography>
                    <Typography variant="body2" color="text.secondary">{c.desc}</Typography>
                  </Box>
                </Box>
              </CardActionArea>
            </Paper>
          </Grid>
        ))}
      </Grid>
    </Box>
  );
};

export default DashboardPage;
