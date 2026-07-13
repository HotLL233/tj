import React, { useEffect, useState, useCallback } from 'react';
import {
  Box, Typography, IconButton, TextField, CircularProgress, Snackbar, Alert, Chip,
  Table, TableBody, TableCell, TableContainer, TableHead, TableRow, Paper, TablePagination,
  Button, Collapse, Select, MenuItem, FormControl,
} from '@mui/material';
import ArrowBackIcon from '@mui/icons-material/ArrowBack';
import ExpandMoreIcon from '@mui/icons-material/ExpandMore';
import ExpandLessIcon from '@mui/icons-material/ExpandLess';
import SaveIcon from '@mui/icons-material/Save';
import { useNavigate } from 'react-router-dom';
import type { WorkRecord, RdRecordColumn, Project, Method } from '../types';
import { getRdRecords, sampleRdRecord, getGroups, getRdRecordColumns, getProjects, getMethods, updateRdRecord } from '../api/client';
import { useUser } from '../UserContext';


const R = '2px';

// 从方法名中提取仪器标签（@符号后的[...]内容）
const extractInstrumentFromMethodName = (methodName: string): string | null => {
  if (!methodName) return null;
  const atIndex = methodName.indexOf('@');
  if (atIndex === -1) return null;
  const afterAt = methodName.substring(atIndex + 1);
  const bracketStart = afterAt.indexOf('[');
  if (bracketStart === -1) return null;
  const bracketEnd = afterAt.indexOf(']', bracketStart);
  if (bracketEnd === -1) return null;
  return afterAt.substring(bracketStart + 1, bracketEnd);
};

// 字段名 → 显示值的映射
const getFieldValue = (rec: WorkRecord, fieldKey: string): string => {
  switch (fieldKey) {
    case 'seq_no': return '';
    case 'user_name': return rec.user_name || '-';
    case 'division_id': return rec.division_id?.toString() || '-';
    case 'lab_name': return rec.group_name || '-';
    case 'project_name': return rec.project_name || '-';
    case 'detection_type': return rec.method_type || '-';
    case 'method_name': return rec.method_name || '-';
    case 'sampling_person': return rec.sampler || '-';
    case 'sampling_time': return rec.sampled_at ? rec.sampled_at.replace('T', ' ').substring(0, 19) : '-';
    case 'status': return rec.status || '待取样';
    case 'notes': return rec.notes || '-';
    default: return '-';
  }
};

