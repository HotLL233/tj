import React, { useEffect, useState } from 'react';
import {
  Box, Paper, Typography, Button, Table, TableHead, TableRow, TableCell, TableBody, IconButton,
  Dialog, DialogTitle, DialogContent, DialogActions, TextField, Stack, Alert, Snackbar, Chip,
  Checkbox, FormControlLabel, Divider,
} from '@mui/material';
import EditIcon from '@mui/icons-material/Edit';
import DeleteIcon from '@mui/icons-material/Delete';
import VerifiedUserIcon from '@mui/icons-material/VerifiedUser';
import { useAuth } from '../context/AuthContext';
import { getRoles, createRole, updateRole, deleteRole, setRolePermissions } from '../api/client';
import type { RoleWithPermissions } from '../types/lims';
import { PERMISSIONS, ALL_PERMISSION, hasPermission } from '../constants/permissions';

const AdminRolesPage: React.FC = () => {
  const { hasPerm } = useAuth();
  const [roles, setRoles] = useState<RoleWithPermissions[]>([]);
  const [err, setErr] = useState('');
  const [msg, setMsg] = useState('');

  const load = () => { getRoles().then((r) => { if (r.code === 0) setRoles(r.data); else setErr(r.message); }).catch((e) => setErr(e.message)); };
  useEffect(load, []); // eslint-disable-line
  if (!hasPerm('role:manage')) return <Alert severity="error">无权限访问</Alert>;

  // 角色基本信息编辑
  const [open, setOpen] = useState(false);
  const [editing, setEditing] = useState<RoleWithPermissions | null>(null);
  const [form, setForm] = useState({ name: '', description: '' });

  // 权限矩阵编辑
  const [permOpen, setPermOpen] = useState(false);
  const [permRole, setPermRole] = useState<RoleWithPermissions | null>(null);
  const [permSel, setPermSel] = useState<string[]>([]);

  const openNew = () => { setEditing(null); setForm({ name: '', description: '' }); setOpen(true); };
  const openEdit = (r: RoleWithPermissions) => { setEditing(r); setForm({ name: r.name, description: r.description }); setOpen(true); };
  const save = async () => {
    try {
      if (editing) { const r = await updateRole(editing.id, { name: form.name, description: form.description }); if (r.code !== 0) throw new Error(r.message); }
      else { const r = await createRole({ name: form.name, description: form.description }); if (r.code !== 0) throw new Error(r.message); }
      setOpen(false); setMsg('已保存'); load();
    } catch (e) { setErr(e instanceof Error ? e.message : '保存失败'); }
  };
  const onDelete = async (r: RoleWithPermissions) => {
    if (r.is_system) { setErr('系统内置角色不可删除'); return; }
    if (!confirm(`删除角色「${r.name}」？`)) return;
    try { const res = await deleteRole(r.id); if (res.code !== 0) throw new Error(res.message); setMsg('已删除'); load(); } catch (e) { setErr(e instanceof Error ? e.message : '删除失败'); }
  };

  const openPerm = (r: RoleWithPermissions) => {
    setPermRole(r);
    setPermSel(r.permissions.includes(ALL_PERMISSION) ? [ALL_PERMISSION] : r.permissions);
    setPermOpen(true);
  };
  const togglePerm = (key: string) => {
    setPermSel((prev) => (prev.includes(key) ? prev.filter((k) => k !== key) : [...prev, key]));
  };
  const savePerm = async () => {
    if (!permRole) return;
    try {
      const res = await setRolePermissions(permRole.id, permSel);
      if (res.code !== 0) throw new Error(res.message);
      setPermOpen(false); setMsg('权限已更新'); load();
    } catch (e) { setErr(e instanceof Error ? e.message : '更新失败'); }
  };

  // 按 group 分组展示权限矩阵
  const groups = PERMISSIONS.reduce<Record<string, typeof PERMISSIONS>>((acc, p) => {
    (acc[p.group] = acc[p.group] || []).push(p);
    return acc;
  }, {});

  const isAll = (r: RoleWithPermissions) => r.permissions.includes(ALL_PERMISSION);

  return (
    <Box>
      <Stack direction="row" justifyContent="space-between" alignItems="center" sx={{ mb: 2 }}>
        <Typography variant="h4" fontWeight={700}>角色管理</Typography>
        <Button variant="contained" onClick={openNew}>新增角色</Button>
      </Stack>
      <Paper elevation={1} sx={{ p: 1 }}>
        <Table size="small">
          <TableHead><TableRow><TableCell>角色名</TableCell><TableCell>说明</TableCell><TableCell>类型</TableCell><TableCell>权限数</TableCell><TableCell>操作</TableCell></TableRow></TableHead>
          <TableBody>
            {roles.map((r) => (
              <TableRow key={r.id}>
                <TableCell>{r.name}</TableCell>
                <TableCell>{r.description}</TableCell>
                <TableCell>{r.is_system ? <Chip size="small" label="内置" color="info" /> : <Chip size="small" label="自定义" />}</TableCell>
                <TableCell>{isAll(r) ? <Chip size="small" label="全部" color="primary" /> : r.permissions.length}</TableCell>
                <TableCell>
                  <IconButton size="small" title="权限" onClick={() => openPerm(r)}><VerifiedUserIcon fontSize="small" /></IconButton>
                  <IconButton size="small" onClick={() => openEdit(r)}><EditIcon fontSize="small" /></IconButton>
                  <IconButton size="small" color="error" disabled={!!r.is_system} onClick={() => onDelete(r)}><DeleteIcon fontSize="small" /></IconButton>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </Paper>

      <Dialog open={open} onClose={() => setOpen(false)} maxWidth="xs" fullWidth>
        <DialogTitle>{editing ? '编辑角色' : '新增角色'}</DialogTitle>
        <DialogContent>
          <TextField label="角色名" fullWidth margin="normal" disabled={!!editing?.is_system} value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} />
          <TextField label="说明" fullWidth margin="normal" value={form.description} onChange={(e) => setForm({ ...form, description: e.target.value })} />
        </DialogContent>
        <DialogActions><Button onClick={() => setOpen(false)}>取消</Button><Button variant="contained" onClick={save}>保存</Button></DialogActions>
      </Dialog>

      <Dialog open={permOpen} onClose={() => setPermOpen(false)} maxWidth="sm" fullWidth>
        <DialogTitle>权限设置：{permRole?.name}</DialogTitle>
        <DialogContent dividers>
          <FormControlLabel
            control={<Checkbox checked={permSel.includes(ALL_PERMISSION)} disabled={!!permRole?.is_system} onChange={() => setPermSel((prev) => (prev.includes(ALL_PERMISSION) ? [] : [ALL_PERMISSION]))} />}
            label={<strong>全部权限（* 通配）</strong>}
          />
          <Divider sx={{ my: 1 }} />
          {Object.entries(groups).map(([g, items]) => (
            <Box key={g} sx={{ mb: 1 }}>
              <Typography variant="subtitle2" color="text.secondary">{g}</Typography>
              <Stack direction="row" flexWrap="wrap" gap={1}>
                {items.map((p) => (
                  <FormControlLabel
                    key={p.key}
                    disabled={permSel.includes(ALL_PERMISSION)}
                    control={<Checkbox size="small" checked={hasPermission(permSel, p.key)} onChange={() => togglePerm(p.key)} />}
                    label={p.label}
                  />
                ))}
              </Stack>
            </Box>
          ))}
        </DialogContent>
        <DialogActions><Button onClick={() => setPermOpen(false)}>取消</Button><Button variant="contained" onClick={savePerm}>保存</Button></DialogActions>
      </Dialog>

      <Snackbar open={!!msg} autoHideDuration={2500} onClose={() => setMsg('')} message={msg} />
      <Snackbar open={!!err} autoHideDuration={4000} onClose={() => setErr('')}><Alert severity="error" onClose={() => setErr('')}>{err}</Alert></Snackbar>
    </Box>
  );
};

export default AdminRolesPage;
