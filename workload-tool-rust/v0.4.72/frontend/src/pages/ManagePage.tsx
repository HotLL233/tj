import React, { useState, useEffect, useCallback, useMemo } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  Box, Typography, Paper, Button, CircularProgress, TextField, FormControl, InputLabel, Select,
  MenuItem, Switch, FormControlLabel, IconButton, Chip, TableContainer,
  Table, TableHead, TableBody, TableRow, TableCell, Alert, Snackbar,
  Checkbox, FormGroup, useMediaQuery, useTheme, Autocomplete,
  Dialog, DialogTitle, DialogContent, DialogActions, Tooltip, Grid,
  Popover, List, ListItem, ListItemText, ListItemIcon,
} from '@mui/material';
import AddIcon from '@mui/icons-material/Add';
import DeleteIcon from '@mui/icons-material/Delete';
import EditIcon from '@mui/icons-material/Edit';
import FolderIcon from '@mui/icons-material/Folder';
import ListAltIcon from '@mui/icons-material/ListAlt';
import DashboardIcon from '@mui/icons-material/Dashboard';
import BuildIcon from '@mui/icons-material/Build';
import ScienceIcon from '@mui/icons-material/Science';

import PageLayoutAdmin from '../components/PageLayoutAdmin';
import DeleteSweepIcon from '@mui/icons-material/DeleteSweep';
import ReceiptLongIcon from '@mui/icons-material/ReceiptLong';
import HistoryIcon from '@mui/icons-material/History';
import CloudUploadIcon from '@mui/icons-material/CloudUpload';
import BackupIcon from '@mui/icons-material/Backup';
import VisibilityIcon from '@mui/icons-material/Visibility';
import MenuBookIcon from '@mui/icons-material/MenuBook';
import BusinessIcon from '@mui/icons-material/Business';
import ViewWeekIcon from '@mui/icons-material/ViewWeek';
import AssessmentIcon from '@mui/icons-material/Assessment';
import ArrowBackIcon from '@mui/icons-material/ArrowBack';
import PeopleIcon from '@mui/icons-material/People';
import VerifiedUserIcon from '@mui/icons-material/VerifiedUser';
import RefreshIcon from '@mui/icons-material/Refresh';
import { getGroups, createGroup, updateGroup, deleteGroup, getProjects, createProject, updateProject, deleteProject, getRecords, restoreRecord, getAuditLogs, batchProjectCoefficient, getBackupStatus, backupNow, getBackupConfig, updateBackupConfig, deleteBackup, restoreBackup, restoreBackupFile, getMethodTypes, createMethodType, updateMethodType, deleteMethodType, getMethods, createMethod, updateMethod, deleteMethod, methodImport, getImportMappings, getHelpDocuments, uploadHelpDocument, updateHelpDocument, deleteHelpDocument, getHelpDocumentFileUrl, getHelpArticles, deleteHelpArticle, updateHelpArticle, getSampleInfoTypesAll, getSampleInfoRecords, updateSampleInfo, deleteSampleInfo, getSampleInfoTypes, getSampleInfoStats, createSampleInfoType, updateSampleInfoType, deleteSampleInfoType, deleteSampleInfoTypePermanent, exportSampleInfo, getDivisions, createDivision, updateDivision, deleteDivision, setDivisionLabs, getSampleInfoColumns, createSampleInfoColumn, updateSampleInfoColumn, deleteSampleInfoColumn, reorderSampleInfoColumns, userList, userRegister, updateUser, deleteUser, deleteUserPermanent, getRoles, updateSetting, importUsers } from '../api/client';
import type { ProjectGroup, Project, WorkRecord, AuditLog, BackupStatus, MethodType, Method, ImportMapping, HelpDocument, HelpArticle, SampleInfoType, SampleInfoRecord, Division, SampleInfoColumn, User, UserUpdate, Role, RoleWithPermissions, SystemSetting, HomeCard, StatCard, ManageTab } from '../types';
import ConfirmDialog from '../components/ConfirmDialog';
import InlineEditCard from '../components/InlineEditCard';
import { useUser } from '../UserContext';
import { hasPermission } from '../constants/permissions';
import AdminRdRecordColumns from './AdminRdRecordColumns';
import ManageFormConfig from '../components/ManageFormConfig';
import ManageExportConfig from '../components/ManageExportConfig';

type TV = 'projects' | 'groups' | 'methods' | 'divisions' | 'trash' | 'audit' | 'backup' | 'help' | 'sampleinfo' | 'users' | 'roles' | 'layouts' | 'theme' | 'home' | 'stats' | 'forms' | 'exports';

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

