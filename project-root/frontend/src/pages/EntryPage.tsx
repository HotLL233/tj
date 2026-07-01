import React, { useEffect, useState, useCallback } from 'react';
import {
  Box, Typography, TextField, IconButton, CircularProgress, Snackbar, Alert, Chip,
} from '@mui/material';
import ArrowBackIcon from '@mui/icons-material/ArrowBack';
import { useParams, useNavigate } from 'react-router-dom';
import type { Project, MethodType } from '../types';
import { getProjects, createRecord, getMethodTypes } from '../api/client';
import ProjectRow from '../components/ProjectRow';

const R = '2px';

const typeColorMap: Record<string, 'info'|'success'|'warning'|'primary'|'default'> = {
  '液相': 'info', '气相': 'success', '理化': 'warning', '检测类型': 'primary',
};

const EntryPage: React.FC = () => {
  const { groupId } = useParams<{ groupId: string }>();
  const gid = Number(groupId) || 0;
  const navigate = useNavigate();

  const [projects, setProjects] = useState<Project[]>([]);
  const [loading, setLoading] = useState(true);
  const [userName, setUserName] = useState('');
  const [dateTime, setDateTime] = useState(() => {
    const now = new Date();
    return now.toISOString().slice(0, 16);
  });
  const [snackMsg, setSnackMsg] = useState('');
  const [snackErr, setSnackErr] = useState(false);

  const [mts, setMts] = useState<MethodType[]>([]);
  const [typeFilter, setTypeFilter] = useState('全部');

  const loadProjects = useCallback(async () => {
    if (!gid) return;
    try {
      const r = await getProjects({ group_id: gid, active_only: true });
      if (r.code === 0 && r.data) setProjects(r.data);
    } catch {} finally { setLoading(false); }
  }, [gid]);

  const loadMethodTypes = useCallback(async () => {
    try { const r = await getMethodTypes(); if (r.code === 0 && r.data) setMts(r.data); } catch {}
  }, []);

  useEffect(() => { loadProjects(); loadMethodTypes(); }, [loadProjects, loadMethodTypes]);

  // 仅显示方法(非研发项目)
  const methods = projects.filter(p => p.method_type !== '研发项目');
  const filtered = typeFilter === '全部'
    ? methods
    : methods.filter(p => p.method_type === typeFilter);

  const handleSubmit = async (projectId: number, quantity: number) => {
    if (!userName.trim()) { setSnackMsg('请输入用户名'); setSnackErr(true); return false; }
    try {
      const r = await createRecord({ project_id: projectId, user_name: userName, quantity, recorded_at: dateTime });
      if (r.code === 0) { setSnackMsg(`录入成功: ${userName} ×${quantity}`); setSnackErr(false); return true; }
      setSnackMsg(r.message); setSnackErr(true); return false;
    } catch { setSnackMsg('录入失败'); setSnackErr(true); return false; }
  };

  if (loading) return <Box sx={{ display: 'flex', justifyContent: 'center', mt: 8 }}><CircularProgress /></Box>;

  return (<Box>
    <Box sx={{ display: 'flex', alignItems: 'center', gap: 1.5, mb: 2 }}>
      <IconButton onClick={() => navigate('/workload')} sx={{ bgcolor: 'rgba(30,136,229,0.08)', '&:hover': { bgcolor: 'rgba(30,136,229,0.15)' } }}>
        <ArrowBackIcon />
      </IconButton>
      <Box><Typography variant="h5" fontWeight={700}>工作量录入</Typography><Typography variant="caption" color="text.secondary">选择检测方法并录入数量</Typography></Box>
    </Box>

    {/* 用户 & 时间 */}
    <Box sx={{ display: 'flex', gap: 2, mb: 2, flexWrap: 'wrap', alignItems: 'center' }}>
      <TextField label="用户名" size="small" value={userName} onChange={e => setUserName(e.target.value)} sx={{ width: 140, '& .MuiOutlinedInput-root': { borderRadius: R } }} />
      <TextField label="日期时间" type="datetime-local" size="small" value={dateTime} onChange={e => setDateTime(e.target.value)} InputLabelProps={{ shrink: true }} sx={{ width: 200, '& .MuiOutlinedInput-root': { borderRadius: R } }} />
    </Box>

    {/* 类型筛选按钮栏 — 始终可见，随时切换 */}
    <Box sx={{ display: 'flex', gap: 1, mb: 2, flexWrap: 'wrap', alignItems: 'center' }}>
      <Chip
        label={`全部 (${methods.length})`} size="medium"
        color={typeFilter === '全部' ? 'primary' : 'default'}
        variant={typeFilter === '全部' ? 'filled' : 'outlined'}
        onClick={() => setTypeFilter('全部')}
        sx={{ borderRadius: R, cursor: 'pointer', fontWeight: typeFilter === '全部' ? 700 : 400 }}
      />
      {mts.filter(t => t.name !== '检测类型').map(t => {
        const cnt = methods.filter(p => p.method_type === t.name).length;
        return (
          <Chip key={t.id}
            label={`${t.name} (${cnt})`} size="medium"
            color={typeFilter === t.name ? (typeColorMap[t.name] || 'primary') : 'default'}
            variant={typeFilter === t.name ? 'filled' : 'outlined'}
            onClick={() => setTypeFilter(t.name)}
            sx={{ borderRadius: R, cursor: 'pointer', fontWeight: typeFilter === t.name ? 700 : 400 }}
          />
        );
      })}
    </Box>

    {/* 项目列表 */}
    {filtered.length === 0
      ? <Typography color="text.secondary" textAlign="center" sx={{ py: 6 }}>{typeFilter !== '全部' ? `无 "${typeFilter}" 类型的检测方法` : '该实验室暂无方法'}</Typography>
      : filtered.map(p => <ProjectRow key={p.id} project={p} onSubmit={handleSubmit} />)}

    <Snackbar open={!!snackMsg} autoHideDuration={3000} onClose={() => setSnackMsg('')} anchorOrigin={{ vertical: 'bottom', horizontal: 'center' }}>
      <Alert severity={snackErr ? 'error' : 'success'} sx={{ borderRadius: R }} onClose={() => setSnackMsg('')}>{snackMsg}</Alert>
    </Snackbar>
  </Box>);
};

export default EntryPage;
