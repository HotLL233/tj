import React, { useEffect, useState, useCallback } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { Box, Typography, TextField, IconButton, Alert, CircularProgress, Paper, Snackbar, useMediaQuery, useTheme } from '@mui/material';
import ArrowBackIcon from '@mui/icons-material/ArrowBack';
import dayjs from 'dayjs'; import ProjectRow from '../components/ProjectRow'; import { getProjects, createRecord, getGroups } from '../api/client';
import type { Project, ProjectGroup } from '../types';
import { useUser } from '../UserContext';

const R = '2px';
const EntryPage: React.FC = () => {
  const { groupId } = useParams<{ groupId: string }>(); const navigate = useNavigate(); const theme = useTheme(); const isMobile = useMediaQuery(theme.breakpoints.down('sm'));
  const [projects, setProjects] = useState<Project[]>([]); const [loading, setLoading] = useState(true); const [error, setError] = useState('');
  const [groupName, setGroupName] = useState(''); const { userName, setUserName } = useUser();
  const [recordedAt, setRecordedAt] = useState(() => dayjs().format('YYYY-MM-DDTHH:mm'));
  const [snackbar, setSnackbar] = useState<{ open: boolean; message: string; severity: 'success' | 'error' }>({ open: false, message: '', severity: 'success' });

  const loadProjects = async () => { setLoading(true); setError(''); try { const [projRes, groupsRes] = await Promise.all([getProjects({ group_id: Number(groupId), active_only: true }), getGroups()]); if (projRes.code === 0) setProjects(projRes.data as Project[]); else setError(projRes.message); if (groupsRes.code === 0) { const gs = groupsRes.data as ProjectGroup[]; const g = gs.find(g => g.id === Number(groupId)); if (g) setGroupName(g.name); } } catch { setError('加载项目失败'); } finally { setLoading(false); } };
  useEffect(() => { loadProjects(); }, [groupId]);

  const handleSubmit = useCallback(async (projectId: number, quantity: number): Promise<boolean> => { if (!userName.trim()) { setSnackbar({ open: true, message: '请先输入用户名', severity: 'error' }); return false; } try { const isoDate = dayjs(recordedAt).format('YYYY-MM-DDTHH:mm:ss'); const res = await createRecord({ project_id: projectId, user_name: userName.trim(), quantity, recorded_at: isoDate }); if (res.code === 0) { setSnackbar({ open: true, message: '录入成功', severity: 'success' }); return true; } else { setSnackbar({ open: true, message: res.message, severity: 'error' }); return false; } } catch { setSnackbar({ open: true, message: '提交失败', severity: 'error' }); return false; } }, [userName, recordedAt]);

  if (loading) return <Box sx={{ display: 'flex', justifyContent: 'center', alignItems: 'center', minHeight: '60vh' }}><CircularProgress /></Box>;
  if (error) return <Box sx={{ p: 2 }}><Alert severity="error">{error}</Alert></Box>;

  return (<Box sx={{ maxWidth: 640, mx: 'auto' }}><Box sx={{ display: 'flex', alignItems: 'center', mb: 3, gap: 1.5 }}><IconButton onClick={() => navigate('/')} sx={{ bgcolor: 'rgba(30,136,229,0.08)', '&:hover': { bgcolor: 'rgba(30,136,229,0.15)' } }}><ArrowBackIcon sx={{ color: '#1e88e5' }} /></IconButton><Box><Typography variant="h5" fontWeight={700}>录入工作量</Typography><Typography variant="body2" color="text.secondary">{groupName || `分组 ${groupId}`} · {dayjs(recordedAt).format('YYYY-MM-DD HH:mm')}</Typography></Box></Box>
    <Paper elevation={0} sx={{ mb: 3, borderRadius: R, background: 'linear-gradient(145deg,#ffffff,#f5f5f5)', border: '1px solid rgba(0,0,0,0.06)', boxShadow: '0 4px 20px rgba(0,0,0,0.06)', overflow: 'hidden' }}><Box sx={{ p: 2, display: 'flex', flexDirection: isMobile ? 'column' : 'row', gap: 2, alignItems: isMobile ? 'stretch' : 'center' }}><TextField size="small" label="用户名" value={userName} onChange={e => setUserName(e.target.value)} placeholder="请输入您的姓名" sx={{ flex: 1, '& .MuiOutlinedInput-root': { borderRadius: R } }} required /><TextField size="small" label="日期时间" type="datetime-local" value={recordedAt} onChange={e => setRecordedAt(e.target.value)} InputLabelProps={{ shrink: true }} sx={{ width: isMobile ? '100%' : 220, '& .MuiOutlinedInput-root': { borderRadius: R } }} /></Box></Paper>
    {projects.length === 0 ? <Box sx={{ textAlign: 'center', py: 6 }}><Typography color="text.secondary">该分组下暂无项目</Typography></Box> : <Box sx={{ display: 'flex', flexDirection: 'column' }}>{projects.map(project => <ProjectRow key={project.id} project={project} onSubmit={handleSubmit} />)}</Box>}
    <Snackbar open={snackbar.open} autoHideDuration={3000} onClose={() => setSnackbar({ ...snackbar, open: false })} anchorOrigin={{ vertical: 'bottom', horizontal: 'center' }}><Alert severity={snackbar.severity} onClose={() => setSnackbar({ ...snackbar, open: false })} variant="filled" sx={{ borderRadius: R }}>{snackbar.message}</Alert></Snackbar>
  </Box>);
};
export default EntryPage;
