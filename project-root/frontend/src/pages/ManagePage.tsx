import React, { useState, useEffect, useCallback } from 'react';
import {
  Box, Typography, Paper, Button, CircularProgress, Dialog, DialogTitle,
  DialogContent, DialogActions, TextField, FormControl, InputLabel, Select,
  MenuItem, Switch, FormControlLabel, IconButton, Chip, TableContainer,
  Table, TableHead, TableBody, TableRow, TableCell, Alert, Snackbar,
  Autocomplete, Checkbox, FormGroup,
} from '@mui/material';
import AddIcon from '@mui/icons-material/Add'; import EditIcon from '@mui/icons-material/Edit';
import DeleteIcon from '@mui/icons-material/Delete'; import FolderIcon from '@mui/icons-material/Folder';
import ListAltIcon from '@mui/icons-material/ListAlt'; import ScienceIcon from '@mui/icons-material/Science';
import DeleteSweepIcon from '@mui/icons-material/DeleteSweep'; import ReceiptLongIcon from '@mui/icons-material/ReceiptLong';
import HistoryIcon from '@mui/icons-material/History'; import TuneIcon from '@mui/icons-material/Tune';
import CloudUploadIcon from '@mui/icons-material/CloudUpload'; import BackupIcon from '@mui/icons-material/Backup';
import { getGroups, createGroup, updateGroup, deleteGroup, getProjects, createProject, updateProject, deleteProject, getRecords, restoreRecord, getAuditLogs, batchProjectCoefficient, getBackupStatus, backupNow, getBackupConfig, updateBackupConfig, deleteBackup, restoreBackup, getMethodTypes, createMethodType, updateMethodType, deleteMethodType, getMethods, createMethod, updateMethod, deleteMethod, methodImport } from '../api/client';
import type { ProjectGroup, Project, WorkRecord, AuditLog, BackupStatus, MethodType, Method } from '../types';
import ConfirmDialog from '../components/ConfirmDialog';
type TV = 'projects' | 'groups' | 'methods' | 'trash' | 'audit' | 'backup';

const R = '2px'; const cSx={borderRadius:R,fontWeight:700,border:'1px solid rgba(0,0,0,0.08)'}; const lSx={p:2,mb:1.5,borderRadius:R, border:'1px solid rgba(0,0,0,0.06)',transition:'all 0.2s','&:hover':{boxShadow:'0 4px 20px rgba(0,0,0,0.08)'}}; const tSx={borderRadius:R,border:'1px solid rgba(0,0,0,0.06)',overflow:'auto'};
const AL: Record<string,string>={create:'创建',update:'更新',delete:'删除',restore:'恢复',import:'导入'}; const AC: Record<string,'success'|'error'|'info'|'warning'|'default'>={create:'success',update:'info',delete:'error',restore:'warning',import:'warning'}; const TL: Record<string,string>={work_records:'工作记录',projects:'项目',project_groups:'分组',samples:'送样记录'};

const TC = [
  { key: 'projects', label: '研发项目管理', icon: <ListAltIcon />, desc: '研发项目及关联实验室' },
  { key: 'groups', label: '实验室管理', icon: <FolderIcon />, desc: '新增编辑实验室映射录入选项卡' },
  { key: 'methods', label: '检测方法管理', icon: <ScienceIcon />, desc: '液相/气相/理化/ICP/热分析等检测方法' },
  { key: 'trash', label: '回收站', icon: <DeleteSweepIcon />, desc: '恢复已删除的记录' },
  { key: 'audit', label: '审计日志', icon: <ReceiptLongIcon />, desc: '操作记录追溯' },
  { key: 'backup', label: '数据备份', icon: <BackupIcon />, desc: '备份恢复与自动备份设置' },
] as { key: TV; label: string; icon: React.ReactNode; desc: string }[];

