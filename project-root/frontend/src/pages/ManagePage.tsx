import React, { useEffect, useState, useCallback } from 'react';
import {
  Box,
  Typography,
  TextField,
  Button,
  IconButton,
  Paper,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogContentText,
  DialogActions,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  Switch,
  FormControlLabel,
  Alert,
  CircularProgress,
  TableContainer,
  Table,
  TableHead,
  TableBody,
  TableRow,
  TableCell,
  Chip,
  useMediaQuery,
  useTheme,
} from '@mui/material';
import AddIcon from '@mui/icons-material/Add';
import EditIcon from '@mui/icons-material/Edit';
import DeleteIcon from '@mui/icons-material/Delete';
import RestoreIcon from '@mui/icons-material/Restore';
import HistoryIcon from '@mui/icons-material/History';
import FolderIcon from '@mui/icons-material/Folder';
import ListAltIcon from '@mui/icons-material/ListAlt';
import DeleteSweepIcon from '@mui/icons-material/DeleteSweep';
import ReceiptLongIcon from '@mui/icons-material/ReceiptLong';
import {
  getGroups,
  createGroup,
  updateGroup,
  deleteGroup,
  getProjects,
  createProject,
  updateProject,
  deleteProject,
  getRecords,
  restoreRecord,
  getAuditLogs,
} from '../api/client';
import type { ProjectGroup, Project, WorkRecord, AuditLog } from '../types';

type TabValue = 'groups' | 'projects' | 'trash' | 'audit';

const ACTION_LABELS: Record<string, string> = {
  create: '创建',
  delete: '删除',
  restore: '恢复',
  update: '修改',
};

const ACTION_COLORS: Record<string, 'success' | 'error' | 'info' | 'warning'> = {
  create: 'success',
  delete: 'error',
  restore: 'info',
  update: 'warning',
};

const TABLE_LABELS: Record<string, string> = {
  work_records: '工作量记录',
  project_groups: '分组',
  projects: '项目',
};

/** Tab card definitions with orange-red theme */
const TAB_CARDS: { key: TabValue; label: string; icon: React.ReactNode; desc: string }[] = [
  { key: 'groups', label: '分组管理', icon: <FolderIcon />, desc: '创建和编辑实验室分组' },
  { key: 'projects', label: '项目管理', icon: <ListAltIcon />, desc: '管理分组下的项目' },
  { key: 'trash', label: '回收站', icon: <DeleteSweepIcon />, desc: '恢复已删除的记录' },
  { key: 'audit', label: '审计日志', icon: <ReceiptLongIcon />, desc: '操作记录追溯' },
];

/** Shared uiverse card style */
const cardSx = {
  borderRadius: 4,
  background: 'linear-gradient(145deg, #ffffff, #f5f5f5)',
  border: '1px solid rgba(0,0,0,0.06)',
  boxShadow: '0 4px 20px rgba(0,0,0,0.06), 0 1px 3px rgba(0,0,0,0.04)',
  transition: 'all 0.3s cubic-bezier(0.4, 0, 0.2, 1)',
};

/** Shared list card style */
const listCardSx = {
  p: 2,
  borderRadius: 3,
  mb: 1,
  background: 'linear-gradient(145deg, #ffffff, #fafafa)',
  border: '1px solid rgba(0,0,0,0.05)',
  boxShadow: '0 2px 10px rgba(0,0,0,0.03)',
  transition: 'all 0.2s cubic-bezier(0.4, 0, 0.2, 1)',
  '&:hover': {
    boxShadow: '0 4px 16px rgba(0,0,0,0.07)',
    borderColor: 'rgba(0,0,0,0.1)',
  },
};

const tablePaperSx = {
  borderRadius: 4,
  background: 'linear-gradient(145deg, #ffffff, #fafafa)',
  border: '1px solid rgba(0,0,0,0.05)',
  boxShadow: '0 2px 16px rgba(0,0,0,0.04)',
};

