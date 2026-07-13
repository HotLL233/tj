import React, { useState, useEffect, useCallback } from 'react';
import { useSearchParams, useNavigate } from 'react-router-dom';
import {
  Box, Typography, Paper, TextField, Button, Grid, IconButton,
  Chip, Snackbar, Alert, Select, MenuItem, FormControl, InputLabel,
  TablePagination, Collapse, CircularProgress,
} from '@mui/material';
import ArrowBackIcon from '@mui/icons-material/ArrowBack';
import ExpandMoreIcon from '@mui/icons-material/ExpandMore';
import ExpandLessIcon from '@mui/icons-material/ExpandLess';
import {
  getSampleInfoRecords, createSampleInfo, updateSampleInfo, updateSampleInfoStatus, getSampleInfoTypes,
} from '../api/client';
import type { SampleInfoRecord, SampleInfoType } from '../types';

const R = '2px';
const PAGE_SIZE = 20;

const STATUS_OPTIONS = ['全部', '待检测', '待取样', '已取样', '检测完成'] as const;
const STATUS_COLORS: Record<string, 'warning' | 'info' | 'success' | 'default'> = {
  '待检测': 'warning', '待取样': 'info', '已取样': 'success', '检测完成': 'default',
};
const NEXT_STATUS: Record<string, string | null> = {
  '待检测': '待取样', '待取样': '已取样', '已取样': '检测完成', '检测完成': null,
};
const NEXT_STATUS_LABEL: Record<string, string | null> = {
  '待检测': '取样', '待取样': '已取样', '已取样': '检测完成', '检测完成': null,
};

const DEFAULT_FORM = { batch_no: '', user_name: '', lab_name: '', project_name: '', main_components: '', notes: '' };

