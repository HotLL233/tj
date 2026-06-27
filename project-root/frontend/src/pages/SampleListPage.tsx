import React, { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Box, Typography, IconButton, Alert, CircularProgress, Table, TableBody, TableCell, TableContainer, TableHead, TableRow, Paper, TextField, FormControl, InputLabel, Select, MenuItem, Dialog, DialogTitle, DialogContent, DialogActions, Button, Snackbar, Pagination, useMediaQuery, useTheme } from '@mui/material';
import ArrowBackIcon from '@mui/icons-material/ArrowBack'; import EditIcon from '@mui/icons-material/Edit'; import DeleteIcon from '@mui/icons-material/Delete';
import dayjs from 'dayjs';
import { getSamples, getGroups, getProjects, updateSample, deleteSample } from '../api/client';
import type { SampleRecord, ProjectGroup, Project, PaginatedResponse } from '../types';

const R = '2px';
const SampleListPage: React.FC = () => {
  const navigate = useNavigate(); const theme = useTheme(); const isMobile = useMediaQuery(theme.breakpoints.down('sm'));
  const [loading, setLoading] = useState(true); const [error, setError] = useState('');
  const [page, setPage] = useState(1); const [total, setTotal] = useState(0);
  const [items, setItems] = useState<SampleRecord[]>([]);
  const [groups, setGroups] = useState<ProjectGroup[]>([]);
  const [projects, setProjects] = useState<Project[]>([]);
  const [filterGroup, setFilterGroup] = useState<number | ''>('');
  const [filterUser, setFilterUser] = useState('');
  const [editOpen, setEditOpen] = useState(false);
  const [editingItem, setEditingItem] = useState<SampleRecord | null>(null);
  const [editName, setEditName] = useState(''); const [editCount, setEditCount] = useState(''); const [editUnit, setEditUnit] = useState('');
  const [editBatch, setEditBatch] = useState(''); const [editNotes, setEditNotes] = useState('');
  const [snackbar, setSnackbar] = useState<{ open: boolean; message: string; severity: 'success' | 'error' }>({ open: false, message: '', severity: 'success' });

  const load = async (p = page) => {
    setLoading(true); setError('');
    try {
      const [res, gRes, pRes] = await Promise.all([
        getSamples({ group_id: filterGroup || undefined, user_name: filterUser || undefined, page: p, page_size: 20 }),
        getGroups(),
        getProjects({ active_only: true }),
      ]);
      if (res.code === 0) {
        const data = res.data as PaginatedResponse<SampleRecord>;
        setItems(data.items); setTotal(data.total); setPage(data.page);
      } else setError(res.message);
      if (gRes.code === 0) setGroups(gRes.data as ProjectGroup[]);
      if (pRes.code === 0) setProjects(pRes.data as Project[]);
    } catch { setError('加载失败'); } finally { setLoading(false); }
  };

  useEffect(() => { load(); }, []);
  const handleFilter = () => { setPage(1); load(1); };

  const openEdit = (r: SampleRecord) => {
    setEditingItem(r); setEditName(r.sample_name); setEditCount(String(r.sample_count));
    setEditUnit(r.unit); setEditBatch(r.batch_no); setEditNotes(r.notes); setEditOpen(true);
  };
  const handleUpdate = async () => {
    if (!editingItem) return;
    try {
      const res = await updateSample(editingItem.id, {
        sample_name: editName, sample_count: parseInt(editCount, 10) || 1,
        unit: editUnit, batch_no: editBatch, notes: editNotes,
      });
      if (res.code === 0) { setSnackbar({ open: true, message: '修改成功', severity: 'success' }); setEditOpen(false); load(); }
      else setSnackbar({ open: true, message: res.message, severity: 'error' });
    } catch { setSnackbar({ open: true, message: '修改失败', severity: 'error' }); }
  };
  const handleDelete = async (id: number) => {
    if (!window.confirm('确定删除该送样记录？')) return;
    try {
      const res = await deleteSample(id);
      if (res.code === 0) { setSnackbar({ open: true, message: '已删除', severity: 'success' }); load(); }
      else setSnackbar({ open: true, message: res.message, severity: 'error' });
    } catch { setSnackbar({ open: true, message: '删除失败', severity: 'error' }); }
  };

  const getProjectName = (pid: number) => projects.find(p => p.id === pid)?.name || '';
  const getGroupName = (gid: number) => groups.find(g => g.id === gid)?.name || '';

  return (
    <Box sx={{ maxWidth: 1000, mx: 'auto' }}>
      <Box sx={{ display: 'flex', alignItems: 'center', mb: 3, gap: 1.5 }}>
        <IconButton onClick={() => navigate('/')} sx={{ bgcolor: 'rgba(30,136,229,0.08)', '&:hover': { bgcolor: 'rgba(30,136,229,0.15)' } }}>
          <ArrowBackIcon sx={{ color: '#1e88e5' }} />
        </IconButton>
        <Typography variant="h5" fontWeight={700}>送样记录</Typography>
      </Box>

      <Paper elevation={0} sx={{ mb: 2, p: 2, borderRadius: R, border: '1px solid rgba(0,0,0,0.06)', display: 'flex', flexDirection: isMobile ? 'column' : 'row', gap: 2, alignItems: isMobile ? 'stretch' : 'center' }}>
        <FormControl size="small" sx={{ minWidth: 140 }}>
          <InputLabel>实验室</InputLabel>
          <Select value={filterGroup} label="实验室" onChange={e => setFilterGroup(Number(e.target.value) || '')}>
            <MenuItem value="">全部</MenuItem>
            {groups.map(g => <MenuItem key={g.id} value={g.id}>{g.name}</MenuItem>)}
          </Select>
        </FormControl>
        <TextField size="small" label="送样人" value={filterUser} onChange={e => setFilterUser(e.target.value)} sx={{ width: 140 }} />
        <Button variant="contained" onClick={handleFilter} sx={{ borderRadius: R, bgcolor: '#1976d2' }}>查询</Button>
      </Paper>

      {loading ? <Box sx={{ display: 'flex', justifyContent: 'center', py: 4 }}><CircularProgress /></Box> :
        error ? <Alert severity="error">{error}</Alert> :
          <>
            <TableContainer component={Paper} elevation={0} sx={{ borderRadius: R, border: '1px solid rgba(0,0,0,0.06)' }}>
              <Table size="small">
                <TableHead><TableRow>
                  <TableCell>送样时间</TableCell>
                  <TableCell>实验室</TableCell>
                  <TableCell>项目方法</TableCell>
                  <TableCell>送样人</TableCell>
                  <TableCell>样品名称</TableCell>
                  <TableCell>数量</TableCell>
                  <TableCell>操作</TableCell>
                </TableRow></TableHead>
                <TableBody>
                  {items.map(r => (
                    <TableRow key={r.id}>
                      <TableCell>{r.submitted_at?.slice(0, 16)}</TableCell>
                      <TableCell>{getGroupName(r.group_id)}</TableCell>
                      <TableCell>{getProjectName(r.project_id)}</TableCell>
                      <TableCell>{r.user_name}</TableCell>
                      <TableCell>{r.sample_name}</TableCell>
                      <TableCell>{r.sample_count}{r.unit}</TableCell>
                      <TableCell>
                        <IconButton size="small" onClick={() => openEdit(r)} title="编辑"><EditIcon fontSize="small" sx={{ color: '#1976d2' }} /></IconButton>
                        <IconButton size="small" onClick={() => handleDelete(r.id)} title="删除"><DeleteIcon fontSize="small" color="error" /></IconButton>
                      </TableCell>
                    </TableRow>
                  ))}
                  {items.length === 0 && <TableRow><TableCell colSpan={7} align="center" sx={{ py: 4, color: 'text.secondary' }}>暂无送样记录</TableCell></TableRow>}
                </TableBody>
              </Table>
            </TableContainer>
            {total > 20 && <Box sx={{ display: 'flex', justifyContent: 'center', mt: 2 }}><Pagination count={Math.ceil(total / 20)} page={page} onChange={(_, p) => { setPage(p); load(p); }} /></Box>}
          </>
      }

      <Dialog open={editOpen} onClose={() => setEditOpen(false)} fullWidth maxWidth="sm">
        <DialogTitle>编辑送样记录</DialogTitle>
        <DialogContent sx={{ display: 'flex', flexDirection: 'column', gap: 2, pt: 2 }}>
          <TextField label="样品名称" value={editName} onChange={e => setEditName(e.target.value)} size="small" />
          <TextField label="数量" type="number" value={editCount} onChange={e => setEditCount(e.target.value)} size="small" />
          <TextField label="单位" value={editUnit} onChange={e => setEditUnit(e.target.value)} size="small" />
          <TextField label="批次号" value={editBatch} onChange={e => setEditBatch(e.target.value)} size="small" />
          <TextField label="备注" value={editNotes} onChange={e => setEditNotes(e.target.value)} size="small" />
        </DialogContent>
        <DialogActions><Button onClick={() => setEditOpen(false)}>取消</Button><Button onClick={handleUpdate} variant="contained" sx={{ borderRadius: R }}>保存</Button></DialogActions>
      </Dialog>

      <Snackbar open={snackbar.open} autoHideDuration={3000} onClose={() => setSnackbar({ ...snackbar, open: false })} anchorOrigin={{ vertical: 'bottom', horizontal: 'center' }}>
        <Alert severity={snackbar.severity} onClose={() => setSnackbar({ ...snackbar, open: false })} variant="filled" sx={{ borderRadius: R }}>{snackbar.message}</Alert>
      </Snackbar>
    </Box>
  );
};
export default SampleListPage;
