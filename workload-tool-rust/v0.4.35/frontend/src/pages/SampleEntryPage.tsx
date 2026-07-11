import React, { useEffect, useState, useCallback, useMemo, useRef } from 'react';
import {
  Box, Typography, TextField, CircularProgress, Snackbar, Alert, Chip,
  Button, Checkbox, Autocomplete, useMediaQuery, useTheme,
  Table, TableBody, TableCell, TableContainer, TableHead, TableRow, Paper, TablePagination,
} from '@mui/material';
import AddIcon from '@mui/icons-material/Add';
import DeleteIcon from '@mui/icons-material/Delete';
import RefreshIcon from '@mui/icons-material/Refresh';
import SendIcon from '@mui/icons-material/Send';
import { useParams, useNavigate } from 'react-router-dom';
import type { Project, Method, MethodType, WorkRecord, ProjectGroup, Division, RdRecordColumn } from '../types';
import { getProjects, getMethods, createRdRecord, getMethodTypes, getGroups, getRdRecords, sampleRdRecord, getDivisions, getRdRecordColumns } from '../api/client';
import { useUser } from '../UserContext';

const R = '2px';

interface RowState {
  id: number; // local row id
  checked: boolean;
  user_name: string;
  project_id: number | null;
  project_name: string;
  division_id: number | null;
  method_id: number | null;
  method_name: string;
  method_type: string;  // v0.4.28: 改为可编辑，级联过滤
  quantity: number;
  batch_no: string;
  notes: string;
}

let rowIdCounter = 1;

const createEmptyRow = (defaultUser: string, defaultDivisionId: number | null | undefined): RowState => ({
  id: rowIdCounter++,
  checked: false,
  user_name: defaultUser,
  project_id: null,
  project_name: '',
  division_id: defaultDivisionId ?? null,
  method_id: null,
  method_name: '',
  method_type: '',
  quantity: 1,
  batch_no: '',
  notes: '',
});

