import React, { useEffect, useState, useCallback, useMemo } from 'react';
import {
  Box, Typography, TextField, IconButton, CircularProgress, Snackbar, Alert, Chip,
  useMediaQuery, useTheme,
  Table, TableBody, TableCell, TableContainer, TableHead, TableRow, Paper, TablePagination,
} from '@mui/material';
import ArrowBackIcon from '@mui/icons-material/ArrowBack';
import CheckCircleIcon from '@mui/icons-material/CheckCircle'; import CheckCircleOutlineIcon from '@mui/icons-material/CheckCircleOutline';
import { useParams, useNavigate } from 'react-router-dom';
import type { Project, Method, MethodType, WorkRecord, ProjectGroup } from '../types';
import { getProjects, getMethods, createRdRecord, getMethodTypes, getGroups, getRdRecords, sampleRdRecord } from '../api/client';

const R = '2px';

const typeColorMap: Record<string, 'info'|'success'|'warning'|'primary'|'default'> = {
  '液相': 'info', '气相': 'success', '理化': 'warning', '检测类型': 'primary',
};

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

const EntryPage: React.FC = () => {
  const { groupId } = useParams<{ groupId: string }>();
  const gid = Number(groupId) || 0;
  const navigate = useNavigate();

  const [projects, setProjects] = useState<Project[]>([]);
  const [allMethods, setAllMethods] = useState<Method[]>([]);
  const [loading, setLoading] = useState(true);
  const [userName, setUserName] = useState('');
  const [dateTime, setDateTime] = useState(() => {
    const now = new Date();
    const y = now.getFullYear();
    const m = String(now.getMonth() + 1).padStart(2, '0');
    const d = String(now.getDate()).padStart(2, '0');
    const hh = String(now.getHours()).padStart(2, '0');
    const mm = String(now.getMinutes()).padStart(2, '0');
    return `${y}-${m}-${d}T${hh}:${mm}`;
  });
  const [snackMsg, setSnackMsg] = useState('');
  const [snackErr, setSnackErr] = useState(false);

  const [mts, setMts] = useState<MethodType[]>([]);
  const [typeFilter, setTypeFilter] = useState('全部');
  const [projectFilter, setProjectFilter] = useState('全部');
  const [groups, setGroups] = useState<ProjectGroup[]>([]);

  // 今日记录
  const [todayRecords, setTodayRecords] = useState<WorkRecord[]>([]);
  const [recordsLoading, setRecordsLoading] = useState(false);
  const [recordsPage, setRecordsPage] = useState(0);
  const [recordsTotal, setRecordsTotal] = useState(0);
  const pageSize = 20;

  const theme = useTheme();
  const isMobile = useMediaQuery(theme.breakpoints.down('sm'));

  const labName = groups.find(g => g.id === gid)?.name || '';

  const loadMethods = useCallback(async () => {
    try { const r = await getMethods(); if (r.code === 0 && r.data) setAllMethods(r.data); } catch {}
  }, []);

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

  const loadGroups = useCallback(async () => {
    try { const r = await getGroups(); if (r.code === 0 && r.data) setGroups(r.data); } catch {}
  }, []);

  // 获取今日日期字符串 YYYY-MM-DD
  const getTodayStr = useCallback(() => {
    const now = new Date();
    const y = now.getFullYear();
    const m = String(now.getMonth() + 1).padStart(2, '0');
    const d = String(now.getDate()).padStart(2, '0');
    return `${y}-${m}-${d}`;
  }, []);

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

  useEffect(() => { loadMethods(); loadProjects(); loadMethodTypes(); loadGroups(); }, [loadMethods, loadProjects, loadMethodTypes, loadGroups]);
  useEffect(() => { loadTodayRecords(); }, [loadTodayRecords]);

  // 该实验室关联的方法 = 该实验室的研发项目所关联的方法
  const linkedMethods = useMemo(() => {
    if (!projects.length || !allMethods.length) return [] as Method[];
    const linkedIds = new Set<number>();
    projects.forEach(p => (p.method_ids || []).forEach(id => linkedIds.add(id)));
    return allMethods.filter(m => linkedIds.has(m.id));
  }, [projects, allMethods]);

  // 按项目过滤的方法
  const projectFilteredMethods = useMemo(() => {
    if (projectFilter === '全部') return linkedMethods;
    const selectedProject = projects.find(p => p.name === projectFilter);
    if (!selectedProject) return linkedMethods;
    const selectedIds = new Set(selectedProject.method_ids || []);
    return linkedMethods.filter(m => selectedIds.has(m.id));
  }, [linkedMethods, projectFilter, projects]);

  const filtered = useMemo(() => {
    if (typeFilter === '全部') return projectFilteredMethods;
    return projectFilteredMethods.filter(m => (m.type_names || []).includes(typeFilter));
  }, [projectFilteredMethods, typeFilter]);

  const refreshRecords = useCallback(() => {
    setRecordsPage(0);
    const today = getTodayStr();
    setRecordsLoading(true);
    getRdRecords({ group_id: gid, start: today, end: today, page: 1, page_size: pageSize })
      .then(r => { if (r.code === 0 && r.data) { setTodayRecords(r.data.items); setRecordsTotal(r.data.total); } })
      .catch(() => {})
      .finally(() => setRecordsLoading(false));
  }, [gid, getTodayStr]);

  const handleSubmit = async (method: Method, quantity: number) => {
    if (!userName.trim()) { setSnackMsg('请输入送样人'); setSnackErr(true); return false; }
    // 找到关联该方法的研发项目 ID（取第一个）
    const linkedProject = projects.find(p => (p.method_ids || []).includes(method.id));
    const projectId = linkedProject?.id;
    if (!projectId) { setSnackMsg('该方法未关联任何研发项目'); setSnackErr(true); return false; }
    try {
      const divId = groups.find(g => g.id === gid)?.division_id ?? null;
      const r = await createRdRecord({ project_id: projectId, method_id: method.id, user_name: userName, quantity, recorded_at: dateTime, group_id: gid, division_id: divId });
      if (r.code === 0) {
        setSnackMsg(`录入成功: ${userName} ×${quantity}`); setSnackErr(false);
        // 自动刷新今日记录
        refreshRecords();
        return true;
      }
      setSnackMsg(r.message); setSnackErr(true); return false;
    } catch { setSnackMsg('录入失败'); setSnackErr(true); return false; }
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

  if (loading) return <Box sx={{ display: 'flex', justifyContent: 'center', mt: 8 }}><CircularProgress /></Box>;

  return (<Box>
    <Box sx={{ display: 'flex', alignItems: 'center', gap: 1.5, mb: 2 }}>
      <IconButton onClick={() => navigate('/sample')} sx={{ bgcolor: 'rgba(230,81,0,0.08)', '&:hover': { bgcolor: 'rgba(230,81,0,0.15)' } }}>
        <ArrowBackIcon />
      </IconButton>
      <Box><Typography variant="h5" fontWeight={700}>研发送样录入</Typography><Typography variant="caption" color="text.secondary">选择检测方法并录入数量</Typography>{labName && <Chip label={`实验室: ${labName}`} size="small" color="primary" variant="outlined" sx={{ ml: 1, borderRadius: R, height: 22, fontSize: '0.7rem' }} />}</Box>
    </Box>

    {/* 用户 & 时间 */}
    <Box sx={{ display: 'flex', gap: 2, mb: 2, flexWrap: 'wrap', alignItems: 'center', flexDirection: isMobile ? 'column' : 'row' }}>
      <TextField label="送样人" size="small" value={userName} onChange={e => setUserName(e.target.value)} sx={{ width: isMobile ? '100%' : 140, '& .MuiOutlinedInput-root': { borderRadius: R } }} />
      <TextField label="日期时间" type="datetime-local" size="small" value={dateTime} onChange={e => setDateTime(e.target.value)} InputLabelProps={{ shrink: true }} sx={{ width: isMobile ? '100%' : 200, '& .MuiOutlinedInput-root': { borderRadius: R } }} />
    </Box>

    {/* 项目筛选栏 */}
    {projects.length > 0 && (
      <Box sx={{ display: 'flex', gap: 1, mb: 1, flexWrap: 'wrap', alignItems: 'center' }}>
        <Chip
          label={`全部 (${linkedMethods.length})`} size="medium"
          color={projectFilter === '全部' ? 'primary' : 'default'}
          variant={projectFilter === '全部' ? 'filled' : 'outlined'}
          onClick={() => { setProjectFilter('全部'); setTypeFilter('全部'); }}
          sx={{ borderRadius: R, cursor: 'pointer', fontWeight: projectFilter === '全部' ? 700 : 400 }}
        />
        {projects.map(p => {
          const cnt = linkedMethods.filter(m => (p.method_ids || []).includes(m.id)).length;
          return (
            <Chip key={p.id}
              label={`${p.name} (${cnt})`} size="medium"
              color={projectFilter === p.name ? 'primary' : 'default'}
              variant={projectFilter === p.name ? 'filled' : 'outlined'}
              onClick={() => { setProjectFilter(p.name); setTypeFilter('全部'); }}
              sx={{ borderRadius: R, cursor: 'pointer', fontWeight: projectFilter === p.name ? 700 : 400 }}
            />
          );
        })}
      </Box>
    )}

    {/* 类型筛选按钮栏 — 基于方法的 type_names */}
    <Box sx={{ display: 'flex', gap: 1, mb: 2, flexWrap: 'wrap', alignItems: 'center' }}>
      <Chip
        label={`全部 (${projectFilteredMethods.length})`} size="medium"
        color={typeFilter === '全部' ? 'primary' : 'default'}
        variant={typeFilter === '全部' ? 'filled' : 'outlined'}
        onClick={() => setTypeFilter('全部')}
        sx={{ borderRadius: R, cursor: 'pointer', fontWeight: typeFilter === '全部' ? 700 : 400 }}
      />
      {mts.filter(t => t.name !== '检测类型').map(t => {
        const cnt = projectFilteredMethods.filter(m => (m.type_names || []).includes(t.name)).length;
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

    {/* 方法列表 */}
    {filtered.length === 0
      ? <Typography color="text.secondary" textAlign="center" sx={{ py: 6 }}>{typeFilter !== '全部' ? `无 "${typeFilter}" 类型的检测方法` : '该实验室暂无关联方法'}</Typography>
      : filtered.map(m => (
        <MethodRow key={m.id} method={m} linkedProjectName={projects.find(p => (p.method_ids || []).includes(m.id))?.name} onSubmit={handleSubmit} isMobile={isMobile} />
      ))}

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
        <TableContainer component={Paper} variant="outlined" sx={{ borderRadius: R, boxShadow: 'none', '& .MuiPaper-root': { borderRadius: R } }}>
          <Table size="small" sx={{ minWidth: 960 }}>
            <TableHead>
              <TableRow sx={{ bgcolor: 'rgba(230,81,0,0.06)' }}>
                <TableCell sx={{ fontWeight: 700, fontSize: '0.8rem', whiteSpace: 'nowrap', width: 40, textAlign: 'center' }}>序号</TableCell>
                <TableCell sx={{ fontWeight: 700, fontSize: '0.8rem', whiteSpace: 'nowrap' }}>状态</TableCell>
                <TableCell sx={{ fontWeight: 700, fontSize: '0.8rem', whiteSpace: 'nowrap' }}>送样时间</TableCell>
                <TableCell sx={{ fontWeight: 700, fontSize: '0.8rem', whiteSpace: 'nowrap' }}>实验室</TableCell>
                <TableCell sx={{ fontWeight: 700, fontSize: '0.8rem', whiteSpace: 'nowrap' }}>研发项目</TableCell>
                <TableCell sx={{ fontWeight: 700, fontSize: '0.8rem', whiteSpace: 'nowrap' }}>送样人</TableCell>
                <TableCell sx={{ fontWeight: 700, fontSize: '0.8rem', whiteSpace: 'nowrap' }}>方法</TableCell>
                <TableCell sx={{ fontWeight: 700, fontSize: '0.8rem', whiteSpace: 'nowrap' }}>类型</TableCell>
                <TableCell sx={{ fontWeight: 700, fontSize: '0.8rem', whiteSpace: 'nowrap' }}>仪器</TableCell>
                <TableCell sx={{ fontWeight: 700, fontSize: '0.8rem', whiteSpace: 'nowrap' }}>数量</TableCell>
                <TableCell sx={{ fontWeight: 700, fontSize: '0.8rem', whiteSpace: 'nowrap' }}>取样人</TableCell>
                <TableCell sx={{ fontWeight: 700, fontSize: '0.8rem', whiteSpace: 'nowrap' }}>取样时间</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {todayRecords.map((rec, idx) => {
                const status = rec.status || '待取样';
                const isSampled = status === '已取样';
                return (
                <TableRow key={rec.id} hover sx={{ '&:last-child td': { borderBottom: 0 } }}>
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
                  <TableCell sx={{ fontSize: '0.8rem', maxWidth: 140, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{rec.method_name || '-'}</TableCell>
                  <TableCell sx={{ fontSize: '0.8rem', whiteSpace: 'nowrap' }}>{rec.method_type || '-'}</TableCell>
                  <TableCell sx={{ fontSize: '0.8rem', whiteSpace: 'nowrap' }}>
                    {extractInstrumentFromMethodName(rec.method_name || '') ? (
                      <Chip label={extractInstrumentFromMethodName(rec.method_name || '')} size="small" sx={{ bgcolor: '#00897b', color: '#fff', borderRadius: R, height: 20, fontSize: '0.7rem' }} />
                    ) : '-'}
                  </TableCell>
                  <TableCell sx={{ fontSize: '0.8rem', fontWeight: 600, whiteSpace: 'nowrap' }}>{rec.quantity}</TableCell>
                  <TableCell sx={{ fontSize: '0.8rem', whiteSpace: 'nowrap' }}>
                    {isSampled ? (
                      <Typography variant="body2" sx={{ color: '#2e7d32', fontWeight: 600 }}>{rec.sampler || '-'}</Typography>
                    ) : (
                      <TextField
                        size="small"
                        placeholder="取样人"
                        onBlur={async (e) => {
                          const val = e.target.value.trim();
                          if (val) {
                            try {
                              await sampleRdRecord(rec.id, val);
                              setSnackMsg('取样成功'); setSnackErr(false);
                              refreshRecords();
                            } catch { setSnackMsg('取样失败'); setSnackErr(true); }
                          }
                        }}
                        onKeyDown={async (e) => {
                          if (e.key === 'Enter') {
                            const val = (e.target as HTMLInputElement).value.trim();
                            if (val) {
                              try {
                                await sampleRdRecord(rec.id, val);
                                setSnackMsg('取样成功'); setSnackErr(false);
                                refreshRecords();
                              } catch { setSnackMsg('取样失败'); setSnackErr(true); }
                            }
                          }
                        }}
                        sx={{ '& .MuiOutlinedInput-root': { borderRadius: R, fontSize: '0.75rem' }, width: 80 }}
                        inputProps={{ style: { padding: '2px 8px' } }}
                      />
                    )}
                  </TableCell>
                  <TableCell sx={{ fontSize: '0.8rem', whiteSpace: 'nowrap' }}>
                    {rec.sampled_at ? rec.sampled_at.replace('T', ' ').substring(0, 19) : '-'}
                  </TableCell>
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

/** 内联方法行组件 — 显示方法名、类型标签、系数、关联项目名，并提供录入入口 */
const MethodRow: React.FC<{
  method: Method;
  linkedProjectName?: string;
  onSubmit: (method: Method, quantity: number) => Promise<boolean>;
  isMobile: boolean;
}> = ({ method, linkedProjectName, onSubmit, isMobile }) => {
  const [q, setQ] = useState<number | ''>('');
  const [l, setL] = useState(false);
  const [s, setS] = useState(false);

  const h = async () => {
    if (q === '' || Number(q) < 1) return;
    setL(true);
    const ok = await onSubmit(method, Number(q));
    setL(false);
    if (ok) { setS(true); setTimeout(() => { setS(false); setQ(''); }, 2000); }
  };

  return (
    <Box sx={{
      display: 'flex', alignItems: 'center', gap: 1.5, py: 1.5, px: 2, mb: 1, borderRadius: '2px',
      flexDirection: isMobile ? 'column' : 'row',
      background: s ? 'linear-gradient(145deg,#e8f5e9,#f1f8e9)' : 'linear-gradient(145deg,#ffffff,#fafafa)',
      border: '1px solid', borderColor: s ? '#a5d6a7' : 'rgba(0,0,0,0.06)',
      borderLeft: '4px solid', borderLeftColor: s ? '#43a047' : '#ef6c00',
      boxShadow: '0 2px 12px rgba(0,0,0,0.04)',
      transition: 'all 0.3s',
      '&:hover': { boxShadow: '0 4px 20px rgba(0,0,0,0.08)', transform: 'translateY(-1px)' }
    }}>
      <Box sx={{ flex: 1, width: isMobile ? '100%' : undefined }}>
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, flexWrap: 'wrap' }}>
          <Typography variant="body1" fontWeight={600} sx={{ wordBreak: 'break-word' }}>
            {method.name}
          </Typography>
          {method.amount != null && method.amount > 0 && (
            <Chip label={`¥${method.amount.toFixed(2)}`} size="small" variant="outlined" sx={{ borderRadius: '2px', height: 22, fontSize: '0.7rem', borderColor: '#2e7d32', color: '#2e7d32', fontWeight: 600 }} />
          )}
          {(method.type_names || []).map(t => (
            <Chip key={t} label={t} size="small" color={typeColorMap[t] || 'default'} sx={{ borderRadius: '2px', height: 20, fontSize: '0.65rem' }} />
          ))}
          {linkedProjectName && (
            <Chip label={`项目: ${linkedProjectName}`} size="small" color="success" variant="outlined" sx={{ borderRadius: '2px', height: 20, fontSize: '0.65rem' }} />
          )}
        </Box>
        <Typography variant="caption" color="text.secondary" sx={{ wordBreak: 'break-word', display: 'block', mt: 0.3 }}>
          {method.full_name && <span>{method.full_name}</span>}
        </Typography>
      </Box>
      <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, width: isMobile ? '100%' : undefined, justifyContent: isMobile ? 'flex-end' : undefined }}>
        <TextField type="number" size="small" value={q} onChange={e => setQ(e.target.value === '' ? '' : Number(e.target.value))}
          inputProps={{ min: 1, style: { textAlign: 'center', width: 60 } }}
          sx={{ width: 80, '& .MuiOutlinedInput-root': { borderRadius: '2px', '& fieldset': { borderColor: 'rgba(0,0,0,0.1)' } } }}
          disabled={l || s} onKeyDown={e => { if (e.key === 'Enter') h(); }} />
        <IconButton onClick={h} disabled={l || s || q === '' || Number(q) < 1}
          sx={{ borderRadius: '50%', bgcolor: s ? '#e8f5e9' : '#fff3e0', color: s ? '#43a047' : '#e65100', '&:disabled': { color: 'rgba(0,0,0,0.2)', bgcolor: 'transparent' } }} size="medium">
          {l ? <CircularProgress size={24} sx={{ color: '#e65100' }} /> : s ? <CheckCircleIcon className="animate-checkmark" /> : <CheckCircleOutlineIcon />}
        </IconButton>
      </Box>
    </Box>
  );
};

export default EntryPage;
