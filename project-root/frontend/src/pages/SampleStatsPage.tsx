import React, { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Box, Typography, IconButton, CircularProgress, Alert, Paper, Table, TableBody, TableCell, TableContainer, TableHead, TableRow, Stack } from '@mui/material';
import ArrowBackIcon from '@mui/icons-material/ArrowBack';
import { getSampleStats } from '../api/client';
import type { SampleStats } from '../types';

const R = '2px';
const SampleStatsPage: React.FC = () => {
  const n = useNavigate();
  const [ld, setLd] = useState(true); const [er, setEr] = useState('');
  const [data, setData] = useState<SampleStats | null>(null);

  const load = async () => {
    setLd(true); setEr('');
    try {
      const r = await getSampleStats();
      if (r.code === 0) setData(r.data as SampleStats); else setEr(r.message);
    } catch { setEr('加载失败'); } finally { setLd(false); }
  };
  useEffect(() => { load(); }, []);

  const StatCard = ({ title, value, unit }: { title: string; value: number; unit: string }) => (
    <Paper elevation={0} sx={{ flex: 1, p: 2.5, textAlign: 'center', borderRadius: R, border: '1px solid rgba(0,0,0,0.06)' }}>
      <Typography variant="h4" fontWeight={700} color="#e65100">{value}</Typography>
      <Typography variant="body2" color="text.secondary">{unit}</Typography>
      <Typography variant="caption" color="text.secondary">{title}</Typography>
    </Paper>
  );

  if (ld) return <Box sx={{ display: 'flex', justifyContent: 'center', pt: 8 }}><CircularProgress /></Box>;
  if (er) return <Box sx={{ p: 2 }}><Alert severity="error">{er}</Alert></Box>;

  return (<Box>
    <Box sx={{ display: 'flex', alignItems: 'center', gap: 1.5, mb: 3 }}>
      <IconButton onClick={() => n('/sample')} sx={{ bgcolor: 'rgba(230,81,0,0.08)', '&:hover': { bgcolor: 'rgba(230,81,0,0.15)' } }}>
        <ArrowBackIcon sx={{ color: '#e65100' }} />
      </IconButton>
      <Typography variant="h5" fontWeight={700} color="#e65100">送样统计</Typography>
    </Box>

    {data && <>
      <Stack direction={{ xs: 'column', sm: 'row' }} spacing={2} sx={{ mb: 3 }}>
        <StatCard title="送样记录" value={data.total_count} unit="条" />
        <StatCard title="样品总数" value={data.total_samples} unit="个" />
      </Stack>

      <Paper elevation={0} sx={{ mb: 3, borderRadius: R, border: '1px solid rgba(0,0,0,0.06)' }}>
        <Typography variant="subtitle1" fontWeight={600} sx={{ p: 2, pb: 0 }}>按实验室</Typography>
        <TableContainer><Table size="small">
          <TableHead><TableRow><TableCell>实验室</TableCell><TableCell align="right">送样次数</TableCell><TableCell align="right">样品总数</TableCell></TableRow></TableHead>
          <TableBody>{data.by_group.map(g => <TableRow key={g.group_name}><TableCell>{g.group_name}</TableCell><TableCell align="right">{g.count}</TableCell><TableCell align="right">{g.total_samples}</TableCell></TableRow>)}</TableBody>
        </Table></TableContainer>
      </Paper>

      <Paper elevation={0} sx={{ mb: 3, borderRadius: R, border: '1px solid rgba(0,0,0,0.06)' }}>
        <Typography variant="subtitle1" fontWeight={600} sx={{ p: 2, pb: 0 }}>按项目方法 (Top 20)</Typography>
        <TableContainer><Table size="small">
          <TableHead><TableRow><TableCell>项目</TableCell><TableCell>实验室</TableCell><TableCell align="right">送样次数</TableCell><TableCell align="right">样品总数</TableCell></TableRow></TableHead>
          <TableBody>{data.by_project.map(p => <TableRow key={p.project_name}><TableCell>{p.project_name}</TableCell><TableCell>{p.group_name}</TableCell><TableCell align="right">{p.count}</TableCell><TableCell align="right">{p.total_samples}</TableCell></TableRow>)}</TableBody>
        </Table></TableContainer>
      </Paper>

      <Paper elevation={0} sx={{ borderRadius: R, border: '1px solid rgba(0,0,0,0.06)' }}>
        <Typography variant="subtitle1" fontWeight={600} sx={{ p: 2, pb: 0 }}>按送样人 (Top 20)</Typography>
        <TableContainer><Table size="small">
          <TableHead><TableRow><TableCell>送样人</TableCell><TableCell align="right">送样次数</TableCell><TableCell align="right">样品总数</TableCell></TableRow></TableHead>
          <TableBody>{data.by_user.map(u => <TableRow key={u.user_name}><TableCell>{u.user_name}</TableCell><TableCell align="right">{u.count}</TableCell><TableCell align="right">{u.total_samples}</TableCell></TableRow>)}</TableBody>
        </Table></TableContainer>
      </Paper>
    </>}
  </Box>);
};
export default SampleStatsPage;