const SampleInfoEntry: React.FC = () => {
  const [sp] = useSearchParams();
  const n = useNavigate();
  const dt = sp.get('type') || '';

  // 表单
  const [form, setForm] = useState(DEFAULT_FORM);
  const [submittedAt, setSubmittedAt] = useState(new Date().toISOString().slice(0, 16));
  const [snack, setSnack] = useState<{ open: boolean; msg: string; sev: 'success' | 'error' }>({ open: false, msg: '', sev: 'success' });

  // 列表
  const [records, setRecords] = useState<SampleInfoRecord[]>([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(0);
  const [statusFilter, setStatusFilter] = useState('全部');
  const [ld, setLd] = useState(false);
  const [expandedId, setExpandedId] = useState<number | null>(null);

  // 编辑
  const [editingId, setEditingId] = useState<number | null>(null);
  const [editForm, setEditForm] = useState<Record<string, string>>({});

  // 检测类型（用于提交时附带 label）
  const [types, setTypes] = useState<SampleInfoType[]>([]);
  useEffect(() => {
    getSampleInfoTypes().then(r => { if (r.code === 0 && r.data) setTypes(r.data); }).catch(() => {});
  }, []);

  const load = useCallback(async () => {
    setLd(true);
    try {
      const r = await getSampleInfoRecords({
        type_key: dt || undefined,
        status: statusFilter === '全部' ? undefined : statusFilter,
        page: page + 1,
        page_size: PAGE_SIZE,
      });
      if (r.data) {
        setRecords(r.data.items);
        setTotal(r.data.total);
      }
    } catch (e: any) { setSnack({ open: true, msg: e.message || '加载失败', sev: 'error' }); }
    setLd(false);
  }, [dt, statusFilter, page]);

  useEffect(() => { load(); }, [load]);

  const setF = (k: string, v: string) => setForm(p => ({ ...p, [k]: v }));

  const doSubmit = async () => {
    if (!form.batch_no || !form.user_name || !form.lab_name || !form.project_name || !form.main_components) {
      setSnack({ open: true, msg: '请填写所有必填字段', sev: 'error' }); return;
    }
    // 检测时间由后端在「检测完成」时自动生成，前端不传
    const typeLabel = types.find(t => t.type_key === dt)?.label || dt;
    try {
      await createSampleInfo({
        batch_no: form.batch_no, user_name: form.user_name, lab_name: form.lab_name,
        project_name: form.project_name, submitted_at: submittedAt,
        main_components: form.main_components, detection_type: typeLabel,
        type_key: dt, notes: form.notes || undefined,
      });
      setSnack({ open: true, msg: '登记成功', sev: 'success' });
      setForm(DEFAULT_FORM);
      setSubmittedAt(new Date().toISOString().slice(0, 16));
      setPage(0); load();
    } catch (e: any) { setSnack({ open: true, msg: e.message || '提交失败', sev: 'error' }); }
  };

  const doReset = () => { setForm(DEFAULT_FORM); setSubmittedAt(new Date().toISOString().slice(0, 16)); };

  const doExpand = (id: number) => setExpandedId(p => p === id ? null : id);

  const doEdit = (rec: SampleInfoRecord) => {
    setEditingId(rec.id);
    setEditForm({ batch_no: rec.batch_no, user_name: rec.user_name, lab_name: rec.lab_name, project_name: rec.project_name, submitted_at: rec.submitted_at, main_components: rec.main_components, notes: rec.notes });
  };

  const doCancelEdit = () => { setEditingId(null); setEditForm({}); };

  const doSaveEdit = async (id: number) => {
    try {
      await updateSampleInfo(id, {
        batch_no: editForm.batch_no, user_name: editForm.user_name, lab_name: editForm.lab_name,
        project_name: editForm.project_name, submitted_at: editForm.submitted_at,
        main_components: editForm.main_components, notes: editForm.notes,
      });
      setSnack({ open: true, msg: '保存成功', sev: 'success' });
      setEditingId(null); setEditForm({}); load();
    } catch (e: any) { setSnack({ open: true, msg: e.message || '保存失败', sev: 'error' }); }
  };

  const doStatusFlow = async (id: number, curStatus: string) => {
    const nxt = NEXT_STATUS[curStatus];
    if (!nxt) return;
    try {
      await updateSampleInfoStatus(id, nxt);
      setSnack({ open: true, msg: `状态已流转: ${curStatus} → ${nxt}`, sev: 'success' });
      if (expandedId === id) setExpandedId(null);
      load();
    } catch (e: any) { setSnack({ open: true, msg: e.message || '状态流转失败', sev: 'error' }); }
  };

  const fmtDate = (s: string) => s ? s.slice(0, 16).replace('T', ' ') : '';

  return (
    <Box sx={{ maxWidth: 1100, mx: 'auto', mt: { xs: 1, md: 3 }, px: { xs: 1, md: 2 } }}>
      {/* 顶部 */}
      <Box sx={{ display: 'flex', alignItems: 'center', mb: 2, gap: 1 }}>
        <IconButton onClick={() => n('/sample-info')} size="small"><ArrowBackIcon /></IconButton>
        <Typography variant="h5" fontWeight={700} color="#2e7d32">样品信息登记</Typography>
      </Box>

      {/* === 部分 A：登记表单 === */}
      <Paper elevation={0} sx={{ p: 3, mb: 3, borderRadius: R, border: '2px solid #2e7d32', background: 'linear-gradient(145deg,#ffffff,#f1f8e9)' }}>
        <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2, flexWrap: 'wrap', gap: 1 }}>
          <Box>
            <Typography variant="h6" fontWeight={700} color="#2e7d32">样品信息登记</Typography>
            <Typography variant="body2" color="text.secondary">
              检测类型: {dt || '全部'} · 序号: <Box component="span" sx={{ color: '#999' }}>自动</Box>
            </Typography>
          </Box>
          <Chip label="待检测" color="warning" size="small" />
        </Box>

        <Grid container spacing={2}>
          <Grid item xs={12} sm={6}>
            <TextField label="样品批号" required fullWidth size="small" value={form.batch_no} onChange={e => setF('batch_no', e.target.value)} />
          </Grid>
          <Grid item xs={12} sm={6}>
            <TextField label="实验室/车间" required fullWidth size="small" value={form.lab_name} onChange={e => setF('lab_name', e.target.value)} />
          </Grid>
          <Grid item xs={12} sm={6}>
            <TextField label="送样人" required fullWidth size="small" value={form.user_name} onChange={e => setF('user_name', e.target.value)} />
          </Grid>
          <Grid item xs={12} sm={6}>
            <TextField label="所属项目" required fullWidth size="small" value={form.project_name} onChange={e => setF('project_name', e.target.value)} />
          </Grid>
          <Grid item xs={12} sm={6}>
            <TextField label="送样时间" type="datetime-local" required fullWidth size="small" value={submittedAt} onChange={e => setSubmittedAt(e.target.value)} InputLabelProps={{ shrink: true }} />
          </Grid>
          <Grid item xs={12} sm={6}>
            <TextField label="序号" fullWidth size="small" value="自动生成" disabled InputProps={{ sx: { color: '#999' } }} />
          </Grid>
          <Grid item xs={12} sm={6}>
            <TextField label="检测类型" fullWidth size="small" value={dt || '全部'} disabled InputProps={{ sx: { color: '#999' } }} />
          </Grid>
          <Grid item xs={12}>
            <TextField label="样品主要成分" required fullWidth size="small" value={form.main_components} onChange={e => setF('main_components', e.target.value)} />
          </Grid>
          <Grid item xs={12}>
            <TextField label="注意事项" fullWidth size="small" multiline rows={2} value={form.notes} onChange={e => setF('notes', e.target.value)} />
          </Grid>
          <Grid item xs={12}>
            <Box sx={{ display: 'flex', gap: 2, justifyContent: 'flex-end' }}>
              <Button variant="outlined" onClick={doReset} sx={{ borderRadius: R }}>重置</Button>
              <Button variant="contained" onClick={doSubmit} sx={{ borderRadius: R, bgcolor: '#2e7d32', '&:hover': { bgcolor: '#1b5e20' } }}>提交登记</Button>
            </Box>
          </Grid>
        </Grid>
      </Paper>

      {/* === 部分 B：记录列表 === */}
      <Paper elevation={0} sx={{ p: { xs: 1, md: 2 }, borderRadius: R, border: '1px solid #e0e0e0' }}>
        <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2, flexWrap: 'wrap', gap: 1 }}>
          <Typography variant="h6" fontWeight={700}>登记记录</Typography>
          <FormControl size="small" sx={{ minWidth: 120 }}>
            <InputLabel>状态</InputLabel>
            <Select value={statusFilter} label="状态" onChange={e => { setStatusFilter(e.target.value); setPage(0); }}>
              {STATUS_OPTIONS.map(s => <MenuItem key={s} value={s}>{s}</MenuItem>)}
            </Select>
          </FormControl>
        </Box>

        {ld ? <Box sx={{ textAlign: 'center', py: 4 }}><CircularProgress size={32} /></Box> : (
          <>
            {/* 表头 */}
            <Box sx={{ display: 'flex', px: 2, py: 1, bgcolor: '#f5f5f5', borderRadius: R, fontWeight: 600, fontSize: '0.875rem', color: '#666', flexWrap: 'wrap' }}>
              <Box sx={{ width: 70 }}>状态</Box>
              <Box sx={{ width: 50 }}>序号</Box>
              <Box sx={{ flex: 1, minWidth: 100 }}>样品批号</Box>
              <Box sx={{ flex: 1, minWidth: 80 }}>送样人</Box>
              <Box sx={{ flex: 1, minWidth: 80 }}>检测类型</Box>
              <Box sx={{ flex: 2, minWidth: 120 }}>样品主要成分</Box>
              <Box sx={{ width: 30 }} />
            </Box>

            {records.map(r => {
              const isDone = r.status === '检测完成';
              return (
                <Box key={r.id}>
                  <Box
                    onClick={() => doExpand(r.id)}
                    sx={{
                      display: 'flex', px: 2, py: 1.5, borderBottom: '1px solid #f0f0f0', cursor: 'pointer',
                      alignItems: 'center', flexWrap: 'wrap',
                      opacity: isDone ? 0.5 : 1,
                      '&:hover': { bgcolor: '#fafafa' },
                    }}
                  >
                    <Box sx={{ width: 70 }}>
                      <Chip label={r.status} color={STATUS_COLORS[r.status] || 'default'} size="small" variant={isDone ? 'outlined' : 'filled'} />
                    </Box>
                    <Box sx={{ width: 50, fontSize: '0.875rem', color: '#999' }}>#{r.seq_no}</Box>
                    <Box sx={{ flex: 1, minWidth: 100, fontSize: '0.875rem' }}>{r.batch_no}</Box>
                    <Box sx={{ flex: 1, minWidth: 80, fontSize: '0.875rem' }}>{r.user_name}</Box>
                    <Box sx={{ flex: 1, minWidth: 80, fontSize: '0.875rem' }}>{r.detection_type}</Box>
                    <Box sx={{ flex: 2, minWidth: 120, fontSize: '0.875rem', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{r.main_components}</Box>
                    <Box sx={{ width: 30 }}>{expandedId === r.id ? <ExpandLessIcon fontSize="small" /> : <ExpandMoreIcon fontSize="small" />}</Box>
                  </Box>

                  {/* 展开详情 */}
                  <Collapse in={expandedId === r.id}>
                    <Paper elevation={0} sx={{ mx: 2, mb: 1, p: 2, border: '1px solid #e8e8e8', borderRadius: R, bgcolor: '#fafafa' }}>
                      <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
                        <Typography variant="subtitle2" fontWeight={700} color="#2e7d32">登记详情 · 序号 #{r.seq_no}</Typography>
                        <Chip label={r.status} color={STATUS_COLORS[r.status] || 'default'} size="small" />
                      </Box>

                      {editingId === r.id ? (
                        /* 编辑模式 */
                        <Grid container spacing={2}>
                          <Grid item xs={12} sm={6}>
                            <TextField label="样品批号" fullWidth size="small" value={editForm.batch_no || ''} onChange={e => setEditForm(p => ({ ...p, batch_no: e.target.value }))} />
                          </Grid>
                          <Grid item xs={12} sm={6}>
                            <TextField label="送样人" fullWidth size="small" value={editForm.user_name || ''} onChange={e => setEditForm(p => ({ ...p, user_name: e.target.value }))} />
                          </Grid>
                          <Grid item xs={12} sm={6}>
                            <TextField label="实验室/车间" fullWidth size="small" value={editForm.lab_name || ''} onChange={e => setEditForm(p => ({ ...p, lab_name: e.target.value }))} />
                          </Grid>
                          <Grid item xs={12} sm={6}>
                            <TextField label="所属项目" fullWidth size="small" value={editForm.project_name || ''} onChange={e => setEditForm(p => ({ ...p, project_name: e.target.value }))} />
                          </Grid>
                          <Grid item xs={12} sm={6}>
                            <TextField label="送样时间" type="datetime-local" fullWidth size="small" value={editForm.submitted_at || ''} onChange={e => setEditForm(p => ({ ...p, submitted_at: e.target.value }))} InputLabelProps={{ shrink: true }} />
                          </Grid>
                          <Grid item xs={12}>
                            <TextField label="样品主要成分" fullWidth size="small" value={editForm.main_components || ''} onChange={e => setEditForm(p => ({ ...p, main_components: e.target.value }))} />
                          </Grid>
                          <Grid item xs={12}>
                            <TextField label="注意事项" fullWidth size="small" multiline rows={2} value={editForm.notes || ''} onChange={e => setEditForm(p => ({ ...p, notes: e.target.value }))} />
                          </Grid>
                          <Grid item xs={12}>
                            <Box sx={{ display: 'flex', gap: 1, justifyContent: 'flex-end' }}>
                              <Button variant="outlined" size="small" onClick={doCancelEdit} sx={{ borderRadius: R }}>取消</Button>
                              <Button variant="contained" size="small" onClick={() => doSaveEdit(r.id)} sx={{ borderRadius: R, bgcolor: '#2e7d32', '&:hover': { bgcolor: '#1b5e20' } }}>保存</Button>
                            </Box>
                          </Grid>
                        </Grid>
                      ) : (
                        /* 只读模式 */
                        <>
                          <Grid container spacing={2} sx={{ mb: 2 }}>
                            <Grid item xs={12} sm={6}><Typography variant="caption" color="text.secondary">样品批号</Typography><Typography variant="body2">{r.batch_no}</Typography></Grid>
                            <Grid item xs={12} sm={6}><Typography variant="caption" color="text.secondary">送样人</Typography><Typography variant="body2">{r.user_name}</Typography></Grid>
                            <Grid item xs={12} sm={6}><Typography variant="caption" color="text.secondary">实验室/车间</Typography><Typography variant="body2">{r.lab_name}</Typography></Grid>
                            <Grid item xs={12} sm={6}><Typography variant="caption" color="text.secondary">所属项目</Typography><Typography variant="body2">{r.project_name}</Typography></Grid>
                            <Grid item xs={12} sm={6}><Typography variant="caption" color="text.secondary">送样时间</Typography><Typography variant="body2">{fmtDate(r.submitted_at)}</Typography></Grid>
                            <Grid item xs={12} sm={6}><Typography variant="caption" color="text.secondary">检测时间</Typography><Typography variant="body2">{fmtDate(r.detection_date)}</Typography></Grid>
                            <Grid item xs={12}><Typography variant="caption" color="text.secondary">样品主要成分</Typography><Typography variant="body2">{r.main_components}</Typography></Grid>
                            {r.notes && <Grid item xs={12}><Typography variant="caption" color="text.secondary">注意事项</Typography><Typography variant="body2">{r.notes}</Typography></Grid>}
                          </Grid>
                          <Box sx={{ display: 'flex', gap: 1, justifyContent: 'flex-end' }}>
                            {!isDone && <Button variant="outlined" size="small" onClick={() => doEdit(r)} sx={{ borderRadius: R }}>编辑</Button>}
                            {NEXT_STATUS[r.status] && (
                              <Button variant="contained" size="small" onClick={() => doStatusFlow(r.id, r.status)}
                                sx={{ borderRadius: R, bgcolor: '#2e7d32', '&:hover': { bgcolor: '#1b5e20' } }}>
                                {NEXT_STATUS_LABEL[r.status]}
                              </Button>
                            )}
                          </Box>
                        </>
                      )}
                    </Paper>
                  </Collapse>
                </Box>
              );
            })}

            {records.length === 0 && <Box sx={{ textAlign: 'center', py: 6, color: '#999' }}>暂无登记记录</Box>}

            <TablePagination
              component="div" count={total} page={page} onPageChange={(_, p) => setPage(p)}
              rowsPerPage={PAGE_SIZE} rowsPerPageOptions={[PAGE_SIZE]}
              onRowsPerPageChange={e => { setPage(0); }}
              labelRowsPerPage="每页"
            />
          </>
        )}
      </Paper>

      <Snackbar open={snack.open} autoHideDuration={3000} onClose={() => setSnack(s => ({ ...s, open: false }))} anchorOrigin={{ vertical: 'top', horizontal: 'center' }}>
        <Alert severity={snack.sev} sx={{ width: '100%' }} onClose={() => setSnack(s => ({ ...s, open: false }))}>{snack.msg}</Alert>
      </Snackbar>
    </Box>
  );
};
export default SampleInfoEntry;
