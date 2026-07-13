import React, { useEffect, useState, useMemo } from 'react';
import { useNavigate } from 'react-router-dom';
import { Box, Typography, IconButton, Fab, Button, CircularProgress, Alert, TextField, InputAdornment } from '@mui/material';
import ArrowBackIcon from '@mui/icons-material/ArrowBack'; import ScienceIcon from '@mui/icons-material/Science'; import SearchIcon from '@mui/icons-material/Search'; import BarChartIcon from '@mui/icons-material/BarChart'; import SettingsIcon from '@mui/icons-material/Settings';
import GroupCard from '../components/GroupCard'; import RecordsCard from '../components/RecordsCard'; import { getGroups } from '../api/client'; import type { ProjectGroup } from '../types';

const R = '2px';
const SamplePortal: React.FC = () => {
  const n = useNavigate(); const [gs, setGs] = useState<ProjectGroup[]>([]); const [ld, setLd] = useState(true); const [er, setEr] = useState(''); const [sq, setSq] = useState('');
  const lg = async () => { setLd(true); setEr(''); try { const r = await getGroups(); if (r.code === 0) setGs(r.data as ProjectGroup[]); else setEr(r.message); } catch { setEr('加载失败'); } finally { setLd(false); } };
  useEffect(() => { lg(); }, []);
  const fg = useMemo(() => {
    let filtered = gs.filter(g => g.show_in_rd !== false && !g.name.includes('方法') && g.name !== '研发项目');
    if (sq.trim()) { const q = sq.trim().toLowerCase(); filtered = filtered.filter(g => g.name.toLowerCase().includes(q)); }
    return filtered;
  }, [gs, sq]);
  const totalPending = gs.reduce((sum, g) => sum + (g.rd_record_count || 0), 0);
  if (ld) return <Box sx={{ display: 'flex', justifyContent: 'center', pt: 8 }}><CircularProgress /></Box>;
  if (er) return <Box sx={{ p: 2 }}><Alert severity="error" action={<Typography component="button" onClick={lg} sx={{ cursor: 'pointer', border: 'none', bgcolor: 'transparent', color: 'inherit', textDecoration: 'underline' }}>重试</Typography>}>{er}</Alert></Box>;

  return (<Box>
    <Box sx={{ display: 'flex', alignItems: 'center', gap: 1.5, mb: 3 }}>
      <IconButton onClick={() => n('/')} sx={{ bgcolor: 'rgba(230,81,0,0.08)', '&:hover': { bgcolor: 'rgba(230,81,0,0.15)' } }}>
        <ArrowBackIcon sx={{ color: '#e65100' }} />
      </IconButton>
      <Box sx={{ flex: 1 }}><Typography variant="h5" fontWeight={700} color="#e65100">研发送样</Typography><Typography variant="body2" color="text.secondary">选择实验室，开始研发送样录入</Typography></Box>
      <Button variant="outlined" startIcon={<BarChartIcon />} onClick={() => n('/sample/stats')}
        sx={{ borderRadius: R, borderColor: '#e65100', color: '#e65100', '&:hover': { borderColor: '#bf360c', bgcolor: 'rgba(230,81,0,0.04)' } }}>
        查看统计
      </Button>
    </Box>
    <TextField size="small" placeholder="搜索实验室..." value={sq} onChange={e => setSq(e.target.value)} InputProps={{ startAdornment: <InputAdornment position="start"><SearchIcon /></InputAdornment> }} sx={{ mb: 3, maxWidth: 400, '& .MuiOutlinedInput-root': { borderRadius: R } }} />
    {fg.length === 0 ? <Box sx={{ textAlign: 'center', py: 6 }}><Typography color="text.secondary">{sq ? '未找到' : '暂无分组'}</Typography></Box> : (
      <Box sx={{ display: 'grid', gridTemplateColumns: { xs: 'repeat(3,1fr)', sm: 'repeat(2,1fr)', md: 'repeat(3,1fr)' }, gap: 2.5 }}>
        <RecordsCard pendingCount={totalPending} onClick={() => n('/sample-records')} themeColor="#e65100" />
        {fg.map(g => <GroupCard key={g.id} group={g} onClick={() => n(`/sample/${g.id}`)} themeColor="#e65100" />)}
      </Box>
    )}
    <Box sx={{ display: { xs: 'none', md: 'flex' }, gap: 1, mt: 4, justifyContent: 'center' }}>
      <Fab variant="extended" size="small" onClick={() => n('/manage')} sx={{ boxShadow: 1 }}><SettingsIcon sx={{ mr: 0.5 }} />管理</Fab>
    </Box>
    <Box sx={{ display: { xs: 'flex', md: 'none' }, position: 'fixed', bottom: 72, right: 16, zIndex: 100, flexDirection: 'column', gap: 1 }}>
      <Fab size="small" onClick={() => n('/manage')}><SettingsIcon /></Fab>
    </Box>
  </Box>);
};
export default SamplePortal;
