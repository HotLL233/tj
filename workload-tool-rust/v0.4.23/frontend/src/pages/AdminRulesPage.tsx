import React, { useEffect, useState } from 'react';
import {
  Box, Paper, Typography, Button, Table, TableHead, TableRow, TableCell, TableBody, IconButton,
  Dialog, DialogTitle, DialogContent, DialogActions, TextField, Stack, Alert, Snackbar, Chip, MenuItem,
} from '@mui/material';
import EditIcon from '@mui/icons-material/Edit';
import DeleteIcon from '@mui/icons-material/Delete';
import { useAuth } from '../context/AuthContext';
import { getApprovalRules, createApprovalRule, updateApprovalRule, deleteApprovalRule } from '../api/client';
import type { ApprovalRule } from '../types/lims';

// 业务类型中文映射（与后端 approval_service.apply_effect 调度保持一致）
const BIZ_TYPES: { key: string; label: string }[] = [
  { key: 'instrument_booking', label: '仪器预约' },
  { key: 'purchase_requisition', label: '采购申请' },
  { key: 'purchase_order', label: '采购订单' },
  { key: 'inventory_out', label: '库存出库' },
];
const bizLabel = (k: string) => BIZ_TYPES.find((b) => b.key === k)?.label ?? k;

const emptyForm = {
  biz_type: 'instrument_booking',
  name: '',
  applicant_role: '',
  applicant: '',
  object_type: '',
  object_value: '',
  approver_role: '',
  approver: '',
  priority: 100,
};