const ManagePage: React.FC = () => {
  const theme = useTheme();
  const isMobile = useMediaQuery(theme.breakpoints.down('sm'));

  const [tab, setTab] = useState<TabValue>('groups');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [success, setSuccess] = useState('');

  // Groups
  const [groups, setGroups] = useState<ProjectGroup[]>([]);
  const [groupDialogOpen, setGroupDialogOpen] = useState(false);
  const [groupForm, setGroupForm] = useState({
    id: 0,
    name: '',
    sort_order: 0,
  });
  const [groupEditing, setGroupEditing] = useState(false);

  // Projects
  const [projects, setProjects] = useState<Project[]>([]);
  const [selectedGroup, setSelectedGroup] = useState<number>(0);
  const [projectDialogOpen, setProjectDialogOpen] = useState(false);
  const [projectForm, setProjectForm] = useState({
    id: 0,
    name: '',
    full_name: '',
    notes: '',
    group_id: 0,
    sort_order: 0,
    is_active: 1,
  });
  const [projectEditing, setProjectEditing] = useState(false);

  // Trash
  const [trashRecords, setTrashRecords] = useState<WorkRecord[]>([]);

  // Audit logs
  const [auditLogs, setAuditLogs] = useState<AuditLog[]>([]);
  const [auditPage, setAuditPage] = useState(1);
  const [auditTotal, setAuditTotal] = useState(0);

  // Delete confirmation
  const [confirmOpen, setConfirmOpen] = useState(false);
  const [confirmAction, setConfirmAction] = useState<() => void>(() => {});

  const loadGroups = useCallback(async () => {
    try {
      const res = await getGroups();
      if (res.code === 0) setGroups(res.data as ProjectGroup[]);
    } catch {
      /* ignore */
    }
  }, []);

  const loadProjects = useCallback(async () => {
    try {
      const res = await getProjects();
      if (res.code === 0) setProjects(res.data as Project[]);
    } catch {
      /* ignore */
    }
  }, []);

  const loadTrash = useCallback(async () => {
    try {
      const res = await getRecords({ include_deleted: true, page_size: 200 });
      if (res.code === 0) {
        const data = res.data as { items: WorkRecord[] };
        const items = data.items || [];
        setTrashRecords(items.filter((r) => r.deleted_at !== null));
      }
    } catch {
      /* ignore */
    }
  }, []);

  const loadAuditLogs = useCallback(async (page = 1) => {
    try {
      const res = await getAuditLogs({ page, page_size: 50 });
      if (res.code === 0) {
        const data = res.data as { items: AuditLog[]; total: number };
        setAuditLogs(data.items || []);
        setAuditTotal(data.total || 0);
        setAuditPage(page);
      }
    } catch {
      /* ignore */
    }
  }, []);

  useEffect(() => {
    setLoading(true);
    Promise.all([loadGroups(), loadProjects(), loadTrash()]).finally(() =>
      setLoading(false)
    );
  }, [loadGroups, loadProjects, loadTrash]);

  useEffect(() => {
    if (tab === 'audit') loadAuditLogs(1);
  }, [tab, loadAuditLogs]);

  const showMessage = (msg: string, isError = false) => {
    if (isError) {
      setError(msg);
      setTimeout(() => setError(''), 3000);
    } else {
      setSuccess(msg);
      setTimeout(() => setSuccess(''), 3000);
    }
  };

  // --- Group handlers ---
  const openGroupCreate = () => {
    setGroupForm({ id: 0, name: '', sort_order: 0 });
    setGroupEditing(false);
    setGroupDialogOpen(true);
  };

  const openGroupEdit = (group: ProjectGroup) => {
    setGroupForm({
      id: group.id,
      name: group.name,
      sort_order: group.sort_order,
    });
    setGroupEditing(true);
    setGroupDialogOpen(true);
  };

  const handleGroupSave = async () => {
    if (!groupForm.name.trim()) {
      showMessage('请输入分组名称', true);
      return;
    }
    try {
      if (groupEditing) {
        const res = await updateGroup(groupForm.id, {
          name: groupForm.name,
          sort_order: groupForm.sort_order,
        });
        if (res.code === 0) {
          showMessage('更新成功');
          loadGroups();
          setGroupDialogOpen(false);
        } else showMessage(res.message, true);
      } else {
        const res = await createGroup({
          name: groupForm.name,
          sort_order: groupForm.sort_order,
        });
        if (res.code === 0) {
          showMessage('创建成功');
          loadGroups();
          setGroupDialogOpen(false);
        } else showMessage(res.message, true);
      }
    } catch {
      showMessage('操作失败', true);
    }
  };

  const confirmDeleteGroup = (id: number) => {
    setConfirmAction(() => async () => {
      const res = await deleteGroup(id);
      if (res.code === 0) {
        showMessage('删除成功');
        loadGroups();
      } else showMessage(res.message, true);
      setConfirmOpen(false);
    });
    setConfirmOpen(true);
  };

  // --- Project handlers ---
  const openProjectCreate = () => {
    if (!selectedGroup) {
      showMessage('请先选择分组', true);
      return;
    }
    setProjectForm({
      id: 0,
      name: '',
      full_name: '',
      notes: '',
      group_id: selectedGroup,
      sort_order: 0,
      is_active: 1,
    });
    setProjectEditing(false);
    setProjectDialogOpen(true);
  };

  const openProjectEdit = (project: Project) => {
    setProjectForm({
      id: project.id,
      name: project.name,
      full_name: project.full_name || '',
      notes: project.notes || '',
      group_id: project.group_id,
      sort_order: project.sort_order,
      is_active: project.is_active,
    });
    setProjectEditing(true);
    setProjectDialogOpen(true);
  };

  const handleProjectSave = async () => {
    if (!projectForm.name.trim()) {
      showMessage('请输入项目名称', true);
      return;
    }
    try {
      if (projectEditing) {
        const res = await updateProject(projectForm.id, {
          name: projectForm.name,
          full_name: projectForm.full_name,
          notes: projectForm.notes,
          sort_order: projectForm.sort_order,
          is_active: projectForm.is_active,
        });
        if (res.code === 0) {
          showMessage('更新成功');
          loadProjects();
          setProjectDialogOpen(false);
        } else showMessage(res.message, true);
      } else {
        const res = await createProject({
          group_id: projectForm.group_id,
          name: projectForm.name,
          sort_order: projectForm.sort_order,
        });
        if (res.code === 0) {
          showMessage('创建成功');
          loadProjects();
          setProjectDialogOpen(false);
        } else showMessage(res.message, true);
      }
    } catch {
      showMessage('操作失败', true);
    }
  };

  const confirmDeleteProject = (id: number) => {
    setConfirmAction(() => async () => {
      const res = await deleteProject(id);
      if (res.code === 0) {
        showMessage('删除成功');
        loadProjects();
      } else showMessage(res.message, true);
      setConfirmOpen(false);
    });
    setConfirmOpen(true);
  };

  const filteredProjects = selectedGroup
    ? projects.filter((p) => p.group_id === selectedGroup)
    : projects;

  // --- Trash handlers ---
  const handleRestore = async (id: number) => {
    const res = await restoreRecord(id);
    if (res.code === 0) {
      showMessage('恢复成功');
      loadTrash();
    } else showMessage(res.message, true);
  };

  if (loading && tab !== 'audit') {
    return (
      <Box sx={{ display: 'flex', justifyContent: 'center', py: 4 }}>
        <CircularProgress />
      </Box>
    );
  }

  return (
    <Box>
      {/* Page title with orange-red gradient */}
      <Typography
        variant="h5"
        fontWeight={700}
        sx={{
          mb: 3,
          px: 1,
          background: 'linear-gradient(135deg, #f4511e, #e53935)',
          WebkitBackgroundClip: 'text',
          WebkitTextFillColor: 'transparent',
          backgroundClip: 'text',
        }}
      >
        管理
      </Typography>

      {error && (
        <Alert severity="error" sx={{ mx: 1, mb: 2, borderRadius: 3 }}>
          {error}
        </Alert>
      )}
      {success && (
        <Alert severity="success" sx={{ mx: 1, mb: 2, borderRadius: 3 }}>
          {success}
        </Alert>
      )}

      {/* Horizontal Tab Cards — replace traditional Tabs */}
      <Box
        sx={{
          display: 'grid',
          gridTemplateColumns: {
            xs: 'repeat(2, 1fr)',
            sm: 'repeat(4, 1fr)',
          },
          gap: 1.5,
          mb: 3,
          px: 1,
        }}
      >
        {TAB_CARDS.map((card) => {
          const isActive = tab === card.key;
          return (
            <Paper
              key={card.key}
              elevation={0}
              onClick={() => setTab(card.key)}
              sx={{
                ...cardSx,
                p: isMobile ? 1.5 : 2,
                cursor: 'pointer',
                textAlign: 'center',
                borderColor: isActive ? '#f4511e60' : 'rgba(0,0,0,0.06)',
                background: isActive
                  ? 'linear-gradient(145deg, #fff5f3, #ffebee)'
                  : 'linear-gradient(145deg, #ffffff, #f5f5f5)',
                '&:hover': {
                  transform: 'translateY(-3px)',
                  boxShadow: '0 8px 30px rgba(244,81,30,0.12), 0 2px 6px rgba(0,0,0,0.06)',
                  borderColor: '#f4511e40',
                },
              }}
            >
              <Box
                sx={{
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  mb: 0.8,
                  color: isActive ? '#f4511e' : '#888',
                }}
              >
                {card.icon}
              </Box>
              <Typography
                variant={isMobile ? 'body2' : 'subtitle2'}
                fontWeight={isActive ? 700 : 500}
                color={isActive ? '#f4511e' : 'text.primary'}
                sx={{ mb: 0.3 }}
              >
                {card.label}
              </Typography>
              {!isMobile && (
                <Typography variant="caption" color="text.secondary">
                  {card.desc}
                </Typography>
              )}
            </Paper>
          );
        })}
      </Box>

      <Box sx={{ px: 1 }}>
        {/* Groups Tab */}
        {tab === 'groups' && (
          <Box>
            <Box sx={{ display: 'flex', justifyContent: 'flex-end', mb: 2 }}>
              <Button
                variant="contained"
                startIcon={<AddIcon />}
                onClick={openGroupCreate}
                sx={{
                  borderRadius: 3,
                  background: 'linear-gradient(135deg, #f4511e, #e53935)',
                  boxShadow: '0 4px 14px rgba(244,81,30,0.3)',
                  '&:hover': {
                    background: 'linear-gradient(135deg, #e64a19, #d32f2f)',
                  },
                }}
              >
                新建分组
              </Button>
            </Box>
            {groups.length === 0 ? (
              <Typography
                color="text.secondary"
                textAlign="center"
                sx={{ py: 4 }}
              >
                暂无分组
              </Typography>
            ) : (
              <Box>
                {groups.map((group) => (
                  <Paper key={group.id} elevation={0} sx={listCardSx}>
                    <Box
                      sx={{
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'space-between',
                      }}
                    >
                      <Box sx={{ flex: 1, minWidth: 0 }}>
                        <Typography variant="subtitle1" fontWeight={600}>
                          {group.name}
                        </Typography>
                        <Typography variant="caption" color="text.secondary">
                          {group.project_count} 个项目 · 排序: {group.sort_order}
                        </Typography>
                      </Box>
                      <Box sx={{ display: 'flex', gap: 0.5, flexShrink: 0 }}>
                        <IconButton
                          onClick={() => openGroupEdit(group)}
                          size="small"
                          sx={{ color: '#f4511e' }}
                        >
                          <EditIcon fontSize="small" />
                        </IconButton>
                        <IconButton
                          onClick={() => confirmDeleteGroup(group.id)}
                          size="small"
                          color="error"
                        >
                          <DeleteIcon fontSize="small" />
                        </IconButton>
                      </Box>
                    </Box>
                  </Paper>
                ))}
              </Box>
            )}
          </Box>
        )}

        {/* Projects Tab */}
        {tab === 'projects' && (
          <Box>
            <Box
              sx={{
                display: 'flex',
                gap: 1,
                mb: 2,
                flexWrap: 'wrap',
                justifyContent: 'space-between',
              }}
            >
              <FormControl size="small" sx={{ minWidth: 150 }}>
                <InputLabel>选择分组</InputLabel>
                <Select
                  value={selectedGroup}
                  label="选择分组"
                  onChange={(e) =>
                    setSelectedGroup(Number(e.target.value))
                  }
                  sx={{ borderRadius: 3 }}
                >
                  <MenuItem value={0}>全部分组</MenuItem>
                  {groups.map((g) => (
                    <MenuItem key={g.id} value={g.id}>
                      {g.name}
                    </MenuItem>
                  ))}
                </Select>
              </FormControl>
              <Button
                variant="contained"
                startIcon={<AddIcon />}
                onClick={openProjectCreate}
                sx={{
                  borderRadius: 3,
                  background: 'linear-gradient(135deg, #f4511e, #e53935)',
                  boxShadow: '0 4px 14px rgba(244,81,30,0.3)',
                  '&:hover': {
                    background: 'linear-gradient(135deg, #e64a19, #d32f2f)',
                  },
                }}
              >
                新建项目
              </Button>
            </Box>
            {filteredProjects.length === 0 ? (
              <Typography
                color="text.secondary"
                textAlign="center"
                sx={{ py: 4 }}
              >
                暂无项目
              </Typography>
            ) : (
              <Box>
                {filteredProjects.map((project) => (
                  <Paper key={project.id} elevation={0} sx={listCardSx}>
                    <Box
                      sx={{
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'space-between',
                      }}
                    >
                      <Box sx={{ flex: 1, minWidth: 0 }}>
                        <Typography variant="subtitle1" fontWeight={600}>
                          {project.name}
                        </Typography>
                        <Typography variant="caption" color="text.secondary">
                          分组: {project.group_name} ·{' '}
                          {project.is_active ? (
                            <Chip label="启用" size="small" color="success" sx={{ borderRadius: 2, height: 20, fontSize: '0.7rem' }} />
                          ) : (
                            <Chip label="已停用" size="small" color="default" sx={{ borderRadius: 2, height: 20, fontSize: '0.7rem' }} />
                          )}{' '}
                          · 排序: {project.sort_order}
                        </Typography>
                      </Box>
                      <Box sx={{ display: 'flex', gap: 0.5, flexShrink: 0 }}>
                        <IconButton
                          onClick={() => openProjectEdit(project)}
                          size="small"
                          sx={{ color: '#f4511e' }}
                        >
                          <EditIcon fontSize="small" />
                        </IconButton>
                        <IconButton
                          onClick={() => confirmDeleteProject(project.id)}
                          size="small"
                          color="error"
                        >
                          <DeleteIcon fontSize="small" />
                        </IconButton>
                      </Box>
                    </Box>
                  </Paper>
                ))}
              </Box>
            )}
          </Box>
        )}

        {/* Trash Tab */}
        {tab === 'trash' && (
          <Box>
            <Typography
              variant="body2"
              color="text.secondary"
              sx={{ mb: 2 }}
            >
              共 {trashRecords.length} 条已删除记录
            </Typography>
            {trashRecords.length === 0 ? (
              <Typography
                color="text.secondary"
                textAlign="center"
                sx={{ py: 4 }}
              >
                回收站为空
              </Typography>
            ) : (
              <TableContainer
                component={Paper}
                className="table-responsive"
                sx={tablePaperSx}
              >
                <Table size="small">
                  <TableHead>
                    <TableRow>
                      <TableCell sx={{ fontWeight: 600 }}>项目</TableCell>
                      <TableCell sx={{ fontWeight: 600 }}>用户</TableCell>
                      <TableCell align="right" sx={{ fontWeight: 600 }}>数量</TableCell>
                      <TableCell sx={{ fontWeight: 600 }}>删除时间</TableCell>
                      <TableCell sx={{ fontWeight: 600 }}>操作</TableCell>
                    </TableRow>
                  </TableHead>
                  <TableBody>
                    {trashRecords.map((r) => (
                      <TableRow key={r.id} hover>
                        <TableCell>
                          {r.project_name || r.group_name}
                        </TableCell>
                        <TableCell>{r.user_name}</TableCell>
                        <TableCell align="right">{r.quantity}</TableCell>
                        <TableCell>{r.deleted_at}</TableCell>
                        <TableCell>
                          <Button
                            size="small"
                            startIcon={<RestoreIcon />}
                            onClick={() => handleRestore(r.id)}
                            sx={{
                              borderRadius: 3,
                              color: '#f4511e',
                              '&:hover': { bgcolor: 'rgba(244,81,30,0.08)' },
                            }}
                          >
                            恢复
                          </Button>
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </TableContainer>
            )}
          </Box>
        )}

        {/* Audit Log Tab */}
        {tab === 'audit' && (
          <Box>
            <Typography
              variant="body2"
              color="text.secondary"
              sx={{ mb: 2, display: 'flex', alignItems: 'center', gap: 1 }}
            >
              <HistoryIcon fontSize="small" />
              共 {auditTotal} 条操作记录
            </Typography>
            {auditLogs.length === 0 ? (
              <Typography
                color="text.secondary"
                textAlign="center"
                sx={{ py: 4 }}
              >
                暂无审计日志
              </Typography>
            ) : (
              <>
                <TableContainer
                  component={Paper}
                  className="table-responsive"
                  sx={tablePaperSx}
                >
                  <Table size="small">
                    <TableHead>
                      <TableRow>
                        <TableCell sx={{ fontWeight: 600 }}>时间</TableCell>
                        <TableCell sx={{ fontWeight: 600 }}>操作类型</TableCell>
                        <TableCell sx={{ fontWeight: 600 }}>操作对象</TableCell>
                        <TableCell sx={{ fontWeight: 600 }}>记录ID</TableCell>
                        <TableCell sx={{ fontWeight: 600 }}>操作人</TableCell>
                      </TableRow>
                    </TableHead>
                    <TableBody>
                      {auditLogs.map((log) => (
                        <TableRow key={log.id} hover>
                          <TableCell sx={{ whiteSpace: 'nowrap' }}>
                            {log.created_at}
                          </TableCell>
                          <TableCell>
                            <Chip
                              label={ACTION_LABELS[log.action] || log.action}
                              size="small"
                              color={ACTION_COLORS[log.action] || 'default'}
                              variant="outlined"
                              sx={{ borderRadius: 2 }}
                            />
                          </TableCell>
                          <TableCell>
                            {TABLE_LABELS[log.table_name] || log.table_name}
                          </TableCell>
                          <TableCell>{log.record_id}</TableCell>
                          <TableCell>{log.user_name}</TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </TableContainer>
                <Box
                  sx={{ display: 'flex', justifyContent: 'center', mt: 2, gap: 1 }}
                >
                  <Button
                    size="small"
                    disabled={auditPage <= 1}
                    onClick={() => loadAuditLogs(auditPage - 1)}
                    sx={{ borderRadius: 3 }}
                  >
                    上一页
                  </Button>
                  <Typography variant="body2" sx={{ alignSelf: 'center' }}>
                    {auditPage} / {Math.max(1, Math.ceil(auditTotal / 50))}
                  </Typography>
                  <Button
                    size="small"
                    disabled={auditPage * 50 >= auditTotal}
                    onClick={() => loadAuditLogs(auditPage + 1)}
                    sx={{ borderRadius: 3 }}
                  >
                    下一页
                  </Button>
                </Box>
              </>
            )}
          </Box>
        )}
      </Box>

      {/* Group Dialog */}
      <Dialog
        open={groupDialogOpen}
        onClose={() => setGroupDialogOpen(false)}
        maxWidth="sm"
        fullWidth
        PaperProps={{ sx: { borderRadius: 4 } }}
      >
        <DialogTitle sx={{ fontWeight: 700 }}>
          {groupEditing ? '编辑分组' : '新建分组'}
        </DialogTitle>
        <DialogContent>
          <TextField
            autoFocus
            label="分组名称"
            fullWidth
            value={groupForm.name}
            onChange={(e) =>
              setGroupForm({ ...groupForm, name: e.target.value })
            }
            sx={{ mt: 1, '& .MuiOutlinedInput-root': { borderRadius: 3 } }}
          />
          <TextField
            label="排序"
            type="number"
            fullWidth
            value={groupForm.sort_order}
            onChange={(e) =>
              setGroupForm({
                ...groupForm,
                sort_order: Number(e.target.value),
              })
            }
            sx={{ mt: 2, '& .MuiOutlinedInput-root': { borderRadius: 3 } }}
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setGroupDialogOpen(false)} sx={{ borderRadius: 3 }}>取消</Button>
          <Button onClick={handleGroupSave} variant="contained" sx={{ borderRadius: 3 }}>
            保存
          </Button>
        </DialogActions>
      </Dialog>

      {/* Project Dialog */}
      <Dialog
        open={projectDialogOpen}
        onClose={() => setProjectDialogOpen(false)}
        maxWidth="sm"
        fullWidth
        PaperProps={{ sx: { borderRadius: 4 } }}
      >
        <DialogTitle sx={{ fontWeight: 700 }}>
          {projectEditing ? '编辑项目' : '新建项目'}
        </DialogTitle>
        <DialogContent>
          {!projectEditing && (
            <FormControl fullWidth sx={{ mt: 1 }}>
              <InputLabel>所属分组</InputLabel>
              <Select
                value={projectForm.group_id}
                label="所属分组"
                onChange={(e) =>
                  setProjectForm({
                    ...projectForm,
                    group_id: Number(e.target.value),
                  })
                }
                sx={{ borderRadius: 3 }}
              >
                {groups.map((g) => (
                  <MenuItem key={g.id} value={g.id}>
                    {g.name}
                  </MenuItem>
                ))}
              </Select>
            </FormControl>
          )}
          <TextField
            label="项目名称"
            fullWidth
            value={projectForm.name}
            onChange={(e) =>
              setProjectForm({ ...projectForm, name: e.target.value })
            }
            sx={{ mt: 2, '& .MuiOutlinedInput-root': { borderRadius: 3 } }}
          />
          <TextField
            label="全称"
            fullWidth
            value={projectForm.full_name}
            onChange={(e) =>
              setProjectForm({ ...projectForm, full_name: e.target.value })
            }
            sx={{ mt: 2, '& .MuiOutlinedInput-root': { borderRadius: 3 } }}
            helperText="方法全称，可自定义修改"
          />
          <TextField
            label="备注"
            fullWidth
            multiline
            minRows={2}
            maxRows={4}
            value={projectForm.notes}
            onChange={(e) =>
              setProjectForm({ ...projectForm, notes: e.target.value })
            }
            sx={{ mt: 2, '& .MuiOutlinedInput-root': { borderRadius: 3 } }}
            helperText="项目备注信息"
          />
          <TextField
            label="排序"
            type="number"
            fullWidth
            value={projectForm.sort_order}
            onChange={(e) =>
              setProjectForm({
                ...projectForm,
                sort_order: Number(e.target.value),
              })
            }
            sx={{ mt: 2, '& .MuiOutlinedInput-root': { borderRadius: 3 } }}
          />
          {projectEditing && (
            <FormControlLabel
              control={
                <Switch
                  checked={projectForm.is_active === 1}
                  onChange={(e) =>
                    setProjectForm({
                      ...projectForm,
                      is_active: e.target.checked ? 1 : 0,
                    })
                  }
                />
              }
              label="启用"
              sx={{ mt: 1 }}
            />
          )}
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setProjectDialogOpen(false)} sx={{ borderRadius: 3 }}>取消</Button>
          <Button onClick={handleProjectSave} variant="contained" sx={{ borderRadius: 3 }}>
            保存
          </Button>
        </DialogActions>
      </Dialog>

      {/* Confirm Dialog */}
      <Dialog
        open={confirmOpen}
        onClose={() => setConfirmOpen(false)}
        PaperProps={{ sx: { borderRadius: 4 } }}
      >
        <DialogTitle sx={{ fontWeight: 700 }}>确认操作</DialogTitle>
        <DialogContent>
          <DialogContentText>
            确定要执行此操作吗？此操作不可撤销。
          </DialogContentText>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setConfirmOpen(false)} sx={{ borderRadius: 3 }}>取消</Button>
          <Button
            onClick={confirmAction}
            color="error"
            variant="contained"
            sx={{ borderRadius: 3 }}
          >
            确认
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
};

export default ManagePage;
