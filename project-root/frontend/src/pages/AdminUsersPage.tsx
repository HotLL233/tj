import React, { useEffect, useState } from 'react';
import {
  Box, Paper, Typography, Button, Table, TableHead, TableRow, TableCell, TableBody, IconButton,
  Dialog, DialogTitle, DialogContent, DialogActions, TextField, Chip, Stack, Alert, Snackbar, MenuItem,
} from '@mui/material';
import EditIcon from '@mui/icons-material/Edit';
import DeleteIcon from '@mui/icons-material/Delete';
import KeyIcon from '@mui/icons-material/Key';
import { useAuth } from '../context/AuthContext';
import { getUsers, createUser, updateUser, deleteUser, resetPassword, getRoles } from '../api/client';
import type { UserPublic, RoleWithPermissions } from '../types/lims';

const AdminUsersPage: React.FC = () => {
  const { user: me, hasPerm } = useAuth();
  const [users, setUsers] = useState<UserPublic[]>([]);
  const [roles, setRoles] = useState<RoleWithPermissions[]>([]);
  const [err, setErr] = useState('');
  const [msg, setMsg] = useState('');

  const load = () => { getUsers().then((r) => { if (r.code === 0) setUsers(r.data); }).catch((e) => setErr(e.message)); getRoles().then((r) => { if (r.code === 0) setRoles(r.data); }).catch(() => {}); };
  useEffect(load, []); // eslint-disable-line
  if (!hasPerm('user:manage')) return <Alert severity="error">无权限访问</Alert>;

  const [open, setOpen] = useState(false);
  const [editing, setEditing] = useState<UserPublic | null>(null);
  const [form, setForm] = useState({ username: '', display_name: '', password: '', role_id: '', is_active: 1 });
  const [pwdOpen, setPwdOpen] = useState(false);
  const [pwdUser, setPwdUser] = useState<UserPublic | null>(null);
  const [newPwd, setNewPwd] = useState('');

  const openNew = () => { setEditing(null); setForm({ username: '', display_name: '', password: '', role_id: roles[0]?.id ? String(roles[0].id) : '', is_active: 1 }); setOpen(true); };
  const openEdit = (u: UserPublic) => { setEditing(u); setForm({ username: u.username, display_name: u.display_name, password: '', role_id: String(u.role_id), is_active: u.is_active }); setOpen(true); };
  const save = async () => {
    try {
      if (editing) { const r = await updateUser(editing.id, { display_name: form.display_name, role_id: Number(form.role_id), is_active: form.is_active }); if (r.code !== 0) throw new Error(r.message); }
      else { const r = await createUser({ username: form.username, display_name: form.display_name || undefined, password: form.password, role_id: Number(form.role_id) }); if (r.code !== 0) throw new Error(r.message); }
      setOpen(false); setMsg('已保存'); load();
    } catch (e) { setErr(e instanceof Error ? e.message : '保存失败'); }
  };
  const onDelete = async (u: UserPublic) => { if (u.id === me?.id) { setErr('不能删除当前登录账号'); return; } if (!confirm(`停用用户「${u.username}」？`)) return; try { const r = await deleteUser(u.id); if (r.code !== 0) throw new Error(r.message); setMsg('已停用'); load(); } catch (e) { setErr(e instanceof Error ? e.message : '删除失败'); } };
  const openPwd = (u: UserPublic) => { setPwdUser(u); setNewPwd(''); setPwdOpen(true); };
  const onResetPwd = async () => { if (!pwdUser) return; if (newPwd.length < 6) { setErr('密码至少 6 位'); return; } try { const r = await resetPassword(pwdUser.id, { new_password: newPwd }); if (r.code !== 0) throw new Error(r.message); setPwdOpen(false); setMsg('密码已重置'); } catch (e) { setErr(e instanceof Error ? e.message : '重置失败'); } };

  return (
    <Box>
      <Stack direction="row" justifyContent="space-between" alignItems="center" sx={{ mb: 2 }}>
        <Typography variant="h4" fontWeight={700}>用户管理</Typography>
        <Button variant="contained" onClick={openNew}>新增用户</Button>
      </Stack>
      <Paper elevation={1} sx={{ p: 1 }}>
        <Table size="small">
          <TableHead><TableRow><TableCell>用户名</TableCell><TableCell>显示名</TableCell><TableCell>角色</TableCell><TableCell>状态</TableCell><TableCell>强制改密</TableCell><TableCell>操作</TableCell></TableRow></TableHead>
          <TableBody>
            {users.map((u) => (
              <TableRow key={u.id}>
                <TableCell>{u.username}</TableCell><TableCell>{u.display_name}</TableCell><TableCell>{u.role_name}</TableCell>
                <TableCell><Chip size="small" label={u.is_active ? '启用' : '停用'} color={u.is_active ? 'success' : 'default'} /></TableCell>
                <TableCell>{u.must_change_password ? <Chip size="small" label="是" color="warning" /> : '否'}</TableCell>
                <TableCell>
                  <IconButton size="small" onClick={() => openEdit(u)}><EditIcon fontSize="small" /></IconButton>
                  <IconButton size="small" title="重置密码" onClick={() => openPwd(u)}><KeyIcon fontSize="small" /></IconButton>
                  <IconButton size="small" color="error" onClick={() => onDelete(u)}><DeleteIcon fontSize="small" /></IconButton>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </Paper>

      <Dialog open={open} onClose={() => setOpen(false)} maxWidth="xs" fullWidth>
        <DialogTitle>{editing ? '编辑用户' : '新增用户'}</DialogTitle>
        <DialogContent>
          <TextField label="用户名" fullWidth margin="normal" disabled={!!editing} value={form.username} onChange={(e) => setForm({ ...form, username: e.target.value })} />
          <TextField label="显示名" fullWidth margin="normal" value={form.display_name} onChange={(e) => setForm({ ...form, display_name: e.target.value })} />
          {!editing && <TextField label="初始密码" type="password" fullWidth margin="normal" value={form.password} onChange={(e) => setForm({ ...form, password: e.target.value })} />}
          <TextField select label="角色" fullWidth margin="normal" value={form.role_id} onChange={(e) => setForm({ ...form, role_id: e.target.value })}>
            {roles.map((r) => <MenuItem key={r.id} value={String(r.id)}>{r.name}</MenuItem>)}
          </TextField>
          <TextField select label="状态" fullWidth margin="normal" value={form.is_active} onChange={(e) => setForm({ ...form, is_active: Number(e.target.value) })}>
            <MenuItem value={1}>启用</MenuItem><MenuItem value={0}>停用</MenuItem>
          </TextField>
        </DialogContent>
        <DialogActions><Button onClick={() => setOpen(false)}>取消</Button><Button variant="contained" onClick={save}>保存</Button></DialogActions>
      </Dialog>

      <Dialog open={pwdOpen} onClose={() => setPwdOpen(false)} maxWidth="xs" fullWidth>
        <DialogTitle>重置密码：{pwdUser?.username}</DialogTitle>
        <DialogContent>
          <TextField label="新密码" type="password" fullWidth margin="normal" value={newPwd} onChange={(e) => setNewPwd(e.target.value)} />
        </DialogContent>
        <DialogActions><Button onClick={() => setPwdOpen(false)}>取消</Button><Button variant="contained" onClick={onResetPwd}>重置</Button></DialogActions>
      </Dialog>

      <Snackbar open={!!msg} autoHideDuration={2500} onClose={() => setMsg('')} message={msg} />
      <Snackbar open={!!err} autoHideDuration={4000} onClose={() => setErr('')}><Alert severity="error" onClose={() => setErr('')}>{err}</Alert></Snackbar>
    </Box>
  );
};

export default AdminUsersPage;