const AdminRulesPage: React.FC = () => {
  const { hasPerm } = useAuth();
  const [rules, setRules] = useState<ApprovalRule[]>([]);
  const [err, setErr] = useState('');
  const [msg, setMsg] = useState('');

  const load = () => { getApprovalRules().then((r) => { if (r.code === 0) setRules(r.data); else setErr(r.message); }).catch((e) => setErr(e.message)); };
  useEffect(load, []); // eslint-disable-line
  if (!hasPerm('approval_rule:manage')) return <Alert severity="error">无权限访问</Alert>;

  const [open, setOpen] = useState(false);
  const [editing, setEditing] = useState<ApprovalRule | null>(null);
  const [form, setForm] = useState({ ...emptyForm });

  const openNew = () => { setEditing(null); setForm({ ...emptyForm }); setOpen(true); };
  const openEdit = (r: ApprovalRule) => { setEditing(r); setForm({ biz_type: r.biz_type, name: r.name, applicant_role: r.applicant_role ?? '', applicant: r.applicant ?? '', object_type: r.object_type ?? '', object_value: r.object_value ?? '', approver_role: r.approver_role ?? '', approver: r.approver ?? '', priority: r.priority }); setOpen(true); };
  const save = async () => {
    try {
      const payload = {
        biz_type: form.biz_type,
        name: form.name,
        applicant_role: form.applicant_role || null,
        applicant: form.applicant || null,
        object_type: form.object_type || null,
        object_value: form.object_value || null,
        approver_role: form.approver_role || null,
        approver: form.approver || null,
        priority: Number(form.priority) || 100,
      };
      if (editing) { const r = await updateApprovalRule(editing.id, { ...payload, is_active: editing.is_active }); if (r.code !== 0) throw new Error(r.message); }
      else { const r = await createApprovalRule(payload); if (r.code !== 0) throw new Error(r.message); }
      setOpen(false); setMsg('已保存'); load();
    } catch (e) { setErr(e instanceof Error ? e.message : '保存失败'); }
  };
  const onDelete = async (r: ApprovalRule) => { if (!confirm(`删除审批规则「${r.name || bizLabel(r.biz_type)}」？`)) return; try { const res = await deleteApprovalRule(r.id); if (res.code !== 0) throw new Error(res.message); setMsg('已删除'); load(); } catch (e) { setErr(e instanceof Error ? e.message : '删除失败'); } };

  return (
    <Box>
      <Stack direction="row" justifyContent="space-between" alignItems="center" sx={{ mb: 2 }}>
        <Typography variant="h4" fontWeight={700}>审批规则</Typography>
        <Button variant="contained" onClick={openNew}>新增规则</Button>
      </Stack>
      <Paper elevation={1} sx={{ p: 1 }}>
        <Table size="small">
          <TableHead><TableRow><TableCell>名称</TableCell><TableCell>业务类型</TableCell><TableCell>适用角色/用户</TableCell><TableCell>审批人</TableCell><TableCell>优先级</TableCell><TableCell>状态</TableCell><TableCell>操作</TableCell></TableRow></TableHead>
          <TableBody>
            {rules.map((r) => (
              <TableRow key={r.id}>
                <TableCell>{r.name || '-'}</TableCell>
                <TableCell>{bizLabel(r.biz_type)}</TableCell>
                <TableCell>{[r.applicant_role && `角色:${r.applicant_role}`, r.applicant && `用户:${r.applicant}`].filter(Boolean).join(' / ') || '全部'}</TableCell>
                <TableCell>{[r.approver_role && `角色:${r.approver_role}`, r.approver && `用户:${r.approver}`].filter(Boolean).join(' / ') || '默认通过'}</TableCell>
                <TableCell>{r.priority}</TableCell>
                <TableCell>{r.is_active ? <Chip size="small" label="启用" color="success" /> : <Chip size="small" label="停用" />}</TableCell>
                <TableCell>
                  <IconButton size="small" onClick={() => openEdit(r)}><EditIcon fontSize="small" /></IconButton>
                  <IconButton size="small" color="error" onClick={() => onDelete(r)}><DeleteIcon fontSize="small" /></IconButton>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </Paper>

      <Dialog open={open} onClose={() => setOpen(false)} maxWidth="sm" fullWidth>
        <DialogTitle>{editing ? '编辑规则' : '新增规则'}</DialogTitle>
        <DialogContent>
          <TextField select label="业务类型" fullWidth margin="normal" value={form.biz_type} onChange={(e) => setForm({ ...form, biz_type: e.target.value })}>
            {BIZ_TYPES.map((b) => <MenuItem key={b.key} value={b.key}>{b.label}</MenuItem>)}
          </TextField>
          <TextField label="规则名称" fullWidth margin="normal" value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} />
          <TextField label="适用角色(可选)" fullWidth margin="normal" value={form.applicant_role} onChange={(e) => setForm({ ...form, applicant_role: e.target.value })} placeholder="如 实验员" />
          <TextField label="适用用户(可选)" fullWidth margin="normal" value={form.applicant} onChange={(e) => setForm({ ...form, applicant: e.target.value })} placeholder="指定用户名" />
          <TextField label="对象类型(可选)" fullWidth margin="normal" value={form.object_type} onChange={(e) => setForm({ ...form, object_type: e.target.value })} placeholder="如 instrument_id" />
          <TextField label="对象值(可选)" fullWidth margin="normal" value={form.object_value} onChange={(e) => setForm({ ...form, object_value: e.target.value })} />
          <TextField label="审批角色(可选)" fullWidth margin="normal" value={form.approver_role} onChange={(e) => setForm({ ...form, approver_role: e.target.value })} placeholder="如 主管" />
          <TextField label="审批用户(可选)" fullWidth margin="normal" value={form.approver} onChange={(e) => setForm({ ...form, approver: e.target.value })} placeholder="指定审批人" />
          <TextField label="优先级" type="number" fullWidth margin="normal" value={form.priority} onChange={(e) => setForm({ ...form, priority: Number(e.target.value) })} helperText="数值越小优先级越高" />
        </DialogContent>
        <DialogActions><Button onClick={() => setOpen(false)}>取消</Button><Button variant="contained" onClick={save}>保存</Button></DialogActions>
      </Dialog>

      <Snackbar open={!!msg} autoHideDuration={2500} onClose={() => setMsg('')} message={msg} />
      <Snackbar open={!!err} autoHideDuration={4000} onClose={() => setErr('')}><Alert severity="error" onClose={() => setErr('')}>{err}</Alert></Snackbar>
    </Box>
  );
};

export default AdminRulesPage;
