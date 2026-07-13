import React, { useState, useEffect, useCallback, useMemo } from 'react';
import {
  Box, Typography, Paper, Button, CircularProgress, TextField, FormControl, InputLabel, Select,
  MenuItem, Switch, FormControlLabel, IconButton, Chip, TableContainer,
  Table, TableHead, TableBody, TableRow, TableCell, Alert, Snackbar,
  Checkbox, FormGroup, useMediaQuery, useTheme, Autocomplete,
  Dialog, DialogTitle, DialogContent, DialogActions, Tooltip, Grid,
} from '@mui/material';
import AddIcon from '@mui/icons-material/Add';
import DeleteIcon from '@mui/icons-material/Delete';
import EditIcon from '@mui/icons-material/Edit';
import FolderIcon from '@mui/icons-material/Folder';
import ListAltIcon from '@mui/icons-material/ListAlt';
import ScienceIcon from '@mui/icons-material/Science';
import DeleteSweepIcon from '@mui/icons-material/DeleteSweep';
import ReceiptLongIcon from '@mui/icons-material/ReceiptLong';
import HistoryIcon from '@mui/icons-material/History';
import CloudUploadIcon from '@mui/icons-material/CloudUpload';
import BackupIcon from '@mui/icons-material/Backup';
import VisibilityIcon from '@mui/icons-material/Visibility';
import MenuBookIcon from '@mui/icons-material/MenuBook';
import { getGroups, createGroup, updateGroup, deleteGroup, getProjects, createProject, updateProject, deleteProject, getRecords, restoreRecord, getAuditLogs, batchProjectCoefficient, getBackupStatus, backupNow, getBackupConfig, updateBackupConfig, deleteBackup, restoreBackup, restoreBackupFile, getMethodTypes, createMethodType, updateMethodType, deleteMethodType, getMethods, createMethod, updateMethod, deleteMethod, methodImport, getImportMappings, getHelpDocuments, uploadHelpDocument, updateHelpDocument, deleteHelpDocument, getHelpDocumentFileUrl, getHelpArticles, deleteHelpArticle, updateHelpArticle } from '../api/client';
import type { ProjectGroup, Project, WorkRecord, AuditLog, BackupStatus, MethodType, Method, ImportMapping, HelpDocument, HelpArticle } from '../types';
import ConfirmDialog from '../components/ConfirmDialog';
import InlineEditCard from '../components/InlineEditCard';

type TV = 'projects' | 'groups' | 'methods' | 'trash' | 'audit' | 'backup' | 'help';

const R = '2px';
const cSx = { borderRadius: R, fontWeight: 700, border: '1px solid rgba(0,0,0,0.08)' };
const tSx = { borderRadius: R, border: '1px solid rgba(0,0,0,0.06)', overflow: 'auto' };
const AL: Record<string, string> = { create: '创建', update: '更新', delete: '删除', restore: '恢复', import: '导入' };
const AC: Record<string, 'success' | 'error' | 'info' | 'warning' | 'default'> = { create: 'success', update: 'info', delete: 'error', restore: 'warning', import: 'warning' };
const TL: Record<string, string> = { work_records: '工作记录', rd_work_records: '研发送样记录', projects: '项目', project_groups: '分组', samples: '送样记录(已退役)' };

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

const TC = [
  { key: 'projects', label: '研发项目管理', icon: <ListAltIcon />, desc: '研发项目及关联实验室' },
  { key: 'groups', label: '实验室管理', icon: <FolderIcon />, desc: '新增编辑实验室映射录入选项卡' },
  { key: 'methods', label: '检测方法管理', icon: <ScienceIcon />, desc: '液相/气相/理化/ICP/热分析等检测方法' },
  { key: 'trash', label: '回收站', icon: <DeleteSweepIcon />, desc: '恢复已删除的记录' },
  { key: 'audit', label: '审计日志', icon: <ReceiptLongIcon />, desc: '操作记录追溯' },
  { key: 'backup', label: '数据备份', icon: <BackupIcon />, desc: '备份恢复与自动备份设置' },
  { key: 'help', label: '教程与帮助', icon: <MenuBookIcon />, desc: '上传编辑帮助文档，管理显隐' },
] as { key: TV; label: string; icon: React.ReactNode; desc: string }[];

