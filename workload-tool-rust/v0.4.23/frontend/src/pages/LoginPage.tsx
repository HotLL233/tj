import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Box, Paper, TextField, Button, Typography, Alert, CircularProgress, Stack } from '@mui/material';
import LockOutlinedIcon from '@mui/icons-material/LockOutlined';
import { useAuth } from '../context/AuthContext';

const LoginPage: React.FC = () => {
  const { login, changePassword } = useAuth();
  const navigate = useNavigate();
  const [username, setUsername] = useState('admin');
  const [password, setPassword] = useState('admin123');
  const [err, setErr] = useState('');
  const [busy, setBusy] = useState(false);

  // 首次强制改密
  const [needChange, setNeedChange] = useState(false);
  const [newPwd, setNewPwd] = useState('');
  const [confirmPwd, setConfirmPwd] = useState('');
  const [changeErr, setChangeErr] = useState('');

  const onSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setErr(''); setBusy(true);
    try {
      const data = await login(username.trim(), password);
      if (data.must_change_password) {
        setNeedChange(true);
      } else {
        navigate('/');
      }
    } catch (e2) {
      setErr(e2 instanceof Error ? e2.message : '登录失败');
    } finally {
      setBusy(false);
    }
  };

  const onForceChange = async (e: React.FormEvent) => {
    e.preventDefault();
    setChangeErr('');
    if (newPwd.length < 6) { setChangeErr('密码至少 6 位'); return; }
    if (newPwd !== confirmPwd) { setChangeErr('两次输入的密码不一致'); return; }
    try {
      await changePassword(undefined, newPwd);
      navigate('/');
    } catch (e2) {
      setChangeErr(e2 instanceof Error ? e2.message : '修改失败');
    }
  };

  return (
    <Box sx={{ minHeight: '100vh', display: 'flex', alignItems: 'center', justifyContent: 'center', background: 'linear-gradient(135deg,#667eea 0%,#764ba2 100%)' }}>
      <Paper elevation={3} sx={{ p: 4, width: 380, maxWidth: '90vw' }}>
        <Stack spacing={2} alignItems="center">
          <LockOutlinedIcon color="primary" sx={{ fontSize: 40 }} />
          <Typography variant="h5" fontWeight={700}>本地化 LIMS</Typography>
          <Typography variant="body2" color="text.secondary">混合本地实验室信息管理系统</Typography>
        </Stack>

        {!needChange ? (
          <Box component="form" onSubmit={onSubmit} sx={{ mt: 3 }}>
            {err && <Alert severity="error" sx={{ mb: 2 }}>{err}</Alert>}
            <TextField label="用户名" fullWidth required value={username} onChange={(e) => setUsername(e.target.value)} margin="normal" />
            <TextField label="密码" type="password" fullWidth required value={password} onChange={(e) => setPassword(e.target.value)} margin="normal" />
            <Button type="submit" variant="contained" fullWidth disabled={busy} sx={{ mt: 2, py: 1.2 }}>
              {busy ? <CircularProgress size={22} color="inherit" /> : '登 录'}
            </Button>
            <Typography variant="caption" color="text.secondary" sx={{ display: 'block', mt: 2, textAlign: 'center' }}>
              初始账号 admin / admin123（首次登录需修改密码）
            </Typography>
          </Box>
        ) : (
          <Box component="form" onSubmit={onForceChange} sx={{ mt: 3 }}>
            <Alert severity="warning" sx={{ mb: 2 }}>首次登录，请修改初始密码</Alert>
            {changeErr && <Alert severity="error" sx={{ mb: 2 }}>{changeErr}</Alert>}
            <TextField label="新密码" type="password" fullWidth required value={newPwd} onChange={(e) => setNewPwd(e.target.value)} margin="normal" />
            <TextField label="确认新密码" type="password" fullWidth required value={confirmPwd} onChange={(e) => setConfirmPwd(e.target.value)} margin="normal" />
            <Button type="submit" variant="contained" fullWidth sx={{ mt: 2, py: 1.2 }}>确认修改</Button>
          </Box>
        )}
      </Paper>
    </Box>
  );
};

export default LoginPage;
