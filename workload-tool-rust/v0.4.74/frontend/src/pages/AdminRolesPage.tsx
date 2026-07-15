import React, { useEffect, useState } from 'react';
import {
  Box, Paper, Typography, Button, Table, TableHead, TableRow, TableCell, TableBody, IconButton,
  Dialog, DialogTitle, DialogContent, DialogActions, TextField, Stack, Alert, Snackbar, Chip,
  Checkbox, FormControlLabel, Divider, Tooltip,
} from '@mui/material';
import EditIcon from '@mui/icons-material/Edit';
import DeleteIcon from '@mui/icons-material/Delete';
import VerifiedUserIcon from '@mui/icons-material/VerifiedUser';
import { useUser } from '../UserContext';
import { getRoles, createRole, updateRole, deleteRole, setRolePermissions, getPermissionWhitelist } from '../api/client';
import type { RoleWithPermissions } from '../types';
import { PERMISSIONS, PERMISSION_GROUPS, ALL_PERMISSION, hasPermission } from '../constants/permissions';

const AdminRolesPage: React.FC = () => {
  const { user } = useUser();
  const [roles, setRoles] = useState<RoleWithPermissions[]>([]);
  const [err, setErr] = useState('');
  const [msg, setMsg] = useState('');

  // v0.4.33: 角色模板
  const ROLE_TEMPLATES = [
    { label: '请选择', name: '', description: '', permissions: [] as string[] },
    { label: '系统管理员', name: '系统管理员', description: '拥有全部权限', permissions: ['*'] },
    { label: '分析检测员', name: '分析检测员', description: '分析检测相关管理权限', permissions: ['entry:workload', 'manage:projects', 'manage:groups', 'manage:methods', 'manage:trash', 'manage:audit', 'manage:help'] },
    { label: '研发送样员', name: '研发送样员', description: '研发送样相关管理权限', permissions: ['entry:sample', 'manage:sampleinfo', 'manage:help'] },
    { label: '样品登记员', name: '样品登记员', description: '样品信息登记相关管理权限', permissions: ['entry:sample-info', 'manage:sampleinfo', 'manage:help'] },
    { label: '查看者', name: '查看者', description: '仅查看门户入口', permissions: ['entry:sample', 'entry:workload', 'entry:sample-info', 'manage:help'] },
    { label: '分析员', name: '分析员', description: '分析检测+研发送样取样', permissions: ['entry:workload', 'entry:sample', 'sample:collect'] },
    { label: '实验员', name: '实验员', description: '仅研发送样+样品信息登记', permissions: ['entry:sample', 'entry:sample-info'] },
  ];

  const load = () => {
    getRoles().then((r) => { if (r.code === 0 && r.data) setRoles(r.data); else setErr(r.message); }).catch((e) => setErr(e.message));
  };
  useEffect(load, []); // eslint-disable-line

  // 角色基本信息编辑
  const [open, setOpen] = useState(false);
  const [editing, setEditing] = useState<RoleWithPermissions | null>(null);
  const [form, setForm] = useState({ name: '', description: '', sort_order: 0 });

  // 权限矩阵编辑
  const [permOpen, setPermOpen] = useState(false);
  const [permRole, setPermRole] = useState<RoleWithPermissions | null>(null);
  const [permSel, setPermSel] = useState<string[]>([]);

  const openNew = () => { setEditing(null); setForm({ name: '', description: '', sort_order: roles.length }); setOpen(true); };
  const openEdit = (r: RoleWithPermissions) => { setEditing(r); setForm({ name: r.name, description: r.description, sort_order: r.sort_order }); setOpen(true); };
  const save = async () => {
    try {
      if (editing) {
        const r = await updateRole(editing.id, { name: form.name, description: form.description, sort_order: form.sort_order });
        if (r.code !== 0) throw new Error(r.message);
      } else {
        const r = await createRole({ name: form.name, description: form.description, sort_order: form.sort_order });
        if (r.code !== 0) throw new Error(r.message);
      }
      setOpen(false); setMsg('已保存'); load();
    } catch (e) { setErr(e instanceof Error ? e.message : '保存失败'); }
  };
  const onDelete = async (r: RoleWithPermissions) => {
    if (r.is_system) { setErr('系统内置角色不可删除'); return; }
    if (!window.confirm(`删除角色「${r.name}」？`)) return;
    try { const res = await deleteRole(r.id); if (res.code !== 0) throw new Error(res.message); setMsg('已删除'); load(); }
    catch (e) { setErr(e instanceof Error ? e.message : '删除失败'); }
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
  const grouped = PERMISSION_GROUPS.reduce<Record<string, typeof PERMISSIONS>>((acc, g) => {
    acc[g] = PERMISSIONS.filter((p) => p.group === g);
    return acc;
  }, {});

  const isAll = (r: RoleWithPermissions) => r.permissions.includes(ALL_PERMISSION);

  // 非管理员无权限访问
  if (!user?.is_admin) return <Alert severity="error">无权限访问</Alert>;

  return (
    <Box>
      <Stack direction="row" justifyContent="space-between" alignItems="center" sx={{ mb: 2 }}>
        <Typography variant="h4" fontWeight={700}>角色管理</Typography>
        <Button variant="contained" onClick={openNew}>新增角色</Button>
      </Stack>
      <Paper elevation={1} sx={{ p: 1 }}>
        <Table size="small">
          <TableHead><TableRow>
            <TableCell>角色名</TableCell><TableCell>说明</TableCell><TableCell>类型</TableCell>
            <TableCell>权限数</TableCell><TableCell>操作</TableCell>
          </TableRow></TableHead>
          <TableBody>
            {roles.map((r) => (
              <TableRow key={r.id}>
                <TableCell>{r.name}</TableCell>
                <TableCell>{r.description}</TableCell>
                <TableCell>{r.is_system ? <Chip size="small" label="内置" color="info" /> : <Chip size="small" label="自定义" />}</TableCell>
                <TableCell>{isAll(r) ? <Chip size="small" label="全部" color="primary" /> : r.permissions.length}</TableCell>
                <TableCell>
                  <Tooltip title="权限设置"><IconButton size="small" onClick={() => openPerm(r)}><VerifiedUserIcon fontSize="small" /></IconButton></Tooltip>
                  <Tooltip title="编辑"><IconButton size="small" onClick={() => openEdit(r)}><EditIcon fontSize="small" /></IconButton></Tooltip>
                  <Tooltip title="删除"><IconButton size="small" color="error" disabled={!!r.is_system} onClick={() => onDelete(r)}><DeleteIcon fontSize="small" /></IconButton></Tooltip>
                </TableCell>
              </TableRow>
            ))}
            {roles.length === 0 && (
              <TableRow><TableCell colSpan={5} align="center" sx={{ color: '#999', py: 3 }}>暂无角色</TableCell></TableRow>
            )}
          </TableBody>
        </Table>
      </Paper>

      <Dialog open={open} onClose={() => setOpen(false)} maxWidth="xs" fullWidth>
        <DialogTitle>{editing ? '编辑角色' : '新增角色'}</DialogTitle>
        <DialogContent>
          {!editing && (
            <TextField label="基于模板" fullWidth margin="normal" select value={''}
              onChange={(e) => {
                const tmpl = ROLE_TEMPLATES.find(t => t.label === e.target.value);
                if (tmpl && tmpl.name) {
                  setForm({ name: tmpl.name, description: tmpl.description, sort_order: roles.length });
                }
              }}
              SelectProps={{ native: true }}>
              {ROLE_TEMPLATES.map(t => (
                <option key={t.label} value={t.label}>{t.label}</option>
              ))}
            </TextField>
          )}
          <TextField label="角色名" fullWidth margin="normal" disabled={!!editing?.is_system} value={form.name}
            onChange={(e) => setForm({ ...form, name: e.target.value })} />
          <TextField label="说明" fullWidth margin="normal" value={form.description}
            onChange={(e) => setForm({ ...form, description: e.target.value })} />
        </DialogContent>
        <DialogActions><Button onClick={() => setOpen(false)}>取消</Button><Button variant="contained" onClick={save}>保存</Button></DialogActions>
      </Dialog>

      <Dialog open={permOpen} onClose={() => setPermOpen(false)} maxWidth="sm" fullWidth>
        <DialogTitle>权限设置：{permRole?.name}</DialogTitle>
        <DialogContent dividers>
          <FormControlLabel
            control={<Checkbox checked={permSel.includes(ALL_PERMISSION)} disabled={!!permRole?.is_system}
              onChange={() => setPermSel((prev) => (prev.includes(ALL_PERMISSION) ? [] : [ALL_PERMISSION]))} />}
            label={<strong>全部权限（* 通配）</strong>}
          />
          <Divider sx={{ my: 1 }} />
          {PERMISSION_GROUPS.map((g) => (
            <Box key={g} sx={{ mb: 1 }}>
              <Typography variant="subtitle2" color="text.secondary">{g}</Typography>
              <Stack direction="row" flexWrap="wrap" gap={1}>
                {grouped[g].map((p) => (
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