const ManagePage: React.FC = () => {
  const [tb, setTb] = useState<TV>('projects');
  const [ld, setLd] = useState(false); const [msg, setMsg] = useState(''); const [err, setErr] = useState(false);
  const sm = useCallback((m: string, isErr?: boolean) => { setMsg(m); setErr(!!isErr); }, []);
  const theme = useTheme();
  const isMobile = useMediaQuery(theme.breakpoints.down('sm'));
  const isTablet = useMediaQuery(theme.breakpoints.down('md'));
  const [co, setCo] = useState(false); const [ca, setCa] = useState<() => Promise<void>>(() => async () => { setCo(false); });

  // v0.3.18: 编辑弹窗状态
  const [projectEditOpen, setProjectEditOpen] = useState(false);
  const [projectEditItem, setProjectEditItem] = useState<Project | null>(null);
  const [methodEditOpen, setMethodEditOpen] = useState(false);
  const [methodEditItem, setMethodEditItem] = useState<Method | null>(null);
  const [groupEditOpen, setGroupEditOpen] = useState(false);
  const [groupEditItem, setGroupEditItem] = useState<ProjectGroup | null>(null);

  // v0.3.18: 方法一览弹窗状态
  const [methodOverviewOpen, setMethodOverviewOpen] = useState(false);
  const [methodOverviewData, setMethodOverviewData] = useState<Method[]>([]);
  const [methodOverviewEditingId, setMethodOverviewEditingId] = useState<number | null>(null);
  const [methodOverviewEditData, setMethodOverviewEditData] = useState<Partial<Method>>({});

  // v0.3.23: 项目一览弹窗状态
  const [projectOverviewOpen, setProjectOverviewOpen] = useState(false);
  const [projectOverviewData, setProjectOverviewData] = useState<Project[]>([]);

  // v0.3.23: 实验室一览弹窗状态
  const [groupOverviewOpen, setGroupOverviewOpen] = useState(false);
  const [groupOverviewData, setGroupOverviewData] = useState<ProjectGroup[]>([]);

  // groups
  const [gs, setGs] = useState<ProjectGroup[]>([]);
  const lg = useCallback(async () => { try { const r = await getGroups(); if (r.code === 0 && r.data) setGs(r.data); } catch {} }, []);

  // projects (v0.2.17 simplified)
  const [ps, setPs] = useState<Project[]>([]);
  const lp = useCallback(async () => { try { const r = await getProjects(); if (r.code === 0 && r.data) setPs(r.data); } catch {} }, []);

  // 项目编辑 - 方法类型筛选
  const [projectMethodTypeFilter, setProjectMethodTypeFilter] = useState('');
  const [ml, setMl] = useState<Method[]>([]);
  const [methodFilter, setMethodFilter] = useState('');
  const [importMappingOpen, setImportMappingOpen] = useState(false);
  const [importMappings, setImportMappings] = useState<ImportMapping[]>([]);
  const lm = useCallback(async () => { try { const r = await getMethods(); if (r.code === 0 && r.data) setMl(r.data); } catch {} }, []);

  // v0.3.0: 打开导入映射预览对话框
  const handleOpenImport = async () => {
    try {
      const r = await getImportMappings();
      if (r.code === 0 && r.data) { setImportMappings(r.data); }
    } catch { setImportMappings([]); }
    setImportMappingOpen(true);
  };

  // method types
  const [mts, setMts] = useState<MethodType[]>([]);
  const [mtd, setMtd] = useState(false); const [mtf, setMtf] = useState({ id: 0, name: '', sort_order: 10 });
  const lmt = useCallback(async () => { try { const r = await getMethodTypes(); if (r.code === 0 && r.data) setMts(r.data); } catch {} }, []);
  const hmt = async () => { if (!mtf.name.trim()) { sm('请输入类型名称', true); return; } try { if (mtf.id > 0) { const r = await updateMethodType(mtf.id, { name: mtf.name, sort_order: mtf.sort_order }); if (r.code === 0) { sm('更新成功'); lmt(); setMtd(false); } else sm(r.message, true); } else { const r = await createMethodType({ name: mtf.name, sort_order: mtf.sort_order }); if (r.code === 0) { sm('创建成功'); lmt(); setMtd(false); } else sm(r.message, true); } } catch { sm('操作失败', true); } };

  // trash
  const [rs, setRs] = useState<WorkRecord[]>([]); const [rc, setRc] = useState(0);
  const lt = useCallback(async () => { try { const r = await getRecords({}); if (r.code === 0 && r.data) { setRs(r.data.items.filter((x: any) => false)); setRc(0); } try { const all = await getAuditLogs({ page_size: 3 }); if (all.code === 0 && all.data) setRc(all.data.total || 0); } catch {} } catch {} }, []);
  const [tr, setTr] = useState<WorkRecord[]>([]);
  const loadTrash = async () => { try { const r = await getRecords({ include_deleted: true }); if (r.code === 0 && r.data) setTr(r.data.items.filter((x: any) => x.deleted_at != null)); } catch {} };

  // audit
  const [al, setAl] = useState<AuditLog[]>([]); const [at, setAt] = useState(0); const [ap, setAp] = useState(1);
  const la = useCallback(async (p: number) => { try { const r = await getAuditLogs({ page: p, page_size: 50 }); if (r.code === 0 && r.data) { setAl(r.data.items); setAt(r.data.total); setAp(p); } } catch {} }, []);

  // backup
  const [bk, setBk] = useState<BackupStatus | null>(null); const [bkAuto, setBkAuto] = useState(false);
  const [bkInt, setBkInt] = useState(24); const [maxBk, setMaxBk] = useState(10); const [bkR, setBkR] = useState(''); const [bkN, setBkN] = useState(false);
  const loadBk = async () => { try { const r = await getBackupStatus(); if (r.code === 0 && r.data) { setBk(r.data); setBkAuto(r.data.auto_enabled); setBkInt(r.data.auto_interval_hours); setMaxBk(r.data.max_backup_count || 10); } } catch {} };

  // help documents
  const [helpDocs, setHelpDocs] = useState<HelpDocument[]>([]);
  const [helpTitle, setHelpTitle] = useState('');
  const [helpFile, setHelpFile] = useState<File | null>(null);
  const [helpEditId, setHelpEditId] = useState<number | null>(null);
  const [helpEditTitle, setHelpEditTitle] = useState('');
  const loadHelpDocs = async () => { try { const r = await getHelpDocuments(false); if (r.code === 0 && r.data) setHelpDocs(r.data); } catch {} };

  // help articles
  const [helpArticles, setHelpArticles] = useState<HelpArticle[]>([]);
  const loadHelpArticles = async () => { try { const r = await getHelpArticles(false); if (r.code === 0 && r.data) setHelpArticles(r.data); } catch {} };

  // v0.3.15: 初始加载时也加载方法类型，否则项目编辑对话框的"关联检测方法"按类型分组时 mts 为空
  useEffect(() => { setLd(true); Promise.all([lg(), lp(), lt(), lm(), lmt()]).finally(() => setLd(false)); }, [lg, lp, lt, lm, lmt]);
  useEffect(() => { if (tb === 'audit') la(1); if (tb === 'backup') loadBk(); if (tb === 'methods') { lmt(); lm(); } if (tb === 'trash') loadTrash(); if (tb === 'help') { loadHelpDocs(); loadHelpArticles(); } }, [tb, la, lmt, lm]);

  // v0.3.18: 项目保存
  const handleSaveProject = async (project: Project) => {
    const body: any = {
      name: project.name,
      full_name: project.full_name,
      notes: project.notes,
      sort_order: project.sort_order,
      is_active: project.is_active,
      lab_ids: project.lab_ids,
      method_ids: project.method_ids,
    };
    if (project.id > 0) {
      const r = await updateProject(project.id, body);
      if (r.code === 0) { sm('更新成功'); lp(); lg(); setProjectEditOpen(false); setProjectEditItem(null); } else sm(r.message, true);
    } else {
      const r = await createProject(body);
      if (r.code === 0) { sm('创建成功'); lp(); lg(); setProjectEditOpen(false); setProjectEditItem(null); } else sm(r.message, true);
    }
  };

  // v0.3.18: 方法保存
  const handleSaveMethod = async (method: Method) => {
    const body: any = {
      name: method.name,
      full_name: method.full_name,
      coefficient: method.coefficient,
      multiplier: method.multiplier ?? 1.0,
      amount: method.amount,
      notes: method.notes,
      type_ids: method.type_ids,
    };
    if (method.id > 0) {
      const r = await updateMethod(method.id, body);
      if (r.code === 0) { sm('更新成功'); lm(); setMethodEditOpen(false); setMethodEditItem(null); } else sm(r.message, true);
    } else {
      const r = await createMethod(body);
      if (r.code === 0) { sm('创建成功'); lm(); setMethodEditOpen(false); setMethodEditItem(null); } else sm(r.message, true);
    }
  };

  // v0.3.18: 实验室保存
    const handleSaveGroup = async (group: ProjectGroup) => {
    if (group.id > 0) {
      const r = await updateGroup(group.id, { name: group.name, sort_order: group.sort_order, show_in_work: group.show_in_work, show_in_rd: group.show_in_rd });
      if (r.code === 0) { sm('更新成功'); lg(); setGroupEditOpen(false); setGroupEditItem(null); } else sm(r.message, true);
    } else {
      const r = await createGroup({ name: group.name, sort_order: group.sort_order, show_in_work: group.show_in_work, show_in_rd: group.show_in_rd });
      if (r.code === 0) { sm('创建成功'); lg(); setGroupEditOpen(false); setGroupEditItem(null); } else sm(r.message, true);
    }
  };

  // v0.3.18: 打开方法一览弹窗
  const handleOpenMethodOverview = () => {
    setMethodOverviewData([...ml]);
    setMethodOverviewOpen(true);
    setMethodOverviewEditingId(null);
    setMethodOverviewEditData({});
  };

  // v0.3.18: 方法一览表格行内编辑保存
  const handleSaveMethodOverview = async () => {
    if (methodOverviewEditingId === null) return;
    const method = methodOverviewData.find(m => m.id === methodOverviewEditingId);
    if (!method) return;

    const body: any = {
      name: methodOverviewEditData.name ?? method.name,
      full_name: methodOverviewEditData.full_name ?? method.full_name,
      coefficient: methodOverviewEditData.coefficient ?? method.coefficient,
      multiplier: methodOverviewEditData.multiplier ?? method.multiplier ?? 1.0,
      amount: methodOverviewEditData.amount ?? method.amount,
      notes: methodOverviewEditData.notes ?? method.notes,
      type_ids: methodOverviewEditData.type_ids ?? method.type_ids,
    };

    try {
      const r = await updateMethod(methodOverviewEditingId, body);
      if (r.code === 0) {
        sm('更新成功');
        lm();
        setMethodOverviewEditingId(null);
        setMethodOverviewEditData({});
        setMethodOverviewData([...ml]);
      } else {
        sm(r.message, true);
      }
    } catch {
      sm('操作失败', true);
    }
  };

  if (ld) return <Box sx={{ display: 'flex', justifyContent: 'center', mt: 8 }}><CircularProgress /></Box>;

  return (<Box>
    <Typography variant="h5" fontWeight={700} sx={{ mb: 3, px: 1, background: 'linear-gradient(135deg,#f4511e,#e53935)', WebkitBackgroundClip:'text', WebkitTextFillColor:'transparent', backgroundClip:'text' }}>系统管理</Typography>
    {msg && <Alert severity={err ? 'error' : 'success'} sx={{ mb: 2, borderRadius: R }} onClose={() => setMsg('')}>{msg}</Alert>}

    {/* 卡片网格 */}
    <Box sx={{ display: 'grid', gridTemplateColumns: { xs: '1fr 1fr 1fr', sm: '1fr 1fr 1fr', md: '1fr 1fr 1fr' }, gap: 1.5, mb: 3 }}>
      {TC.map(c => (
        <Paper key={c.key} elevation={0} onClick={() => setTb(c.key)} sx={{ p: isMobile ? 1.5 : 2.5, borderRadius: R, cursor: 'pointer', border: '2px solid', borderColor: tb === c.key ? '#f4511e' : 'rgba(0,0,0,0.06)', transition: 'all 0.2s', '&:hover': { borderColor: '#f4511e', boxShadow: '0 4px 24px rgba(244,81,30,0.12)' } }}>
          <Box sx={{ display: 'flex', flexDirection: isMobile ? 'column' : 'row', alignItems: isMobile ? 'center' : 'center', gap: isMobile ? 0.5 : 1.5, mb: 0.5, textAlign: isMobile ? 'center' : 'left' }}>
            <Box sx={{ color: tb === c.key ? '#f4511e' : 'text.secondary' }}>{c.icon}</Box>
            <Typography variant={isMobile ? 'caption' : 'subtitle1'} fontWeight={700}>{c.label}</Typography>
          </Box>
          {!isMobile && <Typography variant="caption" color="text.secondary">{c.desc}</Typography>}
        </Paper>
      ))}
    </Box>

    {/* ── 1. 研发项目管理 (v0.2.17 简化) ── */}
    {tb === 'projects' && <Box>
      <Box sx={{ display: 'flex', gap: 1, mb: 2, flexWrap: 'wrap', justifyContent: 'flex-end' }}>
        <Button variant="contained" startIcon={<AddIcon />} onClick={() => {
          setProjectEditItem({ id: 0, name: '', full_name: '', notes: '', sort_order: 0, is_active: true, lab_ids: [], method_ids: [], lab_names: [], method_names: [], type_names: [], created_at: '', updated_at: '' } as Project);
          setProjectEditOpen(true);
        }} size="small" sx={{ borderRadius: R, background: 'linear-gradient(135deg,#f4511e,#e53935)', boxShadow: '0 4px 14px rgba(244,81,30,0.3)' }}>新建研发项目</Button>
        <Button variant="outlined" startIcon={<VisibilityIcon />} size="small"
          onClick={() => {
            setProjectOverviewData([...ps]);
            setProjectOverviewOpen(true);
          }}
          sx={{ borderRadius: R, borderColor: '#1976d2', color: '#1976d2' }}
        >
          项目一览
        </Button>
      </Box>
      {ps.length === 0 ? <Typography color="text.secondary" textAlign="center" sx={{ py: 4 }}>暂无研发项目</Typography>
        : ps.map(p => (
            <InlineEditCard<Project>
              key={p.id}
              item={p}
              isExpanded={false}
              onToggle={() => {}}
              renderView={(item) => (
                <Box>
                  <Typography variant="subtitle1" fontWeight={600}>{item.name}</Typography>
                  <Typography variant="caption" color="text.secondary">
                    简称: {item.name} · 全称: {item.full_name || '—'}
                    {item.is_active === false && <Chip label="已停用" size="small" color="error" sx={{ ml: 0.5, borderRadius: R, height: 20, fontSize: '0.65rem' }} />}
                  </Typography>
                </Box>
              )}
              renderEdit={() => <></>}
              onSave={async () => {}}
              onDelete={async () => {
                setCa(() => async () => {
                  const r = await deleteProject(p.id);
                  if (r.code === 0) { sm('删除成功'); lp(); } else sm(r.message, true);
                  setCo(false);
                });
                setCo(true);
              }}
            >
              {/* 操作按钮 */}
              <Box sx={{ display: 'flex', gap: 0.5, mt: 0.5 }}>
                <IconButton size="small" onClick={() => { setProjectEditItem(p); setProjectEditOpen(true); }} sx={{ color: '#f4511e' }}>
                  <EditIcon fontSize="small" />
                </IconButton>
                <IconButton size="small" color="error" onClick={() => {
                  setCa(() => async () => {
                    const r = await deleteProject(p.id);
                    if (r.code === 0) { sm('删除成功'); lp(); } else sm(r.message, true);
                    setCo(false);
                  });
                  setCo(true);
                }}>
                  <DeleteIcon fontSize="small" />
                </IconButton>
              </Box>
              {/* 关联标签展示 */}
              <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 0.5, mt: 0.5 }}>
                {(p.lab_names || []).map(n => (
                  <Chip
                    key={n}
                    label={n}
                    size="small"
                    onDelete={() => {
                      const group = gs.find(g => g.name === n);
                      if (group) {
                        const newLabIds = (p.lab_ids || []).filter(id => id !== group.id);
                        updateProject(p.id, { ...p, lab_ids: newLabIds }).then(r => {
                          if (r.code === 0) { lp(); }
                        });
                      }
                    }}
                    sx={{ borderRadius: '10px', height: 20, fontSize: '0.7rem', backgroundColor: 'var(--color-background-info, #e3f2fd)' }}
                  />
                ))}
                {(p.method_names || []).map(n => (
                  <Chip
                    key={n}
                    label={n}
                    size="small"
                    color="info"
                    onDelete={() => {
                      const method = ml.find(m => m.name === n);
                      if (method) {
                        const newMethodIds = (p.method_ids || []).filter(id => id !== method.id);
                        updateProject(p.id, { ...p, method_ids: newMethodIds }).then(r => {
                          if (r.code === 0) { lp(); }
                        });
                      }
                    }}
                    sx={{ borderRadius: '10px', height: 20, fontSize: '0.7rem' }}
                  />
                ))}
              </Box>
            </InlineEditCard>
          ))}
    </Box>}

    {/* ── 2. 实验室管理 ── */}
    {tb === 'groups' && <Box>
      <Box sx={{ display: 'flex', justifyContent: 'flex-end', mb: 2, gap: 1 }}>
        <Button variant="contained" startIcon={<AddIcon />} onClick={() => {
          setGroupEditItem({ id: 0, name: '', sort_order: 0, created_at: '', updated_at: '' } as ProjectGroup);
          setGroupEditOpen(true);
        }} size="small" sx={{ borderRadius: R, background: 'linear-gradient(135deg,#f4511e,#e53935)', boxShadow: '0 4px 14px rgba(244,81,30,0.3)' }}>新建实验室</Button>
        <Button variant="outlined" startIcon={<VisibilityIcon />} size="small"
          onClick={() => {
            setGroupOverviewData([...gs]);
            setGroupOverviewOpen(true);
          }}
          sx={{ borderRadius: R, borderColor: '#1976d2', color: '#1976d2' }}>
          实验室一览
        </Button>
      </Box>
      <Typography variant="caption" color="text.secondary" sx={{ mb: 1, display: 'block' }}>实验室分组直接映射为工作量录入界面的选项卡</Typography>
      {gs.length === 0 ? <Typography color="text.secondary" textAlign="center" sx={{ py: 4 }}>暂无实验室分组</Typography>
        : gs.map(g => (
            <InlineEditCard<ProjectGroup>
              key={g.id}
              item={g}
              isExpanded={false}
              onToggle={() => {}}
              renderView={(item) => (
                <Box>
                  <Typography variant="subtitle1" fontWeight={600}>{item.name}</Typography>
                  <Typography variant="caption" color="text.secondary">排序: {item.sort_order}</Typography>
                </Box>
              )}
              renderEdit={() => <></>}
              onSave={async () => {}}
              onDelete={async () => {
                setCa(() => async () => {
                  const r = await deleteGroup(g.id);
                  if (r.code === 0) { sm('删除成功'); lg(); } else sm(r.message, true);
                  setCo(false);
                });
                setCo(true);
              }}
            >
              {/* 操作按钮 */}
              <Box sx={{ display: 'flex', gap: 0.5, mt: 0.5 }}>
                <IconButton size="small" onClick={() => { setGroupEditItem(g); setGroupEditOpen(true); }} sx={{ color: '#f4511e' }}>
                  <EditIcon fontSize="small" />
                </IconButton>
                <IconButton size="small" color="error" onClick={() => {
                  setCa(() => async () => {
                    const r = await deleteGroup(g.id);
                    if (r.code === 0) { sm('删除成功'); lg(); } else sm(r.message, true);
                    setCo(false);
                  });
                  setCo(true);
                }}>
                  <DeleteIcon fontSize="small" />
                </IconButton>
              </Box>
            </InlineEditCard>
          ))}
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
          <Button variant="outlined" startIcon={<VisibilityIcon />} size="small"
            onClick={handleOpenMethodOverview}
            sx={{ borderRadius: R, borderColor: '#1976d2', color: '#1976d2' }}>
            方法一览
          </Button>
          <FormControl size="small" sx={{ minWidth: 120, '& .MuiOutlinedInput-root': { borderRadius: R } }}>
            <InputLabel>类型筛选</InputLabel>
            <Select
              value={methodFilter}
              label="类型筛选"
              onChange={e => setMethodFilter(e.target.value)}
            >
              <MenuItem value="">全部</MenuItem>
              {mts.filter(t => t.name !== '检测类型').map(t => (
                <MenuItem key={t.id} value={t.name}>{t.name}</MenuItem>
              ))}
            </Select>
          </FormControl>
          <Button variant="outlined" startIcon={<CloudUploadIcon />} size="small"
            onClick={handleOpenImport}
            sx={{ borderRadius: R, borderColor: '#00897b', color: '#00897b', width: isMobile ? '100%' : undefined }}>
            导入方法
          </Button>
          <Button variant="contained" startIcon={<AddIcon />} onClick={() => {
            setMethodEditItem({ id: 0, name: '', full_name: '', coefficient: 1.0, multiplier: 1.0, amount: 0, notes: '', type_ids: [] as number[], type_names: [] as string[], is_active: true, created_at: '', updated_at: '' } as Method);
            setMethodEditOpen(true);
          }} size="small" sx={{ borderRadius: R, background: 'linear-gradient(135deg,#f4511e,#e53935)', boxShadow: '0 4px 14px rgba(244,81,30,0.3)' }}>新建方法</Button>
        </Box>
      </Box>
      {ml.filter(m => !methodFilter || m.type_names.includes(methodFilter)).length === 0 ? <Typography color="text.secondary" textAlign="center" sx={{ py: 4 }}>暂无检测方法数据，请先导入方法或手动创建</Typography>
        : ml.filter(m => !methodFilter || m.type_names.includes(methodFilter)).map(m => (
            <InlineEditCard<Method>
              key={m.id}
              item={m}
              isExpanded={false}
              onToggle={() => {}}
              renderView={(item) => (
                <Box>
                  <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, flexWrap: 'wrap' }}>
                    <Typography variant="subtitle1" fontWeight={600}>{item.name}</Typography>
                    {/* 仪器标签 */}
                    {extractInstrumentFromMethodName(item.name) && (
                      <Chip
                        label={`仪器: ${extractInstrumentFromMethodName(item.name)}`}
                        size="small"
                        sx={{
                          borderRadius: R,
                          fontSize: '0.7rem',
                          bgcolor: '#00897b',
                          color: '#fff',
                          fontWeight: 600
                        }}
                      />
                    )}
                  </Box>
                  <Typography variant="caption" color="text.secondary">
                    类型: {item.type_names.length > 0 ? item.type_names.map(t => <Chip key={t} label={t} size="small" variant="outlined" sx={{ borderRadius: R, height: 20, fontSize: '0.7rem', mr: 0.5 }} />) : <Chip label="未分类" size="small" sx={{ borderRadius: R, height: 20, fontSize: '0.7rem' }} />}
                    系数: <Chip label={(item.coefficient ?? 1).toFixed(1)} size="small" variant="outlined" sx={{ borderRadius: R, height: 20, fontSize: '0.7rem', ml: 0.5 }} />
                    单价倍率: <Chip label={(item.multiplier ?? 1.0).toFixed(1)} size="small" variant="outlined" color="secondary" sx={{ borderRadius: R, height: 20, fontSize: '0.7rem', ml: 0.5 }} />
                    单价: <Chip label={((item.amount ?? 0).toFixed(2))} size="small" variant="outlined" color="primary" sx={{ borderRadius: R, height: 20, fontSize: '0.7rem', ml: 0.5 }} />
                  </Typography>
                </Box>
              )}
              renderEdit={() => <></>}
              onSave={async () => {}}
              onDelete={async () => {
                setCa(() => async () => {
                  const r = await deleteMethod(m.id);
                  if (r.code === 0) { sm('删除成功'); lm(); } else sm(r.message, true);
                  setCo(false);
                });
                setCo(true);
              }}
            >
              {/* 操作按钮 */}
              <Box sx={{ display: 'flex', gap: 0.5, mt: 0.5 }}>
                <IconButton size="small" onClick={() => { setMethodEditItem(m); setMethodEditOpen(true); }} sx={{ color: '#f4511e' }}>
                  <EditIcon fontSize="small" />
                </IconButton>
                <IconButton size="small" color="error" onClick={() => {
                  setCa(() => async () => {
                    const r = await deleteMethod(m.id);
                    if (r.code === 0) { sm('删除成功'); lm(); } else sm(r.message, true);
                    setCo(false);
                  });
                  setCo(true);
                }}>
                  <DeleteIcon fontSize="small" />
                </IconButton>
              </Box>
              {/* 关联类型标签展示 */}
              <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 0.5, mt: 0.5 }}>
                {(m.type_names || []).map(n => (
                  <Chip
                    key={n}
                    label={n}
                    size="small"
                    onDelete={() => {
                      const type = mts.find(t => t.name === n);
                      if (type) {
                        const newTypeIds = (m.type_ids || []).filter(id => id !== type.id);
                        updateMethod(m.id, { ...m, type_ids: newTypeIds }).then(r => {
                          if (r.code === 0) { lm(); }
                        });
                      }
                    }}
                    sx={{ borderRadius: '10px', height: 20, fontSize: '0.7rem', backgroundColor: 'var(--color-background-info, #e3f2fd)' }}
                  />
                ))}
              </Box>
            </InlineEditCard>
          ))}
    </Box>}

    {/* ── 回收站 ── */}
    {tb === 'trash' && <Box>
      <Typography variant="body2" color="text.secondary" sx={{ mb: 2, display: 'flex', alignItems: 'center', gap: 1 }}><DeleteSweepIcon fontSize="small" />共 {tr.length} 条已删除记录</Typography>
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
          <TableCell sx={{ fontWeight: 600 }}>时间</TableCell><TableCell sx={{ fontWeight: 600 }}>操作类型</TableCell><TableCell sx={{ fontWeight: 600 }}>操作对象</TableCell><TableCell sx={{ fontWeight: 600 }}>记录ID</TableCell><TableCell sx={{ fontWeight: 600 }}>操作人</TableCell><TableCell sx={{ fontWeight: 600 }}>详情</TableCell>
        </TableRow></TableHead><TableBody>{al.map(l => <TableRow key={l.id} hover><TableCell sx={{ whiteSpace: 'nowrap' }}>{l.created_at}</TableCell><TableCell><Chip label={AL[l.action] || l.action} size="small" color={AC[l.action] || 'default'} variant="outlined" sx={{ borderRadius: R }} /></TableCell><TableCell>{TL[l.table_name] || l.table_name}</TableCell><TableCell>{l.record_id}</TableCell><TableCell>{l.user_name}</TableCell><TableCell sx={{ maxWidth: 300 }}><Tooltip title={l.detail || ''} arrow placement="top"><Typography variant="body2" sx={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{l.detail}</Typography></Tooltip></TableCell></TableRow>)}</TableBody></Table></TableContainer>
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
        {bk.tables && bk.tables.length > 0 && <Box sx={{ mt: 1, display: 'flex', gap: 1.5, flexWrap: 'wrap' }}>{bk.tables.map(t => <Chip key={t.table} label={`${t.label || t.table}: ${t.rows}条`} size="small" variant="outlined" sx={{ borderRadius: R, fontSize: '0.7rem' }} />)}</Box>}
        <Typography variant="caption" color="text.secondary" sx={{ mt: 0.5, display: 'block' }}>各数据表当前记录条数（已排除删除数据）</Typography>
        <Typography variant="body2" sx={{ fontFamily: 'monospace', fontSize: '0.72rem', mt: 1, color: 'text.disabled', wordBreak: 'break-all' }}>备份路径: {bk.backups_dir}</Typography>
      </Paper>}
      <Paper elevation={0} sx={{ p: 2, mb: 3, borderRadius: R, border: '1px solid rgba(0,0,0,0.06)' }}>
        <Typography variant="subtitle2" fontWeight={700} gutterBottom>自动备份设置</Typography>
        <Typography variant="caption" color="text.secondary" sx={{ mt: 0.5, display: 'block' }}>开启后系统将按设定间隔自动生成数据库备份文件</Typography>
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 2, flexWrap: 'wrap' }}>
          <FormControlLabel control={<Switch checked={bkAuto} onChange={async (e) => { const v = e.target.checked; setBkAuto(v); await updateBackupConfig({ enabled: v, interval_hours: bkInt }); sm('设置已保存'); }} />} label="启用自动备份" />
          <TextField label="间隔(小时)" type="number" size="small" value={bkInt} onChange={e => setBkInt(Number(e.target.value) || 1)} inputProps={{ min: 1 }} sx={{ width: 100, '& .MuiOutlinedInput-root': { borderRadius: R } }} />
          <TextField label="最大备份数" type="number" size="small" value={maxBk} onChange={e => setMaxBk(Number(e.target.value) || 1)} inputProps={{ min: 1, max: 50 }} sx={{ width: 100, '& .MuiOutlinedInput-root': { borderRadius: R } }} />
          <Button size="small" variant="outlined" onClick={async () => { await updateBackupConfig({ enabled: bkAuto, interval_hours: bkInt, max_backup_count: maxBk }); sm('设置已保存'); }} sx={{ borderRadius: R }}>保存设置</Button>
        </Box>
        <Typography variant="caption" color="text.secondary" sx={{ mt: 1, display: 'block' }}>修改后需重启程序生效</Typography>
      </Paper>
      {bk && bk.backup_files.length > 0 && <Paper elevation={0} sx={{ p: 2, borderRadius: R, border: '1px solid rgba(0,0,0,0.06)' }}>
        <Typography variant="subtitle2" fontWeight={700} gutterBottom>备份文件列表 ({bk.backup_count} 个)</Typography>
        <Typography variant="caption" color="text.secondary" sx={{ mt: 0.5, display: 'block' }}>可下载备份文件用于数据迁移，或恢复到指定历史版本</Typography>
        {bk.backup_files.map(f => <Box key={f.name} sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', py: 0.5, borderBottom: '1px solid rgba(0,0,0,0.04)' }}><Box><Typography variant="body2" sx={{ fontFamily: 'monospace', fontSize: '0.8rem' }}>{f.name}</Typography><Typography variant="caption" color="text.secondary">{(f.size / 1024).toFixed(1)} KB · {f.time || ''}</Typography></Box><Box sx={{ display: 'flex', gap: 0.5 }}><Button size="small" variant="outlined" sx={{ borderRadius: R, fontSize: '0.7rem', py: 0, minWidth: 'auto' }} onClick={async () => { if (!window.confirm(`恢复备份「${f.name}」将替换当前数据库，确定继续？`)) return; try { const r = await restoreBackupFile(f.name); sm(r.message || '恢复成功'); await loadBk(); } catch { sm('恢复失败', true); } }}>恢复</Button><IconButton size="small" color="error" onClick={async () => { if (!window.confirm('确定删除?')) return; try { const r = await deleteBackup(f.name); sm(r.message || '已删除'); await loadBk(); } catch { sm('删除失败', true); } }}><DeleteIcon fontSize="small" /></IconButton></Box></Box>)}
      </Paper>}
    </Box>}

    {/* ── 教程与帮助 ── */}
    {tb === 'help' && <Box>
      {/* 上传区域 */}
      <Paper elevation={0} sx={{ p: 2, mb: 3, borderRadius: R, border: '1px solid rgba(0,0,0,0.06)' }}>
        <Typography variant="subtitle2" fontWeight={700} gutterBottom>上传文档</Typography>
        <Box sx={{ display: 'flex', gap: 1.5, flexWrap: 'wrap', alignItems: 'center', mt: 1 }}>
          <TextField
            label="文档标题"
            size="small"
            value={helpTitle}
            onChange={e => setHelpTitle(e.target.value)}
            sx={{ width: isMobile ? '100%' : 240, '& .MuiOutlinedInput-root': { borderRadius: R } }}
          />
          <Button variant="outlined" component="label" size="small" sx={{ borderRadius: R }}>
            {helpFile ? helpFile.name : '选择文件'}
            <input type="file" hidden onChange={e => { if (e.target.files?.[0]) setHelpFile(e.target.files[0]); }} />
          </Button>
          <Button
            variant="contained" size="small"
            disabled={!helpFile}
            onClick={async () => {
              if (!helpFile) return;
              const fd = new FormData();
              fd.append('file', helpFile);
              fd.append('title', helpTitle || helpFile.name);
              try {
                const r = await uploadHelpDocument(fd);
                if (r.code === 0) { sm('上传成功'); setHelpTitle(''); setHelpFile(null); loadHelpDocs(); }
                else sm(r.message, true);
              } catch (e: any) { sm('上传失败: ' + (e?.message || '未知错误'), true); }
            }}
            sx={{ borderRadius: R, background: 'linear-gradient(135deg,#667eea,#764ba2)' }}
          >
            上传
          </Button>
        </Box>
      </Paper>

      {/* 文档列表 */}
      <Typography variant="subtitle2" fontWeight={700} sx={{ mb: 1.5 }}>
        文档列表 · 共 {helpDocs.length} 篇
      </Typography>
      {helpDocs.length === 0 ? (
        <Typography color="text.secondary" textAlign="center" sx={{ py: 4 }}>暂无文档</Typography>
      ) : (
        <TableContainer component={Paper} sx={tSx}>
          <Table size="small">
            <TableHead>
              <TableRow>
                <TableCell sx={{ fontWeight: 600 }}>标题</TableCell>
                <TableCell sx={{ fontWeight: 600 }}>类型</TableCell>
                <TableCell sx={{ fontWeight: 600 }}>大小</TableCell>
                <TableCell sx={{ fontWeight: 600 }}>显隐</TableCell>
                <TableCell sx={{ fontWeight: 600 }} align="right">操作</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {helpDocs.map(doc => (
                <TableRow key={doc.id} hover>
                  <TableCell sx={{ maxWidth: 280 }}>
                    {helpEditId === doc.id ? (
                      <Box sx={{ display: 'flex', gap: 0.5 }}>
                        <TextField
                          size="small"
                          value={helpEditTitle}
                          onChange={e => setHelpEditTitle(e.target.value)}
                          sx={{ '& .MuiOutlinedInput-root': { borderRadius: R, fontSize: '0.8rem' } }}
                        />
                        <Button
                          size="small" variant="contained"
                          onClick={async () => {
                            try {
                              const r = await updateHelpDocument(doc.id, { title: helpEditTitle });
                              if (r.code === 0) { sm('更新成功'); setHelpEditId(null); loadHelpDocs(); }
                              else sm(r.message, true);
                            } catch { sm('更新失败', true); }
                          }}
                          sx={{ borderRadius: R, fontSize: '0.7rem', py: 0 }}
                        >
                          保存
                        </Button>
                        <Button size="small" onClick={() => setHelpEditId(null)} sx={{ borderRadius: R, fontSize: '0.7rem', py: 0 }}>取消</Button>
                      </Box>
                    ) : (
                      <Typography variant="body2" sx={{ fontWeight: 500 }}>{doc.title}</Typography>
                    )}
                  </TableCell>
                  <TableCell>
                    <Chip label={doc.file_type.toUpperCase()} size="small" variant="outlined" sx={{ borderRadius: R, fontSize: '0.7rem' }} />
                  </TableCell>
                  <TableCell sx={{ whiteSpace: 'nowrap' }}>
                    <Typography variant="body2" color="text.secondary">
                      {doc.file_size < 1024 ? `${doc.file_size} B` : `${(doc.file_size / 1024).toFixed(1)} KB`}
                    </Typography>
                  </TableCell>
                  <TableCell>
                    <Chip
                      label={doc.is_visible ? '显示' : '隐藏'}
                      size="small"
                      color={doc.is_visible ? 'primary' : 'default'}
                      variant={doc.is_visible ? 'filled' : 'outlined'}
                      clickable
                      onClick={async () => {
                        try {
                          const r = await updateHelpDocument(doc.id, { is_visible: !doc.is_visible });
                          if (r.code === 0) { sm('更新成功'); loadHelpDocs(); }
                          else sm(r.message, true);
                        } catch { sm('更新失败', true); }
                      }}
                      sx={{ borderRadius: R, cursor: 'pointer' }}
                    />
                  </TableCell>
                  <TableCell align="right">
                    <Box sx={{ display: 'flex', gap: 0.5, justifyContent: 'flex-end' }}>
                      <IconButton
                        size="small"
                        onClick={() => { setHelpEditId(doc.id); setHelpEditTitle(doc.title); }}
                        sx={{ color: '#f4511e' }}
                      >
                        <EditIcon fontSize="small" />
                      </IconButton>
                      <IconButton size="small" color="error" onClick={() => {
                        setCa(() => async () => {
                          const r = await deleteHelpDocument(doc.id);
                          if (r.code === 0) { sm('删除成功'); loadHelpDocs(); }
                          else sm(r.message, true);
                          setCo(false);
                        });
                        setCo(true);
                      }}>
                        <DeleteIcon fontSize="small" />
                      </IconButton>
                    </Box>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      )}
      {/* 文章列表 */}
      <Typography variant="subtitle2" fontWeight={700} sx={{ mb: 1.5, mt: 4 }}>
        文章列表 · 共 {helpArticles.length} 篇
      </Typography>
      {helpArticles.length === 0 ? (
        <Typography color="text.secondary" textAlign="center" sx={{ py: 4 }}>暂无文章（上传 Word/PDF 后自动生成）</Typography>
      ) : (
        <TableContainer component={Paper} sx={tSx}>
          <Table size="small">
            <TableHead>
              <TableRow>
                <TableCell sx={{ fontWeight: 600 }}>标题</TableCell>
                <TableCell sx={{ fontWeight: 600 }}>源文件</TableCell>
                <TableCell sx={{ fontWeight: 600 }}>显隐</TableCell>
                <TableCell sx={{ fontWeight: 600 }}>创建时间</TableCell>
                <TableCell sx={{ fontWeight: 600 }} align="right">操作</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {helpArticles.map(article => (
                <TableRow key={article.id} hover>
                  <TableCell sx={{ maxWidth: 280 }}>
                    <Typography variant="body2" sx={{ fontWeight: 500 }}>{article.title}</Typography>
                  </TableCell>
                  <TableCell>
                    <Typography variant="body2" color="text.secondary" sx={{ fontSize: '0.8rem' }}>
                      {article.source_file || '-'}
                    </Typography>
                  </TableCell>
                  <TableCell>
                    <Chip
                      label={article.is_visible ? '显示' : '隐藏'}
                      size="small"
                      color={article.is_visible ? 'primary' : 'default'}
                      variant={article.is_visible ? 'filled' : 'outlined'}
                      clickable
                      onClick={async () => {
                        try {
                          const r = await updateHelpArticle(article.id, { is_visible: !article.is_visible });
                          if (r.code === 0) { sm('更新成功'); loadHelpArticles(); }
                          else sm(r.message, true);
                        } catch { sm('更新失败', true); }
                      }}
                      sx={{ borderRadius: R, cursor: 'pointer' }}
                    />
                  </TableCell>
                  <TableCell sx={{ whiteSpace: 'nowrap' }}>
                    <Typography variant="body2" color="text.secondary" sx={{ fontSize: '0.8rem' }}>
                      {article.created_at ? new Date(article.created_at).toLocaleDateString('zh-CN') : '-'}
                    </Typography>
                  </TableCell>
                  <TableCell align="right">
                    <IconButton size="small" color="error" onClick={() => {
                      setCa(() => async () => {
                        const r = await deleteHelpArticle(article.id);
                        if (r.code === 0) { sm('删除成功'); loadHelpArticles(); }
                        else sm(r.message, true);
                        setCo(false);
                      });
                      setCo(true);
                    }}>
                      <DeleteIcon fontSize="small" />
                    </IconButton>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      )}
    </Box>}

    {/* ── 对话框（保留：方法类型管理、确认对话框、导入映射、v0.3.18 编辑弹窗、方法一览） ── */}

    {/* v0.3.18: 项目编辑弹窗 */}
    <Dialog open={projectEditOpen} onClose={() => { setProjectEditOpen(false); setProjectEditItem(null); }} maxWidth="md" fullWidth PaperProps={{ sx: { borderRadius: R } }}>
      <DialogTitle sx={{ fontWeight: 700 }}>{projectEditItem?.id ? '编辑项目' : '新建项目'}</DialogTitle>
      <DialogContent>
        {projectEditItem && <>
          <TextField label="项目名称" fullWidth value={projectEditItem.name} onChange={e => setProjectEditItem({ ...projectEditItem, name: e.target.value })} sx={{ mt: 2, mb: 2, '& .MuiOutlinedInput-root': { borderRadius: R } }} />
          <TextField label="全称" fullWidth value={projectEditItem.full_name || ''} onChange={e => setProjectEditItem({ ...projectEditItem, full_name: e.target.value })} sx={{ mb: 2, '& .MuiOutlinedInput-root': { borderRadius: R } }} />
          <Typography variant="body2" sx={{ fontWeight: 600, mb: 1 }}>关联实验室</Typography>
          <Box sx={{ maxHeight: 200, overflow: 'auto', border: '1px solid rgba(0,0,0,0.12)', borderRadius: R, p: 1, mb: 2 }}>
            <FormGroup>
              {gs.filter(g => g.name !== '研发项目').map(g => (
                <FormControlLabel key={g.id}
                  control={<Checkbox checked={(projectEditItem.lab_ids || []).includes(g.id)} onChange={(e) => {
                    if (e.target.checked) { setProjectEditItem({ ...projectEditItem, lab_ids: [...(projectEditItem.lab_ids || []), g.id] }); }
                    else { setProjectEditItem({ ...projectEditItem, lab_ids: (projectEditItem.lab_ids || []).filter(id => id !== g.id) }); }
                  }} />}
                  label={g.name} />
              ))}
              {gs.filter(g => g.name !== '研发项目').length === 0 && <Typography variant="caption" color="text.secondary">暂无实验室</Typography>}
            </FormGroup>
          </Box>
          <Typography variant="body2" sx={{ fontWeight: 600, mb: 1 }}>关联检测方法</Typography>
          <FormControl size="small" sx={{ minWidth: 120, mb: 1, '& .MuiOutlinedInput-root': { borderRadius: R } }}>
            <InputLabel>按类型筛选</InputLabel>
            <Select
              value={projectMethodTypeFilter}
              label="按类型筛选"
              onChange={e => setProjectMethodTypeFilter(e.target.value)}
            >
              <MenuItem value="">全部</MenuItem>
              {mts.filter(t => t.name !== '检测类型').map(t => (
                <MenuItem key={t.id} value={t.name}>{t.name}</MenuItem>
              ))}
            </Select>
          </FormControl>
          <Box sx={{ maxHeight: 300, overflow: 'auto', border: '1px solid rgba(0,0,0,0.12)', borderRadius: R, p: 1, mb: 2 }}>
            {(projectMethodTypeFilter ? mts.filter(t => t.name === projectMethodTypeFilter) : mts.filter(t => t.name !== '检测类型')).map(type => {
              const typeMethods = ml.filter(m => m.type_names.includes(type.name));
              if (typeMethods.length === 0) return null;
              return (
                <Box key={type.id} sx={{ mb: 1 }}>
                  <Typography variant="caption" fontWeight={600} color="text.secondary">{type.name}</Typography>
                  <FormGroup row>
                    {typeMethods.map(m => (
                      <FormControlLabel
                        key={m.id}
                        control={<Checkbox
                          checked={(projectEditItem.method_ids || []).includes(m.id)}
                          onChange={(e) => {
                            if (e.target.checked) {
                              setProjectEditItem({ ...projectEditItem, method_ids: [...(projectEditItem.method_ids || []), m.id] });
                            } else {
                              setProjectEditItem({ ...projectEditItem, method_ids: (projectEditItem.method_ids || []).filter(id => id !== m.id) });
                            }
                          }}
                          size="small"
                        />}
                        label={<Typography variant="caption">{m.name}</Typography>}
                        sx={{ mr: 2, mb: 0.5 }}
                      />
                    ))}
                  </FormGroup>
                </Box>
              );
            })}
            {ml.length === 0 && <Typography variant="caption" color="text.secondary">暂无检测方法，请先导入或创建</Typography>}
          </Box>
          <FormControlLabel control={<Switch checked={projectEditItem.is_active} onChange={e => setProjectEditItem({ ...projectEditItem, is_active: e.target.checked })} />} label="启用" />
          <TextField label="备注" fullWidth multiline minRows={2} maxRows={4} value={projectEditItem.notes || ''} onChange={e => setProjectEditItem({ ...projectEditItem, notes: e.target.value })} sx={{ mt: 2, '& .MuiOutlinedInput-root': { borderRadius: R } }} />
        </>}
      </DialogContent>
      <DialogActions>
        <Button onClick={() => { setProjectEditOpen(false); setProjectEditItem(null); }} sx={{ borderRadius: R }}>取消</Button>
        <Button onClick={() => { if (projectEditItem) handleSaveProject(projectEditItem); }} variant="contained" sx={{ borderRadius: R }}>保存</Button>
      </DialogActions>
    </Dialog>

    {/* v0.3.18: 方法编辑弹窗 */}
    <Dialog open={methodEditOpen} onClose={() => { setMethodEditOpen(false); setMethodEditItem(null); }} maxWidth="sm" fullWidth PaperProps={{ sx: { borderRadius: R } }}>
      <DialogTitle sx={{ fontWeight: 700 }}>{methodEditItem?.id ? '编辑方法' : '新建方法'}</DialogTitle>
      <DialogContent>
        {methodEditItem && <>
          <TextField label="方法名称" fullWidth value={methodEditItem.name} onChange={e => setMethodEditItem({ ...methodEditItem, name: e.target.value })} sx={{ mt: 2, mb: 2, '& .MuiOutlinedInput-root': { borderRadius: R } }} />
          <TextField label="全称" fullWidth value={methodEditItem.full_name || ''} onChange={e => setMethodEditItem({ ...methodEditItem, full_name: e.target.value })} sx={{ mb: 2, '& .MuiOutlinedInput-root': { borderRadius: R } }} />
          <TextField label="管理系数" type="number" fullWidth value={methodEditItem.coefficient} onChange={e => setMethodEditItem({ ...methodEditItem, coefficient: Number(e.target.value) || 1.0 })} sx={{ mb: 2, '& .MuiOutlinedInput-root': { borderRadius: R } }} inputProps={{ min: 0, step: 0.1 }} />
          <TextField label="单价倍率" type="number" fullWidth value={methodEditItem.multiplier ?? 1.0} onChange={e => setMethodEditItem({ ...methodEditItem, multiplier: e.target.value === '' ? 1.0 : Number(e.target.value) })} sx={{ mb: 2, '& .MuiOutlinedInput-root': { borderRadius: R } }} inputProps={{ min: 0, step: 0.1 }} />
          <TextField label="单价" type="number" fullWidth value={methodEditItem.amount} onChange={e => setMethodEditItem({ ...methodEditItem, amount: Number(e.target.value) || 0 })} sx={{ mb: 2, '& .MuiOutlinedInput-root': { borderRadius: R } }} inputProps={{ min: 0, step: 0.01 }} />
          <TextField label="备注" fullWidth multiline minRows={2} value={methodEditItem.notes || ''} onChange={e => setMethodEditItem({ ...methodEditItem, notes: e.target.value })} sx={{ mb: 2, '& .MuiOutlinedInput-root': { borderRadius: R } }} />
          <Typography variant="body2" sx={{ fontWeight: 600, mb: 1 }}>类型归属</Typography>
          <FormGroup row>
            {mts.filter(t => t.name !== '检测类型').map(t => (
              <FormControlLabel key={t.id}
                control={<Checkbox checked={(methodEditItem.type_ids || []).includes(t.id)} onChange={(e) => {
                  if (e.target.checked) { setMethodEditItem({ ...methodEditItem, type_ids: [...(methodEditItem.type_ids || []), t.id] }); }
                  else { setMethodEditItem({ ...methodEditItem, type_ids: (methodEditItem.type_ids || []).filter(id => id !== t.id) }); }
                }} />}
                label={t.name}
              />
            ))}
          </FormGroup>
        </>}
      </DialogContent>
      <DialogActions>
        <Button onClick={() => { setMethodEditOpen(false); setMethodEditItem(null); }} sx={{ borderRadius: R }}>取消</Button>
        <Button onClick={() => { if (methodEditItem) handleSaveMethod(methodEditItem); }} variant="contained" sx={{ borderRadius: R }}>保存</Button>
      </DialogActions>
    </Dialog>

    {/* v0.3.18: 实验室编辑弹窗 */}
    <Dialog open={groupEditOpen} onClose={() => { setGroupEditOpen(false); setGroupEditItem(null); }} maxWidth="sm" fullWidth PaperProps={{ sx: { borderRadius: R } }}>
      <DialogTitle sx={{ fontWeight: 700 }}>{groupEditItem?.id ? '编辑实验室' : '新建实验室'}</DialogTitle>
      <DialogContent>
        {groupEditItem && <>
          <TextField label="实验室名称" fullWidth value={groupEditItem.name} onChange={e => setGroupEditItem({ ...groupEditItem, name: e.target.value })} sx={{ mt: 2, mb: 2, '& .MuiOutlinedInput-root': { borderRadius: R } }} />
          <TextField label="排序" type="number" fullWidth value={groupEditItem.sort_order} onChange={e => setGroupEditItem({ ...groupEditItem, sort_order: Number(e.target.value) || 0 })} sx={{ '& .MuiOutlinedInput-root': { borderRadius: R } }} />
          <Box sx={{ display: 'flex', gap: 1, mt: 2 }}>
            <Chip
              label="分析检测"
              clickable
              color={groupEditItem?.show_in_work !== false ? 'primary' : 'default'}
              variant={groupEditItem?.show_in_work !== false ? 'filled' : 'outlined'}
              sx={{ borderRadius: R }}
              onClick={() => setGroupEditItem({ ...groupEditItem!, show_in_work: groupEditItem?.show_in_work === false ? true : false })}
            />
            <Chip
              label="研发送样"
              clickable
              color={groupEditItem?.show_in_rd !== false ? 'primary' : 'default'}
              variant={groupEditItem?.show_in_rd !== false ? 'filled' : 'outlined'}
              sx={{ borderRadius: R }}
              onClick={() => setGroupEditItem({ ...groupEditItem!, show_in_rd: groupEditItem?.show_in_rd === false ? true : false })}
            />
          </Box>
        </>}
      </DialogContent>
      <DialogActions>
        <Button onClick={() => { setGroupEditOpen(false); setGroupEditItem(null); }} sx={{ borderRadius: R }}>取消</Button>
        <Button onClick={() => { if (groupEditItem) handleSaveGroup(groupEditItem); }} variant="contained" sx={{ borderRadius: R }}>保存</Button>
      </DialogActions>
    </Dialog>

    {/* v0.3.19: 方法一览弹窗（直接内联编辑，无需双击/操作列） */}
    <Dialog open={methodOverviewOpen} onClose={() => setMethodOverviewOpen(false)} maxWidth="lg" fullWidth PaperProps={{ sx: { borderRadius: R } }}>
      <DialogTitle sx={{ fontWeight: 700 }}>方法一览</DialogTitle>
      <DialogContent>
        <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
          共 {methodOverviewData.length} 条方法，直接在单元格内编辑，失焦自动保存
        </Typography>
        <TableContainer component={Paper} sx={{ borderRadius: R, border: '1px solid rgba(0,0,0,0.06)', overflowX: 'auto' }}>
          <Table size="small" stickyHeader>
            <TableHead>
              <TableRow>
                <TableCell sx={{ fontWeight: 700, minWidth: 200 }}>方法名称</TableCell>
                <TableCell sx={{ fontWeight: 700 }}>类型</TableCell>
                <TableCell sx={{ fontWeight: 700 }}>系数</TableCell>
                <TableCell sx={{ fontWeight: 700 }}>单价倍率</TableCell>
                <TableCell sx={{ fontWeight: 700 }}>单价</TableCell>
                <TableCell sx={{ fontWeight: 700, minWidth: 100 }}>对应仪器</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {methodOverviewData.map(m => {
                const instrument = extractInstrumentFromMethodName(m.name);
                return (
                  <TableRow key={m.id} hover>
                    <TableCell sx={{ p: 0.5 }}>
                      <TextField
                        size="small"
                        fullWidth
                        defaultValue={m.name}
                        onBlur={async (e) => {
                          const val = e.target.value;
                          if (val !== m.name) {
                            try {
                              await updateMethod(m.id, { ...m, name: val });
                              sm('已保存');
                              // 立即更新 methodOverviewData，确保 UI 实时刷新
                              setMethodOverviewData(prev => prev.map(item =>
                                item.id === m.id ? { ...item, name: val } : item
                              ));
                              lm();
                            } catch { sm('保存失败', true); }
                          }
                        }}
                        sx={{ '& .MuiOutlinedInput-root': { borderRadius: R, fontSize: '0.8rem' } }}
                      />
                    </TableCell>
                    <TableCell sx={{ p: 0.5 }}>
                      <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 0.3 }}>
                        {mts.filter(t => t.name !== '检测类型').map(t => (
                          <Chip
                            key={t.id}
                            label={t.name}
                            size="small"
                            clickable
                            color={(m.type_ids || []).includes(t.id) ? 'primary' : 'default'}
                            variant={(m.type_ids || []).includes(t.id) ? 'filled' : 'outlined'}
                            sx={{ borderRadius: R, fontSize: '0.7rem', height: 22 }}
                            onClick={async () => {
                              const current = m.type_ids || [];
                              const next = current.includes(t.id)
                                ? current.filter(id => id !== t.id)
                                : [...current, t.id];
                              try {
                                await updateMethod(m.id, { ...m, type_ids: next });
                                sm('已保存');
                                // 立即更新 methodOverviewData，确保 UI 实时刷新
                                setMethodOverviewData(prev => prev.map(item =>
                                  item.id === m.id ? { ...item, type_ids: next } : item
                                ));
                                lm();
                              } catch { sm('保存失败', true); }
                            }}
                          />
                        ))}
                      </Box>
                    </TableCell>
                    <TableCell sx={{ p: 0.5 }}>
                      <TextField
                        size="small"
                        type="number"
                        defaultValue={m.coefficient ?? 1.0}
                        onBlur={async (e) => {
                          const val = Number(e.target.value) || 1.0;
                          if (val !== (m.coefficient ?? 1.0)) {
                            try {
                              await updateMethod(m.id, { ...m, coefficient: val });
                              sm('已保存');
                              // 立即更新 methodOverviewData，确保 UI 实时刷新
                              setMethodOverviewData(prev => prev.map(item =>
                                item.id === m.id ? { ...item, coefficient: val } : item
                              ));
                              lm();
                            } catch { sm('保存失败', true); }
                          }
                        }}
                        sx={{ width: 70, '& .MuiOutlinedInput-root': { borderRadius: R, fontSize: '0.8rem' } }}
                        inputProps={{ min: 0, step: 0.1 }}
                      />
                    </TableCell>
                    <TableCell sx={{ p: 0.5 }}>
                      <TextField
                        size="small"
                        type="number"
                        defaultValue={m.multiplier ?? 1.0}
                        onBlur={async (e) => {
                          const val = Number(e.target.value) || 1.0;
                          if (val !== (m.multiplier ?? 1.0)) {
                            try {
                              await updateMethod(m.id, { ...m, multiplier: val });
                              sm('已保存');
                              setMethodOverviewData(prev => prev.map(item =>
                                item.id === m.id ? { ...item, multiplier: val } : item
                              ));
                              lm();
                            } catch { sm('保存失败', true); }
                          }
                        }}
                        sx={{ width: 70, '& .MuiOutlinedInput-root': { borderRadius: R, fontSize: '0.8rem' } }}
                        inputProps={{ min: 0, step: 0.1 }}
                      />
                    </TableCell>
                    <TableCell sx={{ p: 0.5 }}>
                      <TextField
                        size="small"
                        type="number"
                        defaultValue={m.amount ?? 0}
                        onBlur={async (e) => {
                          const val = Number(e.target.value) || 0;
                          if (val !== (m.amount ?? 0)) {
                            try {
                              await updateMethod(m.id, { ...m, amount: val });
                              sm('已保存');
                              // 立即更新 methodOverviewData，确保 UI 实时刷新
                              setMethodOverviewData(prev => prev.map(item =>
                                item.id === m.id ? { ...item, amount: val } : item
                              ));
                              lm();
                            } catch { sm('保存失败', true); }
                          }
                        }}
                        sx={{ width: 90, '& .MuiOutlinedInput-root': { borderRadius: R, fontSize: '0.8rem' } }}
                        inputProps={{ min: 0, step: 0.01 }}
                      />
                    </TableCell>
                    <TableCell>
                      <Chip
                        label={instrument || '无'}
                        size="small"
                        sx={{
                          borderRadius: R,
                          fontSize: '0.7rem',
                          bgcolor: instrument ? '#00897b' : 'default',
                          color: instrument ? '#fff' : 'inherit',
                        }}
                      />
                    </TableCell>
                  </TableRow>
                );
              })}
            </TableBody>
          </Table>
        </TableContainer>
      </DialogContent>
      <DialogActions>
        <Button onClick={() => { setMethodOverviewOpen(false); }} sx={{ borderRadius: R }}>关闭</Button>
      </DialogActions>
    </Dialog>

    {/* v0.3.23: 项目一览弹窗 */}
    <Dialog open={projectOverviewOpen} onClose={() => setProjectOverviewOpen(false)} maxWidth="lg" fullWidth PaperProps={{ sx: { borderRadius: R } }}>
      <DialogTitle sx={{ fontWeight: 700 }}>项目一览</DialogTitle>
      <DialogContent>
        <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
          共 {projectOverviewData.length} 个项目，直接在单元格内编辑，失焦自动保存
        </Typography>
        <TableContainer component={Paper} sx={{ borderRadius: R, border: '1px solid rgba(0,0,0,0.06)' }}>
          <Table size="small" stickyHeader>
            <TableHead>
              <TableRow>
                <TableCell sx={{ fontWeight: 700 }}>项目名称</TableCell>
                <TableCell sx={{ fontWeight: 700 }}>全名</TableCell>
                <TableCell sx={{ fontWeight: 700 }}>排序</TableCell>
                <TableCell sx={{ fontWeight: 700 }}>关联实验室</TableCell>
                <TableCell sx={{ fontWeight: 700 }}>关联方法</TableCell>
                <TableCell sx={{ fontWeight: 700 }}>状态</TableCell>
                <TableCell sx={{ fontWeight: 700 }} align="center">操作</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {projectOverviewData.map(p => (
                <TableRow key={p.id} hover>
                  <TableCell sx={{ p: 0.5 }}>
                    <TextField
                      size="small"
                      fullWidth
                      defaultValue={p.name}
                      onBlur={async (e) => {
                        const val = e.target.value;
                        if (val !== p.name) {
                          try {
                            await updateProject(p.id, { ...p, name: val });
                            sm('已保存');
                            setProjectOverviewData(prev => prev.map(item =>
                              item.id === p.id ? { ...item, name: val } : item
                            ));
                            lp();
                          } catch { sm('保存失败', true); }
                        }
                      }}
                      sx={{ '& .MuiOutlinedInput-root': { borderRadius: R, fontSize: '0.8rem' } }}
                    />
                  </TableCell>
                  <TableCell sx={{ p: 0.5 }}>
                    <TextField
                      size="small"
                      fullWidth
                      defaultValue={p.full_name || ''}
                      onBlur={async (e) => {
                        const val = e.target.value;
                        if (val !== (p.full_name || '')) {
                          try {
                            await updateProject(p.id, { ...p, full_name: val });
                            sm('已保存');
                            setProjectOverviewData(prev => prev.map(item =>
                              item.id === p.id ? { ...item, full_name: val } : item
                            ));
                            lp();
                          } catch { sm('保存失败', true); }
                        }
                      }}
                      sx={{ '& .MuiOutlinedInput-root': { borderRadius: R, fontSize: '0.8rem' } }}
                    />
                  </TableCell>
                  <TableCell sx={{ p: 0.5 }}>
                    <TextField
                      size="small"
                      type="number"
                      defaultValue={p.sort_order || 0}
                      onBlur={async (e) => {
                        const val = Number(e.target.value) || 0;
                        if (val !== (p.sort_order || 0)) {
                          try {
                            await updateProject(p.id, { ...p, sort_order: val });
                            sm('已保存');
                            setProjectOverviewData(prev => prev.map(item =>
                              item.id === p.id ? { ...item, sort_order: val } : item
                            ));
                            lp();
                          } catch { sm('保存失败', true); }
                        }
                      }}
                      sx={{ width: 70, '& .MuiOutlinedInput-root': { borderRadius: R, fontSize: '0.8rem' } }}
                    />
                  </TableCell>
                  <TableCell>
                    {p.lab_names && p.lab_names.length > 0 ? p.lab_names.join('、') : '无'}
                  </TableCell>
                  <TableCell>
                    {p.method_names && p.method_names.length > 0 ? p.method_names.join('、') : '无'}
                  </TableCell>
                  <TableCell>
                    <Chip
                      label={p.is_active ? '激活' : '停用'}
                      size="small"
                      color={p.is_active ? 'success' : 'default'}
                      sx={{ borderRadius: R, fontSize: '0.7rem' }}
                    />
                  </TableCell>
                  <TableCell align="center">
                    <IconButton size="small" onClick={() => { setProjectEditItem(p); setProjectEditOpen(true); }} sx={{ color: '#f4511e' }}>
                      <EditIcon fontSize="small" />
                    </IconButton>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      </DialogContent>
      <DialogActions>
        <Button onClick={() => { setProjectOverviewOpen(false); }} sx={{ borderRadius: R }}>关闭</Button>
      </DialogActions>
    </Dialog>

    {/* v0.3.23: 实验室一览弹窗 */}
    <Dialog open={groupOverviewOpen} onClose={() => setGroupOverviewOpen(false)} maxWidth="md" fullWidth PaperProps={{ sx: { borderRadius: R } }}>
      <DialogTitle sx={{ fontWeight: 700 }}>实验室一览</DialogTitle>
      <DialogContent>
        <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
          共 {groupOverviewData.length} 个实验室，直接在单元格内编辑，失焦自动保存
        </Typography>
        <TableContainer component={Paper} sx={{ borderRadius: R, border: '1px solid rgba(0,0,0,0.06)' }}>
          <Table size="small" stickyHeader>
            <TableHead>
              <TableRow>
                <TableCell sx={{ fontWeight: 700 }}>实验室名称</TableCell>
                <TableCell sx={{ fontWeight: 700 }}>关联项目</TableCell>
                <TableCell sx={{ fontWeight: 700 }}>排序</TableCell>
                <TableCell sx={{ fontWeight: 700 }}>关联分组</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {groupOverviewData.map(g => (
                <TableRow key={g.id} hover>
                  <TableCell sx={{ p: 0.5 }}>
                    <TextField
                      size="small"
                      fullWidth
                      defaultValue={g.name}
                      onBlur={async (e) => {
                        const val = e.target.value;
                        if (val !== g.name) {
                          try {
                            await updateGroup(g.id, { ...g, name: val });
                            sm('已保存');
                            setGroupOverviewData(prev => prev.map(item =>
                              item.id === g.id ? { ...item, name: val } : item
                            ));
                            lg();
                          } catch { sm('保存失败', true); }
                        }
                      }}
                      sx={{ '& .MuiOutlinedInput-root': { borderRadius: R, fontSize: '0.8rem' } }}
                    />
                  </TableCell>
                  <TableCell>
                    {g.project_names || '无'}
                  </TableCell>
                  <TableCell sx={{ p: 0.5 }}>
                    <TextField
                      size="small"
                      type="number"
                      defaultValue={g.sort_order || 0}
                      onBlur={async (e) => {
                        const val = Number(e.target.value) || 0;
                        if (val !== (g.sort_order || 0)) {
                          try {
                            await updateGroup(g.id, { ...g, sort_order: val });
                            sm('已保存');
                            setGroupOverviewData(prev => prev.map(item =>
                              item.id === g.id ? { ...item, sort_order: val } : item
                            ));
                            lg();
                          } catch { sm('保存失败', true); }
                        }
                      }}
                      sx={{ width: 70, '& .MuiOutlinedInput-root': { borderRadius: R, fontSize: '0.8rem' } }}
                    />
                  </TableCell>
                  <TableCell sx={{ p: 0.5 }}>
                    <Box sx={{ display: 'flex', gap: 0.5 }}>
                      <Chip
                        label="分析检测"
                        size="small"
                        clickable
                        color={g.show_in_work !== false ? 'primary' : 'default'}
                        variant={g.show_in_work !== false ? 'filled' : 'outlined'}
                        sx={{ borderRadius: R, fontSize: '0.7rem', height: 22 }}
                        onClick={async () => {
                          const next = g.show_in_work === false ? true : false;
                          try {
                            await updateGroup(g.id, { ...g, show_in_work: next });
                            sm('已保存');
                            setGroupOverviewData(prev => prev.map(item =>
                              item.id === g.id ? { ...item, show_in_work: next } : item
                            ));
                            lg();
                          } catch { sm('保存失败', true); }
                        }}
                      />
                      <Chip
                        label="研发送样"
                        size="small"
                        clickable
                        color={g.show_in_rd !== false ? 'primary' : 'default'}
                        variant={g.show_in_rd !== false ? 'filled' : 'outlined'}
                        sx={{ borderRadius: R, fontSize: '0.7rem', height: 22 }}
                        onClick={async () => {
                          const next = g.show_in_rd === false ? true : false;
                          try {
                            await updateGroup(g.id, { ...g, show_in_rd: next });
                            sm('已保存');
                            setGroupOverviewData(prev => prev.map(item =>
                              item.id === g.id ? { ...item, show_in_rd: next } : item
                            ));
                            lg();
                          } catch { sm('保存失败', true); }
                        }}
                      />
                    </Box>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      </DialogContent>
      <DialogActions>
        <Button onClick={() => { setGroupOverviewOpen(false); }} sx={{ borderRadius: R }}>关闭</Button>
      </DialogActions>
    </Dialog>

    {/* 方法类型对话框 */}
    <Dialog open={mtd} onClose={() => setMtd(false)} maxWidth="sm" fullWidth PaperProps={{ sx: { borderRadius: R } }}>
      <DialogTitle sx={{ fontWeight: 700 }}>{mtf.id > 0 ? '编辑类型' : '新增类型'}</DialogTitle>
      <DialogContent>
        {mts.length > 0 && <TableContainer component={Paper} sx={{ mb: 2, borderRadius: R, border: '1px solid rgba(0,0,0,0.06)' }}>
          <Table size="small">
            <TableHead><TableRow>
              <TableCell sx={{ fontWeight: 600 }}>名称</TableCell>
              <TableCell sx={{ fontWeight: 600 }}>排序</TableCell>
              <TableCell sx={{ fontWeight: 600 }} align="right">操作</TableCell>
            </TableRow></TableHead>
            <TableBody>
              {mts.map(t => <TableRow key={t.id} hover>
                <TableCell>{t.name}</TableCell>
                <TableCell>{t.sort_order}</TableCell>
                <TableCell align="right">
                  <IconButton size="small" onClick={() => setMtf({ id: t.id, name: t.name, sort_order: t.sort_order || 10 })} sx={{ color: '#f4511e' }}><EditIcon fontSize="small" /></IconButton>
                  <IconButton size="small" color="error" onClick={() => { setCa(() => async () => { const r = await deleteMethodType(t.id); if (r.code === 0) { sm('删除成功'); lmt(); } else sm(r.message, true); setCo(false); }); setCo(true); }}><DeleteIcon fontSize="small" /></IconButton>
                </TableCell>
              </TableRow>)}
            </TableBody>
          </Table>
        </TableContainer>}
        <TextField label="类型名称" fullWidth value={mtf.name} onChange={e => setMtf({ ...mtf, name: e.target.value })} sx={{ mt: 1, '& .MuiOutlinedInput-root': { borderRadius: R } }} helperText="如: 液相、气相、理化、检测类型等" />
        <TextField label="排序" type="number" fullWidth value={mtf.sort_order} onChange={e => setMtf({ ...mtf, sort_order: Number(e.target.value) || 10 })} sx={{ mt: 2, '& .MuiOutlinedInput-root': { borderRadius: R } }} />
      </DialogContent>
      <DialogActions><Button onClick={() => setMtd(false)} sx={{ borderRadius: R }}>取消</Button><Button onClick={hmt} variant="contained" sx={{ borderRadius: R }}>保存</Button></DialogActions>
    </Dialog>

    <ConfirmDialog open={co} title="确认操作" message="确定要执行此操作吗？" confirmText="确定" cancelText="取消" onConfirm={ca} onCancel={() => setCo(false)} />

    {/* v0.3.0: 导入映射预览对话框 */}
    <Dialog open={importMappingOpen} onClose={() => setImportMappingOpen(false)} maxWidth="md" fullWidth PaperProps={{ sx: { borderRadius: R } }}>
      <DialogTitle sx={{ fontWeight: 700 }}>列头映射预览</DialogTitle>
      <DialogContent>
        <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
          下表展示当前列头匹配规则。导入 Excel 时将按此规则对每列进行分类。
        </Typography>
        <TableContainer component={Paper} sx={{ borderRadius: R, border: '1px solid rgba(0,0,0,0.06)' }}>
          <Table size="small">
            <TableHead>
              <TableRow>
                <TableCell sx={{ fontWeight: 600 }}>匹配模式</TableCell>
                <TableCell sx={{ fontWeight: 600 }}>目标表</TableCell>
                <TableCell sx={{ fontWeight: 600 }}>默认类型</TableCell>
                <TableCell sx={{ fontWeight: 600 }} align="right">优先级</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {importMappings.map(m => (
                <TableRow key={m.id} hover>
                  <TableCell sx={{ fontFamily: 'monospace', fontSize: '0.8rem' }}>{m.header_pattern}</TableCell>
                  <TableCell>
                    <Chip
                      label={m.target_table === 'project_groups' ? '实验室' : m.target_table === 'projects' ? '研发项目' : '检测方法'}
                      size="small"
                      color={m.target_table === 'project_groups' ? 'primary' : m.target_table === 'projects' ? 'success' : 'default'}
                      variant="outlined"
                      sx={{ borderRadius: R }}
                    />
                  </TableCell>
                  <TableCell>{m.default_type || '—'}</TableCell>
                  <TableCell align="right">{m.priority}</TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
        <Typography variant="caption" color="text.secondary" sx={{ mt: 1, display: 'block' }}>
          通配符 * 匹配任意字符。优先级数值越小越优先匹配。
        </Typography>
      </DialogContent>
      <DialogActions>
        <Button onClick={() => setImportMappingOpen(false)} sx={{ borderRadius: R }}>取消</Button>
        <Button variant="contained" component="label" startIcon={<CloudUploadIcon />}
          sx={{ borderRadius: R, background: 'linear-gradient(135deg,#00897b,#43a047)' }}>
          选择文件导入
          <input type="file" accept=".xlsx" hidden onChange={async (e) => {
            const f = e.target.files?.[0]; if (!f) return;
            setImportMappingOpen(false);
            try { const r = await methodImport(f); sm(`导入成功: ${r.data?.total_methods || 0}条方法, ${r.data?.total_groups || 0}个分组`); lm(); lg(); lp(); }
            catch { sm('导入失败', true); } e.target.value = '';
          }} />
        </Button>
      </DialogActions>
    </Dialog>
  </Box>);
};

export default ManagePage;