const RdRecordsPage: React.FC = () => {
  const navigate = useNavigate();
  const { hasPermission } = useUser();
  const [records, setRecords] = useState<WorkRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(0);
  const [groups, setGroups] = useState<{ id: number; name: string }[]>([]);
  const [snackMsg, setSnackMsg] = useState('');
  const [snackErr, setSnackErr] = useState(false);
  const [columns, setColumns] = useState<RdRecordColumn[]>([]);
  const pageSize = 20;

  // v0.4.34: 行内编辑状态
  const [expandedId, setExpandedId] = useState<number | null>(null);
  const [editForm, setEditForm] = useState<Record<string, any>>({});
  const [saving, setSaving] = useState(false);

  // v0.4.34: 级联数据（用于行内编辑的下拉选择）
  const [projects, setProjects] = useState<Project[]>([]);
  const [methods, setMethods] = useState<Method[]>([]);

  const loadColumns = useCallback(async () => {
    try {
      const r = await getRdRecordColumns();
      if (r.code === 0 && r.data) setColumns(r.data.filter(c => c.show_in_list));
    } catch {}
  }, []);

  const loadRecords = useCallback(async (p: number) => {
    setLoading(true);
    try {
      const r = await getRdRecords({ page: p + 1, page_size: pageSize });
      if (r.code === 0 && r.data) {
        setRecords(r.data.items);
        setTotal(r.data.total);
      }
    } catch {} finally { setLoading(false); }
  }, []);

  const loadGroups = useCallback(async () => {
    try { const r = await getGroups(); if (r.code === 0 && r.data) setGroups(r.data); } catch {}
  }, []);

  // v0.4.34: 加载级联数据
  const loadCascadeData = useCallback(async () => {
    try {
      const [pr, mr] = await Promise.all([getProjects(), getMethods()]);
      if (pr.code === 0 && pr.data) setProjects(pr.data);
      if (mr.code === 0 && mr.data) setMethods(mr.data);
    } catch {}
  }, []);

  useEffect(() => { loadColumns(); loadRecords(0); loadGroups(); loadCascadeData(); }, []);

  const handlePageChange = (_e: unknown, newPage: number) => {
    setPage(newPage);
    setExpandedId(null);
    setLoading(true);
    getRdRecords({ page: newPage + 1, page_size: pageSize })
      .then(r => { if (r.code === 0 && r.data) { setRecords(r.data.items); setTotal(r.data.total); } })
      .catch(() => {})
      .finally(() => setLoading(false));
  };

  const getGroupName = (rec: WorkRecord) => rec.group_name || groups.find(g => g.name === rec.group_name)?.name || '-';

  const isStatusCol = (name: string) => name === 'status';
  const isSeqNoCol = (name: string) => name === 'seq_no';

  // v0.4.34: 取样
  const handleSample = async (id: number) => {
    try {
      await sampleRdRecord(id);
      setSnackMsg('取样成功'); setSnackErr(false);
      loadRecords(page);
    } catch { setSnackMsg('取样失败'); setSnackErr(true); }
  };

  // v0.4.34: 展开/折叠行内编辑
  const toggleExpand = (rec: WorkRecord) => {
    if (expandedId === rec.id) {
      setExpandedId(null);
      setEditForm({});
      return;
    }
    // 已取样不可展开编辑
    if (rec.status === '已取样') return;
    setExpandedId(rec.id);
    setEditForm({
      user_name: rec.user_name || '',
      project_id: rec.project_id,
      method_id: rec.method_id ?? '',
      quantity: rec.quantity,
      batch_no: rec.batch_no || '',
      notes: rec.notes || '',
    });
  };

  // v0.4.34: 保存行内编辑
  const handleSave = async (rec: WorkRecord) => {
    setSaving(true);
    try {
      const data: any = {};
      if (editForm.user_name !== rec.user_name) data.user_name = editForm.user_name;
      if (Number(editForm.quantity) !== rec.quantity) data.quantity = Number(editForm.quantity);
      if (editForm.batch_no !== (rec.batch_no || '')) data.batch_no = editForm.batch_no;
      if (editForm.notes !== (rec.notes || '')) data.notes = editForm.notes;
      if (editForm.project_id !== rec.project_id) data.project_id = Number(editForm.project_id);
      const newMid = editForm.method_id === '' ? null : Number(editForm.method_id);
      if (newMid !== rec.method_id) data.method_id = newMid;
      if (Object.keys(data).length === 0) {
        setSnackMsg('没有需要修改的字段'); setSnackErr(true);
        setSaving(false);
        return;
      }
      await updateRdRecord(rec.id, data);
      setSnackMsg('保存成功'); setSnackErr(false);
      setExpandedId(null);
      setEditForm({});
      loadRecords(page);
    } catch (e: any) {
      setSnackMsg(e.message || '保存失败'); setSnackErr(true);
    } finally { setSaving(false); }
  };

  // v0.4.34: 根据当前 project_id 获取可用的方法列表
  const getAvailableMethods = (projectId: number | null) => {
    if (!projectId) return [];
    const proj = projects.find(p => p.id === projectId);
    if (!proj) return [];
    return methods.filter(m => (proj.method_ids || []).includes(m.id));
  };

  return (
    <Box>
      <Box sx={{ display: 'flex', alignItems: 'center', gap: 1.5, mb: 2 }}>
        
        <IconButton onClick={() => navigate(-1)} sx={{ bgcolor: 'rgba(230,81,0,0.08)', '&:hover': { bgcolor: 'rgba(230,81,0,0.15)' } }}>
          <ArrowBackIcon />
        </IconButton>
        <Typography variant="h5" fontWeight={700}>研发送样记录</Typography>
        
      </Box>

    {loading && records.length === 0 ? (
      
      <Box sx={{ display: 'flex', justifyContent: 'center', py: 8 }}><CircularProgress /></Box>
      
    ) : records.length === 0 ? (
      
      <Typography color="text.secondary" textAlign="center" sx={{ py: 6, fontSize: '0.875rem' }}>暂无记录</Typography>
      
    ) : (
      
      <TableContainer component={Paper} variant="outlined" sx={{ borderRadius: R, boxShadow: 'none', '& .MuiPaper-root': { borderRadius: R }, overflowX: 'auto' }}>
        <Table size="small" sx={{ minWidth: columns.length * 100 + 60 }}>
          <TableHead>
            <TableRow sx={{ bgcolor: 'rgba(230,81,0,0.06)' }}>
              {columns.map(col => (
                <TableCell key={col.name} sx={{
                  fontWeight: 700, fontSize: '0.8rem', whiteSpace: 'nowrap',
                  width: isSeqNoCol(col.name) ? 40 : undefined,
                  minWidth: col.width || 80,
                  textAlign: isSeqNoCol(col.name) ? 'center' : 'left',
                }}>
                  {col.label}
                </TableCell>
              ))}
              <TableCell sx={{ fontWeight: 700, fontSize: '0.8rem', whiteSpace: 'nowrap', width: 50 }}>操作</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {records.map((rec, idx) => {
              const status = rec.status || '待取样';
              const isSampled = status === '已取样';
              return (
              <React.Fragment key={rec.id}>
              <TableRow hover sx={{ '&:last-child td': { borderBottom: 0 } }}>
                {columns.map(col => {
                  if (isSeqNoCol(col.name)) {
                    return (
                      <TableCell key={col.name} sx={{ fontSize: '0.8rem', textAlign: 'center' }}>
                        {page * pageSize + idx + 1}
                      </TableCell>
                    );
                  }
                  if (isStatusCol(col.name)) {
                    return (
                      <TableCell key={col.name} sx={{ fontSize: '0.8rem', whiteSpace: 'nowrap' }}>
                        <Typography variant="body2" sx={{
                          display: 'inline-block', px: 1, py: 0.3, borderRadius: R, fontSize: '0.75rem', fontWeight: 600,
                          bgcolor: isSampled ? '#c8e6c9' : '#fff9c4',
                          color: isSampled ? '#2e7d32' : '#f57f17',
                        }}>{status}</Typography>
                      </TableCell>
                    );
                  }
                  if (col.name === 'lab_name') {
                    return (
                      <TableCell key={col.name} sx={{ fontSize: '0.8rem', maxWidth: col.width || 100, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                        {getGroupName(rec)}
                      </TableCell>
                    );
                  }
                  if (col.name === 'method_name') {
                    return (
                      <TableCell key={col.name} sx={{ fontSize: '0.8rem', maxWidth: col.width || 140, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                        {rec.method_name || '-'}
                      </TableCell>
                    );
                  }
                  if (col.name === 'sampling_person') {
                    return (
                      <TableCell key={col.name} sx={{ fontSize: '0.8rem', whiteSpace: 'nowrap' }}>
                        {isSampled ? (
                          <Typography variant="body2" sx={{ color: '#2e7d32', fontWeight: 600 }}>{rec.sampler || '-'}</Typography>
                        ) : hasPermission('sample:collect') ? (
                          <Button variant="contained" size="small" sx={{ borderRadius: R, bgcolor: '#2e7d32', '&:hover': { bgcolor: '#1b5e20' }, fontSize: '0.7rem', minWidth: 0, px: 1.5, py: 0 }}
                            onClick={() => handleSample(rec.id)}>
                            取样
                          </Button>
                        ) : (
                          <Typography variant="body2" sx={{ color: '#999' }}>待取样</Typography>
                        )}
                      </TableCell>
                    );
                  }
                  return (
                    <TableCell key={col.name} sx={{ fontSize: '0.8rem', whiteSpace: 'nowrap', maxWidth: col.width || 100, overflow: 'hidden', textOverflow: 'ellipsis' }}>
                      {getFieldValue(rec, col.name)}
                    </TableCell>
                  );
                })}
                <TableCell sx={{ fontSize: '0.8rem', whiteSpace: 'nowrap' }}>
                  {isSampled ? (
                    <Typography variant="body2" sx={{ color: '#ccc', fontSize: '0.75rem' }}>-</Typography>
                  ) : (
                    <IconButton size="small" onClick={() => toggleExpand(rec)} sx={{ color: '#2e7d32' }}>
                      {expandedId === rec.id ? <ExpandLessIcon fontSize="small" /> : <ExpandMoreIcon fontSize="small" />}
                    </IconButton>
                  )}
                </TableCell>
              </TableRow>
              {/* 行内编辑展开行 */}
              {expandedId === rec.id && (
                <TableRow>
                  <TableCell colSpan={columns.length + 1} sx={{ p: 0, borderBottom: '1px solid #e0e0e0' }}>
                    <Collapse in={expandedId === rec.id}>
                      <Box sx={{ p: 2, bgcolor: '#fafff5', borderTop: '2px solid #2e7d32' }}>
                        <Typography variant="subtitle2" fontWeight={600} color="#2e7d32" sx={{ mb: 1.5 }}>
                          编辑记录 #{page * pageSize + idx + 1}
                        </Typography>
                        <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 2, mb: 2 }}>
                          <TextField label="送样人" size="small" value={editForm.user_name || ''}
                            onChange={e => setEditForm(p => ({ ...p, user_name: e.target.value }))}
                            sx={{ minWidth: 140, '& .MuiOutlinedInput-root': { borderRadius: R, fontSize: '0.8rem' } }} />
                          <FormControl size="small" sx={{ minWidth: 160 }}>
                            <Select value={editForm.project_id ?? ''} displayEmpty
                              onChange={e => {
                                const pid = e.target.value === '' ? null : Number(e.target.value);
                                setEditForm(p => ({ ...p, project_id: pid, method_id: '' }));
                              }}
                              sx={{ fontSize: '0.8rem', borderRadius: R }}>
                              <MenuItem value=""><em>选择项目</em></MenuItem>
                              {projects.map(p => <MenuItem key={p.id} value={p.id}>{p.name}</MenuItem>)}
                            </Select>
                          </FormControl>
                          <FormControl size="small" sx={{ minWidth: 180 }}>
                            <Select value={editForm.method_id} displayEmpty
                              onChange={e => setEditForm(p => ({ ...p, method_id: e.target.value }))}
                              sx={{ fontSize: '0.8rem', borderRadius: R }}>
                              <MenuItem value=""><em>选择方法</em></MenuItem>
                              {getAvailableMethods(editForm.project_id || null).map(m => (
                                <MenuItem key={m.id} value={m.id}>{m.name}</MenuItem>
                              ))}
                            </Select>
                          </FormControl>
                          <TextField label="数量" type="number" size="small" value={editForm.quantity ?? 1}
                            onChange={e => setEditForm(p => ({ ...p, quantity: Math.max(1, Number(e.target.value) || 1) }))}
                            inputProps={{ min: 1, style: { textAlign: 'center' } }}
                            sx={{ width: 100, '& .MuiOutlinedInput-root': { borderRadius: R, fontSize: '0.8rem' } }} />
                          <TextField label="批号" size="small" value={editForm.batch_no || ''}
                            onChange={e => setEditForm(p => ({ ...p, batch_no: e.target.value }))}
                            sx={{ minWidth: 140, '& .MuiOutlinedInput-root': { borderRadius: R, fontSize: '0.8rem' } }} />
                          <TextField label="注意事项" size="small" value={editForm.notes || ''}
                            onChange={e => setEditForm(p => ({ ...p, notes: e.target.value }))}
                            sx={{ minWidth: 200, '& .MuiOutlinedInput-root': { borderRadius: R, fontSize: '0.8rem' } }} />
                        </Box>
                        <Box sx={{ display: 'flex', gap: 1, justifyContent: 'flex-end' }}>
                          <Button variant="outlined" size="small" onClick={() => { setExpandedId(null); setEditForm({}); }}
                            sx={{ borderRadius: R, fontSize: '0.75rem' }}>取消</Button>
                          <Button variant="contained" size="small" startIcon={<SaveIcon />} disabled={saving}
                            onClick={() => handleSave(rec)}
                            sx={{ borderRadius: R, bgcolor: '#2e7d32', '&:hover': { bgcolor: '#1b5e20' }, fontSize: '0.75rem' }}>
                            {saving ? '保存中...' : '保存'}
                          </Button>
                        </Box>
                      </Box>
                    </Collapse>
                  </TableCell>
                </TableRow>
              )}
              </React.Fragment>
              );
            })}
          </TableBody>
        </Table>
        {total > pageSize && (
          <TablePagination
            component="div"
            count={total}
            page={page}
            onPageChange={handlePageChange}
            rowsPerPage={pageSize}
            rowsPerPageOptions={[pageSize]}
            labelDisplayedRows={({ from, to, count }) => `${from}-${to} / ${count}`}
            sx={{ '& .MuiTablePagination-toolbar': { minHeight: 40 }, '& .MuiTablePagination-selectLabel': { fontSize: '0.75rem' }, '& .MuiTablePagination-displayedRows': { fontSize: '0.75rem' } }}
          />
        )}
      </TableContainer>
      
    )}

    <Snackbar open={!!snackMsg} autoHideDuration={3000} onClose={() => setSnackMsg('')} anchorOrigin={{ vertical: 'bottom', horizontal: 'center' }}>
      <Alert severity={snackErr ? 'error' : 'success'} sx={{ borderRadius: R }} onClose={() => setSnackMsg('')}>{snackMsg}</Alert>
    </Snackbar>
  </Box>
  );
};
export default RdRecordsPage;
