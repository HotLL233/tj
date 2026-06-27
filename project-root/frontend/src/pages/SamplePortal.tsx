import React, { useEffect, useState, useMemo } from 'react';
import { useNavigate } from 'react-router-dom';
import { Box, Typography, IconButton, Fab, CircularProgress, Alert, TextField, InputAdornment } from '@mui/material';
import ArrowBackIcon from '@mui/icons-material/ArrowBack'; import ScienceIcon from '@mui/icons-material/Science'; import SearchIcon from '@mui/icons-material/Search'; import AssessmentIcon from '@mui/icons-material/Assessment'; import ListAltIcon from '@mui/icons-material/ListAlt';
import GroupCard from '../components/GroupCard'; import { getGroups } from '../api/client'; import type { ProjectGroup } from '../types';

const R = '2px';
const SamplePortal: React.FC = () => {
  const n = useNavigate(); const [gs, setGs] = useState<ProjectGroup[]>([]); const [ld, setLd] = useState(true); const [er, setEr] = useState(''); const [sq, setSq] = useState('');
  const lg = async () => { setLd(true); setEr(''); try { const r = await getGroups(); if (r.code === 0) setGs(r.data as ProjectGroup[]); else setEr(r.message); } catch { setEr('加载失败'); } finally { setLd(false); } };
  useEffect(() => { lg(); }, []);
  const fg = useMemo(() => { if (!sq.trim()) return gs; const q = sq.trim().toLowerCase(); return gs.filter(g => g.name.toLowerCase().includes(q)); }, [gs, sq]);
  if (ld) return <Box sx={{ display: 'flex', justifyContent: 'center', pt: 8 }}><CircularProgress /></Box>;
  if (er) return <Box sx={{ p: 2 }}><Alert severity="error" action={<Typography component="button" onClick={lg} sx={{ cursor: 'pointer', border: 'none', bgcolor: 'transparent', color: 'inherit', textDecoration: 'underline' }}>重试</Typography>}>{er}</Alert></Box>;

  return (<Box>
    <Box sx={{ display: 'flex', alignItems: 'center', gap: 1.5, mb: 3 }}>
      <IconButton onClick={() => n('/')} sx={{ bgcolor: 'rgba(230,81,0,0.08)', '&:hover': { bgcolor: 'rgba(230,81,0,0.15)' } }}>
        <ArrowBackIcon sx={{ color: '#e65100' }} />
      </IconButton>
      <Box><Typography variant="h5" fontWeight={700} color="#e65100">实验室送样</Typography><Typography variant="body2" color="text.secondary">选择实验室，开始送样录入</Typography></Box>
    </Box>
    <TextField size="small" placeholder="搜索实验室..." value={sq} onChange={e => setSq(e.target.value)} InputProps={{ startAdornment: <InputAdornment position="start"><SearchIcon /></InputAdornment> }} sx={{ mb: 3, maxWidth: 400, '& .MuiOutlinedInput-root': { borderRadius: R } }} />
    {fg.length === 0 ? <Box sx={{ textAlign: 'center', py: 6 }}><Typography color="text.secondary">{sq ? '未找到' : '暂无分组'}</Typography></Box> : (
      <Box sx={{ display: 'grid', gridTemplateColumns: { xs: '1fr', sm: 'repeat(2,1fr)', md: 'repeat(3,1fr)' }, gap: 2.5 }}>
        {fg.map(g => <GroupCard key={g.id} group={g} onClick={() => n(`/sample/${g.id}`)} />)}
        <GroupCard group={{ id: -1, name: '送样记录', sort_order: 99, project_count: 0, created_at: '' } as ProjectGroup} onClick={() => n('/sample/list')} />
      </Box>
    )}
    <Box sx={{ display: { xs: 'none', md: 'flex' }, gap: 1, mt: 4, justifyContent: 'center' }}>
      <Fab variant="extended" size="small" onClick={() => n('/sample/stats')} sx={{ bgcolor: '#e65100', color: '#fff', '&:hover': { bgcolor: '#bf360c' } }}><AssessmentIcon sx={{ mr: 0.5 }} />送样统计</Fab>
      <Fab variant="extended" size="small" onClick={() => n('/sample/list')} sx={{ boxShadow: 1 }}><ListAltIcon sx={{ mr: 0.5 }} />查看记录</Fab>
    </Box>
    <Box sx={{ display: { xs: 'flex', md: 'none' }, position: 'fixed', bottom: 72, right: 16, zIndex: 100, flexDirection: 'column', gap: 1 }}>
      <Fab size="small" onClick={() => n('/sample/stats')} sx={{ bgcolor: '#e65100', color: '#fff' }}><AssessmentIcon /></Fab>
      <Fab size="small" onClick={() => n('/sample/list')}><ListAltIcon /></Fab>
    </Box>
  </Box>);
};
export default SamplePortal;