const ManagePage: React.FC = () => {
  const navigate = useNavigate();
  const { user } = useUser();
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

  // v0.4.24: 事业部
  const [divs, setDivs] = useState<Division[]>([]);
  const ldiv = useCallback(async () => { try { const r = await getDivisions(); if (r.code === 0 && r.data) setDivs(r.data); } catch {} }, []);
  const [divEditOpen, setDivEditOpen] = useState(false);
  const [divForm, setDivForm] = useState({ id: 0, name: '', sort_order: 10, color: '#1976d2', group_ids: [] as number[] });
  const hdiv = async () => {
    if (!divForm.name.trim()) { sm('请输入部门名称', true); return; }
    try {
      if (divForm.id > 0) {
        const r = await updateDivision(divForm.id, { name: divForm.name, sort_order: divForm.sort_order, color: divForm.color });
        if (r.code === 0) {
          // 保存关联实验室
          await setDivisionLabs(divForm.id, divForm.group_ids);
          sm('更新成功'); ldiv(); setDivEditOpen(false);
        } else sm(r.message, true);
      } else {
        const r = await createDivision({ name: divForm.name, sort_order: divForm.sort_order, color: divForm.color });
        if (r.code === 0 && r.data) {
          // 保存关联实验室
          await setDivisionLabs(r.data.id, divForm.group_ids);
          sm('创建成功'); ldiv(); setDivEditOpen(false);
        } else sm(r.message, true);
      }
    } catch { sm('操作失败', true); }
  };
  useEffect(() => { ldiv(); }, []);

  // projects (v0.2.17 simplified)
  const [ps, setPs] = useState<Project[]>([]);
  const lp = useCallback(async () => { try { const r = await getProjects(); if (r.code === 0 && r.data) setPs(r.data); } catch {} }, []);

  // 项目编辑 - 方法类型筛选
  const [projectMethodTypeFilter, setProjectMethodTypeFilter] = useState('');
  const [ml, setMl] = useState<Method[]>([]);
  const [methodFilter, setMethodFilter] = useState('');
  const [importMappingOpen, setImportMappingOpen] = useState(false);
  const [importMappings, setImportMappings] = useState<ImportMapping[]>([]);

  // v0.4.35: 动态 Tab 配置
  const defaultTC: { key: TV; label: string; perm?: string; icon: React.ReactNode; desc: string }[] = [
    { key: 'projects', label: '研发项目管理', perm: 'manage:projects', icon: <ListAltIcon />, desc: '研发项目及关联实验室' },
    { key: 'groups', label: '实验室管理', perm: 'manage:groups', icon: <FolderIcon />, desc: '新增编辑实验室映射录入选项卡' },
    { key: 'divisions', label: '部门管理', perm: 'manage:divisions', icon: <BusinessIcon />, desc: '管理部门及下属实验室' },
    { key: 'methods', label: '检测方法管理', perm: 'manage:methods', icon: <ScienceIcon />, desc: '液相/气相/理化/ICP/热分析等检测方法' },
    { key: 'trash', label: '回收站', perm: 'manage:trash', icon: <DeleteSweepIcon />, desc: '恢复已删除的记录' },
    { key: 'audit', label: '审计日志', perm: 'manage:audit', icon: <ReceiptLongIcon />, desc: '操作记录追溯' },
    { key: 'backup', label: '数据备份', perm: 'manage:backup', icon: <BackupIcon />, desc: '备份恢复与自动备份设置' },
    { key: 'help', label: '教程与帮助', perm: 'manage:help', icon: <MenuBookIcon />, desc: '上传编辑帮助文档，管理显隐' },
    { key: 'sampleinfo', label: '样品信息登记管理', perm: 'manage:sampleinfo', icon: <ScienceIcon />, desc: '检测类型 · 记录查询 · 独立统计' },
    { key: 'users', label: '用户管理', perm: 'manage:users', icon: <PeopleIcon />, desc: '注册/编辑用户、分配权限' },
    { key: 'roles', label: '角色管理', perm: 'manage:roles', icon: <VerifiedUserIcon />, desc: '角色分级与入口可见性' },
    { key: 'layouts', label: '页面布局管理', perm: 'manage:settings', icon: <DashboardIcon />, desc: '编辑各页面的布局及功能文案' },
    { key: 'forms', label: '录入表单配置', perm: 'manage:settings', icon: <ScienceIcon />, desc: '统一配置研发送样/样品信息/分析检测的录入字段' },
    { key: 'exports', label: '导出模板配置', perm: 'manage:settings', icon: <ListAltIcon />, desc: '配置分析检测统计/研发送样统计/样品信息登记的Excel导出模板' },
  ];
  const [tc, setTc] = useState(defaultTC);

  // v0.4.35: 设置编辑器状态
  const [themeForm, setThemeForm] = useState({ primaryColor: '#667eea', secondaryColor: '#764ba2', bgColor: '#f8fafc', cardRadius: 2, loginBg: 'linear-gradient(135deg, #f0f4f8, #e8f5e9)', loginButtonColor: '#f4511e', logoText: '知微' });
  const [homeCardsForm, setHomeCardsForm] = useState<HomeCard[]>([]);
  const [statsCardsForm, setStatsCardsForm] = useState<StatCard[]>([]);
  const [settingsLoading, setSettingsLoading] = useState(false);
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
  const [trTypes, setTrTypes] = useState<SampleInfoType[]>([]);
  const [trProjects, setTrProjects] = useState<any[]>([]);
  const [trUsers, setTrUsers] = useState<User[]>([]);
  const [trashTab, setTrashTab] = useState<'records' | 'types' | 'projects' | 'users'>('records');
  const loadTrash = async () => {
    try { const r = await getRecords({ include_deleted: true }); if (r.code === 0 && r.data) setTr(r.data.items.filter((x: any) => x.deleted_at != null)); } catch {};
    try { const r = await getSampleInfoTypesAll(); if (r.code === 0 && r.data) setTrTypes(r.data.filter((t: any) => !t.is_active)); } catch {};
    try { const r = await getProjects(); if (r.code === 0 && r.data) setTrProjects(r.data.filter((p: any) => !p.is_active)); } catch {};
    try { const r = await userList(); if (r.code === 0 && r.data) setTrUsers(r.data.filter((u: any) => !u.is_active)); } catch {};
  };
  // v0.4.63: 导入用户对话框
  const [importDialogOpen, setImportDialogOpen] = useState(false);
  const [importFile, setImportFile] = useState<File | null>(null);
  const [importResult, setImportResult] = useState<any>(null);

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

  // ========== v0.4.23: 样品信息登记管理 ==========
  // ① 检测类型
  const [siTypes, setSiTypes] = useState<SampleInfoType[]>([]);
  const loadSiTypes = useCallback(async () => { try { const r = await getSampleInfoTypesAll(); if (r.code === 0 && r.data) setSiTypes(r.data); } catch {} }, []);
  const [siTypeEdit, setSiTypeEdit] = useState<SampleInfoType | null>(null);
  // v0.4.61: 独立追踪"新建类型"表单是否打开（不受 form 值影响）
  const [siTypeNewOpen, setSiTypeNewOpen] = useState(false);
  const [siTypeForm, setSiTypeForm] = useState({ type_key: '', label: '', description: '', color: '#2e7d32', sort_order: 0, is_active: 1 });

  // ② 记录查询
  const [siRecords, setSiRecords] = useState<SampleInfoRecord[]>([]);
  const [siTotal, setSiTotal] = useState(0);
  const [siPage, setSiPage] = useState(0);
  const [siFilters, setSiFilters] = useState({ start: '', end: '', user_name: '', lab_name: '', project_name: '', type_key: '', status: '' });
  const loadSiRecords = useCallback(async () => {
    try {
      const r = await getSampleInfoRecords({
        start: siFilters.start || undefined,
        end: siFilters.end || undefined,
        user_name: siFilters.user_name || undefined,
        lab_name: siFilters.lab_name || undefined,
        project_name: siFilters.project_name || undefined,
        type_key: siFilters.type_key || undefined,
        status: siFilters.status || undefined,
        page: siPage + 1,
        page_size: 50,
      });
      if (r.code === 0 && r.data) { setSiRecords(r.data.items); setSiTotal(r.data.total); }
    } catch {}
  }, [siFilters, siPage]);

  // ③ 独立统计
  const [siStats, setSiStats] = useState<any>(null);
  const loadSiStats = useCallback(async () => {
    try {
      const r = await getSampleInfoStats({
        start: siFilters.start || undefined,
        end: siFilters.end || undefined,
        type_key: siFilters.type_key || undefined,
        status: siFilters.status || undefined,
      });
      if (r.code === 0 && r.data) setSiStats(r.data);
      else setSiStats(null);
    } catch { setSiStats(null); }
  }, [siFilters]);

  // ④ 自定义列配置
  const [siColumns, setSiColumns] = useState<SampleInfoColumn[]>([]);
  const [siColTypeKey, setSiColTypeKey] = useState<string>('');
  const loadSiColumns = useCallback(async (typeKey?: string) => {
    // v0.4.28: always load all columns, not type-specific
    try {
      const r = await getSampleInfoColumns();
      if (r.code === 0 && r.data) setSiColumns(r.data);
    } catch {}
  }, []);
  const [colEditOpen, setColEditOpen] = useState(false);
  const [colEditItem, setColEditItem] = useState<SampleInfoColumn | null>(null);
  const [colForm, setColForm] = useState({
    field_key: '', label: '', data_type: 'text' as string,
    width: 100, sort_order: 0, options: '',
    is_required: false, show_in_list: true, show_in_export: true, show_in_form: true,
  });

  // 记录行内编辑
  const [siEditId, setSiEditId] = useState<number | null>(null);
  const [siEditForm, setSiEditForm] = useState<Record<string, string>>({});
  // v0.4.28: sampleinfo 子卡片导航
  const [siSubTab, setSiSubTab] = useState<string | null>(null);

  // v0.4.71: 显示类型编辑
  const [sampleTypes, setSampleTypes] = useState<{ type_key: string; label: string }[]>([]);
  const [visibleTypesAnchorEl, setVisibleTypesAnchorEl] = useState<HTMLElement | null>(null);
  const [visibleTypesColId, setVisibleTypesColId] = useState<number | null>(null);
  const [visibleTypesSelected, setVisibleTypesSelected] = useState<string[]>([]);
  useEffect(() => {
    getSampleInfoTypesAll().then(r => { if (r.code === 0 && r.data) setSampleTypes(r.data); }).catch(() => {});
  }, []);

  // v0.4.29: 用户管理
  const [users, setUsers] = useState<User[]>([]);
  const [roles, setRoles] = useState<RoleWithPermissions[]>([]);
  const roleNameOf = (id?: number | null) => roles.find((r) => r.id === id)?.name || '未分配';
  const [userEditOpen, setUserEditOpen] = useState(false);
  const [userEditItem, setUserEditItem] = useState<User | null>(null);
  const [userForm, setUserForm] = useState({
    username: '', password: '', division_id: null as number | null,
    group_id: null as number | null, role_id: null as number | null,
    is_admin: false, is_active: true,
  });
  const loadUsers = useCallback(async () => {
    try {
      const r = await userList();
      if (r.code === 0 && r.data) setUsers(r.data.filter((u: any) => u.is_active));
    } catch {}
  }, []);
  const handleSaveUser = async () => {
    if (!userForm.username.trim()) { sm('请输入用户名', true); return; }
    if (!userEditItem && !userForm.password.trim()) { sm('新用户必须设置密码', true); return; }
    try {
      if (userEditItem) {
        const body: UserUpdate = { username: userForm.username, division_id: userForm.division_id, group_id: userForm.group_id, role_id: userForm.role_id, is_admin: userForm.is_admin, is_active: userForm.is_active };
        if (userForm.password.trim()) body.password = userForm.password;
        await updateUser(userEditItem.id, body);
        sm('用户更新成功');
      } else {
        await userRegister({ username: userForm.username, password: userForm.password, division_id: userForm.division_id, group_id: userForm.group_id, role_id: userForm.role_id });
        sm('用户创建成功');
      }
      setUserEditOpen(false);
      loadUsers();
    } catch (e: any) { sm(e.message, true); }
  };
  const handleDeleteUser = async (id: number) => {
    setCa(() => async () => {
      try { await deleteUser(id); sm('用户已删除'); loadUsers(); setCo(false); }
      catch (e: any) { sm(e.message, true); setCo(false); }
    });
    setCo(true);
  };

  // v0.4.35: 加载管理页 Tab 配置和设备设置
  useEffect(() => {
    fetch('/api/settings/manage-tabs')
      .then(r => r.json())
      .then(d => {
        if (d.data?.value) {
          try {
            const tabs: ManageTab[] = JSON.parse(d.data.value);
            const enabledKeys = new Set(tabs.filter(t => t.enabled !== false).map(t => t.key as TV));
            const filtered = defaultTC.filter(t => enabledKeys.has(t.key));
            if (filtered.length > 0) setTc(filtered);
          } catch {}
        }
      })
      .catch(() => {});
    // 加载主题/首页/统计卡片表单初始值
    fetch('/api/settings/theme')
      .then(r => r.json()).then(d => {
        if (d.data?.value) try { const v = JSON.parse(d.data.value); setThemeForm(prev => ({ ...prev, ...v })); } catch {}
      }).catch(() => {});
    fetch('/api/settings/home-cards')
      .then(r => r.json()).then(d => {
        if (d.data?.value) try { setHomeCardsForm(JSON.parse(d.data.value)); } catch {}
      }).catch(() => {});
    fetch('/api/settings/stats-cards')
      .then(r => r.json()).then(d => {
        if (d.data?.value) try { setStatsCardsForm(JSON.parse(d.data.value)); } catch {}
      }).catch(() => {});
  }, []);

  const openSiEdit = (rec: SampleInfoRecord) => {
    setSiEditId(rec.id);
    setSiEditForm({
      batch_no: rec.batch_no, user_name: rec.user_name, lab_name: rec.lab_name,
      project_name: rec.project_name, status: rec.status, main_components: rec.main_components, notes: rec.notes,
    });
  };
  const saveSiEdit = async (id: number) => {
    try {
      const r = await updateSampleInfo(id, { ...siEditForm });
      if (r.code === 0) { sm('保存成功'); setSiEditId(null); setSiEditForm({}); loadSiRecords(); }
      else sm(r.message, true);
    } catch (e: any) { sm(e.message || '保存失败', true); }
  };
  const delSiRecord = (id: number) => {
    setCa(() => async () => {
      const r = await deleteSampleInfo(id);
      if (r.code === 0) { sm('删除成功'); loadSiRecords(); loadSiStats(); }
      else sm(r.message, true);
      setCo(false);
    });
    setCo(true);
  };

  // 导出（独立接口）
  const doExportSi = async () => {
    try {
      await exportSampleInfo({ start: siFilters.start || undefined, end: siFilters.end || undefined });
      sm('导出成功');
    } catch (e: any) { sm(e.message || '导出失败', true); }
  };

  // 类型 CRUD 保存
  const saveSiType = async () => {
    if (!siTypeForm.type_key.trim() || !siTypeForm.label.trim()) { sm('类型标识与名称不能为空', true); return; }
    try {
      if (siTypeEdit) {
        const r = await updateSampleInfoType(siTypeEdit.id, {
          type_key: siTypeForm.type_key, label: siTypeForm.label, description: siTypeForm.description,
          color: siTypeForm.color, sort_order: siTypeForm.sort_order, is_active: siTypeForm.is_active,
        });
        if (r.code === 0) { sm('更新成功'); setSiTypeEdit(null); setSiTypeNewOpen(false); setSiTypeForm({ type_key: '', label: '', description: '', color: '#2e7d32', sort_order: 0, is_active: 1 }); loadSiTypes(); }
        else sm(r.message, true);
      } else {
        const r = await createSampleInfoType({
          type_key: siTypeForm.type_key, label: siTypeForm.label, description: siTypeForm.description,
          color: siTypeForm.color, sort_order: siTypeForm.sort_order,
        });
        if (r.code === 0) { sm('创建成功'); setSiTypeNewOpen(false); setSiTypeForm({ type_key: '', label: '', description: '', color: '#2e7d32', sort_order: 0, is_active: 1 }); loadSiTypes(); }
        else sm(r.message, true);
      }
    } catch (e: any) { sm(e.message || '操作失败', true); }
  };
  const editSiType = (t: SampleInfoType) => {
    setSiTypeEdit(t);
    setSiTypeForm({ type_key: t.type_key, label: t.label, description: t.description, color: t.color, sort_order: t.sort_order, is_active: t.is_active });
  };
  const delSiType = (id: number) => {
    setCa(() => async () => {
      const r = await deleteSampleInfoType(id);
      if (r.code === 0) { sm('已移入回收站'); loadSiTypes(); }
      else sm(r.message, true);
      setCo(false);
    });
    setCo(true);
  };
  const restoreSiType = async (id: number) => {
    const r = await updateSampleInfoType(id, { is_active: 1 });
    if (r.code === 0) { sm('已恢复启用'); loadSiTypes(); }
    else sm(r.message, true);
  };
  // 仅显示启用的检测类型（已停用移至回收站）
  const activeSiTypes = siTypes.filter(t => t.is_active);

  // ④ 列配置 CRUD
  const saveCol = async () => {
    if (!colForm.field_key.trim() || !colForm.label.trim()) { sm('字段标识与显示名称不能为空', true); return; }
    try {
      if (colEditItem) {
        const r = await updateSampleInfoColumn(colEditItem.id, {
          label: colForm.label, data_type: colForm.data_type,
          is_active: colEditItem.is_active, is_required: colForm.is_required,
          width: colForm.width, options: colForm.options || undefined,
          show_in_list: colForm.show_in_list, show_in_export: colForm.show_in_export,
          show_in_form: colForm.show_in_form,
        });
        if (r.code === 0) { sm('更新成功'); setColEditOpen(false); setColEditItem(null); loadSiColumns(); }
        else sm(r.message, true);
      } else {
        const r = await createSampleInfoColumn({
          field_key: colForm.field_key, label: colForm.label, data_type: colForm.data_type,
          width: colForm.width, sort_order: colForm.sort_order,
          options: colForm.options || undefined,
          is_required: colForm.is_required, show_in_list: colForm.show_in_list,
          show_in_export: colForm.show_in_export, show_in_form: colForm.show_in_form,
        });
        if (r.code === 0) { sm('创建成功'); loadSiColumns(); }
        else sm(r.message, true);
      }
    } catch (e: any) { sm(e.message || '操作失败', true); }
  };
  const editCol = (col: SampleInfoColumn) => {
    setColEditItem(col);
    setColForm({
      field_key: col.field_key, label: col.label, data_type: col.data_type,
      width: col.width, sort_order: col.sort_order, options: col.options || '',
      is_required: col.is_required, show_in_list: col.show_in_list,
      show_in_export: col.show_in_export, show_in_form: col.show_in_form,
    });
    setColEditOpen(true);
  };
  const delCol = (id: number) => {
    setCa(() => async () => {
      const r = await deleteSampleInfoColumn(id);
      if (r.code === 0) { sm('删除成功'); loadSiColumns(); }
      else sm(r.message, true);
      setCo(false);
    });
    setCo(true);
  };
  const moveCol = async (idx: number, dir: -1 | 1) => {
    const newCols = [...siColumns];
    const target = idx + dir;
    if (target < 0 || target >= newCols.length) return;
    [newCols[idx], newCols[target]] = [newCols[target], newCols[idx]];
    // 重新计算 sort_order
    const ids = newCols.map((c, i) => ({ id: c.id, sort_order: i }));
    try {
      const r = await reorderSampleInfoColumns(ids);
      if (r.code === 0) { loadSiColumns(); }
      else sm(r.message, true);
    } catch (e: any) { sm(e.message || '排序失败', true); }
  };


  // v0.3.15: 初始加载时也加载方法类型，否则项目编辑对话框的"关联检测方法"按类型分组时 mts 为空
  useEffect(() => { setLd(true); Promise.all([lg(), lp(), lt(), lm(), lmt()]).finally(() => setLd(false)); }, [lg, lp, lt, lm, lmt]);
  useEffect(() => { if (tb === 'audit') la(1); if (tb === 'backup') loadBk(); if (tb === 'methods') { lmt(); lm(); } if (tb === 'trash') loadTrash(); if (tb === 'help') { loadHelpDocs(); loadHelpArticles(); } if (tb === 'users') { userList().then(r => { if (r.code === 0 && r.data) setUsers(r.data.filter((u: any) => u.is_active)); }).catch(() => {}); getRoles().then(r => { if (r.code === 0 && r.data) setRoles(r.data); }).catch(() => {}); } }, [tb, la, lmt, lm, loadUsers]);

  // v0.4.23: 样品信息登记管理数据加载
  useEffect(() => {
    if (tb === 'sampleinfo') { loadSiTypes(); loadSiRecords(); loadSiStats(); loadSiColumns(); }
  }, [tb, loadSiTypes, loadSiRecords, loadSiStats, loadSiColumns]);

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
      high_item: project.high_item,
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
      const r = await updateGroup(group.id, { name: group.name, sort_order: group.sort_order, show_in_work: group.show_in_work, show_in_rd: group.show_in_rd, division_id: group.division_id ?? null });
      if (r.code === 0) { sm('更新成功'); lg(); setGroupEditOpen(false); setGroupEditItem(null); } else sm(r.message, true);
    } else {
      const r = await createGroup({ name: group.name, sort_order: group.sort_order, show_in_work: group.show_in_work, show_in_rd: group.show_in_rd, division_id: group.division_id ?? null });
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

  return (
    
    <Box>

      {/* 标题 */}
      
      <Typography variant="h5" fontWeight={700} sx={{ mb: 3, px: 1, background: 'linear-gradient(135deg,#f4511e,#e53935)', WebkitBackgroundClip:'text', WebkitTextFillColor:'transparent', backgroundClip:'text' }}>系统管理</Typography>
      

      {msg && <Alert severity={err ? 'error' : 'success'} sx={{ mb: 2, borderRadius: R }} onClose={() => setMsg('')}>{msg}</Alert>}

      {/* 管理卡片网格 — 每张卡片独立可编辑 */}
      <Box sx={{ display: 'grid', gridTemplateColumns: { xs: '1fr 1fr 1fr', sm: '1fr 1fr 1fr', md: '1fr 1fr 1fr' }, gap: 1.5, mb: 3 }}>
      {tc.filter(c => user?.is_admin || !c.perm || hasPermission(user?.permissions || [], c.perm)).map(c => (
        
        <Paper elevation={0}
          onClick={() => c.key === 'roles' ? navigate('/admin/roles') : setTb(c.key)}
          sx={{ p: isMobile ? 1.5 : 2.5, borderRadius: R, cursor: 'pointer', border: '2px solid', borderColor: tb === c.key ? '#f4511e' : 'rgba(0,0,0,0.06)', transition: 'all 0.2s', '&:hover': { borderColor: '#f4511e', boxShadow: '0 4px 24px rgba(244,81,30,0.12)' } }}>
          <Box sx={{ display: 'flex', flexDirection: isMobile ? 'column' : 'row', alignItems: isMobile ? 'center' : 'center', gap: isMobile ? 0.5 : 1.5, mb: 0.5, textAlign: isMobile ? 'center' : 'left' }}>
            <Box sx={{ color: tb === c.key ? '#f4511e' : 'text.secondary' }}>{c.icon}</Box>
            <Typography variant={isMobile ? 'caption' : 'subtitle1'} fontWeight={700}>{c.label}</Typography>
          </Box>
          {!isMobile && <Typography variant="caption" color="text.secondary">{c.desc}</Typography>}
        </Paper>
        
      ))}
    </Box>

    {/* ── Tab 内容面板 ── */}
    
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
                    sx={{ borderRadius: '10px', height: 20, fontSize: '0.7rem', backgroundColor: 'var(--color-background-info, #e3f2fd)' }}
                  />
                ))}
                {(p.method_names || []).map(n => (
                  <Chip
                    key={n}
                    label={n}
                    size="small"
                    color="info"
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

    {/* ── 2.5 事业部管理 (v0.4.24) ── */}
    {tb === 'divisions' && <Box>
      <Box sx={{ display: 'flex', justifyContent: 'flex-end', mb: 2 }}>
        <Button variant="contained" startIcon={<AddIcon />} onClick={() => {
          const maxSo = divs.length ? Math.max(...divs.map(d => d.sort_order)) : 0;
          setDivForm({ id: 0, name: '', sort_order: maxSo + 1, color: '#1976d2', group_ids: [] });
          setDivEditOpen(true);
        }} size="small" sx={{ borderRadius: R, background: 'linear-gradient(135deg,#1976d2,#1565c0)', boxShadow: '0 4px 14px rgba(25,118,210,0.3)' }}>新建部门</Button>
      </Box>
      <Typography variant="caption" color="text.secondary" sx={{ mb: 1, display: 'block' }}>按组织架构（部门/事业部）归拢实验室；删除部门采用软删除，关联实验室的归属将自动置空，可在管理页恢复。</Typography>
      {divs.length === 0 ? <Typography color="text.secondary" textAlign="center" sx={{ py: 4 }}>暂无部门</Typography>
        : <TableContainer component={Paper} sx={{ borderRadius: R, border: '1px solid rgba(0,0,0,0.06)' }}>
          <Table size="small">
            <TableHead><TableRow>
              <TableCell sx={{ fontWeight: 700 }}>名称</TableCell>
              <TableCell sx={{ fontWeight: 700 }}>排序</TableCell>
              <TableCell sx={{ fontWeight: 700 }}>下属实验室</TableCell>
              <TableCell sx={{ fontWeight: 700 }} align="right">操作</TableCell>
            </TableRow></TableHead>
            <TableBody>
              {divs.map(d => <TableRow key={d.id} hover>
                <TableCell>{d.name}</TableCell>
                <TableCell>{d.sort_order}</TableCell>
                <TableCell>{d.lab_count ?? 0}</TableCell>
                <TableCell align="right">
                  <IconButton size="small" onClick={() => {
                    const linkedGroupIds = gs.filter(g => g.division_id === d.id).map(g => g.id);
                    setDivForm({ id: d.id, name: d.name, sort_order: d.sort_order, color: d.color || '#1976d2', group_ids: linkedGroupIds });
                    setDivEditOpen(true);
                  }} sx={{ color: '#f4511e' }}><EditIcon fontSize="small" /></IconButton>
                  <IconButton size="small" color="error" onClick={() => { setCa(() => async () => { const r = await deleteDivision(d.id); if (r.code === 0) { sm('删除成功'); ldiv(); lg(); } else sm(r.message, true); setCo(false); }); setCo(true); }}><DeleteIcon fontSize="small" /></IconButton>
                </TableCell>
              </TableRow>)}
            </TableBody>
          </Table>
        </TableContainer>}
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
                    sx={{ borderRadius: '10px', height: 20, fontSize: '0.7rem', backgroundColor: 'var(--color-background-info, #e3f2fd)' }}
                  />
                ))}
              </Box>
            </InlineEditCard>
          ))}
    </Box>}

    {/* ── 回收站 ── */}
    {tb === 'trash' && <Box>
      <Typography variant="body2" color="text.secondary" sx={{ mb: 1, display: 'flex', alignItems: 'center', gap: 1 }}><DeleteSweepIcon fontSize="small" />回收站</Typography>
      <Box sx={{ display: 'flex', gap: 1, mb: 2, flexWrap: 'wrap' }}>
        <Chip label={`工作记录 (${tr.length})`} size="small" onClick={() => setTrashTab('records')} color={trashTab === 'records' ? 'primary' : 'default'} variant={trashTab === 'records' ? 'filled' : 'outlined'} sx={{ borderRadius: R }} />
        <Chip label={`检测类型 (${trTypes.length})`} size="small" onClick={() => setTrashTab('types')} color={trashTab === 'types' ? 'primary' : 'default'} variant={trashTab === 'types' ? 'filled' : 'outlined'} sx={{ borderRadius: R }} />
        <Chip label={`研发项目 (${trProjects.length})`} size="small" onClick={() => setTrashTab('projects')} color={trashTab === 'projects' ? 'primary' : 'default'} variant={trashTab === 'projects' ? 'filled' : 'outlined'} sx={{ borderRadius: R }} />
        <Chip label={`用户 (${trUsers.length})`} size="small" onClick={() => setTrashTab('users')} color={trashTab === 'users' ? 'primary' : 'default'} variant={trashTab === 'users' ? 'filled' : 'outlined'} sx={{ borderRadius: R }} />
      </Box>
      {trashTab === 'records' && (tr.length === 0 ? <Typography color="text.secondary" textAlign="center" sx={{ py: 4 }}>回收站为空</Typography>
        : <TableContainer component={Paper} className="table-responsive" sx={tSx}><Table size="small"><TableHead><TableRow>
          <TableCell sx={{ fontWeight: 600 }}>日期</TableCell><TableCell sx={{ fontWeight: 600 }}>项目</TableCell><TableCell sx={{ fontWeight: 600 }}>用户</TableCell><TableCell align="right" sx={{ fontWeight: 600 }}>数量</TableCell><TableCell align="right" sx={{ fontWeight: 600 }}>操作</TableCell>
        </TableRow></TableHead><TableBody>{tr.map(r => <TableRow key={r.id} hover><TableCell>{r.recorded_at}</TableCell><TableCell>{r.project_name}</TableCell><TableCell>{r.user_name}</TableCell><TableCell align="right">{r.quantity}</TableCell><TableCell align="right"><Button size="small" color="success" onClick={async () => { try { const res = await restoreRecord(r.id); if (res.code === 0) { sm('恢复成功'); loadTrash(); } else sm(res.message, true); } catch { sm('恢复失败', true); } }} sx={{ borderRadius: R }}>恢复</Button></TableCell></TableRow>)}</TableBody></Table></TableContainer>)}
      {trashTab === 'types' && (trTypes.length === 0 ? <Typography color="text.secondary" textAlign="center" sx={{ py: 4 }}>回收站为空</Typography>
        : <TableContainer component={Paper} className="table-responsive" sx={tSx}><Table size="small"><TableHead><TableRow>
          <TableCell sx={{ fontWeight: 600 }}>ID</TableCell><TableCell sx={{ fontWeight: 600 }}>类型标识</TableCell><TableCell sx={{ fontWeight: 600 }}>名称</TableCell><TableCell sx={{ fontWeight: 600 }}>创建时间</TableCell><TableCell align="right" sx={{ fontWeight: 600 }}>操作</TableCell>
        </TableRow></TableHead><TableBody>{trTypes.map(t => <TableRow key={t.id} hover><TableCell>{t.id}</TableCell><TableCell>{t.type_key}</TableCell><TableCell>{t.label}</TableCell><TableCell>{t.created_at}</TableCell><TableCell align="right"><Button size="small" color="success" onClick={async () => { try { const res = await updateSampleInfoType(t.id, { is_active: 1 }); if (res.code === 0) { sm('恢复成功'); loadTrash(); } else sm(res.message, true); } catch { sm('恢复失败', true); } }} sx={{ borderRadius: R, mr: 0.5 }}>恢复</Button><Button size="small" color="error" onClick={async () => { if (!window.confirm('确认彻底删除此类型？')) return; try { await deleteSampleInfoTypePermanent(t.id); sm('已彻底删除'); loadTrash(); } catch { sm('删除失败', true); } }} sx={{ borderRadius: R }}>彻底删除</Button></TableCell></TableRow>)}</TableBody></Table></TableContainer>)}
      {trashTab === 'projects' && (trProjects.length === 0 ? <Typography color="text.secondary" textAlign="center" sx={{ py: 4 }}>回收站为空</Typography>
        : <TableContainer component={Paper} className="table-responsive" sx={tSx}><Table size="small"><TableHead><TableRow>
          <TableCell sx={{ fontWeight: 600 }}>ID</TableCell><TableCell sx={{ fontWeight: 600 }}>名称</TableCell><TableCell sx={{ fontWeight: 600 }}>全称</TableCell><TableCell sx={{ fontWeight: 600 }}>创建时间</TableCell><TableCell align="right" sx={{ fontWeight: 600 }}>操作</TableCell>
        </TableRow></TableHead><TableBody>{trProjects.map(p => <TableRow key={p.id} hover><TableCell>{p.id}</TableCell><TableCell>{p.name}</TableCell><TableCell>{p.full_name || '-'}</TableCell><TableCell>{p.created_at}</TableCell><TableCell align="right"><Button size="small" color="success" onClick={async () => { try { const res = await updateProject(p.id, { is_active: true }); if (res.code === 0) { sm('恢复成功'); loadTrash(); } else sm(res.message, true); } catch { sm('恢复失败', true); } }} sx={{ borderRadius: R, mr: 0.5 }}>恢复</Button></TableCell></TableRow>)}</TableBody></Table></TableContainer>)}
      {trashTab === 'users' && (trUsers.length === 0 ? <Typography color="text.secondary" textAlign="center" sx={{ py: 4 }}>回收站为空</Typography>
        : <TableContainer component={Paper} className="table-responsive" sx={tSx}><Table size="small"><TableHead><TableRow>
          <TableCell sx={{ fontWeight: 600 }}>ID</TableCell><TableCell sx={{ fontWeight: 600 }}>用户名</TableCell><TableCell sx={{ fontWeight: 600 }}>角色</TableCell><TableCell sx={{ fontWeight: 600 }}>部门</TableCell><TableCell sx={{ fontWeight: 600 }}>实验室</TableCell><TableCell align="right" sx={{ fontWeight: 600 }}>操作</TableCell>
        </TableRow></TableHead><TableBody>{trUsers.map(u => <TableRow key={u.id} hover><TableCell>{u.id}</TableCell><TableCell>{u.username}</TableCell><TableCell>{u.role_id || '-'}</TableCell><TableCell>{u.division_name || '-'}</TableCell><TableCell>{u.group_name || '-'}</TableCell><TableCell align="right"><Button size="small" color="success" onClick={async () => { try { const res = await updateUser(u.id, { is_active: true }); if (res.code === 0) { sm('恢复成功'); loadTrash(); } else sm(res.message, true); } catch { sm('恢复失败', true); } }} sx={{ borderRadius: R, mr: 0.5 }}>恢复</Button><Button size="small" color="error" onClick={async () => { if (!window.confirm('确认彻底删除此用户？')) return; try { await deleteUserPermanent(u.id); sm('已彻底删除'); loadTrash(); } catch { sm('删除失败', true); } }} sx={{ borderRadius: R }}>彻底删除</Button></TableCell></TableRow>)}</TableBody></Table></TableContainer>)}
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

    {/* ── 样品信息登记管理 ── */}
    {tb === 'sampleinfo' && <Box>
      <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
        <Typography variant="subtitle1" fontWeight={700} color="#2e7d32">样品信息登记管理</Typography>
        <Button variant="contained" size="small" startIcon={<CloudUploadIcon />} onClick={doExportSi}
          sx={{ borderRadius: R, bgcolor: '#2e7d32', '&:hover': { bgcolor: '#1b5e20' } }}>导出 Excel</Button>
      </Box>

      {/* v0.4.28: 二级子卡片导航 */}
      {!siSubTab && (
        <Box sx={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(200px, 1fr))', gap: 2, mb: 3 }}>
          {[
            { key: 'types', label: '① 检测类型', desc: '管理检测类型', icon: <ScienceIcon sx={{ fontSize: 36, color: '#2e7d32' }} />, color: '#2e7d32' },
            { key: 'records', label: '② 记录查询', desc: '查询/编辑记录', icon: <ListAltIcon sx={{ fontSize: 36, color: '#1976d2' }} />, color: '#1976d2' },
            { key: 'columns', label: '③ 自定义列', desc: '配置字段列', icon: <ViewWeekIcon sx={{ fontSize: 36, color: '#f57c00' }} />, color: '#f57c00' },
            { key: 'stats', label: '④ 独立统计', desc: '数据统计概览', icon: <AssessmentIcon sx={{ fontSize: 36, color: '#7b1fa2' }} />, color: '#7b1fa2' },
            { key: 'rd_columns', label: '⑤ 研发送样列配置', desc: '配置研发送样表格列', icon: <ViewWeekIcon sx={{ fontSize: 36, color: '#e65100' }} />, color: '#e65100' },
          ].map(sc => (
            <Paper key={sc.key} elevation={0} onClick={() => setSiSubTab(sc.key)}
              sx={{ p: 2.5, borderRadius: R, cursor: 'pointer', border: '1px solid rgba(0,0,0,0.08)',
                transition: 'all 0.2s', textAlign: 'center',
                '&:hover': { borderColor: sc.color, boxShadow: '0 4px 20px rgba(0,0,0,0.08)', transform: 'translateY(-2px)' } }}>
              <Box sx={{ mb: 1 }}>{sc.icon}</Box>
              <Typography variant="subtitle1" fontWeight={700}>{sc.label}</Typography>
              <Typography variant="caption" color="text.secondary">{sc.desc}</Typography>
            </Paper>
          ))}
        </Box>
      )}
      {siSubTab && (
        <Box sx={{ mb: 2 }}>
          <Button size="small" startIcon={<ArrowBackIcon />} onClick={() => setSiSubTab(null)}
            sx={{ borderRadius: R, mb: 1 }}>返回样品登记管理</Button>
        </Box>
      )}

      {siSubTab === 'types' && (<>
      {/* ① 检测类型 CRUD */}
      <Paper elevation={0} sx={{ p: 2, mb: 3, borderRadius: R, border: '1px solid rgba(0,0,0,0.08)' }}>
        <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 1.5 }}>
          <Typography variant="subtitle2" fontWeight={700}>① 检测类型</Typography>
          <Button variant="outlined" size="small" startIcon={<AddIcon />} onClick={() => { setSiTypeEdit(null); setSiTypeNewOpen(true); setSiTypeForm({ type_key: '', label: '', description: '', color: '#2e7d32', sort_order: (activeSiTypes.length + 1), is_active: 1 }); }}
            sx={{ borderRadius: R, borderColor: '#2e7d32', color: '#2e7d32' }}>新建类型</Button>
        </Box>

        {/* 新建/编辑表单 */}
        {siTypeEdit !== null || siTypeNewOpen ? (
          <Box sx={{ display: 'flex', gap: 1.5, flexWrap: 'wrap', alignItems: 'center', mb: 2, p: 1.5, bgcolor: '#f5f9f5', borderRadius: R }}>
            <TextField label="类型标识(type_key)" size="small" value={siTypeForm.type_key} onChange={e => setSiTypeForm(p => ({ ...p, type_key: e.target.value }))} sx={{ width: 160 }} />
            <TextField label="名称(label)" size="small" value={siTypeForm.label} onChange={e => setSiTypeForm(p => ({ ...p, label: e.target.value }))} sx={{ width: 140 }} />
            <TextField label="描述" size="small" value={siTypeForm.description} onChange={e => setSiTypeForm(p => ({ ...p, description: e.target.value }))} sx={{ width: 180 }} />
            <TextField label="排序" size="small" type="number" value={siTypeForm.sort_order} onChange={e => setSiTypeForm(p => ({ ...p, sort_order: Number(e.target.value) }))} sx={{ width: 80 }} />
            <TextField label="颜色色值" size="small" value={siTypeForm.color} onChange={e => setSiTypeForm(p => ({ ...p, color: e.target.value }))} sx={{ width: 120 }} InputProps={{ startAdornment: <Box sx={{ width: 16, height: 16, mr: 1, borderRadius: '4px', bgcolor: siTypeForm.color, border: '1px solid #ccc' }} /> }} />
            <FormControlLabel control={<Switch checked={siTypeForm.is_active === 1} onChange={e => setSiTypeForm(p => ({ ...p, is_active: e.target.checked ? 1 : 0 }))} />} label="启用" />
            <Button variant="contained" size="small" onClick={saveSiType} sx={{ borderRadius: R, bgcolor: '#2e7d32' }}>保存</Button>
            <Button size="small" onClick={() => { setSiTypeEdit(null); setSiTypeNewOpen(false); setSiTypeForm({ type_key: '', label: '', description: '', color: '#2e7d32', sort_order: 0, is_active: 1 }); }} sx={{ borderRadius: R }}>取消</Button>
          </Box>
        ) : null}

        <TableContainer component={Paper} sx={tSx}>
          <Table size="small">
            <TableHead><TableRow>
              <TableCell sx={{ fontWeight: 600 }}>名称</TableCell>
              <TableCell sx={{ fontWeight: 600 }}>标识</TableCell>
              <TableCell sx={{ fontWeight: 600 }}>描述</TableCell>
              <TableCell sx={{ fontWeight: 600 }}>颜色</TableCell>
              <TableCell sx={{ fontWeight: 600 }}>排序</TableCell>
              <TableCell sx={{ fontWeight: 600 }}>启用</TableCell>
              <TableCell align="right" sx={{ fontWeight: 600 }}>操作</TableCell>
            </TableRow></TableHead>
            <TableBody>
              {activeSiTypes.length === 0 ? (
                <TableRow><TableCell colSpan={7} align="center" sx={{ color: '#999', py: 3 }}>暂无检测类型</TableCell></TableRow>
              ) : activeSiTypes.map(t => (
                <TableRow key={t.id} hover>
                  <TableCell>{t.label}</TableCell>
                  <TableCell>{t.type_key}</TableCell>
                  <TableCell sx={{ maxWidth: 220 }}>{t.description}</TableCell>
                  <TableCell><Box sx={{ width: 18, height: 18, borderRadius: '4px', bgcolor: t.color, border: '1px solid #ccc' }} /></TableCell>
                  <TableCell>{t.sort_order}</TableCell>
                  <TableCell><Chip label={t.is_active ? '启用' : '已停用'} size="small" color={t.is_active ? 'success' : 'error'} variant={t.is_active ? 'filled' : 'outlined'} /></TableCell>
                  <TableCell align="right">
                    <Box sx={{ display: 'flex', gap: 0.5, justifyContent: 'flex-end' }}>
                      <IconButton size="small" onClick={() => editSiType(t)} sx={{ color: '#2e7d32' }}><EditIcon fontSize="small" /></IconButton>
                      {t.is_active ? (
                        <IconButton size="small" color="error" title="软删除（设为停用）" onClick={() => delSiType(t.id)}><DeleteIcon fontSize="small" /></IconButton>
                      ) : (
                        <IconButton size="small" color="primary" title="恢复启用" onClick={() => restoreSiType(t.id)}><RefreshIcon fontSize="small" /></IconButton>
                      )}
                    </Box>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      </Paper>
      </>)}

      {siSubTab === 'records' && (<>
      {/* 筛选区 */}
      <Paper elevation={0} sx={{ p: 2, mb: 3, borderRadius: R, border: '1px solid rgba(0,0,0,0.08)' }}>
        <Typography variant="subtitle2" fontWeight={700} sx={{ mb: 1.5 }}>记录查询（共 {siTotal} 条）</Typography>
        <Box sx={{ display: 'flex', gap: 1.5, flexWrap: 'wrap', mb: 2, alignItems: 'center' }}>
          <TextField label="起始日期" type="date" size="small" value={siFilters.start} onChange={e => setSiFilters(p => ({ ...p, start: e.target.value }))} InputLabelProps={{ shrink: true }} />
          <TextField label="截止日期" type="date" size="small" value={siFilters.end} onChange={e => setSiFilters(p => ({ ...p, end: e.target.value }))} InputLabelProps={{ shrink: true }} />
          <TextField label="送样人" size="small" value={siFilters.user_name} onChange={e => setSiFilters(p => ({ ...p, user_name: e.target.value }))} />
          <TextField label="实验室" size="small" value={siFilters.lab_name} onChange={e => setSiFilters(p => ({ ...p, lab_name: e.target.value }))} />
          <TextField label="项目" size="small" value={siFilters.project_name} onChange={e => setSiFilters(p => ({ ...p, project_name: e.target.value }))} />
          <FormControl size="small" sx={{ minWidth: 120 }}>
            <InputLabel>检测类型</InputLabel>
            <Select value={siFilters.type_key} label="检测类型" onChange={e => setSiFilters(p => ({ ...p, type_key: e.target.value }))}>
              <MenuItem value="">全部</MenuItem>
              {siTypes.map(t => <MenuItem key={t.id} value={t.type_key}>{t.label}</MenuItem>)}
            </Select>
          </FormControl>
          <FormControl size="small" sx={{ minWidth: 110 }}>
            <InputLabel>状态</InputLabel>
            <Select value={siFilters.status} label="状态" onChange={e => setSiFilters(p => ({ ...p, status: e.target.value }))}>
              {['', '待检测', '待取样', '已取样', '检测完成'].map(s => <MenuItem key={s} value={s}>{s === '' ? '全部' : s}</MenuItem>)}
            </Select>
          </FormControl>
          <Button variant="contained" size="small" onClick={() => { setSiPage(0); }} sx={{ borderRadius: R, bgcolor: '#2e7d32' }}>查询</Button>
          <Button size="small" onClick={() => { setSiFilters({ start: '', end: '', user_name: '', lab_name: '', project_name: '', type_key: '', status: '' }); setSiPage(0); }} sx={{ borderRadius: R }}>重置</Button>
        </Box>

        <TableContainer component={Paper} sx={tSx}>
          <Table size="small">
            <TableHead><TableRow>
              <TableCell sx={{ fontWeight: 600 }}>序号</TableCell>
              <TableCell sx={{ fontWeight: 600 }}>批号</TableCell>
              <TableCell sx={{ fontWeight: 600 }}>送样人</TableCell>
              <TableCell sx={{ fontWeight: 600 }}>实验室</TableCell>
              <TableCell sx={{ fontWeight: 600 }}>项目</TableCell>
              <TableCell sx={{ fontWeight: 600 }}>检测类型</TableCell>
              <TableCell sx={{ fontWeight: 600 }}>状态</TableCell>
              <TableCell sx={{ fontWeight: 600 }}>检测时间</TableCell>
              <TableCell sx={{ fontWeight: 600 }}>主要成分</TableCell>
              <TableCell align="right" sx={{ fontWeight: 600 }}>操作</TableCell>
            </TableRow></TableHead>
            <TableBody>
              {siRecords.length === 0 ? (
                <TableRow><TableCell colSpan={10} align="center" sx={{ color: '#999', py: 3 }}>暂无记录</TableCell></TableRow>
              ) : siRecords.map(r => (
                <TableRow key={r.id} hover>
                  {siEditId === r.id ? (
                    <>
                      <TableCell>#{r.seq_no}</TableCell>
                      <TableCell><TextField size="small" value={siEditForm.batch_no || ''} onChange={e => setSiEditForm(p => ({ ...p, batch_no: e.target.value }))} /></TableCell>
                      <TableCell><TextField size="small" value={siEditForm.user_name || ''} onChange={e => setSiEditForm(p => ({ ...p, user_name: e.target.value }))} /></TableCell>
                      <TableCell><TextField size="small" value={siEditForm.lab_name || ''} onChange={e => setSiEditForm(p => ({ ...p, lab_name: e.target.value }))} /></TableCell>
                      <TableCell><TextField size="small" value={siEditForm.project_name || ''} onChange={e => setSiEditForm(p => ({ ...p, project_name: e.target.value }))} /></TableCell>
                      <TableCell>{r.detection_type}</TableCell>
                      <TableCell>
                        <FormControl size="small">
                          <Select value={siEditForm.status || ''} onChange={e => setSiEditForm(p => ({ ...p, status: e.target.value }))}>
                            {['待检测', '待取样', '已取样', '检测完成'].map(s => <MenuItem key={s} value={s}>{s}</MenuItem>)}
                          </Select>
                        </FormControl>
                      </TableCell>
                      <TableCell>{r.detection_date || '-'}</TableCell>
                      <TableCell><TextField size="small" value={siEditForm.main_components || ''} onChange={e => setSiEditForm(p => ({ ...p, main_components: e.target.value }))} /></TableCell>
                      <TableCell align="right">
                        <Box sx={{ display: 'flex', gap: 0.5, justifyContent: 'flex-end' }}>
                          <Button size="small" variant="contained" onClick={() => saveSiEdit(r.id)} sx={{ borderRadius: R, fontSize: '0.7rem', py: 0 }}>保存</Button>
                          <Button size="small" onClick={() => { setSiEditId(null); setSiEditForm({}); }} sx={{ borderRadius: R, fontSize: '0.7rem', py: 0 }}>取消</Button>
                        </Box>
                      </TableCell>
                    </>
                  ) : (
                    <>
                      <TableCell>#{r.seq_no}</TableCell>
                      <TableCell>{r.batch_no}</TableCell>
                      <TableCell>{r.user_name}</TableCell>
                      <TableCell>{r.lab_name}</TableCell>
                      <TableCell>{r.project_name}</TableCell>
                      <TableCell>{r.detection_type}</TableCell>
                      <TableCell><Chip label={r.status} size="small" color={r.status === '检测完成' ? 'default' : 'warning'} /></TableCell>
                      <TableCell>{r.detection_date || '-'}</TableCell>
                      <TableCell sx={{ maxWidth: 200 }}>{r.main_components}</TableCell>
                      <TableCell align="right">
                        <Box sx={{ display: 'flex', gap: 0.5, justifyContent: 'flex-end' }}>
                          <IconButton size="small" onClick={() => openSiEdit(r)} sx={{ color: '#2e7d32' }}><EditIcon fontSize="small" /></IconButton>
                          <IconButton size="small" color="error" onClick={() => delSiRecord(r.id)}><DeleteIcon fontSize="small" /></IconButton>
                        </Box>
                      </TableCell>
                    </>
                  )}
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
        {siTotal > 50 && (
          <Box sx={{ display: 'flex', gap: 1, justifyContent: 'flex-end', mt: 1 }}>
            <Button size="small" disabled={siPage === 0} onClick={() => setSiPage(p => Math.max(0, p - 1))} sx={{ borderRadius: R }}>上一页</Button>
            <Button size="small" disabled={(siPage + 1) * 50 >= siTotal} onClick={() => setSiPage(p => p + 1)} sx={{ borderRadius: R }}>下一页</Button>
          </Box>
        )}
      </Paper>
      </>)}

      {siSubTab === 'columns' && (<>
      {/* ④ 自定义列配置 */}
      <Paper elevation={0} sx={{ p: 2, mb: 3, borderRadius: R, border: '1px solid rgba(0,0,0,0.08)' }}>
        <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 1.5 }}>
          <Typography variant="subtitle2" fontWeight={700} color="#2e7d32">④ 自定义列配置</Typography>
          <Box sx={{ display: 'flex', gap: 1 }}>
            <Button variant="outlined" size="small" startIcon={<AddIcon />} onClick={() => {
              setColEditItem(null);
              const maxSo = siColumns.length ? Math.max(...siColumns.map(c => c.sort_order)) : 0;
              setColForm({ field_key: '', label: '', data_type: 'text', width: 100, sort_order: maxSo + 1, options: '', is_required: false, show_in_list: true, show_in_export: true, show_in_form: true });
              setColEditOpen(true);
            }} sx={{ borderRadius: R, borderColor: '#2e7d32', color: '#2e7d32' }}>新增列</Button>
          </Box>
        </Box>
        <Typography variant="caption" color="text.secondary" sx={{ mb: 1, display: 'block' }}>
          管理样品信息登记页面显示的列字段。预置字段不可删除，自定义字段可增删改。
          v0.4.28: 列配置对所有检测类型统一生效。
        </Typography>
        {siColumns.length === 0 ? (
          <Typography color="text.secondary" textAlign="center" sx={{ py: 3 }}>暂无列配置</Typography>
        ) : (
          <TableContainer component={Paper} sx={tSx}>
            <Table size="small">
              <TableHead><TableRow>
                <TableCell sx={{ fontWeight: 600, width: 60 }}>排序</TableCell>
                <TableCell sx={{ fontWeight: 600 }}>字段标识</TableCell>
                <TableCell sx={{ fontWeight: 600 }}>显示名称</TableCell>
                <TableCell sx={{ fontWeight: 600 }}>类型</TableCell>
                <TableCell sx={{ fontWeight: 600 }}>显示类型</TableCell>
                <TableCell sx={{ fontWeight: 600 }}>必填</TableCell>
                <TableCell sx={{ fontWeight: 600 }}>启用</TableCell>
                <TableCell sx={{ fontWeight: 600 }}>表单</TableCell>
                <TableCell sx={{ fontWeight: 600 }}>列表</TableCell>
                <TableCell sx={{ fontWeight: 600 }}>导出</TableCell>
                <TableCell sx={{ fontWeight: 600 }}>宽度</TableCell>
                <TableCell align="right" sx={{ fontWeight: 600 }}>操作</TableCell>
              </TableRow></TableHead>
              <TableBody>
                {siColumns.map((col, idx) => (
                  <TableRow key={col.id} hover>
                    <TableCell>
                      <Box sx={{ display: 'flex', gap: 0.5 }}>
                        <IconButton size="small" disabled={idx === 0} onClick={() => moveCol(idx, -1)} sx={{ p: 0.3 }}>
                          <Typography variant="caption" sx={{ fontWeight: 700 }}>▲</Typography>
                        </IconButton>
                        <IconButton size="small" disabled={idx === siColumns.length - 1} onClick={() => moveCol(idx, 1)} sx={{ p: 0.3 }}>
                          <Typography variant="caption" sx={{ fontWeight: 700 }}>▼</Typography>
                        </IconButton>
                      </Box>
                    </TableCell>
                    <TableCell sx={{ fontFamily: 'monospace', fontSize: '0.8rem' }}>{col.field_key}</TableCell>
                    <TableCell>{col.label}</TableCell>
                    <TableCell><Chip label={col.data_type} size="small" variant="outlined" sx={{ borderRadius: R, fontSize: '0.7rem' }} /></TableCell>
                    <TableCell>
                      <Box sx={{ display: 'flex', gap: 0.3, flexWrap: 'wrap', cursor: 'pointer' }}
                        onClick={(e) => {
                          setVisibleTypesColId(col.id);
                          setVisibleTypesSelected(col.visible_types ? [...col.visible_types] : []);
                          setVisibleTypesAnchorEl(e.currentTarget);
                        }}>
                        {col.visible_types && col.visible_types.length > 0 ? (
                          <>
                            {col.visible_types.slice(0, 3).map(tk => {
                              const st = sampleTypes.find(s => s.type_key === tk);
                              return <Chip key={tk} label={st?.label || tk} size="small" variant="outlined" sx={{ borderRadius: R, fontSize: '0.65rem', height: 20 }} />;
                            })}
                            {col.visible_types.length > 3 && (
                              <Chip label={`+${col.visible_types.length - 3}`} size="small" sx={{ borderRadius: R, fontSize: '0.65rem', height: 20, bgcolor: '#e0e0e0' }} />
                            )}
                          </>
                        ) : (
                          <Typography variant="caption" color="text.disabled">全部</Typography>
                        )}
                      </Box>
                    </TableCell>
                    <TableCell><Chip label={col.is_required ? '是' : '否'} size="small" color={col.is_required ? 'warning' : 'default'} variant="outlined" sx={{ borderRadius: R }} /></TableCell>
                    <TableCell><Chip label={col.is_active ? '启用' : '停用'} size="small" color={col.is_active ? 'success' : 'default'} sx={{ borderRadius: R }} /></TableCell>
                    <TableCell><Chip label={col.show_in_form ? '显示' : '隐藏'} size="small" variant="outlined" sx={{ borderRadius: R }} /></TableCell>
                    <TableCell><Chip label={col.show_in_list ? '显示' : '隐藏'} size="small" variant="outlined" sx={{ borderRadius: R }} /></TableCell>
                    <TableCell><Chip label={col.show_in_export ? '显示' : '隐藏'} size="small" variant="outlined" sx={{ borderRadius: R }} /></TableCell>
                    <TableCell>{col.width}</TableCell>
                    <TableCell align="right">
                      <Box sx={{ display: 'flex', gap: 0.5, justifyContent: 'flex-end' }}>
                        <IconButton size="small" onClick={() => editCol(col)} sx={{ color: '#2e7d32' }}><EditIcon fontSize="small" /></IconButton>
                        <IconButton size="small" color="error" onClick={() => delCol(col.id)} disabled={col.is_predefined}><DeleteIcon fontSize="small" /></IconButton>
                      </Box>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </TableContainer>
        )}
      </Paper>
      </>)}

      {/* 显示类型编辑弹窗 */}
      <Popover open={!!visibleTypesAnchorEl} anchorEl={visibleTypesAnchorEl}
        onClose={() => setVisibleTypesAnchorEl(null)}
        anchorOrigin={{ vertical: 'bottom', horizontal: 'left' }}
        slotProps={{ paper: { sx: { borderRadius: R, p: 1.5, minWidth: 200 } } }}>
        <Typography variant="subtitle2" sx={{ mb: 1, fontWeight: 600, fontSize: '0.8rem' }}>选择可见检测类型</Typography>
        <List dense sx={{ maxHeight: 240, overflow: 'auto', py: 0 }}>
          {sampleTypes.map(st => (
            <ListItem key={st.type_key} disableGutters sx={{ py: 0 }}>
              <ListItemIcon sx={{ minWidth: 36 }}>
                <Checkbox size="small" checked={visibleTypesSelected.includes(st.type_key)}
                  onChange={() => {
                    setVisibleTypesSelected(prev =>
                      prev.includes(st.type_key) ? prev.filter(t => t !== st.type_key) : [...prev, st.type_key]
                    );
                  }} />
              </ListItemIcon>
              <ListItemText primary={st.label} primaryTypographyProps={{ fontSize: '0.8rem' }} />
            </ListItem>
          ))}
        </List>
        <Box sx={{ display: 'flex', gap: 0.5, justifyContent: 'flex-end', mt: 1 }}>
          <Button size="small" onClick={() => setVisibleTypesAnchorEl(null)} sx={{ borderRadius: R, fontSize: '0.75rem' }}>取消</Button>
          <Button size="small" variant="contained" sx={{ borderRadius: R, fontSize: '0.75rem', bgcolor: '#2e7d32' }}
            onClick={async () => {
              if (visibleTypesColId === null) return;
              try {
                await fetch('/api/sample-info/columns/' + visibleTypesColId + '/types', {
                  method: 'PUT',
                  headers: { 'Content-Type': 'application/json' },
                  body: JSON.stringify({ type_keys: visibleTypesSelected }),
                });
                setSiColumns(prev => prev.map(c => c.id === visibleTypesColId ? { ...c, visible_types: visibleTypesSelected } : c));
                setVisibleTypesAnchorEl(null);
                sm('显示类型更新成功');
              } catch (e: any) {
                sm(e.message || '更新失败', true);
              }
            }}>保存</Button>
        </Box>
      </Popover>

      {/* 列编辑弹窗 */}
      <Dialog open={colEditOpen} onClose={() => { setColEditOpen(false); setColEditItem(null); }} maxWidth="sm" fullWidth PaperProps={{ sx: { borderRadius: R } }}>
        <DialogTitle sx={{ fontWeight: 700 }}>{colEditItem ? '编辑列' : '新增自定义列'}</DialogTitle>
        <DialogContent>
          <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2, mt: 1 }}>
            {!colEditItem && (
              <TextField label="字段标识 (field_key)" fullWidth size="small" value={colForm.field_key}
                onChange={e => setColForm(p => ({ ...p, field_key: e.target.value }))}
                helperText="英文字母或下划线，创建后不可修改" />
            )}
            <TextField label="显示名称 (label)" fullWidth size="small" value={colForm.label}
              onChange={e => setColForm(p => ({ ...p, label: e.target.value }))} />
            <FormControl size="small" fullWidth>
              <InputLabel>数据类型</InputLabel>
              <Select value={colForm.data_type} label="数据类型" onChange={e => setColForm(p => ({ ...p, data_type: e.target.value }))}>
                <MenuItem value="text">文本 (text)</MenuItem>
                <MenuItem value="number">数字 (number)</MenuItem>
                <MenuItem value="select">下拉 (select)</MenuItem>
                <MenuItem value="date">日期 (date)</MenuItem>
                <MenuItem value="attachment">附件 (attachment)</MenuItem>
              </Select>
            </FormControl>
            {colForm.data_type === 'select' && (
              <TextField label="下拉选项（逗号分隔）" fullWidth size="small" value={colForm.options}
                onChange={e => setColForm(p => ({ ...p, options: e.target.value }))} />
            )}
            <Box sx={{ display: 'flex', gap: 2 }}>
              <TextField label="宽度" type="number" size="small" value={colForm.width}
                onChange={e => {
                  const v = Number(e.target.value);
                  setColForm(p => ({ ...p, width: v || 100 }));
                }} sx={{ width: 100 }} />
              <TextField label="排序" type="number" size="small" value={colForm.sort_order}
                onChange={e => {
                  const v = Number(e.target.value);
                  setColForm(p => ({ ...p, sort_order: v || 0 }));
                }} sx={{ width: 100 }} />
            </Box>
            <FormControlLabel control={<Switch checked={colForm.is_required} onChange={e => setColForm(p => ({ ...p, is_required: e.target.checked }))} />} label="必填" />
            <Box sx={{ display: 'flex', gap: 2 }}>
              <FormControlLabel control={<Switch checked={colForm.show_in_form} onChange={e => setColForm(p => ({ ...p, show_in_form: e.target.checked }))} />} label="在表单中显示" />
              <FormControlLabel control={<Switch checked={colForm.show_in_list} onChange={e => setColForm(p => ({ ...p, show_in_list: e.target.checked }))} />} label="在列表中显示" />
              <FormControlLabel control={<Switch checked={colForm.show_in_export} onChange={e => setColForm(p => ({ ...p, show_in_export: e.target.checked }))} />} label="在导出中显示" />
            </Box>
          </Box>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => { setColEditOpen(false); setColEditItem(null); }} sx={{ borderRadius: R }}>取消</Button>
          <Button onClick={saveCol} variant="contained" sx={{ borderRadius: R, bgcolor: '#2e7d32' }}>保存</Button>
        </DialogActions>
      </Dialog>

      {siSubTab === 'stats' && (<>
      {/* ③ 独立统计 */}
      <Paper elevation={0} sx={{ p: 2, borderRadius: R, border: '1px solid rgba(0,0,0,0.08)' }}>
        <Typography variant="subtitle2" fontWeight={700} sx={{ mb: 1.5 }}>③ 独立统计（不接分析检测 /stats）</Typography>
        {!siStats ? (
          <Typography color="text.secondary">加载中…</Typography>
        ) : (
          <Grid container spacing={2}>
            <Grid item xs={12} sm={6} md={4}>
              <Typography variant="caption" fontWeight={700} color="text.secondary">总记录数</Typography>
              <Typography variant="h4" fontWeight={800} color="#2e7d32">{siStats.total}</Typography>
            </Grid>
            <Grid item xs={12} sm={6} md={4}>
              <Typography variant="caption" fontWeight={700} color="text.secondary">按状态</Typography>
              <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 0.5, mt: 0.5 }}>
                {siStats.by_status.map((s: any) => <Chip key={s.name} label={`${s.name}: ${s.count}`} size="small" variant="outlined" sx={{ borderRadius: R }} />)}
              </Box>
            </Grid>
            <Grid item xs={12} sm={6} md={4}>
              <Typography variant="caption" fontWeight={700} color="text.secondary">按检测类型</Typography>
              <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 0.5, mt: 0.5 }}>
                {siStats.by_type.map((s: any) => <Chip key={s.type_key} label={`${s.label}: ${s.count}`} size="small" variant="outlined" sx={{ borderRadius: R }} />)}
              </Box>
            </Grid>
            <Grid item xs={12} sm={6} md={4}>
              <Typography variant="caption" fontWeight={700} color="text.secondary">按实验室</Typography>
              <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 0.5, mt: 0.5 }}>
                {siStats.by_lab.map((s: any) => <Chip key={s.name} label={`${s.name}: ${s.count}`} size="small" variant="outlined" sx={{ borderRadius: R }} />)}
              </Box>
            </Grid>
            <Grid item xs={12} sm={6} md={4}>
              <Typography variant="caption" fontWeight={700} color="text.secondary">按项目</Typography>
              <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 0.5, mt: 0.5 }}>
                {siStats.by_project.map((s: any) => <Chip key={s.name} label={`${s.name}: ${s.count}`} size="small" variant="outlined" sx={{ borderRadius: R }} />)}
              </Box>
            </Grid>
            <Grid item xs={12} sm={6} md={4}>
              <Typography variant="caption" fontWeight={700} color="text.secondary">按送样人</Typography>
              <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 0.5, mt: 0.5 }}>
                {siStats.by_user.map((s: any) => <Chip key={s.name} label={`${s.name}: ${s.count}`} size="small" variant="outlined" sx={{ borderRadius: R }} />)}
              </Box>
            </Grid>
            <Grid item xs={12}>
              <Typography variant="caption" fontWeight={700} color="text.secondary">按月份</Typography>
              <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 0.5, mt: 0.5 }}>
                {siStats.by_month.map((s: any) => <Chip key={s.month} label={`${s.month}: ${s.count}`} size="small" variant="outlined" sx={{ borderRadius: R }} />)}
              </Box>
            </Grid>
          </Grid>
        )}
      </Paper>
      </>)}

      {siSubTab === 'rd_columns' && <AdminRdRecordColumns />}

    </Box>}

    {/* ── 主题设置 (v0.4.35) ── */}
    {tb === 'theme' && <Box>
      <Typography variant="subtitle1" fontWeight={700} sx={{ mb: 2 }}>⑧ 主题设置</Typography>
      <Paper elevation={0} sx={{ p: 3, borderRadius: R, border: '1px solid rgba(0,0,0,0.08)', maxWidth: 600 }}>
        <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2.5 }}>
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
            <Typography variant="body2" sx={{ minWidth: 100, fontWeight: 600 }}>主色</Typography>
            <input type="color" value={themeForm.primaryColor} onChange={e => setThemeForm(p => ({ ...p, primaryColor: e.target.value }))} style={{ width: 48, height: 36, border: 'none', cursor: 'pointer', padding: 0 }} />
            <TextField size="small" value={themeForm.primaryColor} onChange={e => setThemeForm(p => ({ ...p, primaryColor: e.target.value }))} sx={{ '& .MuiOutlinedInput-root': { borderRadius: R }, width: 180 }} />
          </Box>
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
            <Typography variant="body2" sx={{ minWidth: 100, fontWeight: 600 }}>辅色</Typography>
            <input type="color" value={themeForm.secondaryColor} onChange={e => setThemeForm(p => ({ ...p, secondaryColor: e.target.value }))} style={{ width: 48, height: 36, border: 'none', cursor: 'pointer', padding: 0 }} />
            <TextField size="small" value={themeForm.secondaryColor} onChange={e => setThemeForm(p => ({ ...p, secondaryColor: e.target.value }))} sx={{ '& .MuiOutlinedInput-root': { borderRadius: R }, width: 180 }} />
          </Box>
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
            <Typography variant="body2" sx={{ minWidth: 100, fontWeight: 600 }}>背景色</Typography>
            <input type="color" value={themeForm.bgColor} onChange={e => setThemeForm(p => ({ ...p, bgColor: e.target.value }))} style={{ width: 48, height: 36, border: 'none', cursor: 'pointer', padding: 0 }} />
            <TextField size="small" value={themeForm.bgColor} onChange={e => setThemeForm(p => ({ ...p, bgColor: e.target.value }))} sx={{ '& .MuiOutlinedInput-root': { borderRadius: R }, width: 180 }} />
          </Box>
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
            <Typography variant="body2" sx={{ minWidth: 100, fontWeight: 600 }}>圆角</Typography>
            <input type="range" min={0} max={16} value={themeForm.cardRadius} onChange={e => setThemeForm(p => ({ ...p, cardRadius: Number(e.target.value) }))} style={{ flex: 1 }} />
            <Typography variant="body2" sx={{ minWidth: 30, textAlign: 'right' }}>{themeForm.cardRadius}px</Typography>
          </Box>
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
            <Typography variant="body2" sx={{ minWidth: 100, fontWeight: 600 }}>登录背景</Typography>
            <TextField size="small" fullWidth value={themeForm.loginBg} onChange={e => setThemeForm(p => ({ ...p, loginBg: e.target.value }))} sx={{ '& .MuiOutlinedInput-root': { borderRadius: R } }} />
          </Box>
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
            <Typography variant="body2" sx={{ minWidth: 100, fontWeight: 600 }}>登录按钮色</Typography>
            <input type="color" value={themeForm.loginButtonColor} onChange={e => setThemeForm(p => ({ ...p, loginButtonColor: e.target.value }))} style={{ width: 48, height: 36, border: 'none', cursor: 'pointer', padding: 0 }} />
            <TextField size="small" value={themeForm.loginButtonColor} onChange={e => setThemeForm(p => ({ ...p, loginButtonColor: e.target.value }))} sx={{ '& .MuiOutlinedInput-root': { borderRadius: R }, width: 180 }} />
          </Box>
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
            <Typography variant="body2" sx={{ minWidth: 100, fontWeight: 600 }}>品牌文字</Typography>
            <TextField size="small" value={themeForm.logoText} onChange={e => setThemeForm(p => ({ ...p, logoText: e.target.value }))} sx={{ '& .MuiOutlinedInput-root': { borderRadius: R }, width: 180 }} />
          </Box>
          <Box sx={{ display: 'flex', gap: 1, justifyContent: 'flex-end', mt: 1 }}>
            <Button variant="contained" onClick={async () => {
              setSettingsLoading(true);
              try {
                await updateSetting('theme', themeForm);
                sm('主题设置已保存（刷新页面后生效）');
              } catch { sm('保存失败', true); }
              setSettingsLoading(false);
            }} disabled={settingsLoading} sx={{ borderRadius: R }}>
              {settingsLoading ? <CircularProgress size={20} /> : '保存主题设置'}
            </Button>
          </Box>
        </Box>
      </Paper>
    </Box>}

    {/* ── 首页配置 (v0.4.35) ── */}
    {tb === 'home' && <Box>
      <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
        <Typography variant="subtitle1" fontWeight={700}>⑨ 首页配置</Typography>
        <Button variant="outlined" size="small" startIcon={<AddIcon />} onClick={() => {
          setHomeCardsForm(prev => [...prev, { key: '', title: '', subtitle: '', path: '', perm: '', icon: 'Science', gradient: 'linear-gradient(145deg,#fff3e0,#ffe0b2)', border: '#e65100', titleColor: '#e65100' }]);
        }} sx={{ borderRadius: R }}>新增卡片</Button>
      </Box>
      <Typography variant="caption" color="text.secondary" sx={{ mb: 2, display: 'block' }}>编辑首页 3 张入口卡片。修改后保存生效。</Typography>
      {homeCardsForm.length === 0 ? (
        <Typography color="text.secondary" textAlign="center" sx={{ py: 3 }}>暂无卡片配置</Typography>
      ) : (
        <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1.5 }}>
          {homeCardsForm.map((card, idx) => (
            <Paper key={idx} elevation={0} sx={{ p: 2, borderRadius: R, border: '1px solid rgba(0,0,0,0.08)' }}>
              <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 1.5 }}>
                <Typography variant="subtitle2" fontWeight={700} sx={{ mr: 1 }}>卡片 {idx + 1}</Typography>
                <IconButton size="small" color="error" onClick={() => setHomeCardsForm(prev => prev.filter((_, i) => i !== idx))}><DeleteIcon fontSize="small" /></IconButton>
              </Box>
              <Box sx={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 1.5 }}>
                <TextField label="标识 (key)" size="small" value={card.key} onChange={e => { const v = e.target.value; setHomeCardsForm(prev => prev.map((c, i) => i === idx ? { ...c, key: v } : c)); }} sx={{ '& .MuiOutlinedInput-root': { borderRadius: R } }} />
                <TextField label="标题" size="small" value={card.title} onChange={e => { const v = e.target.value; setHomeCardsForm(prev => prev.map((c, i) => i === idx ? { ...c, title: v } : c)); }} sx={{ '& .MuiOutlinedInput-root': { borderRadius: R } }} />
                <TextField label="副标题" size="small" value={card.subtitle} onChange={e => { const v = e.target.value; setHomeCardsForm(prev => prev.map((c, i) => i === idx ? { ...c, subtitle: v } : c)); }} sx={{ '& .MuiOutlinedInput-root': { borderRadius: R } }} />
                <TextField label="路径" size="small" value={card.path} onChange={e => { const v = e.target.value; setHomeCardsForm(prev => prev.map((c, i) => i === idx ? { ...c, path: v } : c)); }} sx={{ '& .MuiOutlinedInput-root': { borderRadius: R } }} />
                <TextField label="权限点" size="small" value={card.perm} onChange={e => { const v = e.target.value; setHomeCardsForm(prev => prev.map((c, i) => i === idx ? { ...c, perm: v } : c)); }} sx={{ '& .MuiOutlinedInput-root': { borderRadius: R } }} />
                <TextField label="图标名" size="small" value={card.icon} onChange={e => { const v = e.target.value; setHomeCardsForm(prev => prev.map((c, i) => i === idx ? { ...c, icon: v } : c)); }} sx={{ '& .MuiOutlinedInput-root': { borderRadius: R } }} helperText="Science / BarChart / Assignment" />
                <TextField label="渐变背景" size="small" value={card.gradient} onChange={e => { const v = e.target.value; setHomeCardsForm(prev => prev.map((c, i) => i === idx ? { ...c, gradient: v } : c)); }} sx={{ '& .MuiOutlinedInput-root': { borderRadius: R } }} fullWidth />
                <Box sx={{ display: 'flex', gap: 1, alignItems: 'center' }}>
                  <Typography variant="caption">边框色</Typography>
                  <input type="color" value={card.border} onChange={e => { const v = e.target.value; setHomeCardsForm(prev => prev.map((c, i) => i === idx ? { ...c, border: v } : c)); }} style={{ width: 32, height: 28, border: 'none', cursor: 'pointer', padding: 0 }} />
                  <TextField size="small" value={card.border} onChange={e => { const v = e.target.value; setHomeCardsForm(prev => prev.map((c, i) => i === idx ? { ...c, border: v } : c)); }} sx={{ width: 100, '& .MuiOutlinedInput-root': { borderRadius: R } }} />
                </Box>
                <Box sx={{ display: 'flex', gap: 1, alignItems: 'center' }}>
                  <Typography variant="caption">标题色</Typography>
                  <input type="color" value={card.titleColor} onChange={e => { const v = e.target.value; setHomeCardsForm(prev => prev.map((c, i) => i === idx ? { ...c, titleColor: v } : c)); }} style={{ width: 32, height: 28, border: 'none', cursor: 'pointer', padding: 0 }} />
                  <TextField size="small" value={card.titleColor} onChange={e => { const v = e.target.value; setHomeCardsForm(prev => prev.map((c, i) => i === idx ? { ...c, titleColor: v } : c)); }} sx={{ width: 100, '& .MuiOutlinedInput-root': { borderRadius: R } }} />
                </Box>
              </Box>
            </Paper>
          ))}
        </Box>
      )}
      <Box sx={{ mt: 2, display: 'flex', justifyContent: 'flex-end' }}>
        <Button variant="contained" onClick={async () => {
          setSettingsLoading(true);
          try {
            await updateSetting('home-cards', homeCardsForm);
            sm('首页配置已保存');
          } catch { sm('保存失败', true); }
          setSettingsLoading(false);
        }} disabled={settingsLoading} sx={{ borderRadius: R }}>
          {settingsLoading ? <CircularProgress size={20} /> : '保存首页配置'}
        </Button>
      </Box>
    </Box>}

    {/* ── 统计卡片 (v0.4.35) ── */}
    {tb === 'stats' && <Box>
      <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
        <Typography variant="subtitle1" fontWeight={700}>⑩ 统计卡片</Typography>
        <Button variant="outlined" size="small" startIcon={<AddIcon />} onClick={() => {
          setStatsCardsForm(prev => [...prev, { key: '', label: '', color: '#667eea', gradient: 'linear-gradient(135deg,#667eea,#764ba2)' }]);
        }} sx={{ borderRadius: R }}>新增卡片</Button>
      </Box>
      <Typography variant="caption" color="text.secondary" sx={{ mb: 2, display: 'block' }}>编辑统计页卡片。key 决定数据来源：total/records/users/project/lab/type</Typography>
      {statsCardsForm.length === 0 ? (
        <Typography color="text.secondary" textAlign="center" sx={{ py: 3 }}>暂无统计卡片配置</Typography>
      ) : (
        <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1.5 }}>
          {statsCardsForm.map((card, idx) => (
            <Paper key={idx} elevation={0} sx={{ p: 2, borderRadius: R, border: '1px solid rgba(0,0,0,0.08)' }}>
              <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 1.5 }}>
                <Typography variant="subtitle2" fontWeight={700}>卡片 {idx + 1}</Typography>
                <IconButton size="small" color="error" onClick={() => setStatsCardsForm(prev => prev.filter((_, i) => i !== idx))}><DeleteIcon fontSize="small" /></IconButton>
              </Box>
              <Box sx={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 1.5 }}>
                <TextField label="标识 (key)" size="small" value={card.key} onChange={e => { const v = e.target.value; setStatsCardsForm(prev => prev.map((c, i) => i === idx ? { ...c, key: v } : c)); }} sx={{ '& .MuiOutlinedInput-root': { borderRadius: R } }} helperText="total/records/users/project/lab/type" />
                <TextField label="显示名" size="small" value={card.label} onChange={e => { const v = e.target.value; setStatsCardsForm(prev => prev.map((c, i) => i === idx ? { ...c, label: v } : c)); }} sx={{ '& .MuiOutlinedInput-root': { borderRadius: R } }} />
                <Box sx={{ display: 'flex', gap: 1, alignItems: 'center' }}>
                  <Typography variant="caption">颜色</Typography>
                  <input type="color" value={card.color} onChange={e => { const v = e.target.value; setStatsCardsForm(prev => prev.map((c, i) => i === idx ? { ...c, color: v } : c)); }} style={{ width: 32, height: 28, border: 'none', cursor: 'pointer', padding: 0 }} />
                  <TextField size="small" value={card.color} onChange={e => { const v = e.target.value; setStatsCardsForm(prev => prev.map((c, i) => i === idx ? { ...c, color: v } : c)); }} sx={{ width: 140, '& .MuiOutlinedInput-root': { borderRadius: R } }} />
                </Box>
                <TextField label="渐变背景" size="small" value={card.gradient} onChange={e => { const v = e.target.value; setStatsCardsForm(prev => prev.map((c, i) => i === idx ? { ...c, gradient: v } : c)); }} sx={{ '& .MuiOutlinedInput-root': { borderRadius: R } }} />
              </Box>
            </Paper>
          ))}
        </Box>
      )}
      <Box sx={{ mt: 2, display: 'flex', justifyContent: 'flex-end' }}>
        <Button variant="contained" onClick={async () => {
          setSettingsLoading(true);
          try {
            await updateSetting('stats-cards', statsCardsForm);
            sm('统计卡片已保存');
          } catch { sm('保存失败', true); }
          setSettingsLoading(false);
        }} disabled={settingsLoading} sx={{ borderRadius: R }}>
          {settingsLoading ? <CircularProgress size={20} /> : '保存统计卡片'}
        </Button>
      </Box>
    </Box>}

    {/* ── 用户管理 (v0.4.29) ── */}
    {tb === 'users' && <Box>
      <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
        <Typography variant="subtitle1" fontWeight={700} color="#f4511e">用户管理</Typography>
        <Button variant="contained" size="small" startIcon={<AddIcon />}
          onClick={() => {
            setUserEditItem(null);
            setUserForm({ username: '', password: '', division_id: null, group_id: null, role_id: null, is_admin: false, is_active: true });
            setUserEditOpen(true);
          }}
          sx={{ borderRadius: R, bgcolor: '#f4511e', '&:hover': { bgcolor: '#e64a19' } }}>
          新增用户
        </Button>
        <Button variant="outlined" size="small" startIcon={<AddIcon />} onClick={() => { setImportDialogOpen(true); setImportFile(null); setImportResult(null); }} sx={{ borderRadius: R, borderColor: '#2e7d32', color: '#2e7d32' }}>批量导入</Button>
      </Box>

      <TableContainer component={Paper} sx={tSx}>
        <Table size="small">
          <TableHead>
            <TableRow>
              <TableCell sx={{ fontWeight: 600 }}>用户名</TableCell>
              <TableCell sx={{ fontWeight: 600 }}>所属部门</TableCell>
              <TableCell sx={{ fontWeight: 600 }}>所属实验室</TableCell>
              <TableCell sx={{ fontWeight: 600 }}>角色</TableCell>
              <TableCell sx={{ fontWeight: 600 }}>权限</TableCell>
              <TableCell sx={{ fontWeight: 600 }}>状态</TableCell>
              <TableCell sx={{ fontWeight: 600 }}>创建时间</TableCell>
              <TableCell align="right" sx={{ fontWeight: 600 }}>操作</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {users.map(u => (
              <TableRow key={u.id} hover>
                <TableCell sx={{ fontWeight: 600 }}>{u.username}</TableCell>
                <TableCell>{u.division_name || '未分配'}</TableCell>
                <TableCell>{u.group_name || '未分配'}</TableCell>
                <TableCell>{roleNameOf(u.role_id)}</TableCell>
                <TableCell>
                  <Chip label={u.is_admin ? '管理员' : '普通用户'}
                    size="small" color={u.is_admin ? 'warning' : 'default'}
                    variant={u.is_admin ? 'filled' : 'outlined'} />
                </TableCell>
                <TableCell>
                  <Chip label={u.is_active ? '启用' : '停用'}
                    size="small" color={u.is_active ? 'success' : 'default'}
                    variant={u.is_active ? 'filled' : 'outlined'} />
                </TableCell>
                <TableCell>{new Date(u.created_at).toLocaleDateString()}</TableCell>
                <TableCell align="right">
                  <IconButton size="small" onClick={() => {
                    setUserEditItem(u);
                    setUserForm({
                      username: u.username, password: '',
                      division_id: u.division_id ?? null,
                      group_id: u.group_id ?? null,
                      role_id: u.role_id ?? null,
                      is_admin: u.is_admin, is_active: u.is_active,
                    });
                    setUserEditOpen(true);
                  }} sx={{ color: '#f4511e' }}><EditIcon fontSize="small" /></IconButton>
                  <IconButton size="small" color="error" onClick={() => handleDeleteUser(u.id)}>
                    <DeleteIcon fontSize="small" />
                  </IconButton>
                </TableCell>
              </TableRow>
            ))}
            {users.length === 0 && (
              <TableRow><TableCell colSpan={8} align="center" sx={{ color: '#999', py: 3 }}>暂无用户</TableCell></TableRow>
            )}
          </TableBody>
        </Table>
      </TableContainer>
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
          {/* v0.4.64: 高项自定义填写 */}
          <TextField label="关联高项（自定义填写）" fullWidth size="small" placeholder="例如：A 类/重点/高项" value={projectEditItem.high_item || ''} onChange={e => setProjectEditItem({ ...projectEditItem, high_item: e.target.value })} sx={{ mt: 2, '& .MuiOutlinedInput-root': { borderRadius: R } }} />
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
          <FormControl fullWidth sx={{ mt: 2, '& .MuiOutlinedInput-root': { borderRadius: R } }}>
            <InputLabel>所属部门</InputLabel>
            <Select
              value={groupEditItem.division_id ?? ''}
              label="所属部门"
              onChange={e => setGroupEditItem({ ...groupEditItem!, division_id: e.target.value === '' ? null : Number(e.target.value) })}
            >
              <MenuItem value="">未分配</MenuItem>
              {divs.map(d => <MenuItem key={d.id} value={d.id}>{d.name}</MenuItem>)}
            </Select>
          </FormControl>
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

    {/* v0.4.24: 事业部编辑弹窗 */}
    <Dialog open={divEditOpen} onClose={() => setDivEditOpen(false)} maxWidth="sm" fullWidth PaperProps={{ sx: { borderRadius: R } }}>
      <DialogTitle sx={{ fontWeight: 700 }}>{divForm.id > 0 ? '编辑部门' : '新建部门'}</DialogTitle>
      <DialogContent>
        <TextField label="部门名称" fullWidth value={divForm.name} onChange={e => setDivForm({ ...divForm, name: e.target.value })} sx={{ mt: 2, mb: 2, '& .MuiOutlinedInput-root': { borderRadius: R } }} helperText="如: 液相、气相、理化、ICP、热分析、质谱、红外、其他" />
        <TextField label="排序" type="number" fullWidth value={divForm.sort_order} onChange={e => setDivForm({ ...divForm, sort_order: Number(e.target.value) || 0 })} sx={{ mb: 2, '& .MuiOutlinedInput-root': { borderRadius: R } }} />
        <TextField label="颜色" fullWidth value={divForm.color} onChange={e => setDivForm({ ...divForm, color: e.target.value })} sx={{ '& .MuiOutlinedInput-root': { borderRadius: R } }} helperText="十六进制颜色，如 #1976d2（P2 预留）" />
        <Typography variant="body2" sx={{ fontWeight: 600, mb: 1, mt: 2 }}>关联实验室</Typography>
        <Box sx={{ maxHeight: 150, overflow: 'auto', border: '1px solid #e0e0e0', borderRadius: R, p: 1, mb: 2 }}>
          {gs.filter(g => g.name !== '研发项目').map(g => (
            <FormControlLabel key={g.id}
              control={<Checkbox checked={divForm.group_ids.includes(g.id)} onChange={e => setDivForm(p => ({ ...p, group_ids: e.target.checked ? [...p.group_ids, g.id] : p.group_ids.filter(id => id !== g.id) }))} />}
              label={g.name} />
          ))}
        </Box>
      </DialogContent>
      <DialogActions>
        <Button onClick={() => setDivEditOpen(false)} sx={{ borderRadius: R }}>取消</Button>
        <Button onClick={hdiv} variant="contained" sx={{ borderRadius: R }}>保存</Button>
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
                <TableCell sx={{ fontWeight: 700 }}>关联部门</TableCell>
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
                    {g.division_name || '未分配'}
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

    {/* v0.4.29: 用户编辑弹窗 */}
    <Dialog open={userEditOpen} onClose={() => setUserEditOpen(false)}
      maxWidth="sm" fullWidth PaperProps={{ sx: { borderRadius: R } }}>
      <DialogTitle sx={{ fontWeight: 700 }}>
        {userEditItem ? '编辑用户' : '新增用户'}
      </DialogTitle>
      <DialogContent>
        <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2, mt: 1 }}>
          <TextField label="用户名" size="small" value={userForm.username}
            onChange={e => setUserForm(p => ({ ...p, username: e.target.value }))}
            sx={{ '& .MuiOutlinedInput-root': { borderRadius: R } }} />
          <TextField label="密码（留空则不修改）" type="password" size="small"
            value={userForm.password}
            onChange={e => setUserForm(p => ({ ...p, password: e.target.value }))}
            helperText={userEditItem ? '留空则不修改密码' : '新用户必填'}
            sx={{ '& .MuiOutlinedInput-root': { borderRadius: R } }} />
          <FormControl size="small">
            <InputLabel>所属部门</InputLabel>
            <Select value={userForm.division_id ?? ''} label="所属部门"
              onChange={e => setUserForm(p => ({ ...p, division_id: e.target.value ? Number(e.target.value) : null }))}
              sx={{ borderRadius: R }}>
              <MenuItem value="">未分配</MenuItem>
              {divs.map(d => <MenuItem key={d.id} value={d.id}>{d.name}</MenuItem>)}
            </Select>
          </FormControl>
          <FormControl size="small">
            <InputLabel>所属实验室</InputLabel>
            <Select value={userForm.group_id ?? ''} label="所属实验室"
              onChange={e => setUserForm(p => ({ ...p, group_id: e.target.value ? Number(e.target.value) : null }))}
              sx={{ borderRadius: R }}>
              <MenuItem value="">未分配</MenuItem>
              {gs.map(g => <MenuItem key={g.id} value={g.id}>{g.name}</MenuItem>)}
            </Select>
          </FormControl>
          <FormControl size="small">
            <InputLabel>角色</InputLabel>
            <Select value={userForm.role_id ?? ''} label="角色"
              onChange={e => setUserForm(p => ({ ...p, role_id: e.target.value ? Number(e.target.value) : null }))}
              sx={{ borderRadius: R }}>
              <MenuItem value="">未分配</MenuItem>
              {roles.map(r => <MenuItem key={r.id} value={r.id}>{r.name}</MenuItem>)}
            </Select>
          </FormControl>
          <FormControlLabel
            control={<Switch checked={userForm.is_admin}
              onChange={e => setUserForm(p => ({ ...p, is_admin: e.target.checked }))} />}
            label="管理员权限" />
          <FormControlLabel
            control={<Switch checked={userForm.is_active}
              onChange={e => setUserForm(p => ({ ...p, is_active: e.target.checked }))} />}
            label="启用账号" />
        </Box>
      </DialogContent>
      <DialogActions>
        <Button onClick={() => setUserEditOpen(false)} sx={{ borderRadius: R }}>取消</Button>
        <Button variant="contained" onClick={handleSaveUser}
          sx={{ borderRadius: R, bgcolor: '#f4511e', '&:hover': { bgcolor: '#e64a19' } }}>
          保存
        </Button>
      </DialogActions>
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
    {/* v0.4.63: 导入用户对话框 */}
    <Dialog open={importDialogOpen} onClose={() => setImportDialogOpen(false)} maxWidth="sm" fullWidth PaperProps={{ sx: { borderRadius: R } }}>
      <DialogTitle sx={{ fontWeight: 700 }}>批量导入用户</DialogTitle>
      <DialogContent>
        <Box sx={{ mb: 2 }}>
          <Typography variant="body2" color="text.secondary" sx={{ mb: 1 }}>① 下载模板</Typography>
          <Button variant="outlined" size="small" onClick={() => {
            const csv = "\uFEFF" + "用户名,密码,真实姓名,所属部门,所属实验室,角色,启用\nzhangsan,123456,张三,研究部,410,普通用户,1\n";
            const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' });
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url; a.download = 'users_template.csv';
            a.click(); URL.revokeObjectURL(url);
          }} sx={{ borderRadius: R }}>📄 users_template.csv</Button>
        </Box>
        <Box sx={{ mb: 2 }}>
          <Typography variant="body2" color="text.secondary" sx={{ mb: 1 }}>② 上传文件（CSV 格式，第一行为表头）</Typography>
          <Button variant="outlined" component="label" sx={{ borderRadius: R }}>
            选择文件
            <input type="file" hidden accept=".csv,.xlsx,.xls" onChange={e => { setImportFile(e.target.files?.[0] || null); setImportResult(null); e.target.value = ''; }} />
          </Button>
          {importFile && <Chip label={importFile.name} size="small" color="primary" variant="outlined" sx={{ ml: 1, borderRadius: R }} onDelete={() => setImportFile(null)} />}
        </Box>
        {importResult && (
          <Paper elevation={0} sx={{ p: 2, bgcolor: '#f5f9f5', borderRadius: R, border: '1px solid rgba(0,0,0,0.06)' }}>
            <Typography variant="body2" sx={{ color: '#2e7d32' }}>✓ 成功导入 {importResult.created} 条</Typography>
            {importResult.skipped > 0 && <Typography variant="body2" sx={{ color: '#f57c00' }}>⚠ 跳过 {importResult.skipped} 条</Typography>}
            {(importResult.errors || []).map((e: string, i: number) => <Typography key={i} variant="caption" display="block" color="error">{e}</Typography>)}
          </Paper>
        )}
      </DialogContent>
      <DialogActions>
        <Button onClick={() => { setImportDialogOpen(false); setImportFile(null); setImportResult(null); }} sx={{ borderRadius: R }}>关闭</Button>
        <Button variant="contained" disabled={!importFile} onClick={async () => {
          if (!importFile) return;
          try { const r = await importUsers(importFile); if (r.code === 0) { setImportResult(r.data); userList().then(r2 => { if (r2.code === 0 && r2.data) setUsers(r2.data.filter((u: any) => u.is_active)); }).catch(() => {}); } else { sm(r.message, true); } }
          catch { sm('导入失败', true); }
        }} sx={{ borderRadius: R, bgcolor: '#2e7d32' }}>开始导入</Button>
      </DialogActions>
    </Dialog>
    {tb === 'layouts' && <PageLayoutAdmin />}
    {tb === 'forms' && <ManageFormConfig />}
    {tb === 'exports' && <ManageExportConfig />}
  </Box>
  
);
};

export default ManagePage;
