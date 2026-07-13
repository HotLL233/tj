import React, { useState } from 'react';
import {
  Box, Paper, Typography, Stack, TextField, Button, Divider, Alert, Snackbar, Chip, Dialog, DialogTitle, DialogContent, DialogActions,
} from '@mui/material';
import { useAuth } from '../context/AuthContext';
import { changePassword } from '../api/client';
import { PERMISSIONS } from '../constants/permissions';

const ProfilePage: React.FC = () => {
  const { user } = useAuth();
  const [err, setErr] = useState('');
  const [msg, setMsg] = useState('');

  const [open, setOpen] = useState(false);
  const [oldPwd, setOldPwd] = useState('');
  const [newPwd, setNewPwd] = useState('');
  const [confirmPwd, setConfirmPwd] = useState('');

  const permLabels = (user?.permissions ?? [])
    .map((k) => (k === '*' ? '全部权限' : PERMISSIONS.find((p) => p.key === k)?.label ?? k));

  const openPwd = () => { setOldPwd(''); setNewPwd(''); setConfirmPwd(''); setOpen(true); };
  const onSave = async () => {
    if (newPwd.length < 6) { setErr('新密码至少 6 位'); return; }
    if (newPwd !== confirmPwd) { setErr('两次输入的新密码不一致'); return; }
    try {
      const r = await changePassword({ old_password: oldPwd || undefined, new_password: newPwd });
      if (r.code !== 0) throw new Error(r.message);
      setOpen(false); setMsg('密码已修改');
    } catch (e) { setErr(e instanceof Error ? e.message : '修改失败'); }
  };

  if (!user) return <Alert severity="info">加载中…</Alert>;

  return (
    <Box sx={{ maxWidth: 720, mx: 'auto' }}>
      <Typography variant="h4" fontWeight={700} sx={{ mb: 2 }}>个人中心</Typography>
      <Paper elevation={1} sx={{ p: 3 }}>
        <Stack spacing={1.5}>
          <Stack direction="row" justifyContent="space-between"><Typography color="text.secondary">用户名</Typography><Typography>{user.username}</Typography></Stack>
          <Stack direction="row" justifyContent="space-between"><Typography color="text.secondary">显示名</Typography><Typography>{user.display_name || '-'}</Typography></Stack>
          <Stack direction="row" justifyContent="space-between"><Typography color="text.secondary">角色</Typography><Typography>{user.role}</Typography></Stack>
          <Stack direction="row" justifyContent="space-between" alignItems="center">
            <Typography color="text.secondary">强制改密</Typography>
            {user.must_change_password ? <Chip size="small" label="需要" color="warning" /> : <Chip size="small" label="无需" />}
          </Stack>
          <Divider sx={{ my: 1 }} />
          <Box>
            <Typography color="text.secondary" sx={{ mb: 1 }}>权限</Typography>
            <Stack direction="row" flexWrap="wrap" gap={1}>
              {permLabels.map((l) => <Chip key={l} size="small" label={l} color={l === '全部权限' ? 'primary' : 'default'} />)}
            </Stack>
          </Box>
        </Stack>
      </Paper>

      <Stack direction="row" justifyContent="flex-end" sx={{ mt: 2 }}>
        <Button variant="contained" onClick={openPwd}>修改密码</Button>
      </Stack>

      <Dialog open={open} onClose={() => setOpen(false)} maxWidth="xs" fullWidth>
        <DialogTitle>修改密码</DialogTitle>
        <DialogContent>
          <TextField label="旧密码" type="password" fullWidth margin="normal" value={oldPwd} onChange={(e) => setOldPwd(e.target.value)} />
          <TextField label="新密码" type="password" fullWidth margin="normal" value={newPwd} onChange={(e) => setNewPwd(e.target.value)} />
          <TextField label="确认新密码" type="password" fullWidth margin="normal" value={confirmPwd} onChange={(e) => setConfirmPwd(e.target.value)} />
        </DialogContent>
        <DialogActions><Button onClick={() => setOpen(false)}>取消</Button><Button variant="contained" onClick={onSave}>保存</Button></DialogActions>
      </Dialog>

      <Snackbar open={!!msg} autoHideDuration={2500} onClose={() => setMsg('')} message={msg} />
      <Snackbar open={!!err} autoHideDuration={4000} onClose={() => setErr('')}><Alert severity="error" onClose={() => setErr('')}>{err}</Alert></Snackbar>
    </Box>
  );
};

export default ProfilePage;