const SampleEntryPage: React.FC = () => {
  const { groupId } = useParams<{ groupId: string }>();
  const gid = Number(groupId) || 0;
  const navigate = useNavigate();
  const { user, hasPermission } = useUser();

  const [groups, setGroups] = useState<ProjectGroup[]>([]);
  const [projects, setProjects] = useState<Project[]>([]);
  const [allMethods, setAllMethods] = useState<Method[]>([]);
  const [mts, setMts] = useState<MethodType[]>([]);
  const [divs, setDivs] = useState<Division[]>([]);
  const [loading, setLoading] = useState(true);
  const [rdColumns, setRdColumns] = useState<RdRecordColumn[]>([]);

  const [dateTime, setDateTime] = useState(() => {
    const now = new Date();
    const y = now.getFullYear();
    const m = String(now.getMonth() + 1).padStart(2, '0');
    const d = String(now.getDate()).padStart(2, '0');
    const hh = String(now.getHours()).padStart(2, '0');
    const mm = String(now.getMinutes()).padStart(2, '0');
    return `${y}-${m}-${d}T${hh}:${mm}`;
  });

  const [rows, setRows] = useState<RowState[]>([]);
  const [snackMsg, setSnackMsg] = useState('');
  const [snackErr, setSnackErr] = useState(false);

  // 今日记录
  const [todayRecords, setTodayRecords] = useState<WorkRecord[]>([]);
  const [recordsLoading, setRecordsLoading] = useState(false);
  const [recordsPage, setRecordsPage] = useState(0);
  const [recordsTotal, setRecordsTotal] = useState(0);
  // v0.4.34: 选中记录（用于右上角动态状态）
  const [selectedRecordId, setSelectedRecordId] = useState<number | null>(null);
  const pageSize = 20;

  const theme = useTheme();
  const isMobile = useMediaQuery(theme.breakpoints.down('sm'));

  const labName = groups.find(g => g.id === gid)?.name || '';
  const labDivisionId = groups.find(g => g.id === gid)?.division_id;
  const dt = 'rd';

  // v0.4.27-A: auto-fill user info
  useEffect(() => {
    const defaultUser = user?.username || '';
    const defaultDiv = user?.division_id ?? labDivisionId ?? null;
    setRows([createEmptyRow(defaultUser, defaultDiv)]);
  }, [user, labDivisionId]);

  const getTodayStr = useCallback(() => {
    const now = new Date();
    const y = now.getFullYear();
    const m = String(now.getMonth() + 1).padStart(2, '0');
    const d = String(now.getDate()).padStart(2, '0');
    return `${y}-${m}-${d}`;
  }, []);

  const loadData = useCallback(async () => {
    setLoading(true);
    try {
      const [gr, pr, mr, mtr, dr, rcr] = await Promise.all([
        getGroups(), getProjects({ group_id: gid, active_only: true }),
        getMethods(), getMethodTypes(), getDivisions(), getRdRecordColumns(),
      ]);
      if (gr.code === 0 && gr.data) setGroups(gr.data);
      if (pr.code === 0 && pr.data) setProjects(pr.data);
      if (mr.code === 0 && mr.data) setAllMethods(mr.data);
      if (mtr.code === 0 && mtr.data) setMts(mtr.data);
      if (dr.code === 0 && dr.data) setDivs(dr.data);
      if (rcr.code === 0 && rcr.data) setRdColumns(rcr.data);
    } catch {} finally { setLoading(false); }
  }, [gid]);

  const loadTodayRecords = useCallback(async (page?: number) => {
    if (!gid) return;
    setRecordsLoading(true);
    try {
      const today = getTodayStr();
      const r = await getRdRecords({ group_id: gid, start: today, end: today, page: (page ?? recordsPage) + 1, page_size: pageSize });
      if (r.code === 0 && r.data) {
        setTodayRecords(r.data.items);
        setRecordsTotal(r.data.total);
      }
    } catch {} finally { setRecordsLoading(false); }
  }, [gid, recordsPage, getTodayStr]);

  useEffect(() => { loadData(); }, [loadData]);
  useEffect(() => { loadTodayRecords(); }, [loadTodayRecords]);

  // 该实验室的研发项目所关联的方法
  const linkedMethods = useMemo(() => {
    if (!projects.length || !allMethods.length) return [] as Method[];
    const linkedIds = new Set<number>();
    projects.forEach(p => (p.method_ids || []).forEach(id => linkedIds.add(id)));
    return allMethods.filter(m => linkedIds.has(m.id));
  }, [projects, allMethods]);

  // v0.4.28: 级联过滤辅助函数
  const getAvailableTypes = (projectId: number | null): string[] => {
    if (!projectId) return [];
    const proj = projects.find(p => p.id === projectId);
    if (!proj) return [];
    const types = new Set<string>();
    linkedMethods
      .filter(m => (proj.method_ids || []).includes(m.id))
      .forEach(m => (m.type_names || []).forEach(t => types.add(t)));
    return Array.from(types);
  };

  const getAvailableMethods = (projectId: number | null, typeFilter: string): Method[] => {
    if (!projectId) return [];
    const proj = projects.find(p => p.id === projectId);
    if (!proj) return [];
    let methods = linkedMethods.filter(m => (proj.method_ids || []).includes(m.id));
    if (typeFilter) {
      methods = methods.filter(m => (m.type_names || []).includes(typeFilter));
    }
    return methods;
  };

  const refreshRecords = useCallback(() => {
    setRecordsPage(0);
    setSelectedRecordId(null);
    const today = getTodayStr();
    setRecordsLoading(true);
    getRdRecords({ group_id: gid, start: today, end: today, page: 1, page_size: pageSize })
      .then(r => { if (r.code === 0 && r.data) { setTodayRecords(r.data.items); setRecordsTotal(r.data.total); } })
      .catch(() => {})
      .finally(() => setRecordsLoading(false));
  }, [gid, getTodayStr]);

  const addRow = () => {
    const defaultUser = user?.username || '';
    const defaultDiv = user?.division_id ?? labDivisionId ?? null;
    setRows(prev => [...prev, createEmptyRow(defaultUser, defaultDiv)]);
  };

  const deleteChecked = () => {
    setRows(prev => prev.filter(r => !r.checked));
  };

  const reset = () => {
    const defaultUser = user?.username || '';
    const defaultDiv = user?.division_id ?? labDivisionId ?? null;
    setRows([createEmptyRow(defaultUser, defaultDiv)]);
  };

  const toggleCheck = (rowId: number) => {
    setRows(prev => prev.map(r => r.id === rowId ? { ...r, checked: !r.checked } : r));
  };

  const updateRow = (rowId: number, patch: Partial<RowState>) => {
    setRows(prev => prev.map(r => r.id === rowId ? { ...r, ...patch } : r));
  };

  const handleSubmit = async () => {
    const invalidRows = rows.filter(r => !r.user_name.trim() || !r.project_id || r.quantity < 1);
    if (invalidRows.length > 0) {
      setSnackMsg('请填写完整信息：送样人、项目、数量（≥1）');
      setSnackErr(true);
      return;
    }

    let successCount = 0;
    let failCount = 0;
    for (const row of rows) {
      try {
        const body: any = {
          project_id: row.project_id!,
          method_id: row.method_id,
          user_name: row.user_name,
          quantity: row.quantity,
          recorded_at: dateTime,
          group_id: gid,
          division_id: row.division_id ?? labDivisionId ?? null,
          batch_no: row.batch_no || undefined,
          notes: row.notes || undefined,
        };
        await createRdRecord(body);
        successCount++;
      } catch {
        failCount++;
      }
    }

    if (failCount === 0) {
      setSnackMsg(`成功提交 ${successCount} 条记录`);
      setSnackErr(false);
      reset();
      refreshRecords();
    } else {
      setSnackMsg(`成功 ${successCount} 条，失败 ${failCount} 条`);
      setSnackErr(true);
    }
  };

  const handleRecordsPageChange = (_e: unknown, newPage: number) => {
    setRecordsPage(newPage);
    const today = getTodayStr();
    setRecordsLoading(true);
    getRdRecords({ group_id: gid, start: today, end: today, page: newPage + 1, page_size: pageSize })
      .then(r => { if (r.code === 0 && r.data) { setTodayRecords(r.data.items); setRecordsTotal(r.data.total); } })
      .catch(() => {})
      .finally(() => setRecordsLoading(false));
  };

  const handleSample = async (rec: WorkRecord) => {
    try {
      await sampleRdRecord(rec.id);
      setSnackMsg('取样成功'); setSnackErr(false);
      refreshRecords();
    } catch { setSnackMsg('取样失败'); setSnackErr(true); }
  };

  // v0.4.34: 选中的今日记录
  const selectedRecord = todayRecords.find(r => r.id === selectedRecordId);
  const headerStatus = selectedRecord ? (selectedRecord.status || '待取样') : '待取样';

  // 表单列（show_in_form=true）
  const formColumns = rdColumns.filter(c => c.show_in_form);

  if (loading) return <Box sx={{ display: 'flex', justifyContent: 'center', mt: 8 }}><CircularProgress /></Box>;

  return (<Box sx={{ p: 2 }}>
    {/* === 卡片式白色容器，绿色边框 — 与样品信息登记一致 === */}
    <Paper elevation={0} sx={{ p: 2, mb: 2, borderRadius: R, border: '2px solid #2e7d32', background: 'linear-gradient(145deg,#ffffff,#f1f8e9)' }}>

      {/* 顶部标题栏 */}
      <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 1 }}>
        <Box>
          <Typography variant="h6" fontWeight={700} sx={{ cursor: 'pointer' }} onClick={() => navigate('/sample')}>← 研发送样录入</Typography>
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mt: 0.5 }}>
            <Chip label={`实验室: ${labName}`} size="small" color="primary" variant="outlined" />
            <Typography variant="body2" color="text.secondary">检测类型: {dt} · 序号: 自动生成</Typography>
          </Box>
        </Box>
        <Chip label={headerStatus} size="small" sx={{ bgcolor: headerStatus === '已取样' ? '#c8e6c9' : '#fff3e0', color: headerStatus === '已取样' ? '#2e7d32' : '#e65100', fontWeight: 500 }} />
      </Box>

      {/* 公共送样时间（整单公共） */}
      <Box sx={{ mb: 2 }}>
        <Typography variant="body2" sx={{ mb: 0.5 }}>送样时间（整单公共）</Typography>
        <TextField
          type="datetime-local"
          size="small"
          value={dateTime}
          onChange={e => setDateTime(e.target.value)}
          InputLabelProps={{ shrink: true }}
          sx={{ width: isMobile ? '100%' : 240, '& .MuiOutlinedInput-root': { borderRadius: R } }}
        />
      </Box>

      {/* 操作按钮栏 */}
      <Box sx={{ display: 'flex', gap: 1, mb: 2, flexWrap: 'wrap' }}>
        <Button variant="outlined" size="small" startIcon={<AddIcon />} onClick={addRow} sx={{ borderRadius: R }}>
          添加行
        </Button>
        <Button variant="outlined" size="small" startIcon={<DeleteIcon />} color="error" onClick={deleteChecked} sx={{ borderRadius: R }}
          disabled={!rows.some(r => r.checked)}>
          删除选中
        </Button>
        <Button variant="outlined" size="small" startIcon={<RefreshIcon />} onClick={reset} sx={{ borderRadius: R }}>
          重置
        </Button>
      </Box>

      {/* 多行表格 — 动态列 */}
      {rows.length > 0 && (
      <TableContainer component={Paper} variant="outlined" sx={{ borderRadius: R, boxShadow: 'none', mb: 2, overflowX: 'auto' }}>
        <Table size="small" sx={{ minWidth: 1280 }} stickyHeader>
          <TableHead>
            <TableRow sx={{ bgcolor: 'rgba(230,81,0,0.06)' }}>
              <TableCell padding="checkbox" sx={{ fontWeight: 700, fontSize: '0.8rem' }}>
                <Checkbox size="small" checked={rows.length > 0 && rows.every(r => r.checked)}
                  indeterminate={rows.some(r => r.checked) && !rows.every(r => r.checked)}
                  onChange={() => {
                    const allChecked = rows.every(r => r.checked);
                    setRows(prev => prev.map(r => ({ ...r, checked: !allChecked })));
                  }} />
              </TableCell>
              <TableCell sx={{ fontWeight: 700, fontSize: '0.8rem', whiteSpace: 'nowrap', width: 40, textAlign: 'center' }}>序号</TableCell>
              {formColumns.map(col => {
                if (col.name === 'user_name') {
                  return <TableCell key={col.name} sx={{ fontWeight: 700, fontSize: '0.8rem', whiteSpace: 'nowrap', minWidth: col.width || 120 }}>{col.label}</TableCell>;
                }
                if (col.name === 'division_id') {
                  return <TableCell key={col.name} sx={{ fontWeight: 700, fontSize: '0.8rem', whiteSpace: 'nowrap', minWidth: col.width || 140 }}>{col.label}</TableCell>;
                }
                if (col.name === 'project_name') {
                  return <TableCell key={col.name} sx={{ fontWeight: 700, fontSize: '0.8rem', whiteSpace: 'nowrap', minWidth: col.width || 150 }}>{col.label}</TableCell>;
                }
                if (col.name === 'detection_type') {
                  return <TableCell key={col.name} sx={{ fontWeight: 700, fontSize: '0.8rem', whiteSpace: 'nowrap', minWidth: col.width || 110 }}>{col.label}</TableCell>;
                }
                if (col.name === 'method_name') {
                  return <TableCell key={col.name} sx={{ fontWeight: 700, fontSize: '0.8rem', whiteSpace: 'nowrap', minWidth: col.width || 180 }}>{col.label}</TableCell>;
                }
                if (col.name === 'quantity') {
                  return <TableCell key={col.name} sx={{ fontWeight: 700, fontSize: '0.8rem', whiteSpace: 'nowrap', minWidth: col.width || 80 }}>{col.label}</TableCell>;
                }
                if (col.name === 'batch_no') {
                  return <TableCell key={col.name} sx={{ fontWeight: 700, fontSize: '0.8rem', whiteSpace: 'nowrap', minWidth: col.width || 100 }}>{col.label}</TableCell>;
                }
                if (col.name === 'notes') {
                  return <TableCell key={col.name} sx={{ fontWeight: 700, fontSize: '0.8rem', whiteSpace: 'nowrap', minWidth: col.width || 130 }}>{col.label}</TableCell>;
                }
                return <TableCell key={col.name} sx={{ fontWeight: 700, fontSize: '0.8rem', whiteSpace: 'nowrap', minWidth: col.width || 100 }}>{col.label}</TableCell>;
              })}
            </TableRow>
          </TableHead>
          <TableBody>
            {rows.map((row, idx) => {
              const availableTypes = getAvailableTypes(row.project_id);
              const availableMethods = getAvailableMethods(row.project_id, row.method_type);
              return (
              <TableRow key={row.id} hover sx={{ '&:last-child td': { borderBottom: 0 }, height: 48 }}>
                <TableCell padding="checkbox">
                  <Checkbox size="small" checked={row.checked} onChange={() => toggleCheck(row.id)} />
                </TableCell>
                <TableCell sx={{ fontSize: '0.8rem', textAlign: 'center' }}>{idx + 1}</TableCell>
                {formColumns.map(col => {
                  if (col.name === 'user_name') {
                    return (
                      <TableCell key={col.name} sx={{ p: 0.5 }}>
                        <TextField size="small" value={row.user_name} onChange={e => updateRow(row.id, { user_name: e.target.value })}
                          sx={{ width: Math.min(col.width || 120, 140), '& .MuiOutlinedInput-root': { borderRadius: R, fontSize: '0.8rem' } }}
                          inputProps={{ style: { padding: '2px 6px' } }} />
                      </TableCell>
                    );
                  }
                  if (col.name === 'division_id') {
                    return (
                      <TableCell key={col.name} sx={{ p: 0.5 }}>
                        <TextField size="small" select value={row.division_id ?? ''}
                          onChange={e => updateRow(row.id, { division_id: e.target.value ? Number(e.target.value) : null })}
                          sx={{ width: Math.min(col.width || 140, 160), '& .MuiOutlinedInput-root': { borderRadius: R, fontSize: '0.8rem' } }}
                          SelectProps={{ native: true }} inputProps={{ style: { padding: '2px 6px' } }}>
                          <option value="">-</option>
                          {divs.map(d => <option key={d.id} value={d.id}>{d.name}</option>)}
                        </TextField>
                      </TableCell>
                    );
                  }
                  if (col.name === 'project_name') {
                    return (
                      <TableCell key={col.name} sx={{ p: 0.5 }}>
                        <TextField size="small" select value={row.project_id ?? ''}
                          onChange={e => {
                            const pid = e.target.value ? Number(e.target.value) : null;
                            const proj = projects.find(p => p.id === pid);
                            updateRow(row.id, {
                              project_id: pid,
                              project_name: proj?.name || '',
                              method_type: '',
                              method_id: null,
                              method_name: '',
                            });
                          }}
                          sx={{ width: Math.min(col.width || 150, 200), '& .MuiOutlinedInput-root': { borderRadius: R, fontSize: '0.8rem' } }}
                          SelectProps={{ native: true }} inputProps={{ style: { padding: '2px 6px' } }}>
                          <option value="">-</option>
                          {projects.map(p => <option key={p.id} value={p.id}>{p.name}</option>)}
                        </TextField>
                      </TableCell>
                    );
                  }
                  if (col.name === 'detection_type') {
                    return (
                      <TableCell key={col.name} sx={{ p: 0.5 }}>
                        <TextField size="small" select value={row.method_type}
                          onChange={e => {
                            const mt = e.target.value;
                            updateRow(row.id, { method_type: mt, method_id: null, method_name: '' });
                          }}
                          sx={{ width: Math.min(col.width || 110, 140), '& .MuiOutlinedInput-root': { borderRadius: R, fontSize: '0.8rem' } }}
                          SelectProps={{ native: true }} inputProps={{ style: { padding: '2px 6px' } }}
                          disabled={!row.project_id}>
                          <option value="">-</option>
                          {availableTypes.map(t => <option key={t} value={t}>{t}</option>)}
                        </TextField>
                      </TableCell>
                    );
                  }
                  if (col.name === 'method_name') {
                    return (
                      <TableCell key={col.name} sx={{ p: 0.5 }}>
                        <TextField size="small" select value={row.method_id ?? ''}
                          onChange={e => {
                            const mid = e.target.value ? Number(e.target.value) : null;
                            const meth = availableMethods.find(m => m.id === mid);
                            updateRow(row.id, { method_id: mid, method_name: meth?.name || '' });
                          }}
                          sx={{ width: Math.min(col.width || 180, 220), '& .MuiOutlinedInput-root': { borderRadius: R, fontSize: '0.8rem' } }}
                          SelectProps={{ native: true }} inputProps={{ style: { padding: '2px 6px' } }}
                          disabled={!row.project_id}>
                          <option value="">-</option>
                          {availableMethods.map(m => <option key={m.id} value={m.id}>{m.name}</option>)}
                        </TextField>
                      </TableCell>
                    );
                  }
                  if (col.name === 'quantity') {
                    return (
                      <TableCell key={col.name} sx={{ p: 0.5 }}>
                        <TextField type="number" size="small" value={row.quantity}
                          onChange={e => updateRow(row.id, { quantity: Math.max(1, Number(e.target.value) || 1) })}
                          sx={{ width: Math.min(col.width || 80, 100), '& .MuiOutlinedInput-root': { borderRadius: R, fontSize: '0.8rem' } }}
                          inputProps={{ min: 1, style: { padding: '2px 6px', textAlign: 'center' } }} />
                      </TableCell>
                    );
                  }
                  if (col.name === 'batch_no') {
                    return (
                      <TableCell key={col.name} sx={{ p: 0.5 }}>
                        <TextField size="small" value={row.batch_no} onChange={e => updateRow(row.id, { batch_no: e.target.value })}
                          sx={{ width: Math.min(col.width || 100, 140), '& .MuiOutlinedInput-root': { borderRadius: R, fontSize: '0.8rem' } }}
                          inputProps={{ style: { padding: '2px 6px' } }} />
                      </TableCell>
                    );
                  }
                  if (col.name === 'notes') {
                    return (
                      <TableCell key={col.name} sx={{ p: 0.5 }}>
                        <TextField size="small" value={row.notes} onChange={e => updateRow(row.id, { notes: e.target.value })}
                          sx={{ minWidth: Math.min(col.width || 130, 200), '& .MuiOutlinedInput-root': { borderRadius: R, fontSize: '0.8rem' } }}
                          inputProps={{ style: { padding: '2px 6px' } }} />
                      </TableCell>
                    );
                  }
                  // 其他动态列（占位）
                  return <TableCell key={col.name} sx={{ p: 0.5, fontSize: '0.8rem' }}>-</TableCell>;
                })}
              </TableRow>
            );
            })}
          </TableBody>
        </Table>
      </TableContainer>
      )}

      {/* 底栏：提交按钮 */}
      <Box sx={{ display: 'flex', justifyContent: 'flex-end', alignItems: 'center', flexWrap: 'wrap', gap: 1 }}>
        <Button variant="contained" size="small" startIcon={<SendIcon />} onClick={handleSubmit}
          sx={{ borderRadius: R, bgcolor: '#2e7d32', '&:hover': { bgcolor: '#1b5e20' } }}
          disabled={rows.length === 0}>
          提交登记（{rows.length} 行）
        </Button>
      </Box>
    </Paper>

    {/* 今日记录 */}
    <Box sx={{ mt: 3 }}>
      <Typography variant="h6" fontWeight={600} sx={{ mb: 1.5 }}>
        今日记录
        {recordsTotal > 0 && <Typography component="span" variant="body2" color="text.secondary" sx={{ ml: 1 }}>（共 {recordsTotal} 条）</Typography>}
      </Typography>
      {recordsLoading && todayRecords.length === 0 ? (
        <Box sx={{ display: 'flex', justifyContent: 'center', py: 3 }}><CircularProgress size={28} /></Box>
      ) : todayRecords.length === 0 ? (
        <Typography color="text.secondary" textAlign="center" sx={{ py: 3, fontSize: '0.875rem' }}>今天暂无录入记录</Typography>
      ) : (
        <TableContainer component={Paper} variant="outlined" sx={{ borderRadius: R, boxShadow: 'none', overflowX: 'auto' }}>
          <Table size="small" sx={{ minWidth: 960 }}>
            <TableHead>
              <TableRow sx={{ bgcolor: 'rgba(230,81,0,0.06)' }}>
                <TableCell sx={{ fontWeight: 700, fontSize: '0.8rem', whiteSpace: 'nowrap', width: 40, textAlign: 'center' }}>序号</TableCell>
                <TableCell sx={{ fontWeight: 700, fontSize: '0.8rem', whiteSpace: 'nowrap' }}>状态</TableCell>
                <TableCell sx={{ fontWeight: 700, fontSize: '0.8rem', whiteSpace: 'nowrap' }}>送样时间</TableCell>
                <TableCell sx={{ fontWeight: 700, fontSize: '0.8rem', whiteSpace: 'nowrap' }}>实验室</TableCell>
                <TableCell sx={{ fontWeight: 700, fontSize: '0.8rem', whiteSpace: 'nowrap' }}>研发项目</TableCell>
                <TableCell sx={{ fontWeight: 700, fontSize: '0.8rem', whiteSpace: 'nowrap' }}>送样人</TableCell>
                {rdColumns.filter(c => c.name === 'method_name' || c.name === 'detection_type' || c.name === 'quantity' || c.name === 'batch_no' || c.name === 'sampling_person' || c.name === 'sampling_time' || c.name === 'notes').map(col => (
                  <TableCell key={col.name} sx={{ fontWeight: 700, fontSize: '0.8rem', whiteSpace: 'nowrap' }}>{col.label}</TableCell>
                ))}
              </TableRow>
            </TableHead>
            <TableBody>
              {todayRecords.map((rec, idx) => {
                const status = rec.status || '待取样';
                const isSampled = status === '已取样';
                return (
                  <TableRow key={rec.id} hover
                    onClick={() => setSelectedRecordId(rec.id)}
                    selected={selectedRecordId === rec.id}
                    sx={{ '&:last-child td': { borderBottom: 0 }, cursor: 'pointer', '&.Mui-selected': { bgcolor: 'rgba(46,125,50,0.08)' } }}>
                    <TableCell sx={{ fontSize: '0.8rem', textAlign: 'center' }}>{recordsPage * pageSize + idx + 1}</TableCell>
                    <TableCell sx={{ fontSize: '0.8rem', whiteSpace: 'nowrap' }}>
                      <Typography variant="body2" sx={{
                        display: 'inline-block', px: 1, py: 0.3, borderRadius: R, fontSize: '0.75rem', fontWeight: 600,
                        bgcolor: isSampled ? '#c8e6c9' : '#fff9c4',
                        color: isSampled ? '#2e7d32' : '#f57f17',
                      }}>{status}</Typography>
                    </TableCell>
                    <TableCell sx={{ fontSize: '0.8rem', whiteSpace: 'nowrap' }}>
                      {rec.recorded_at ? rec.recorded_at.replace('T', ' ').substring(0, 19) : '-'}
                    </TableCell>
                    <TableCell sx={{ fontSize: '0.8rem', maxWidth: 100, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{labName || '-'}</TableCell>
                    <TableCell sx={{ fontSize: '0.8rem', maxWidth: 120, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{rec.project_name || '-'}</TableCell>
                    <TableCell sx={{ fontSize: '0.8rem', whiteSpace: 'nowrap' }}>{rec.user_name || '-'}</TableCell>
                    {rdColumns.filter(c => c.name === 'method_name' || c.name === 'detection_type' || c.name === 'quantity' || c.name === 'batch_no' || c.name === 'sampling_person' || c.name === 'sampling_time' || c.name === 'notes').map(col => {
                      if (col.name === 'method_name') return <TableCell key={col.name} sx={{ fontSize: '0.8rem', maxWidth: col.width || 140, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{rec.method_name || '-'}</TableCell>;
                      if (col.name === 'detection_type') return <TableCell key={col.name} sx={{ fontSize: '0.8rem', whiteSpace: 'nowrap' }}>{rec.method_type || '-'}</TableCell>;
                      if (col.name === 'quantity') return <TableCell key={col.name} sx={{ fontSize: '0.8rem', fontWeight: 600, whiteSpace: 'nowrap' }}>{rec.quantity}</TableCell>;
                      if (col.name === 'batch_no') return <TableCell key={col.name} sx={{ fontSize: '0.8rem', whiteSpace: 'nowrap' }}>{rec.batch_no || '-'}</TableCell>;
                      if (col.name === 'sampling_person') return (
                        <TableCell key={col.name} sx={{ fontSize: '0.8rem', whiteSpace: 'nowrap' }}>
                          {isSampled ? (
                            <Typography variant="body2" sx={{ color: '#2e7d32', fontWeight: 600 }}>{rec.sampler || '-'}</Typography>
                          ) : hasPermission('sample:collect') ? (
                            <Button variant="contained" size="small" sx={{ borderRadius: R, bgcolor: '#2e7d32', '&:hover': { bgcolor: '#1b5e20' }, fontSize: '0.75rem', minWidth: 0, px: 1.5, py: 0 }}
                              onClick={() => handleSample(rec)}>
                              取样
                            </Button>
                          ) : (
                            <Typography variant="body2" sx={{ color: '#999' }}>待取样</Typography>
                          )}
                        </TableCell>
                      );
                      if (col.name === 'sampling_time') return <TableCell key={col.name} sx={{ fontSize: '0.8rem', whiteSpace: 'nowrap' }}>{rec.sampled_at ? rec.sampled_at.replace('T', ' ').substring(0, 19) : '-'}</TableCell>;
                      return <TableCell key={col.name} sx={{ fontSize: '0.8rem', whiteSpace: 'nowrap' }}>-</TableCell>;
                    })}
                  </TableRow>
                );
              })}
            </TableBody>
          </Table>
          {recordsTotal > pageSize && (
            <TablePagination
              component="div"
              count={recordsTotal}
              page={recordsPage}
              onPageChange={handleRecordsPageChange}
              rowsPerPage={pageSize}
              rowsPerPageOptions={[pageSize]}
              labelDisplayedRows={({ from, to, count }) => `${from}-${to} / ${count}`}
              sx={{ '& .MuiTablePagination-toolbar': { minHeight: 40 }, '& .MuiTablePagination-selectLabel': { fontSize: '0.75rem' }, '& .MuiTablePagination-displayedRows': { fontSize: '0.75rem' } }}
            />
          )}
        </TableContainer>
      )}
    </Box>

    <Snackbar open={!!snackMsg} autoHideDuration={3000} onClose={() => setSnackMsg('')} anchorOrigin={{ vertical: 'bottom', horizontal: 'center' }}>
      <Alert severity={snackErr ? 'error' : 'success'} sx={{ borderRadius: R }} onClose={() => setSnackMsg('')}>{snackMsg}</Alert>
    </Snackbar>
  </Box>);
};

export default SampleEntryPage;
