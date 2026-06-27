import React, { useEffect, useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { Box, Typography, TextField, IconButton, Alert, CircularProgress, Paper, Snackbar, Select, MenuItem, FormControl, InputLabel, useMediaQuery, useTheme } from '@mui/material';
import ArrowBackIcon from '@mui/icons-material/ArrowBack';
import dayjs from 'dayjs';
import { getProjects, getGroups, createSample } from '../api/client';
import type { Project, ProjectGroup } from '../types';
import { useUser } from '../UserContext';

const R = '2px';
const SampleEntryPage: React.FC = () => {
  const { groupId } = useParams<{ groupId: string }>();
  const navigate = useNavigate();
  const theme = useTheme(); const isMobile = useMediaQuery(theme.breakpoints.down('sm'));
  const { userName, setUserName } = useUser();
  const [projects, setProjects] = useState<Project[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [groupName, setGroupName] = useState('');
  const [submittedAt, setSubmittedAt] = useState(() => dayjs().format('YYYY-MM-DDTHH:mm'));
  const [projectId, setProjectId] = useState<number | ''>('');
  const [sampleName, setSampleName] = useState('');
  const [sampleCount, setSampleCount] = useState('1');
  const [unit, setUnit] = useState('个');
  const [batchNo, setBatchNo] = useState('');
  const [notes, setNotes] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const [snackbar, setSnackbar] = useState<{ open: boolean; message: string; severity: 'success' | 'error' }>({ open: false, message: '', severity: 'success' });

  const load = async () => {
    setLoading(true); setError('');
    try {
      const [projRes, groupsRes] = await Promise.all([
        getProjects({ group_id: Number(groupId), active_only: true }),
        getGroups(),
      ]);
      if (projRes.code === 0) setProjects(projRes.data as Project[]);
      else setError(projRes.message);
      if (groupsRes.code === 0) {
        const gs = groupsRes.data as ProjectGroup[];
        const g = gs.find(g => g.id === Number(groupId));
        if (g) setGroupName(g.name);
      }
    } catch { setError('加载失败'); } finally { setLoading(false); }
  };
  useEffect(() => { load(); }, [groupId]);

  const handleSubmit = async () => {
    if (!userName.trim()) { setSnackbar({ open: true, message: '请输入送样人', severity: 'error' }); return; }
    if (!projectId) { setSnackbar({ open: true, message: '请选择项目方法', severity: 'error' }); return; }
    if (!sampleName.trim()) { setSnackbar({ open: true, message: '请输入样品名称', severity: 'error' }); return; }
    const count = parseInt(sampleCount, 10);
    if (isNaN(count) || count <= 0) { setSnackbar({ open: true, message: '数量必须大于0', severity: 'error' }); return; }
    setSubmitting(true);
    try {
      const res = await createSample({
        project_id: projectId as number,
        user_name: userName.trim(),
        sample_name: sampleName.trim(),
        sample_count: count,
        unit: unit || '个',
        batch_no: batchNo.trim() || undefined,
        notes: notes.trim() || undefined,
        submitted_at: dayjs(submittedAt).format('YYYY-MM-DDTHH:mm:ss'),
      });
      if (res.code === 0) {
        setSnackbar({ open: true, message: '送样记录已保存', severity: 'success' });
        setSampleName(''); setSampleCount('1'); setBatchNo(''); setNotes('');
      } else { setSnackbar({ open: true, message: res.message, severity: 'error' }); }
    } catch { setSnackbar({ open: true, message: '提交失败', severity: 'error' }); }
    finally { setSubmitting(false); }
  };

  if (loading) return <Box sx={{ display: 'flex', justifyContent: 'center', alignItems: 'center', minHeight: '60vh' }}><CircularProgress /></Box>;
  if (error) return <Box sx={{ p: 2 }}><Alert severity="error">{error}</Alert></Box>;

  return (
    <Box sx={{ maxWidth: 640, mx: 'auto' }}>
      <Box sx={{ display: 'flex', alignItems: 'center', mb: 3, gap: 1.5 }}>
        <IconButton onClick={() => navigate('/')} sx={{ bgcolor: 'rgba(30,136,229,0.08)', '&:hover': { bgcolor: 'rgba(30,136,229,0.15)' } }}>
          <ArrowBackIcon sx={{ color: '#1e88e5' }} />
        </IconButton>
        <Box>
          <Typography variant="h5" fontWeight={700}>送样录入</Typography>
          <Typography variant="body2" color="text.secondary">{groupName || `分组 ${groupId}`}</Typography>
        </Box>
      </Box>

      <Paper elevation={0} sx={{ mb: 3, borderRadius: R, background: 'linear-gradient(145deg,#ffffff,#f5f5f5)', border: '1px solid rgba(0,0,0,0.06)', boxShadow: '0 4px 20px rgba(0,0,0,0.06)', overflow: 'hidden' }}>
        <Box sx={{ p: 2, display: 'flex', flexDirection: 'column', gap: 2 }}>
          <Box sx={{ display: 'flex', flexDirection: isMobile ? 'column' : 'row', gap: 2 }}>
            <TextField size="small" label="送样人" value={userName} onChange={e => setUserName(e.target.value)} placeholder="请输入姓名" sx={{ flex: 1, '& .MuiOutlinedInput-root': { borderRadius: R } }} required />
            <TextField size="small" label="送样时间" type="datetime-local" value={submittedAt} onChange={e => setSubmittedAt(e.target.value)} InputLabelProps={{ shrink: true }} sx={{ width: isMobile ? '100%' : 220, '& .MuiOutlinedInput-root': { borderRadius: R } }} />
          </Box>
          <FormControl size="small" sx={{ '& .MuiOutlinedInput-root': { borderRadius: R } }}>
            <InputLabel>项目方法</InputLabel>
            <Select value={projectId} label="项目方法" onChange={e => setProjectId(Number(e.target.value))}>
              {projects.map(p => <MenuItem key={p.id} value={p.id}>{p.name}</MenuItem>)}
            </Select>
          </FormControl>
          <Box sx={{ display: 'flex', flexDirection: isMobile ? 'column' : 'row', gap: 2 }}>
            <TextField size="small" label="样品名称" value={sampleName} onChange={e => setSampleName(e.target.value)} placeholder="如: 反应液A" sx={{ flex: 2, '& .MuiOutlinedInput-root': { borderRadius: R } }} required />
            <TextField size="small" label="数量" type="number" value={sampleCount} onChange={e => setSampleCount(e.target.value)} InputProps={{ inputProps: { min: 1 } }} sx={{ width: isMobile ? '100%' : 100, '& .MuiOutlinedInput-root': { borderRadius: R } }} required />
            <TextField size="small" label="单位" value={unit} onChange={e => setUnit(e.target.value)} sx={{ width: isMobile ? '100%' : 80, '& .MuiOutlinedInput-root': { borderRadius: R } }} />
          </Box>
          <Box sx={{ display: 'flex', flexDirection: isMobile ? 'column' : 'row', gap: 2 }}>
            <TextField size="small" label="批次号(可选)" value={batchNo} onChange={e => setBatchNo(e.target.value)} sx={{ flex: 1, '& .MuiOutlinedInput-root': { borderRadius: R } }} />
            <TextField size="small" label="备注(可选)" value={notes} onChange={e => setNotes(e.target.value)} sx={{ flex: 2, '& .MuiOutlinedInput-root': { borderRadius: R } }} />
          </Box>
          <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
            <IconButton onClick={handleSubmit} disabled={submitting} sx={{ bgcolor: '#1976d2', color: '#fff', borderRadius: R, px: 3, py: 1, fontSize: 14, fontWeight: 600, '&:hover': { bgcolor: '#1565c0' } }}>
              {submitting ? '提交中...' : '保存送样记录'}
            </IconButton>
          </Box>
        </Box>
      </Paper>

      <Snackbar open={snackbar.open} autoHideDuration={3000} onClose={() => setSnackbar({ ...snackbar, open: false })} anchorOrigin={{ vertical: 'bottom', horizontal: 'center' }}>
        <Alert severity={snackbar.severity} onClose={() => setSnackbar({ ...snackbar, open: false })} variant="filled" sx={{ borderRadius: R }}>{snackbar.message}</Alert>
      </Snackbar>
    </Box>
  );
};
export default SampleEntryPage;
