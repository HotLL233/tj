import React, { useEffect, useState, useMemo } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  Box,
  Typography,
  Fab,
  CircularProgress,
  Alert,
  TextField,
  InputAdornment,
} from '@mui/material';
import BarChartIcon from '@mui/icons-material/BarChart';
import SearchIcon from '@mui/icons-material/Search';
import GroupCard from '../components/GroupCard';
import { getGroups } from '../api/client';
import type { ProjectGroup } from '../types';

/**
 * HomePage — uiverse.io modern card style.
 * Features hero header with decorative gradient background,
 * search bar, and 3-column lab card grid.
 */
const HomePage: React.FC = () => {
  const navigate = useNavigate();
  const [groups, setGroups] = useState<ProjectGroup[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [search, setSearch] = useState('');

  const loadGroups = async () => {
    setLoading(true);
    setError('');
    try {
      const res = await getGroups();
      if (res.code === 0) {
        setGroups(res.data as ProjectGroup[]);
      } else {
        setError(res.message);
      }
    } catch {
      setError('加载失败，请检查网络连接');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadGroups();
  }, []);

  const filteredGroups = useMemo(() => {
    if (!search.trim()) return groups;
    const q = search.trim().toLowerCase();
    return groups.filter((g) => g.name.toLowerCase().includes(q));
  }, [groups, search]);

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
        <Alert
          severity="error"
          action={
            <Typography
              component="button"
              onClick={loadGroups}
              sx={{
                cursor: 'pointer',
                border: 'none',
                bgcolor: 'transparent',
                color: 'inherit',
                textDecoration: 'underline',
              }}
            >
              重试
            </Typography>
          }
        >
          {error}
        </Alert>
      </Box>
    );
  }

  return (
    <Box>
      {/* Hero Header with decorative gradient */}
      <Box
        sx={{
          position: 'relative',
          borderRadius: 4,
          p: { xs: 3, md: 5 },
          mb: 4,
          background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
          overflow: 'hidden',
        }}
      >
        {/* Decorative background shapes */}
        <Box
          sx={{
            position: 'absolute',
            top: -40,
            right: -30,
            width: 180,
            height: 180,
            borderRadius: '50%',
            background: 'rgba(255,255,255,0.08)',
          }}
        />
        <Box
          sx={{
            position: 'absolute',
            bottom: -50,
            left: -20,
            width: 140,
            height: 140,
            borderRadius: '50%',
            background: 'rgba(255,255,255,0.06)',
          }}
        />
        <Box
          sx={{
            position: 'absolute',
            top: 20,
            left: '40%',
            width: 60,
            height: 60,
            borderRadius: '50%',
            background: 'rgba(255,255,255,0.1)',
          }}
        />

        <Typography
          variant="h4"
          component="h1"
          fontWeight={800}
          sx={{
            color: '#fff',
            mb: 1,
            position: 'relative',
            zIndex: 1,
          }}
        >
          项目分组
        </Typography>
        <Typography
          variant="body1"
          sx={{
            color: 'rgba(255,255,255,0.85)',
            position: 'relative',
            zIndex: 1,
            mb: 2,
          }}
        >
          选择实验室开始录入工作量数据
        </Typography>

        {/* Search bar inside hero */}
        <TextField
          size="small"
          placeholder="搜索实验室..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          InputProps={{
            startAdornment: (
              <InputAdornment position="start">
                <SearchIcon sx={{ color: 'rgba(255,255,255,0.6)' }} />
              </InputAdornment>
            ),
          }}
          sx={{
            position: 'relative',
            zIndex: 1,
            maxWidth: 400,
            '& .MuiOutlinedInput-root': {
              bgcolor: 'rgba(255,255,255,0.15)',
              borderRadius: 3,
              color: '#fff',
              '& fieldset': { borderColor: 'rgba(255,255,255,0.2)' },
              '&:hover fieldset': { borderColor: 'rgba(255,255,255,0.4)' },
              '&.Mui-focused fieldset': { borderColor: 'rgba(255,255,255,0.6)' },
              '& input::placeholder': { color: 'rgba(255,255,255,0.5)' },
            },
          }}
        />
      </Box>

      {/* Grid of Group Cards */}
      {filteredGroups.length === 0 ? (
        <Box sx={{ textAlign: 'center', py: 8 }}>
          <Typography variant="h6" color="text.secondary" gutterBottom>
            {search ? '未找到匹配的实验室' : '暂无项目分组'}
          </Typography>
          <Typography variant="body2" color="text.secondary">
            {search ? '请尝试其他关键词' : '请前往管理页面创建分组和项目'}
          </Typography>
        </Box>
      ) : (
        <Box
          sx={{
            display: 'grid',
            gridTemplateColumns: {
              xs: '1fr',
              sm: 'repeat(2, 1fr)',
              md: 'repeat(3, 1fr)',
            },
            gap: 2.5,
            px: { xs: 0.5, sm: 1 },
          }}
        >
          {filteredGroups.map((group) => (
            <GroupCard
              key={group.id}
              group={group}
              onClick={() => navigate(`/entry/${group.id}`)}
            />
          ))}
        </Box>
      )}

      {/* Desktop Quick Action FAB */}
      <Box sx={{ display: { xs: 'none', md: 'flex' }, gap: 1, mt: 3, justifyContent: 'center' }}>
        <Fab
          variant="extended"
          size="small"
          onClick={() => navigate('/stats')}
          sx={{
            background: 'linear-gradient(135deg, #667eea, #764ba2)',
            color: '#fff',
            boxShadow: '0 4px 14px rgba(102,126,234,0.4)',
            '&:hover': {
              background: 'linear-gradient(135deg, #5a6fd6, #6a4190)',
            },
          }}
        >
          <BarChartIcon sx={{ mr: 0.5 }} />
          查看统计
        </Fab>
      </Box>

      {/* Mobile FAB */}
      <Fab
        onClick={() => navigate('/stats')}
        sx={{
          display: { xs: 'flex', md: 'none' },
          position: 'fixed',
          bottom: 72,
          right: 16,
          zIndex: 100,
          background: 'linear-gradient(135deg, #667eea, #764ba2)',
          color: '#fff',
          boxShadow: '0 4px 16px rgba(102,126,234,0.5)',
          '&:hover': {
            background: 'linear-gradient(135deg, #5a6fd6, #6a4190)',
          },
        }}
      >
        <BarChartIcon />
      </Fab>
    </Box>
  );
};

export default HomePage;