const ManagePage: React.FC = () => {
  const [tb, setTb] = useState<TV>('projects');
  const [ld, setLd] = useState(false); const [msg, setMsg] = useState(''); const [err, setErr] = useState(false);
  const sm = useCallback((m: string, isErr?: boolean) => { setMsg(m); setErr(!!isErr); }, []);
  const [co, setCo] = useState(false); const [ca, setCa] = useState<() => Promise<void>>(() => async () => { setCo(false); });

  // groups
  const [gs, setGs] = useState<ProjectGroup[]>([]);
  const [gd, setGd] = useState(false); const [gf, setGf] = useState({ id: 0, name: '', sort_order: 0 }); const [ged, setGed] = useState(false);
  const lg = useCallback(async () => { try { const r = await getGroups(); if (r.code === 0 && r.data) setGs(r.data); } catch {} }, []);
  const hgs = async () => { if (!gf.name.trim()) { sm('请输入分组名称', true); return; } if (ged) { const r = await updateGroup(gf.id, { name: gf.name, sort_order: gf.sort_order }); if (r.code === 0) { sm('更新成功'); lg(); setGd(false); } else sm(r.message, true); } else { const r = await createGroup({ name: gf.name, sort_order: gf.sort_order }); if (r.code === 0) { sm('创建成功'); lg(); setGd(false); } else sm(r.message, true); } };

  // projects (v0.2.17 simplified)
  const [ps, setPs] = useState<Project[]>([]); const [sg, setSg] = useState(0); const [pd, setPd] = useState(false);
  const [pf, setPf] = useState({ id: 0, name: '', notes: '', lab_ids: [] as number[], method_ids: [] as number[] });
  const [ped, setPed] = useState(false);
  const lp = useCallback(async () => { try { const r = await getProjects({ group_id: sg || undefined }); if (r.code === 0 && r.data) setPs(r.data); } catch {} }, [sg]);
  const hps = async () => { if (!pf.name.trim()) { sm('请输入项目名称', true); return; } if (ped) { const body: any = { name: pf.name, notes: pf.notes, lab_ids: pf.lab_ids, method_ids: pf.method_ids }; const r = await updateProject(pf.id, body); if (r.code === 0) { sm('更新成功'); lp(); setPd(false); } else sm(r.message, true); } else { const body: any = { name: pf.name, notes: pf.notes, lab_ids: pf.lab_ids, method_ids: pf.method_ids }; const r = await createProject(body); if (r.code === 0) { sm('创建成功'); lp(); setPd(false); } else sm(r.message, true); } };
  const [bdo, setBdo] = useState(false); const [bgid, setBgid] = useState(0); const [bgn, setBgn] = useState(''); const [bcoeff, setBcoeff] = useState(1.0);
  const hbc = async () => { if (bcoeff <= 0) { sm('系数必须大于0', true); return; } try { const r = await batchProjectCoefficient({ group_id: bgid, coefficient: bcoeff }); if (r.code === 0) { sm(r.message); lp(); setBdo(false); } else sm(r.message, true); } catch { sm('批量更新失败', true); } };

  // methods (v0.2.17 new — 独立 methods 表)
  const [ml, setMl] = useState<Method[]>([]);
  const [md, setMd] = useState(false); const [mf, setMf] = useState({ id: 0, name: '', full_name: '', coefficient: 1.0, notes: '', type_ids: [] as number[] }); const [med, setMed] = useState(false);
  const lm = useCallback(async () => { try { const r = await getMethods(); if (r.code === 0 && r.data) setMl(r.data); } catch {} }, []);
  const hms = async () => { if (!mf.name.trim()) { sm('请输入方法名称', true); return; } if (med) { const body: any = { name: mf.name, full_name: mf.full_name, coefficient: mf.coefficient, notes: mf.notes, type_ids: mf.type_ids }; const r = await updateMethod(mf.id, body); if (r.code === 0) { sm('更新成功'); lm(); setMd(false); } else sm(r.message, true); } else { const body: any = { name: mf.name, full_name: mf.full_name, coefficient: mf.coefficient, notes: mf.notes, type_ids: mf.type_ids }; const r = await createMethod(body); if (r.code === 0) { sm('创建成功'); lm(); setMd(false); } else sm(r.message, true); } };

  // method types
  const [mts, setMts] = useState<MethodType[]>([]);
  const [mtd, setMtd] = useState(false); const [mtf, setMtf] = useState({ id: 0, name: '', sort_order: 10 });
  const lmt = useCallback(async () => { try { const r = await getMethodTypes(); if (r.code === 0 && r.data) setMts(r.data); } catch {} }, []);
  const hmt = async () => { if (!mtf.name.trim()) { sm('请输入类型名称', true); return; } try { if (mtf.id > 0) { const r = await updateMethodType(mtf.id, { name: mtf.name, sort_order: mtf.sort_order }); if (r.code === 0) { sm('更新成功'); lmt(); setMtd(false); } else sm(r.message, true); } else { const r = await createMethodType({ name: mtf.name, sort_order: mtf.sort_order }); if (r.code === 0) { sm('创建成功'); lmt(); setMtd(false); } else sm(r.message, true); } } catch { sm('操作失败', true); } };

  // trash
  const [rs, setRs] = useState<WorkRecord[]>([]); const [rc, setRc] = useState(0);
  const lt = useCallback(async () => { try { const r = await getRecords({}); if (r.code === 0 && r.data) { setRs(r.data.items.filter((x: any) => false)); setRc(0); } try { const all = await getAuditLogs({ page_size: 3 }); if (all.code === 0 && all.data) setRc(all.data.total || 0); } catch {} } catch {} }, []);
  const [tr, setTr] = useState<WorkRecord[]>([]);
  const loadTrash = async () => { try { const r = await getRecords({}); if (r.code === 0 && r.data) setTr(r.data.items.filter((x: any) => !x.is_active)); } catch {} };

  // audit
  const [al, setAl] = useState<AuditLog[]>([]); const [at, setAt] = useState(0); const [ap, setAp] = useState(1);
  const la = useCallback(async (p: number) => { try { const r = await getAuditLogs({ page: p, page_size: 50 }); if (r.code === 0 && r.data) { setAl(r.data.items); setAt(r.data.total); setAp(p); } } catch {} }, []);

  // backup
  const [bk, setBk] = useState<BackupStatus | null>(null); const [bkAuto, setBkAuto] = useState(false);
  const [bkInt, setBkInt] = useState(24); const [bkR, setBkR] = useState(''); const [bkN, setBkN] = useState(false);
  const loadBk = async () => { try { const r = await getBackupStatus(); if (r.code === 0 && r.data) { setBk(r.data); setBkAuto(r.data.auto_enabled); setBkInt(r.data.auto_interval_hours); } } catch {} };

  useEffect(() => { setLd(true); Promise.all([lg(), lp(), lt()]).finally(() => setLd(false)); }, [lg, lp, lt]);
  useEffect(() => { lp(); }, [sg, lp]);
  useEffect(() => { if (tb === 'audit') la(1); if (tb === 'backup') loadBk(); if (tb === 'methods') { lmt(); lm(); } if (tb === 'trash') loadTrash(); }, [tb, la, lmt, lm]);

  if (ld) return <Box sx={{ display: 'flex', justifyContent: 'center', mt: 8 }}><CircularProgress /></Box>;

  return (<Box>
    <Typography variant="h5" fontWeight={700} sx={{ mb: 3, px: 1, background: 'linear-gradient(135deg,#f4511e,#e53935)', WebkitBackgroundClip:'text', WebkitTextFillColor:'transparent', backgroundClip:'text' }}>系统管理</Typography>
    {msg && <Alert severity={err ? 'error' : 'success'} sx={{ mb: 2, borderRadius: R }} onClose={() => setMsg('')}>{msg}</Alert>}

    {/* 卡片网格 */}
    <Box sx={{ display: 'grid', gridTemplateColumns: { xs: '1fr', sm: '1fr 1fr', md: '1fr 1fr 1fr' }, gap: 2, mb: 3 }}>
      {TC.map(c => (
        <Paper key={c.key} elevation={0} onClick={() => setTb(c.key)} sx={{ p: 2.5, borderRadius: R, cursor: 'pointer', border: '2px solid', borderColor: tb === c.key ? '#f4511e' : 'rgba(0,0,0,0.06)', transition: 'all 0.2s', '&:hover': { borderColor: '#f4511e', boxShadow: '0 4px 24px rgba(244,81,30,0.12)' } }}>
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1.5, mb: 0.5 }}>
            <Box sx={{ color: tb === c.key ? '#f4511e' : 'text.secondary' }}>{c.icon}</Box>
            <Typography variant="subtitle1" fontWeight={700}>{c.label}</Typography>
          </Box>
          <Typography variant="caption" color="text.secondary">{c.desc}</Typography>
        </Paper>
      ))}
    </Box>

    {/* 内容区 */}

    {/* ── 1. 研发项目管理 (v0.2.17 简化) ── */}
    {tb === 'projects' && <Box>
      <Box sx={{ display: 'flex', gap: 1, mb: 2, flexWrap: 'wrap', justifyContent: 'space-between' }}>
        <Box sx={{ display: 'flex', gap: 1 }}>
          <FormControl size="small" sx={{ minWidth: 160 }}><InputLabel>实验室分组</InputLabel>
            <Select value={sg} label="实验室分组" onChange={e => setSg(Number(e.target.value))} sx={{ borderRadius: R }}>
              <MenuItem value={0}>全部实验室</MenuItem>
              {gs.map(g => <MenuItem key={g.id} value={g.id}>{g.name}</MenuItem>)}
            </Select>
          </FormControl>
          {sg > 0 && <Button size="small" variant="outlined" startIcon={<TuneIcon />} onClick={() => { setBgid(sg); setBgn(gs.find(g => g.id === sg)?.name || ''); setBcoeff(1.0); setBdo(true); }} sx={{ borderRadius: R, borderColor: '#9c27b0', color: '#9c27b0' }}>批量系数</Button>}
        </Box>
        <Box sx={{ display: 'flex', gap: 1 }}>
          <Button variant="contained" startIcon={<AddIcon />} onClick={() => { setPf({ id: 0, name: '', notes: '', lab_ids: [], method_ids: [] }); setPed(false); setPd(true); }} size="small" sx={{ borderRadius: R, background: 'linear-gradient(135deg,#f4511e,#e53935)', boxShadow: '0 4px 14px rgba(244,81,30,0.3)' }}>新建研发项目</Button>
        </Box>
      </Box>
      {ps.length === 0 ? <Typography color="text.secondary" textAlign="center" sx={{ py: 4 }}>暂无研发项目</Typography>
        : ps.map(p => <Paper key={p.id} elevation={0} sx={lSx}>
          <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: 1 }}>
            <Box>
              <Typography variant="subtitle1" fontWeight={600}>{p.name}</Typography>
              <Typography variant="caption" color="text.secondary">
                关联实验室: {p.lab_names.length > 0 ? p.lab_names.map(n => <Chip key={n} label={n} size="small" variant="outlined" sx={{ borderRadius: R, height: 20, fontSize: '0.7rem', mr: 0.5 }} />) : '—'}
                {p.method_names.length > 0 && <span> · 关联方法: {p.method_names.map(n => <Chip key={n} label={n} size="small" color="info" variant="outlined" sx={{ borderRadius: R, height: 20, fontSize: '0.7rem', mr: 0.5 }} />)}</span>}
              </Typography>
            </Box>
            <Box sx={{ display: 'flex', gap: 0.5 }}>
              <IconButton onClick={() => { setPf({ id: p.id, name: p.name, notes: p.notes, lab_ids: p.lab_ids || [], method_ids: p.method_ids || [] }); setPed(true); setPd(true); }} size="small" sx={{ color: '#f4511e' }}><EditIcon fontSize="small" /></IconButton>
              <IconButton onClick={() => { setCa(() => async () => { const r = await deleteProject(p.id); if (r.code === 0) { sm('删除成功'); lp(); } else sm(r.message, true); setCo(false); }); setCo(true); }} size="small" color="error"><DeleteIcon fontSize="small" /></IconButton>
            </Box>
          </Box>
        </Paper>)}
    </Box>}

    {/* ── 2. 实验室管理 ── */}
    {tb === 'groups' && <Box>
      <Box sx={{ display: 'flex', justifyContent: 'flex-end', mb: 2 }}>
        <Button variant="contained" startIcon={<AddIcon />} onClick={() => { setGf({ id: 0, name: '', sort_order: 0 }); setGed(false); setGd(true); }} size="small" sx={{ borderRadius: R, background: 'linear-gradient(135deg,#f4511e,#e53935)', boxShadow: '0 4px 14px rgba(244,81,30,0.3)' }}>新建实验室</Button>
      </Box>
      <Typography variant="caption" color="text.secondary" sx={{ mb: 1, display: 'block' }}>实验室分组直接映射为工作量录入界面的选项卡</Typography>
      {gs.length === 0 ? <Typography color="text.secondary" textAlign="center" sx={{ py: 4 }}>暂无实验室分组</Typography>
        : gs.map(g => <Paper key={g.id} elevation={0} sx={lSx}>
          <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
            <Box><Typography variant="subtitle1" fontWeight={600}>{g.name}</Typography><Typography variant="caption" color="text.secondary">排序: {g.sort_order}</Typography></Box>
            <Box sx={{ display: 'flex', gap: 0.5 }}>
              <IconButton onClick={() => { setGf({ id: g.id, name: g.name, sort_order: g.sort_order }); setGed(true); setGd(true); }} size="small" sx={{ color: '#f4511e' }}><EditIcon fontSize="small" /></IconButton>
              <IconButton onClick={() => { setCa(() => async () => { const r = await deleteGroup(g.id); if (r.code === 0) { sm('删除成功'); lg(); } else sm(r.message, true); setCo(false); }); setCo(true); }} size="small" color="error"><DeleteIcon fontSize="small" /></IconButton>
            </Box>
          </Box>
        </Paper>)}
    </Box>}

    {/* ── 3. 检测方法管理 (v0.2.17 独立 methods API) ── */}
    {tb === 'methods' && <Box>
      <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
        <Box>
          <Typography variant="subtitle1" fontWeight={700}>检测方法管理</Typography>
          <Typography variant="caption" color="text.secondary">
            共 {ml.length} 条方法 · 
            <Button size="small" onClick={() => { setMtf({ id: 0, name: '', sort_order: 10 }); setMtd(true); }} sx={{ minWidth: 'auto', p: 0, ml: 0.5, fontSize: '0.7rem' }}>管理类型</Button>
          </Typography>
        </Box>
        <Box sx={{ display: 'flex', gap: 1 }}>
          <Button variant="outlined" component="label" startIcon={<CloudUploadIcon />} size="small"
            sx={{ borderRadius: R, borderColor: '#00897b', color: '#00897b' }}>
            导入方法
            <input type="file" accept=".xlsx" hidden onChange={async (e) => {
              const f = e.target.files?.[0]; if (!f) return;
              try { const r = await methodImport(f); sm(`导入成功: ${r.data?.total_methods || 0}条方法, ${r.data?.total_groups || 0}个分组`); lm(); }
              catch { sm('导入失败', true); } e.target.value = '';
            }} />
          </Button>
          <Button variant="contained" startIcon={<AddIcon />} onClick={() => { setMf({ id: 0, name: '', full_name: '', coefficient: 1.0, notes: '', type_ids: [] }); setMed(false); setMd(true); }} size="small" sx={{ borderRadius: R, background: 'linear-gradient(135deg,#f4511e,#e53935)', boxShadow: '0 4px 14px rgba(244,81,30,0.3)' }}>新建方法</Button>
        </Box>
      </Box>
      {ml.length === 0 ? <Typography color="text.secondary" textAlign="center" sx={{ py: 4 }}>暂无检测方法数据，请先导入方法或手动创建</Typography>
        : ml.map(m => <Paper key={m.id} elevation={0} sx={lSx}>
          <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: 1 }}>
            <Box>
              <Typography variant="subtitle1" fontWeight={600}>{m.name}</Typography>
              <Typography variant="caption" color="text.secondary">
                类型: {m.type_names.length > 0 ? m.type_names.map(t => <Chip key={t} label={t} size="small" variant="outlined" sx={{ borderRadius: R, height: 20, fontSize: '0.7rem', mr: 0.5 }} />) : <Chip label="未分类" size="small" sx={{ borderRadius: R, height: 20, fontSize: '0.7rem' }} />}
                系数: <Chip label={(m.coefficient ?? 1).toFixed(1)} size="small" variant="outlined" sx={{ borderRadius: R, height: 20, fontSize: '0.7rem' }} />
                {m.full_name ? <Chip label={m.full_name} size="small" sx={{ borderRadius: R, height: 20, fontSize: '0.65rem', color: '#666' }} /> : null}
              </Typography>
            </Box>
            <Box sx={{ display: 'flex', gap: 0.5 }}>
              <IconButton onClick={() => { setMf({ id: m.id, name: m.name, full_name: m.full_name || '', coefficient: m.coefficient ?? 1.0, notes: m.notes || '', type_ids: m.type_ids || [] }); setMed(true); setMd(true); }} size="small" sx={{ color: '#f4511e' }}><EditIcon fontSize="small" /></IconButton>
              <IconButton onClick={() => { setCa(() => async () => { const r = await deleteMethod(m.id); if (r.code === 0) { sm('删除成功'); lm(); } else sm(r.message, true); setCo(false); }); setCo(true); }} size="small" color="error"><DeleteIcon fontSize="small" /></IconButton>
            </Box>
          </Box>
        </Paper>)}
    </Box>}

    {/* ── 回收站 ── */}
    {tb === 'trash' && <Box>
      <Typography variant="body2" color="text.secondary" sx={{ mb: 2, display: 'flex', alignItems: 'center', gap: 1 }}><DeleteSweepIcon fontSize="small" />共 {rc} 条已删除记录</Typography>
      {tr.length === 0 ? <Typography color="text.secondary" textAlign="center" sx={{ py: 4 }}>回收站为空</Typography>
        : <TableContainer component={Paper} className="table-responsive" sx={tSx}><Table size="small"><TableHead><TableRow>
          <TableCell sx={{ fontWeight: 600 }}>日期</TableCell><TableCell sx={{ fontWeight: 600 }}>项目</TableCell><TableCell sx={{ fontWeight: 600 }}>用户</TableCell><TableCell align="right" sx={{ fontWeight: 600 }}>数量</TableCell><TableCell align="right" sx={{ fontWeight: 600 }}>操作</TableCell>
        </TableRow></TableHead><TableBody>{tr.map(r => <TableRow key={r.id} hover><TableCell>{r.recorded_at}</TableCell><TableCell>{r.project_name}</TableCell><TableCell>{r.user_name}</TableCell><TableCell align="right">{r.quantity}</TableCell><TableCell align="right"><Button size="small" onClick={async () => { try { const res = await restoreRecord(r.id); if (res.code === 0) { sm('恢复成功'); loadTrash(); } else sm(res.message, true); } catch { sm('恢复失败', true); } }} sx={{ borderRadius: R }}>恢复</Button></TableCell></TableRow>)}</TableBody></Table></TableContainer>}
    </Box>}

    {/* ── 审计日志 ── */}
    {tb === 'audit' && <Box>
      <Typography variant="body2" color="text.secondary" sx={{ mb: 2, display: 'flex', alignItems: 'center', gap: 1 }}><HistoryIcon fontSize="small" />共 {at} 条操作记录</Typography>
      {al.length === 0 ? <Typography color="text.secondary" textAlign="center" sx={{ py: 4 }}>暂无审计日志</Typography> : <>
        <TableContainer component={Paper} className="table-responsive" sx={tSx}><Table size="small"><TableHead><TableRow>
          <TableCell sx={{ fontWeight: 600 }}>时间</TableCell><TableCell sx={{ fontWeight: 600 }}>操作类型</TableCell><TableCell sx={{ fontWeight: 600 }}>操作对象</TableCell><TableCell sx={{ fontWeight: 600 }}>记录ID</TableCell><TableCell sx={{ fontWeight: 600 }}>操作人</TableCell>
        </TableRow></TableHead><TableBody>{al.map(l => <TableRow key={l.id} hover><TableCell sx={{ whiteSpace: 'nowrap' }}>{l.created_at}</TableCell><TableCell><Chip label={AL[l.action] || l.action} size="small" color={AC[l.action] || 'default'} variant="outlined" sx={{ borderRadius: R }} /></TableCell><TableCell>{TL[l.table_name] || l.table_name}</TableCell><TableCell>{l.record_id}</TableCell><TableCell>{l.user_name}</TableCell></TableRow>)}</TableBody></Table></TableContainer>
        <Box sx={{ display: 'flex', justifyContent: 'center', mt: 2, gap: 1 }}>
          <Button size="small" disabled={ap <= 1} onClick={() => la(ap - 1)} sx={{ borderRadius: R }}>上一页</Button>
          <Typography variant="body2">{ap} / {Math.max(1, Math.ceil(at / 50))}</Typography>
          <Button size="small" disabled={ap * 50 >= at} onClick={() => la(ap + 1)} sx={{ borderRadius: R }}>下一页</Button>
        </Box></>}
    </Box>}

    {/* ── 数据备份 ── */}
    {tb === 'backup' && <Box>
      <Box sx={{ display: 'flex', gap: 2, mb: 3, flexWrap: 'wrap' }}>
        <Button variant="contained" startIcon={<BackupIcon />} onClick={async () => { setBkN(true); try { const r = await backupNow(); sm(r.message || '备份成功'); await loadBk(); } catch { sm('备份失败', true); } finally { setBkN(false); } }} disabled={bkN} sx={{ borderRadius: R, background: 'linear-gradient(135deg,#00897b,#43a047)' }}>{bkN ? '备份中...' : '立即备份'}</Button>
        <Button variant="outlined" component="label" startIcon={<CloudUploadIcon />} sx={{ borderRadius: R, borderColor: '#f4511e', color: '#f4511e' }}>恢复备份<input type="file" accept=".db" hidden onChange={async (e) => { const f = e.target.files?.[0]; if (!f) return; if (!window.confirm('恢复将替换当前数据库，确定继续？')) { e.target.value = ''; return; } setBkR('恢复中...'); try { const r = await restoreBackup(f); sm(r.message || '恢复成功', !r.message?.startsWith('恢复成功')); await loadBk(); } catch { sm('恢复失败', true); } finally { setBkR(''); e.target.value = ''; } }} /></Button>
        {bkR && <Chip label={bkR} color="warning" size="small" />}
      </Box>
      {bk && <Paper elevation={0} sx={{ p: 2, mb: 3, borderRadius: R, border: '1px solid rgba(0,0,0,0.06)' }}>
        <Typography variant="subtitle2" fontWeight={700} gutterBottom>数据库状态</Typography>
        <Typography variant="body2">大小: <strong>{(bk.db_size / 1024).toFixed(1)} KB</strong> · 备份数: <strong>{bk.backup_count}</strong> · 上次: <strong>{bk.last_backup || '无'}</strong></Typography>
        {bk.tables && bk.tables.length > 0 && <Box sx={{ mt: 1, display: 'flex', gap: 1.5, flexWrap: 'wrap' }}>{bk.tables.map(t => <Chip key={t.table} label={`${t.table}: ${t.rows}条`} size="small" variant="outlined" sx={{ borderRadius: R, fontSize: '0.7rem' }} />)}</Box>}
      </Paper>}
      <Paper elevation={0} sx={{ p: 2, mb: 3, borderRadius: R, border: '1px solid rgba(0,0,0,0.06)' }}>
        <Typography variant="subtitle2" fontWeight={700} gutterBottom>自动备份设置</Typography>
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 2, flexWrap: 'wrap' }}>
          <FormControlLabel control={<Switch checked={bkAuto} onChange={async (e) => { const v = e.target.checked; setBkAuto(v); await updateBackupConfig({ enabled: v, interval_hours: bkInt }); sm('设置已保存'); }} />} label="启用自动备份" />
          <TextField label="间隔(小时)" type="number" size="small" value={bkInt} onChange={e => setBkInt(Number(e.target.value) || 1)} inputProps={{ min: 1 }} sx={{ width: 100, '& .MuiOutlinedInput-root': { borderRadius: R } }} />
          <Button size="small" variant="outlined" onClick={async () => { await updateBackupConfig({ enabled: bkAuto, interval_hours: bkInt }); sm('设置已保存'); }} sx={{ borderRadius: R }}>保存设置</Button>
        </Box>
        <Typography variant="caption" color="text.secondary" sx={{ mt: 1, display: 'block' }}>修改后需重启程序生效</Typography>
      </Paper>
      {bk && bk.backup_files.length > 0 && <Paper elevation={0} sx={{ p: 2, borderRadius: R, border: '1px solid rgba(0,0,0,0.06)' }}>
        <Typography variant="subtitle2" fontWeight={700} gutterBottom>备份文件列表 ({bk.backup_count} 个)</Typography>
        {bk.backup_files.map(f => <Box key={f.name} sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', py: 0.5, borderBottom: '1px solid rgba(0,0,0,0.04)' }}><Box><Typography variant="body2" sx={{ fontFamily: 'monospace', fontSize: '0.8rem' }}>{f.name}</Typography><Typography variant="caption" color="text.secondary">{(f.size / 1024).toFixed(1)} KB · {f.time || ''}</Typography></Box><IconButton size="small" color="error" onClick={async () => { if (!window.confirm('确定删除?')) return; try { const r = await deleteBackup(f.name); sm(r.message || '已删除'); await loadBk(); } catch { sm('删除失败', true); } }}><DeleteIcon fontSize="small" /></IconButton></Box>)}
      </Paper>}
    </Box>}

    {/* ── 对话框 ── */}
    {/* 项目对话框 (v0.2.17 简化) */}
    <Dialog open={pd} onClose={() => setPd(false)} maxWidth="sm" fullWidth PaperProps={{ sx: { borderRadius: R } }}>
      <DialogTitle sx={{ fontWeight: 700 }}>{ped ? '编辑项目' : '新建研发项目'}</DialogTitle>
      <DialogContent>
        <TextField label="项目名称" fullWidth value={pf.name} onChange={e => setPf({ ...pf, name: e.target.value })} sx={{ mt: 1, '& .MuiOutlinedInput-root': { borderRadius: R } }} />
        <Autocomplete multiple options={gs} getOptionLabel={(g: ProjectGroup) => g.name}
          value={gs.filter(g => (pf.lab_ids || []).includes(g.id))}
          onChange={(_, v) => setPf({ ...pf, lab_ids: v.map(g => g.id) })}
          renderInput={(params) => <TextField {...params} label="关联实验室" sx={{ mt: 2, '& .MuiOutlinedInput-root': { borderRadius: R } }} />}
          renderTags={(value, getTagProps) => value.map((option, index) => (
            <Chip label={option.name} size="small" {...getTagProps({ index })} sx={{ borderRadius: R }} />
          ))}
        />
        <Autocomplete multiple options={ml} getOptionLabel={(m: Method) => m.name}
          value={ml.filter(m => (pf.method_ids || []).includes(m.id))}
          onChange={(_, v) => setPf({ ...pf, method_ids: v.map(m => m.id) })}
          renderInput={(params) => <TextField {...params} label="关联方法" sx={{ mt: 2, '& .MuiOutlinedInput-root': { borderRadius: R } }} />}
          renderTags={(value, getTagProps) => value.map((option, index) => (
            <Chip label={option.name} size="small" {...getTagProps({ index })} sx={{ borderRadius: R }} />
          ))}
        />
        <TextField label="备注" fullWidth multiline minRows={2} maxRows={4} value={pf.notes} onChange={e => setPf({ ...pf, notes: e.target.value })} sx={{ mt: 2, '& .MuiOutlinedInput-root': { borderRadius: R } }} />
      </DialogContent>
      <DialogActions><Button onClick={() => setPd(false)} sx={{ borderRadius: R }}>取消</Button><Button onClick={hps} variant="contained" sx={{ borderRadius: R }}>保存</Button></DialogActions>
    </Dialog>

    {/* 分组对话框 */}
    <Dialog open={gd} onClose={() => setGd(false)} maxWidth="sm" fullWidth PaperProps={{ sx: { borderRadius: R } }}>
      <DialogTitle sx={{ fontWeight: 700 }}>{ged ? '编辑实验室' : '新建实验室'}</DialogTitle>
      <DialogContent>
        <TextField label="实验室名称" fullWidth value={gf.name} onChange={e => setGf({ ...gf, name: e.target.value })} sx={{ mt: 1, '& .MuiOutlinedInput-root': { borderRadius: R } }} />
        <TextField label="排序" type="number" fullWidth value={gf.sort_order} onChange={e => setGf({ ...gf, sort_order: Number(e.target.value) })} sx={{ mt: 2, '& .MuiOutlinedInput-root': { borderRadius: R } }} />
      </DialogContent>
      <DialogActions><Button onClick={() => setGd(false)} sx={{ borderRadius: R }}>取消</Button><Button onClick={hgs} variant="contained" sx={{ borderRadius: R }}>保存</Button></DialogActions>
    </Dialog>

    {/* 批量系数对话框 */}
    <Dialog open={bdo} onClose={() => setBdo(false)} maxWidth="xs" fullWidth PaperProps={{ sx: { borderRadius: R } }}>
      <DialogTitle sx={{ fontWeight: 700 }}>批量设置系数 — {bgn}</DialogTitle>
      <DialogContent>
        <TextField label="系数" type="number" fullWidth value={bcoeff} onChange={e => setBcoeff(Number(e.target.value) || 1.0)} inputProps={{ min: 0, step: 0.1 }} sx={{ mt: 1, '& .MuiOutlinedInput-root': { borderRadius: R } }} helperText="将更新此实验室下所有项目" />
      </DialogContent>
      <DialogActions><Button onClick={() => setBdo(false)} sx={{ borderRadius: R }}>取消</Button><Button onClick={hbc} variant="contained" color="secondary" sx={{ borderRadius: R }}>更新</Button></DialogActions>
    </Dialog>

    {/* 方法对话框 (v0.2.17 新增) */}
    <Dialog open={md} onClose={() => setMd(false)} maxWidth="sm" fullWidth PaperProps={{ sx: { borderRadius: R } }}>
      <DialogTitle sx={{ fontWeight: 700 }}>{med ? '编辑方法' : '新建方法'}</DialogTitle>
      <DialogContent>
        <TextField label="方法名称" fullWidth value={mf.name} onChange={e => setMf({ ...mf, name: e.target.value })} sx={{ mt: 1, '& .MuiOutlinedInput-root': { borderRadius: R } }} />
        <TextField label="全称" fullWidth value={mf.full_name} onChange={e => setMf({ ...mf, full_name: e.target.value })} sx={{ mt: 2, '& .MuiOutlinedInput-root': { borderRadius: R } }} />
        <TextField label="管理系数" type="number" fullWidth value={mf.coefficient} onChange={e => setMf({ ...mf, coefficient: Number(e.target.value) || 1.0 })} sx={{ mt: 2, '& .MuiOutlinedInput-root': { borderRadius: R } }} inputProps={{ min: 0, step: 0.1 }} />
        <TextField label="备注" fullWidth multiline minRows={2} value={mf.notes} onChange={e => setMf({ ...mf, notes: e.target.value })} sx={{ mt: 2, '& .MuiOutlinedInput-root': { borderRadius: R } }} />
        <Typography variant="body2" sx={{ mt: 2, mb: 0.5, fontWeight: 600 }}>类型归属</Typography>
        <FormGroup row>
          {mts.filter(t => !['检测类型','其他'].includes(t.name)).map(t => (
            <FormControlLabel key={t.id}
              control={<Checkbox checked={mf.type_ids.includes(t.id)} onChange={(e) => { if (e.target.checked) { setMf({ ...mf, type_ids: [...mf.type_ids, t.id] }); } else { setMf({ ...mf, type_ids: mf.type_ids.filter(id => id !== t.id) }); } }} />}
              label={t.name}
            />
          ))}
        </FormGroup>
      </DialogContent>
      <DialogActions><Button onClick={() => setMd(false)} sx={{ borderRadius: R }}>取消</Button><Button onClick={hms} variant="contained" sx={{ borderRadius: R }}>保存</Button></DialogActions>
    </Dialog>

    {/* 方法类型对话框 */}
    <Dialog open={mtd} onClose={() => setMtd(false)} maxWidth="xs" fullWidth PaperProps={{ sx: { borderRadius: R } }}>
      <DialogTitle sx={{ fontWeight: 700 }}>{mtf.id > 0 ? '编辑类型' : '新增类型'}</DialogTitle>
      <DialogContent>
        <TextField label="类型名称" fullWidth value={mtf.name} onChange={e => setMtf({ ...mtf, name: e.target.value })} sx={{ mt: 1, '& .MuiOutlinedInput-root': { borderRadius: R } }} helperText="如: 液相、气相、理化、检测类型等" />
        <TextField label="排序" type="number" fullWidth value={mtf.sort_order} onChange={e => setMtf({ ...mtf, sort_order: Number(e.target.value) || 10 })} sx={{ mt: 2, '& .MuiOutlinedInput-root': { borderRadius: R } }} />
      </DialogContent>
      <DialogActions><Button onClick={() => setMtd(false)} sx={{ borderRadius: R }}>取消</Button><Button onClick={hmt} variant="contained" sx={{ borderRadius: R }}>保存</Button></DialogActions>
    </Dialog>

    <ConfirmDialog open={co} title="确认操作" message="确定要执行此操作吗？" confirmText="确定" cancelText="取消" onConfirm={ca} onCancel={() => setCo(false)} />
  </Box>);
};

export default ManagePage;
