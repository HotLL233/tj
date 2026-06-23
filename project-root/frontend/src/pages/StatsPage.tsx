import React, { useEffect, useState, useCallback } from 'react';
import {
  Box,
  Typography,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Paper,
  Button,
  CircularProgress,
  Alert,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  Chip,
  useMediaQuery,
  useTheme,
  IconButton,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  Grid,
} from '@mui/material';
import DownloadIcon from '@mui/icons-material/Download';
import EditIcon from '@mui/icons-material/Edit';
import DeleteForeverIcon from '@mui/icons-material/DeleteForever';
import ViewWeekIcon from '@mui/icons-material/ViewWeek';
import CalendarMonthIcon from '@mui/icons-material/CalendarMonth';
import PeopleIcon from '@mui/icons-material/People';
import FolderIcon from '@mui/icons-material/Folder';
import ScienceIcon from '@mui/icons-material/Science';
import PrecisionManufacturingIcon from '@mui/icons-material/PrecisionManufacturing';
import HistoryIcon from '@mui/icons-material/History';
import ArrowBackIcon from '@mui/icons-material/ArrowBack';
import dayjs from 'dayjs';
import isoWeek from 'dayjs/plugin/isoWeek';
import StatsCards from '../components/StatsCards';
import DateRangePicker from '../components/DateRangePicker';
import ConfirmDialog from '../components/ConfirmDialog';
import {
  getStatsSummary,
  getStatsByUser,
  getStatsByProject,
  getStatsByType,
  getStatsByInstrument,
  exportExcel,
  getGroups,
  getRecords,
  updateRecord,
  deleteRecordsByUser,
} from '../api/client';
import type {
  StatsSummary,
  UserStats,
  ProjectStats,
  TypeStats,
  InstrumentStats,
  ProjectGroup,
  StatsDetail,
  WorkRecord,
} from '../types';

dayjs.extend(isoWeek);

export type TabValue = 'week' | 'month' | 'user' | 'project' | 'type' | 'instrument' | 'user-log';

interface StatCardDef {
  key: TabValue;
  label: string;
  icon: React.ReactNode;
  color: string;
  desc: string;
}

/** uiverse.io card style shared across stat cards */
const cardSx = {
  p: 2.5,
  borderRadius: '12px',
  cursor: 'pointer',
  background: 'linear-gradient(145deg, #ffffff, #f5f5f5)',
  border: '1px solid rgba(0,0,0,0.06)',
  boxShadow: '0 4px 20px rgba(0,0,0,0.06), 0 1px 3px rgba(0,0,0,0.04)',
  transition: 'all 0.3s cubic-bezier(0.4, 0, 0.2, 1)',
  '&:hover': {
    transform: 'translateY(-4px)',
    boxShadow: '0 8px 30px rgba(0,0,0,0.1), 0 2px 6px rgba(0,0,0,0.06)',
  },
};

const STAT_CARDS: StatCardDef[] = [
  { key: 'week', label: '按周统计', icon: <ViewWeekIcon />, color: '#667eea', desc: '每月第几周汇总' },
  { key: 'month', label: '按月统计', icon: <CalendarMonthIcon />, color: '#43a047', desc: '每月汇总数据' },
  { key: 'user', label: '按用户', icon: <PeopleIcon />, color: '#f57c00', desc: '每人工作量' },
  { key: 'project', label: '按项目', icon: <FolderIcon />, color: '#7b1fa2', desc: '各项目汇总' },
  { key: 'type', label: '按类型', icon: <ScienceIcon />, color: '#0097a7', desc: '液相/气相分类' },
  { key: 'instrument', label: '按仪器', icon: <PrecisionManufacturingIcon />, color: '#d32f2f', desc: '各仪器汇总' },
  { key: 'user-log', label: '用户日志', icon: <HistoryIcon />, color: '#5d4037', desc: '逐条记录明细' },
];

