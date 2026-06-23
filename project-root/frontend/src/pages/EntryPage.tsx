import React, { useEffect, useState, useCallback } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import {
  Box,
  Typography,
  TextField,
  IconButton,
  Alert,
  CircularProgress,
  Paper,
  Snackbar,
  useMediaQuery,
  useTheme,
  Chip,
  Collapse,
} from '@mui/material';
import ArrowBackIcon from '@mui/icons-material/ArrowBack';
import ExpandMoreIcon from '@mui/icons-material/ExpandMore';
import ExpandLessIcon from '@mui/icons-material/ExpandLess';
import dayjs from 'dayjs';
import ProjectRow from '../components/ProjectRow';
import { getProjects, createRecord, getGroups } from '../api/client';
import type { Project, ProjectGroup } from '../types';

const USER_NAME_KEY = 'workload_user_name';

/**
 * EntryPage — uiverse.io card style.
 * Blue-cyan themed. Card-based project list with instrument type stripe.
 * Expandable user selector, pill submit buttons.
 */
const EntryPage: React.FC = () => {
  const { groupId } = useParams<{ groupId: string }>();
  const navigate = useNavigate();
  const theme = useTheme();
  const isMobile = useMediaQuery(theme.breakpoints.down('sm'));

  const [projects, setProjects] = useState<Project[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [groupName, setGroupName] = useState('');
  const [userName, setUserName] = useState(() => {
    return localStorage.getItem(USER_NAME_KEY) || '';
  });
  const [recordedAt, setRecordedAt] = useState(() => {
    return dayjs().format('YYYY-MM-DDTHH:mm');
  });
  const [configOpen, setConfigOpen] = useState(false);
  const [snackbar, setSnackbar] = useState<{
    open: boolean;
    message: string;
    severity: 'success' | 'error';
  }>({
    open: false,
    message: '',
    severity: 'success',
  });

  /** Recently used user names for quick selection */
  const recentUsers = (() => {
    try {
      const raw = localStorage.getItem('workload_recent_users');
      return raw ? (JSON.parse(raw) as string[]) : [];
    } catch {
      return [];
    }
  })();

  const saveRecentUser = (name: string) => {
    const trimmed = name.trim();
    if (!trimmed) return;
    const updated = [trimmed, ...recentUsers.filter((u) => u !== trimmed)].slice(0, 8);
    localStorage.setItem('workload_recent_users', JSON.stringify(updated));
  };

  const loadProjects = async () => {
    setLoading(true);
    setError('');
    try {
      const [projRes, groupsRes] = await Promise.all([
        getProjects({ group_id: Number(groupId), active_only: true }),
        getGroups(),
      ]);
      if (projRes.code === 0) {
        setProjects(projRes.data as Project[]);
      } else {
        setError(projRes.message);
      }
      if (groupsRes.code === 0) {
        const gs = groupsRes.data as ProjectGroup[];
        const g = gs.find((g) => g.id === Number(groupId));
        if (g) setGroupName(g.name);
      }
    } catch {
      setError('加载项目失败');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadProjects();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [groupId]);

  useEffect(() => {
    localStorage.setItem(USER_NAME_KEY, userName);
  }, [userName]);

  const handleSubmit = useCallback(
    async (projectId: number, quantity: number): Promise<boolean> => {
      if (!userName.trim()) {
        setSnackbar({
          open: true,
          message: '请先输入用户名',
          severity: 'error',
        });
        return false;
      }
      try {
        const isoDate = dayjs(recordedAt).format('YYYY-MM-DDTHH:mm:ss');
        const res = await createRecord({
          project_id: projectId,
          user_name: userName.trim(),
          quantity,
          recorded_at: isoDate,
        });
        if (res.code === 0) {
          saveRecentUser(userName.trim());
          setSnackbar({ open: true, message: '录入成功', severity: 'success' });
          return true;
        } else {
          setSnackbar({
            open: true,
            message: res.message,
            severity: 'error',
          });
          return false;
        }
      } catch {
        setSnackbar({
          open: true,
          message: '提交失败，请检查网络',
          severity: 'error',
        });
        return false;
      }
    },
    [userName, recordedAt]
  );

  if (loading) {
    return (
      <Box
        sx={{
          display: 'flex',
          justifyContent: 'center',
          alignItems: 'center',
          minHeight: '60vh',
        }}
      >
        <CircularProgress />
      </Box>
    );
  }

  if (error) {
    return (
      <Box sx={{ p: 2 }}>
        <Alert severity="error">{error}</Alert>
      </Box>
    );
  }

  return (
    <Box sx={{ maxWidth: 640, mx: 'auto' }}>
      {/* Header with back button, date, and lab name */}
      <Box
        sx={{
          display: 'flex',
          alignItems: 'center',
          mb: 3,
          gap: 1.5,
        }}
      >
        <IconButton
          onClick={() => navigate('/')}
          sx={{
            bgcolor: 'rgba(30,136,229,0.08)',
            '&:hover': { bgcolor: 'rgba(30,136,229,0.15)' },
          }}
        >
          <ArrowBackIcon sx={{ color: '#1e88e5' }} />
        </IconButton>
        <Box>
          <Typography variant="h5" fontWeight={700}>
            录入工作量
          </Typography>
          <Typography variant="body2" color="text.secondary">
            {groupName || `分组 ${groupId}`} · {dayjs(recordedAt).format('YYYY-MM-DD HH:mm')}
          </Typography>
        </Box>
      </Box>

      {/* Expandable user & date config card */}
      <Paper
        elevation={0}
        sx={{
          mb: 3,
          borderRadius: 4,
          background: 'linear-gradient(145deg, #ffffff, #f5f5f5)',
          border: '1px solid rgba(0,0,0,0.06)',
          boxShadow: '0 4px 20px rgba(0,0,0,0.06)',
          overflow: 'hidden',
        }}
      >
        {/* Always visible: username input */}
        <Box
          sx={{
            p: 2,
            display: 'flex',
            flexDirection: isMobile ? 'column' : 'row',
            gap: 2,
            alignItems: isMobile ? 'stretch' : 'center',
          }}
        >
          <TextField
            size="small"
            label="用户名"
            value={userName}
            onChange={(e) => setUserName(e.target.value)}
            placeholder="请输入您的姓名"
            sx={{
              flex: 1,
              '& .MuiOutlinedInput-root': { borderRadius: 3 },
            }}
            required
          />
          <IconButton
            onClick={() => setConfigOpen(!configOpen)}
            size="small"
            sx={{
              alignSelf: isMobile ? 'flex-end' : 'center',
              bgcolor: configOpen ? 'rgba(30,136,229,0.1)' : 'transparent',
              borderRadius: 2,
              transition: 'all 0.2s',
            }}
          >
            {configOpen ? (
              <ExpandLessIcon sx={{ color: '#1e88e5' }} />
            ) : (
              <ExpandMoreIcon />
            )}
          </IconButton>
        </Box>

        {/* Recent users quick select */}
        {recentUsers.length > 0 && (
          <Box sx={{ px: 2, pb: 1, display: 'flex', gap: 0.75, flexWrap: 'wrap' }}>
            {recentUsers.slice(0, isMobile ? 4 : 8).map((u) => (
              <Chip
                key={u}
                label={u}
                size="small"
                variant={userName === u ? 'filled' : 'outlined'}
                color={userName === u ? 'primary' : 'default'}
                onClick={() => setUserName(u)}
                sx={{
                  borderRadius: 2,
                  cursor: 'pointer',
                  fontWeight: userName === u ? 600 : 400,
                }}
              />
            ))}
          </Box>
        )}

        {/* Expandable: date-time */}
        <Collapse in={configOpen}>
          <Box sx={{ px: 2, pb: 2 }}>
            <TextField
              size="small"
              label="日期时间"
              type="datetime-local"
              value={recordedAt}
              onChange={(e) => setRecordedAt(e.target.value)}
              InputLabelProps={{ shrink: true }}
              fullWidth
              sx={{
                '& .MuiOutlinedInput-root': { borderRadius: 3 },
              }}
            />
          </Box>
        </Collapse>
      </Paper>

      {/* Project card list */}
      {projects.length === 0 ? (
        <Box sx={{ textAlign: 'center', py: 6 }}>
          <Typography color="text.secondary">
            该分组下暂无项目
          </Typography>
        </Box>
      ) : (
        <Box sx={{ display: 'flex', flexDirection: 'column' }}>
          {projects.map((project) => (
            <ProjectRow
              key={project.id}
              project={project}
              onSubmit={handleSubmit}
            />
          ))}
        </Box>
      )}

      {/* Snackbar */}
      <Snackbar
        open={snackbar.open}
        autoHideDuration={3000}
        onClose={() => setSnackbar({ ...snackbar, open: false })}
        anchorOrigin={{ vertical: 'bottom', horizontal: 'center' }}
      >
        <Alert
          severity={snackbar.severity}
          onClose={() => setSnackbar({ ...snackbar, open: false })}
          variant="filled"
          sx={{ borderRadius: 3 }}
        >
          {snackbar.message}
        </Alert>
      </Snackbar>
    </Box>
  );
};

export default EntryPage;