/** Table container shared style */
const tablePaperSx = {
  borderRadius: '8px',
  background: 'linear-gradient(145deg, #ffffff, #fafafa)',
  border: '1px solid rgba(0,0,0,0.05)',
  boxShadow: '0 2px 16px rgba(0,0,0,0.04)',
};

const StatsPage: React.FC = () => {
  const theme = useTheme();
  const isMobile = useMediaQuery(theme.breakpoints.down('sm'));

  const [activeCard, setActiveCard] = useState<TabValue | null>(null);
  const [startDate, setStartDate] = useState(() =>
    dayjs().startOf('isoWeek').format('YYYY-MM-DD')
  );
  const [endDate, setEndDate] = useState(() =>
    dayjs().endOf('isoWeek').format('YYYY-MM-DD')
  );
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  const [summary, setSummary] = useState<StatsSummary | null>(null);
  const [userStats, setUserStats] = useState<UserStats[]>([]);
  const [projectStats, setProjectStats] = useState<ProjectStats[]>([]);
  const [typeStats, setTypeStats] = useState<TypeStats[]>([]);
  const [instrumentStats, setInstrumentStats] = useState<InstrumentStats[]>([]);
  const [details, setDetails] = useState<StatsDetail[]>([]);
  const [groups, setGroups] = useState<ProjectGroup[]>([]);
  const [groupFilter, setGroupFilter] = useState<number>(0);

  // User log
  const [userLogRecords, setUserLogRecords] = useState<WorkRecord[]>([]);
  const [userLogLoading, setUserLogLoading] = useState(false);
  const [userLogPage, setUserLogPage] = useState(1);
  const [userLogTotal, setUserLogTotal] = useState(0);
  const [userFilter, setUserFilter] = useState<string>('');
  const PAGE_SIZE = 100;

  // Edit dialog
  const [editDialogOpen, setEditDialogOpen] = useState(false);
  const [editForm, setEditForm] = useState({ id: 0, user_name: '', quantity: 0, recorded_at: '' });
  const [editError, setEditError] = useState('');

  // Delete user confirmation
  const [deleteUserDialogOpen, setDeleteUserDialogOpen] = useState(false);
  const [deleteUserName, setDeleteUserName] = useState('');
  const [deleteUserLoading, setDeleteUserLoading] = useState(false);

  const loadGroups = async () => {
    try {
      const res = await getGroups();
      if (res.code === 0) {
        setGroups(res.data as ProjectGroup[]);
      }
    } catch {
      /* ignore */
    }
  };

  useEffect(() => {
    loadGroups();
  }, []);

  const startIso = dayjs(startDate).format('YYYY-MM-DDTHH:mm:ss');
  const endIso = dayjs(endDate).endOf('day').format('YYYY-MM-DDTHH:mm:ss');

  const loadUserLog = useCallback(async (page = 1) => {
    setUserLogLoading(true);
    try {
      const params: any = { start: startIso, end: endIso, page, page_size: PAGE_SIZE };
      if (userFilter) params.user_name = userFilter;
      const res = await getRecords(params);
      if (res.code === 0) {
        const data = res.data as { items: WorkRecord[]; total: number };
        setUserLogRecords(data.items || []);
        setUserLogTotal(data.total || 0);
        setUserLogPage(page);
      }
    } catch {
      /* ignore */
    } finally {
      setUserLogLoading(false);
    }
  }, [startIso, endIso, userFilter]);

  const loadData = useCallback(async () => {
    if (!activeCard) return;
    setLoading(true);
    setError('');
    try {
      if (activeCard === 'week' || activeCard === 'month') {
        const res = await getStatsSummary({
          start: startIso,
          end: endIso,
          group_by: activeCard === 'week' ? 'week' : 'month',
        });
        if (res.code === 0) {
          const data = res.data as StatsSummary;
          setSummary(data);
          setDetails(data.details || []);
        } else {
          setError(res.message);
        }
      } else if (activeCard === 'user') {
        const res = await getStatsByUser({ start: startIso, end: endIso });
        if (res.code === 0) {
          setUserStats(res.data as UserStats[]);
        } else {
          setError(res.message);
        }
      } else if (activeCard === 'project') {
        const res = await getStatsByProject({
          start: startIso,
          end: endIso,
          group_id: groupFilter || undefined,
        });
        if (res.code === 0) {
          setProjectStats(res.data as ProjectStats[]);
        } else {
          setError(res.message);
        }
      } else if (activeCard === 'type') {
        const res = await getStatsByType({ start: startIso, end: endIso });
        if (res.code === 0) {
          setTypeStats(res.data as TypeStats[]);
        } else {
          setError(res.message);
        }
      } else if (activeCard === 'instrument') {
        const res = await getStatsByInstrument({ start: startIso, end: endIso });
        if (res.code === 0) {
          setInstrumentStats(res.data as InstrumentStats[]);
        } else {
          setError(res.message);
        }
      } else if (activeCard === 'user-log') {
        await loadUserLog(1);
      }
    } catch {
      setError('加载失败，请检查网络连接');
    } finally {
      setLoading(false);
    }
  }, [activeCard, startIso, endIso, groupFilter]);

  useEffect(() => {
    loadData();
  }, [loadData]);

  // Also load summary separately for StatsCards (always show)
  useEffect(() => {
    const loadSummary = async () => {
      try {
        const res = await getStatsSummary({
          start: startIso,
          end: endIso,
          group_by: 'week',
        });
        if (res.code === 0) {
          setSummary(res.data as StatsSummary);
        }
      } catch {
        /* ignore */
      }
    };
    loadSummary();
  }, [startIso, endIso]);

  const handleExport = async () => {
    try {
      const blob = await exportExcel({
        start: startIso,
        end: endIso,
        group_id: groupFilter || undefined,
      });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `工作量统计_${startDate}_${endDate}.xlsx`;
      a.click();
      URL.revokeObjectURL(url);
    } catch {
      setError('导出失败');
    }
  };

  // --- Edit record handlers ---
  const openEditDialog = (record: WorkRecord) => {
    setEditForm({
      id: record.id,
      user_name: record.user_name,
      quantity: record.quantity,
      recorded_at: dayjs(record.recorded_at).format('YYYY-MM-DDTHH:mm'),
    });
    setEditError('');
    setEditDialogOpen(true);
  };

  const handleEditSave = async () => {
    if (!editForm.user_name.trim()) {
      setEditError('请输入用户名');
      return;
    }
    if (editForm.quantity < 1) {
      setEditError('数量必须大于0');
      return;
    }
    try {
      const res = await updateRecord(editForm.id, {
        user_name: editForm.user_name,
        quantity: editForm.quantity,
        recorded_at: dayjs(editForm.recorded_at).format('YYYY-MM-DDTHH:mm:ss'),
      });
      if (res.code === 0) {
        setEditDialogOpen(false);
        loadUserLog(userLogPage);
      } else {
        setEditError(res.message);
      }
    } catch {
      setEditError('保存失败，请检查网络连接');
    }
  };

  // --- Delete user handlers ---
  const openDeleteUserDialog = (userName: string) => {
    setDeleteUserName(userName);
    setDeleteUserDialogOpen(true);
  };

  const handleDeleteUser = async () => {
    setDeleteUserLoading(true);
    try {
      const res = await deleteRecordsByUser(deleteUserName, {
        start: startIso,
        end: endIso,
      });
      if (res.code === 0) {
        const data = res.data as { deleted_count: number };
        setDeleteUserDialogOpen(false);
        loadUserLog(userLogPage);
        const summaryRes = await getStatsSummary({
          start: startIso,
          end: endIso,
          group_by: 'week',
        });
        if (summaryRes.code === 0) {
          setSummary(summaryRes.data as StatsSummary);
        }
      } else {
        setError(res.message);
        setDeleteUserDialogOpen(false);
      }
    } catch {
      setError('删除失败，请检查网络连接');
      setDeleteUserDialogOpen(false);
    } finally {
      setDeleteUserLoading(false);
    }
  };

  const handleCardClick = (cardKey: TabValue) => {
    setActiveCard(cardKey);
    setError('');
  };

  const handleBackToCards = () => {
    setActiveCard(null);
    setError('');
  };

  // --- Render card grid ---
  const renderCardGrid = () => (
    <Grid container spacing={2}>
      {STAT_CARDS.map((card) => (
        <Grid item xs={12} sm={6} md={4} key={card.key}>
          <Paper
            onClick={() => handleCardClick(card.key)}
            sx={{
              ...cardSx,
              '&:hover': {
                ...cardSx['&:hover'],
                borderColor: `${card.color}50`,
                boxShadow: `0 12px 30px ${card.color}20, 0 4px 10px rgba(0,0,0,0.06)`,
              },
            }}
          >
            <Box sx={{ display: 'flex', alignItems: 'center', gap: 1.5, mb: 1 }}>
              <Box
                sx={{
                  width: 40,
                  height: 40,
                  borderRadius: '8px',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  bgcolor: `${card.color}16`,
                  color: card.color,
                }}
              >
                {card.icon}
              </Box>
              <Typography variant="subtitle1" fontWeight={700}>
                {card.label}
              </Typography>
            </Box>
            <Typography variant="body2" color="text.secondary">
              {card.desc}
            </Typography>
          </Paper>
        </Grid>
      ))}
    </Grid>
  );

  // --- Render content area for active card ---
  const renderContent = () => {
    if (!activeCard) return null;

    return (
      <Box>
        {/* Back button + title */}
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 2 }}>
          <IconButton
            onClick={handleBackToCards}
            size="small"
            sx={{ bgcolor: 'rgba(0,0,0,0.04)', borderRadius: '8px' }}
          >
            <ArrowBackIcon />
          </IconButton>
          <Typography variant="h6" fontWeight={700}>
            {STAT_CARDS.find((c) => c.key === activeCard)?.label ?? ''}
          </Typography>
        </Box>

        {/* Group filter for project tab */}
        {activeCard === 'project' && (
          <Box sx={{ mb: 2 }}>
            <FormControl size="small" sx={{ minWidth: 150 }}>
              <InputLabel>分组筛选</InputLabel>
              <Select
                value={groupFilter}
                label="分组筛选"
                onChange={(e) => setGroupFilter(Number(e.target.value))}
                sx={{ borderRadius: '8px' }}
              >
                <MenuItem value={0}>全部分组</MenuItem>
                {groups.map((g) => (
                  <MenuItem key={g.id} value={g.id}>
                    {g.name}
                  </MenuItem>
                ))}
              </Select>
            </FormControl>
          </Box>
        )}

        {error && (
          <Alert severity="error" sx={{ mb: 2, borderRadius: '8px' }}>
            {error}
          </Alert>
        )}
        {loading ? (
          <Box sx={{ display: 'flex', justifyContent: 'center', py: 4 }}>
            <CircularProgress />
          </Box>
        ) : (
          <Box>
            {/* Week/Month detail table */}
            {(activeCard === 'week' || activeCard === 'month') && (
              <TableContainer component={Paper} className="table-responsive" sx={tablePaperSx}>
                <Table size="small">
                  <TableHead>
                    <TableRow>
                      <TableCell sx={{ fontWeight: 600 }}>时间</TableCell>
                      <TableCell align="right" sx={{ fontWeight: 600 }}>总数量</TableCell>
                      <TableCell align="right" sx={{ fontWeight: 600 }}>记录数</TableCell>
                    </TableRow>
                  </TableHead>
                  <TableBody>
                    {details.length === 0 ? (
                      <TableRow>
                        <TableCell colSpan={3} align="center">
                          暂无数据
                        </TableCell>
                      </TableRow>
                    ) : (
                      details.map((d, idx) => (
                        <TableRow key={idx} hover>
                          <TableCell>{d.period}</TableCell>
                          <TableCell align="right">
                            <Chip
                              label={d.total_quantity}
                              size="small"
                              sx={{
                                bgcolor: 'rgba(102,126,234,0.1)',
                                color: '#667eea',
                                fontWeight: 600,
                                borderRadius: '8px',
                              }}
                            />
                          </TableCell>
                          <TableCell align="right">{d.record_count}</TableCell>
                        </TableRow>
                      ))
                    )}
                  </TableBody>
                </Table>
              </TableContainer>
            )}

            {/* User table */}
            {activeCard === 'user' && (
              <TableContainer component={Paper} className="table-responsive" sx={tablePaperSx}>
                <Table size="small">
                  <TableHead>
                    <TableRow>
                      <TableCell sx={{ fontWeight: 600 }}>用户名</TableCell>
                      <TableCell align="right" sx={{ fontWeight: 600 }}>总数量</TableCell>
                      <TableCell align="right" sx={{ fontWeight: 600 }}>记录数</TableCell>
                    </TableRow>
                  </TableHead>
                  <TableBody>
                    {userStats.length === 0 ? (
                      <TableRow>
                        <TableCell colSpan={3} align="center">
                          暂无数据
                        </TableCell>
                      </TableRow>
                    ) : (
                      userStats.map((u, idx) => (
                        <TableRow key={idx} hover>
                          <TableCell>{u.user_name}</TableCell>
                          <TableCell align="right">
                            <Chip
                              label={u.total_quantity}
                              size="small"
                              sx={{
                                bgcolor: 'rgba(102,126,234,0.1)',
                                color: '#667eea',
                                fontWeight: 600,
                                borderRadius: '8px',
                              }}
                            />
                          </TableCell>
                          <TableCell align="right">{u.record_count}</TableCell>
                        </TableRow>
                      ))
                    )}
                  </TableBody>
                </Table>
              </TableContainer>
            )}

            {/* Project table */}
            {activeCard === 'project' && (
              <TableContainer component={Paper} className="table-responsive" sx={tablePaperSx}>
                <Table size="small">
                  <TableHead>
                    <TableRow>
                      <TableCell sx={{ fontWeight: 600 }}>分组</TableCell>
                      <TableCell sx={{ fontWeight: 600 }}>项目</TableCell>
                      <TableCell align="right" sx={{ fontWeight: 600 }}>总数量</TableCell>
                      <TableCell align="right" sx={{ fontWeight: 600 }}>记录数</TableCell>
                    </TableRow>
                  </TableHead>
                  <TableBody>
                    {projectStats.length === 0 ? (
                      <TableRow>
                        <TableCell colSpan={4} align="center">
                          暂无数据
                        </TableCell>
                      </TableRow>
                    ) : (
                      projectStats.map((p) => (
                        <TableRow key={p.project_id} hover>
                          <TableCell>{p.group_name}</TableCell>
                          <TableCell>{p.project_name}</TableCell>
                          <TableCell align="right">
                            <Chip
                              label={p.total_quantity}
                              size="small"
                              sx={{
                                bgcolor: 'rgba(102,126,234,0.1)',
                                color: '#667eea',
                                fontWeight: 600,
                                borderRadius: '8px',
                              }}
                            />
                          </TableCell>
                          <TableCell align="right">{p.record_count}</TableCell>
                        </TableRow>
                      ))
                    )}
                  </TableBody>
                </Table>
              </TableContainer>
            )}

            {/* Type table */}
            {activeCard === 'type' && (
              <TableContainer component={Paper} className="table-responsive" sx={tablePaperSx}>
                <Table size="small">
                  <TableHead>
                    <TableRow>
                      <TableCell sx={{ fontWeight: 600 }}>类型</TableCell>
                      <TableCell align="right" sx={{ fontWeight: 600 }}>总数量</TableCell>
                      <TableCell align="right" sx={{ fontWeight: 600 }}>记录数</TableCell>
                    </TableRow>
                  </TableHead>
                  <TableBody>
                    {typeStats.length === 0 ? (
                      <TableRow>
                        <TableCell colSpan={3} align="center">
                          暂无数据
                        </TableCell>
                      </TableRow>
                    ) : (
                      typeStats.map((t, idx) => (
                        <TableRow key={idx} hover>
                          <TableCell>
                            <Chip
                              label={t.instrument_type}
                              size="small"
                              color={
                                t.instrument_type === '液相'
                                  ? 'info'
                                  : t.instrument_type === '气相'
                                  ? 'success'
                                  : 'default'
                              }
                              sx={{ borderRadius: '8px' }}
                            />
                          </TableCell>
                          <TableCell align="right">
                            <Chip
                              label={t.total_quantity}
                              size="small"
                              sx={{
                                bgcolor: 'rgba(102,126,234,0.1)',
                                color: '#667eea',
                                fontWeight: 600,
                                borderRadius: '8px',
                              }}
                            />
                          </TableCell>
                          <TableCell align="right">{t.record_count}</TableCell>
                        </TableRow>
                      ))
                    )}
                  </TableBody>
                </Table>
              </TableContainer>
            )}

            {/* Instrument table */}
            {activeCard === 'instrument' && (
              <TableContainer component={Paper} className="table-responsive" sx={tablePaperSx}>
                <Table size="small">
                  <TableHead>
                    <TableRow>
                      <TableCell sx={{ fontWeight: 600 }}>仪器</TableCell>
                      <TableCell sx={{ fontWeight: 600 }}>类型</TableCell>
                      <TableCell align="right" sx={{ fontWeight: 600 }}>总数量</TableCell>
                      <TableCell align="right" sx={{ fontWeight: 600 }}>记录数</TableCell>
                      <TableCell align="right" sx={{ fontWeight: 600 }}>用户数</TableCell>
                    </TableRow>
                  </TableHead>
                  <TableBody>
                    {instrumentStats.length === 0 ? (
                      <TableRow>
                        <TableCell colSpan={5} align="center">
                          暂无数据
                        </TableCell>
                      </TableRow>
                    ) : (
                      instrumentStats.map((inst, idx) => (
                        <TableRow key={idx} hover>
                          <TableCell>
                            <Chip
                              label={inst.instrument}
                              size="small"
                              color={
                                inst.instrument_type === '液相'
                                  ? 'info'
                                  : 'success'
                              }
                              sx={{ borderRadius: '8px' }}
                            />
                          </TableCell>
                          <TableCell>
                            <Chip
                              label={inst.instrument_type}
                              size="small"
                              color={
                                inst.instrument_type === '液相'
                                  ? 'info'
                                  : 'success'
                              }
                              variant="outlined"
                              sx={{ borderRadius: '8px' }}
                            />
                          </TableCell>
                          <TableCell align="right">
                            <Chip
                              label={inst.total_quantity}
                              size="small"
                              sx={{
                                bgcolor: 'rgba(102,126,234,0.1)',
                                color: '#667eea',
                                fontWeight: 600,
                                borderRadius: '8px',
                              }}
                            />
                          </TableCell>
                          <TableCell align="right">
                            {inst.record_count}
                          </TableCell>
                          <TableCell align="right">
                            {inst.user_count}
                          </TableCell>
                        </TableRow>
                      ))
                    )}
                  </TableBody>
                </Table>
              </TableContainer>
            )}

            {/* User Log tab */}
            {activeCard === 'user-log' && (
              <Box>
                <Box sx={{ mb: 2, display: 'flex', gap: 2, alignItems: 'center', flexWrap: 'wrap' }}>
                  <FormControl size="small" sx={{ minWidth: 150 }}>
                    <InputLabel>筛选用户</InputLabel>
                    <Select
                      value={userFilter}
                      label="筛选用户"
                      onChange={(e) => {
                        setUserFilter(e.target.value);
                        setUserLogPage(1);
                      }}
                      sx={{ borderRadius: '8px' }}
                    >
                      <MenuItem value="">全部用户</MenuItem>
                      {[...new Set(userLogRecords.map(r => r.user_name))].map(name => (
                        <MenuItem key={name} value={name}>{name}</MenuItem>
                      ))}
                    </Select>
                  </FormControl>
                  <Typography variant="body2" color="text.secondary">
                    共 {userLogTotal} 条
                  </Typography>
                </Box>
                {userLogLoading ? (
                  <Box sx={{ display: 'flex', justifyContent: 'center', py: 4 }}>
                    <CircularProgress />
                  </Box>
                ) : (
                  <>
                    <TableContainer component={Paper} className="table-responsive" sx={tablePaperSx}>
                      <Table size="small">
                        <TableHead>
                          <TableRow>
                            <TableCell sx={{ fontWeight: 600 }}>序号</TableCell>
                            <TableCell sx={{ fontWeight: 600 }}>日期时间</TableCell>
                            <TableCell sx={{ fontWeight: 600 }}>用户名</TableCell>
                            <TableCell sx={{ fontWeight: 600 }}>实验室</TableCell>
                            <TableCell sx={{ fontWeight: 600 }}>项目</TableCell>
                            <TableCell align="right" sx={{ fontWeight: 600 }}>数量</TableCell>
                            <TableCell sx={{ fontWeight: 600 }}>操作</TableCell>
                          </TableRow>
                        </TableHead>
                        <TableBody>
                          {userLogRecords.length === 0 ? (
                            <TableRow>
                              <TableCell colSpan={7} align="center">
                                暂无数据
                              </TableCell>
                            </TableRow>
                          ) : (
                            userLogRecords.map((r, idx) => (
                              <TableRow key={r.id} hover>
                                <TableCell>
                                  {(userLogPage - 1) * PAGE_SIZE + idx + 1}
                                </TableCell>
                                <TableCell sx={{ whiteSpace: 'nowrap' }}>
                                  {r.recorded_at}
                                </TableCell>
                                <TableCell>{r.user_name}</TableCell>
                                <TableCell>{r.group_name}</TableCell>
                                <TableCell>{r.project_name}</TableCell>
                                <TableCell align="right">
                                  <Chip
                                    label={r.quantity}
                                    size="small"
                                    sx={{
                                      bgcolor: 'rgba(102,126,234,0.1)',
                                      color: '#667eea',
                                      fontWeight: 600,
                                      borderRadius: '8px',
                                    }}
                                  />
                                </TableCell>
                                <TableCell>
                                  <IconButton
                                    size="small"
                                    onClick={() => openEditDialog(r)}
                                    title="编辑"
                                    sx={{ color: '#667eea' }}
                                  >
                                    <EditIcon fontSize="small" />
                                  </IconButton>
                                  <IconButton
                                    size="small"
                                    color="error"
                                    onClick={() => openDeleteUserDialog(r.user_name)}
                                    title="删除该用户所有记录"
                                  >
                                    <DeleteForeverIcon fontSize="small" />
                                  </IconButton>
                                </TableCell>
                              </TableRow>
                            ))
                          )}
                        </TableBody>
                      </Table>
                    </TableContainer>
                    {/* Pagination */}
                    {userLogTotal > PAGE_SIZE && (
                      <Box
                        sx={{
                          display: 'flex',
                          justifyContent: 'center',
                          mt: 2,
                          gap: 1,
                        }}
                      >
                        <Button
                          size="small"
                          disabled={userLogPage <= 1}
                          onClick={() => loadUserLog(userLogPage - 1)}
                        >
                          上一页
                        </Button>
                        <Typography variant="body2" sx={{ alignSelf: 'center' }}>
                          {userLogPage} /{' '}
                          {Math.max(1, Math.ceil(userLogTotal / PAGE_SIZE))}
                        </Typography>
                        <Button
                          size="small"
                          disabled={userLogPage * PAGE_SIZE >= userLogTotal}
                          onClick={() => loadUserLog(userLogPage + 1)}
                        >
                          下一页
                        </Button>
                      </Box>
                    )}
                  </>
                )}
              </Box>
            )}
          </Box>
        )}
      </Box>
    );
  };

  return (
    <Box>
      {/* Page title with gradient accent */}
      <Typography
        variant="h5"
        fontWeight={700}
        sx={{
          mb: 3,
          px: 1,
          background: 'linear-gradient(135deg, #00897b, #43a047)',
          WebkitBackgroundClip: 'text',
          WebkitTextFillColor: 'transparent',
          backgroundClip: 'text',
        }}
      >
        统计分析
      </Typography>

      {/* Summary Cards */}
      {summary && (
        <StatsCards summary={summary} onCardClick={(t) => handleCardClick(t)} />
      )}

      {/* Date Range & Controls */}
      <Box sx={{ mb: 3, px: 1 }}>
        <DateRangePicker
          startDate={startDate}
          endDate={endDate}
          onStartChange={setStartDate}
          onEndChange={setEndDate}
        />
        <Box sx={{ display: 'flex', justifyContent: 'flex-end', mt: 1 }}>
          <Button
            variant="contained"
            startIcon={<DownloadIcon />}
            onClick={handleExport}
            size="small"
            sx={{
              borderRadius: '8px',
              background: 'linear-gradient(135deg, #00897b, #43a047)',
              boxShadow: '0 4px 14px rgba(0,137,123,0.3)',
              '&:hover': {
                background: 'linear-gradient(135deg, #00796b, #388e3c)',
              },
            }}
          >
            导出 Excel
          </Button>
        </Box>
      </Box>

      {/* Card Grid or Content */}
      {activeCard ? renderContent() : renderCardGrid()}

      {/* Edit Record Dialog */}
      <Dialog
        open={editDialogOpen}
        onClose={() => setEditDialogOpen(false)}
        maxWidth="sm"
        fullWidth
        PaperProps={{ sx: { borderRadius: '12px' } }}
      >
        <DialogTitle sx={{ fontWeight: 700 }}>编辑记录</DialogTitle>
        <DialogContent>
          {editError && (
            <Alert severity="error" sx={{ mb: 2, borderRadius: '8px' }}>
              {editError}
            </Alert>
          )}
          <TextField
            label="用户名"
            fullWidth
            value={editForm.user_name}
            onChange={(e) =>
              setEditForm({ ...editForm, user_name: e.target.value })
            }
            sx={{ mt: 1, '& .MuiOutlinedInput-root': { borderRadius: '8px' } }}
          />
          <TextField
            label="数量"
            type="number"
            fullWidth
            value={editForm.quantity}
            onChange={(e) =>
              setEditForm({
                ...editForm,
                quantity: Number(e.target.value),
              })
            }
            sx={{ mt: 2, '& .MuiOutlinedInput-root': { borderRadius: '8px' } }}
            inputProps={{ min: 1 }}
          />
          <TextField
            label="日期时间"
            type="datetime-local"
            fullWidth
            value={editForm.recorded_at}
            onChange={(e) =>
              setEditForm({ ...editForm, recorded_at: e.target.value })
            }
            sx={{ mt: 2, '& .MuiOutlinedInput-root': { borderRadius: '8px' } }}
            InputLabelProps={{ shrink: true }}
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setEditDialogOpen(false)}>取消</Button>
          <Button onClick={handleEditSave} variant="contained">
            保存
          </Button>
        </DialogActions>
      </Dialog>

      {/* Delete User Confirm Dialog */}
      <ConfirmDialog
        open={deleteUserDialogOpen}
        title="删除用户记录"
        message={`确定要删除用户「${deleteUserName}」在所选日期范围内的所有记录吗？此操作不可撤销。`}
        confirmText="删除"
        cancelText="取消"
        onConfirm={handleDeleteUser}
        onCancel={() => setDeleteUserDialogOpen(false)}
      />
    </Box>
  );
};

export default StatsPage;
