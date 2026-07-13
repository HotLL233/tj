import React, { useEffect, useState } from 'react';
import {
  Box, Paper, Typography, Button, Table, TableHead, TableRow, TableCell, TableBody, IconButton,
  Dialog, DialogTitle, DialogContent, DialogActions, TextField, Tabs, Tab, Chip, Stack, Alert, Snackbar, MenuItem,
} from '@mui/material';
import CheckIcon from '@mui/icons-material/Check';
import CloseIcon from '@mui/icons-material/Close';
import EditIcon from '@mui/icons-material/Edit';
import DeleteIcon from '@mui/icons-material/Delete';
import { useAuth } from '../context/AuthContext';
import { getApprovalTasks, decideTask, getApprovalRules, createApprovalRule, updateApprovalRule, deleteApprovalRule } from '../api/client';
import type { ApprovalTask, ApprovalRule } from '../types/lims';

type TabVal = 'todo' | 'mine' | 'rules';

const BIZ_LABELS: Record<string, string> = {
  instrument_booking: '仪器预约', purchase_requisition: '采购申请', purchase_order: '采购单', inventory_out: '库存出库',
};

const ApprovalCenterPage: React.FC = () => {
  const { hasPerm } = useAuth();
  const [tab, setTab] = useState<TabVal>('todo');
  const [tasks, setTasks] = useState<ApprovalTask[]>([]);
  const [rules, setRules] = useState<ApprovalRule[]>([]);
  const [err, setErr] = useState('');
  const [msg, setMsg] = useState('');

  const loadTasks = () => getApprovalTasks({ view: tab === 'rules' ? 'all' : tab }).then((r) => { if (r.code === 0) setTasks(r.data); }).catch((e) => setErr(e.message));
  const loadRules = () => getApprovalRules().then((r) => { if (r.code === 0) setRules(r.data); }).catch((e) => setErr(e.message));
  useEffect(() => { if (tab !== 'rules') loadTasks(); else loadRules(); }, [tab]); // eslint-disable-line

  // 决策
  const [dlg, setDlg] = useState<ApprovalTask | null>(null);
  const [note, setNote] = useState('');
  const decide = async (decision: string) => {
    if (!dlg) return;
    try { const r = await decideTask(dlg.id, { decision, note }); if (r.code !== 0) throw new Error(r.message); setDlg(null); setMsg('已处理'); loadTasks(); } catch (e) { setErr(e instanceof Error ? e.message : '处理失败'); }
  };

  // 规则
  const [ruleOpen, setRuleOpen] = useState(false);
  const [ruleEditing, setRuleEditing] = useState<ApprovalRule | null>(null);
  const [ruleForm, setRuleForm] = useState({ biz_type: 'instrument_booking', name: '', applicant_role: '', applicant: '', approver_role: '', approver: '', priority: 100, is_active: 1 });
  const openRule = (r?: ApprovalRule) => { setRuleEditing(r || null); setRuleForm({ biz_type: r?.biz_type || 'instrument_booking', name: r?.name || '', applicant_role: r?.applicant_role || '', applicant: r?.applicant || '', approver_role: r?.approver_role || '', approver: r?.approver || '', priority: r?.priority || 100, is_active: r?.is_active ?? 1 }); setRuleOpen(true); };
  const saveRule = async () => {
    const payload = { ...ruleForm, applicant_role: ruleForm.applicant_role || null, applicant: ruleForm.applicant || null, approver_role: ruleForm.approver_role || null, approver: ruleForm.approver || null };
    try {
      if (ruleEditing) { const r = await updateApprovalRule(ruleEditing.id, payload); if (r.code !== 0) throw new Error(r.message); }
      else { const r = await createApprovalRule(payload); if (r.code !== 0) throw new Error(r.message); }
      setRuleOpen(false); setMsg('已保存'); loadRules();
    } catch (e) { setErr(e instanceof Error ? e.message : '保存失败'); }
  };
  const delRule = async (r: ApprovalRule) => { if (!confirm('删除该审批规则？')) return; try { const res = await deleteApprovalRule(r.id); if (res.code !== 0) throw new Error(res.message); setMsg('已删除'); loadRules(); } catch (e) { setErr(e instanceof Error ? e.message : '删除失败'); } };

  return (
    <Box>
      <Typography variant="h4" fontWeight={700} gutterBottom>审批中心</Typography>
      <Tabs value={tab} onChange={(_, v) => setTab(v)} sx={{ mb: 2 }}>
        <Tab label="待我审批" value="todo" /><Tab label="我申请的" value="mine" /><Tab label="审批规则" value="rules" />
      </Tabs>

      {tab !== 'rules' && (
        <Paper elevation={1} sx={{ p: 1 }}>
          <Table size="small">
            <TableHead><TableRow><TableCell>业务</TableCell><TableCell>标题</TableCell><TableCell>申请人</TableCell><TableCell>状态</TableCell><TableCell>创建时间</TableCell><TableCell>操作</TableCell></TableRow></TableHead>
            <TableBody>
              {tasks.map((t) => (
                <TableRow key={t.id}>
                  <TableCell>{BIZ_LABELS[t.biz_type] || t.biz_type}</TableCell><TableCell>{t.title}</TableCell><TableCell>{t.applicant}</TableCell>
                  <TableCell><Chip size="small" label={t.status} color={t.status === '已通过' ? 'success' : t.status === '已拒绝' ? 'error' : 'warning'} /></TableCell>
                  <TableCell>{t.created_at}</TableCell>
                  <TableCell>{t.status === '待审批' && hasPerm('approval:approve') && <Button size="small" variant="outlined" onClick={() => { setNote(''); setDlg(t); }}>审批</Button>}</TableCell>
                </TableRow>
              ))}
              {tasks.length === 0 && <TableRow><TableCell colSpan={6} align="center" sx={{ py: 3, color: '#999' }}>暂无任务</TableCell></TableRow>}
            </TableBody>
          </Table>
        </Paper>
      )}

      {tab === 'rules' && (
        <Paper elevation={1} sx={{ p: 1 }}>
          <Stack direction="row" justifyContent="flex-end" sx={{ mb: 1 }}>
            {hasPerm('approval_rule:manage') && <Button variant="contained" onClick={() => openRule()}>新增规则</Button>}
          </Stack>
          <Table size="small">
            <TableHead><TableRow><TableCell>业务</TableCell><TableCell>名称</TableCell><TableCell>申请人条件</TableCell><TableCell>审批人</TableCell><TableCell>优先级</TableCell><TableCell>操作</TableCell></TableRow></TableHead>
            <TableBody>
              {rules.map((r) => (
                <TableRow key={r.id}>
                  <TableCell>{BIZ_LABELS[r.biz_type] || r.biz_type}</TableCell><TableCell>{r.name || '—'}</TableCell>
                  <TableCell>{[r.applicant_role ? `角色:${r.applicant_role}` : '', r.applicant ? `用户:${r.applicant}` : ''].filter(Boolean).join(' / ') || '全部'}</TableCell>
                  <TableCell>{[r.approver_role ? `角色:${r.approver_role}` : '', r.approver ? `用户:${r.approver}` : ''].filter(Boolean).join(' / ') || '—'}</TableCell>
                  <TableCell>{r.priority}</TableCell>
                  <TableCell>{hasPerm('approval_rule:manage') && <><IconButton size="small" onClick={() => openRule(r)}><EditIcon fontSize="small" /></IconButton><IconButton size="small" color="error" onClick={() => delRule(r)}><DeleteIcon fontSize="small" /></IconButton></>}</TableCell>
                </TableRow>
              ))}
              {rules.length === 0 && <TableRow><TableCell colSpan={6} align="center" sx={{ py: 3, color: '#999' }}>暂无规则（将默认通过）</TableCell></TableRow>}
            </TableBody>
          </Table>
        </Paper>
      )}

      {/* 决策 */}
      <Dialog open={!!dlg} onClose={() => setDlg(null)} maxWidth="xs" fullWidth>
        <DialogTitle>审批：{dlg?.title}</DialogTitle>
        <DialogContent>
          <TextField label="审批意见" fullWidth margin="normal" multiline minRows={2} value={note} onChange={(e) => setNote(e.target.value)} />
        </DialogContent>
        <DialogActions>
          <Button color="error" variant="contained" startIcon={<CloseIcon />} onClick={() => decide('reject')}>拒绝</Button>
          <Button color="success" variant="contained" startIcon={<CheckIcon />} onClick={() => decide('approve')}>通过</Button>
        </DialogActions>
      </Dialog>

      {/* 规则 */}
      <Dialog open={ruleOpen} onClose={() => setRuleOpen(false)} maxWidth="sm" fullWidth>
        <DialogTitle>{ruleEditing ? '编辑规则' : '新增规则'}</DialogTitle>
        <DialogContent>
          <TextField select label="业务类型" fullWidth margin="normal" value={ruleForm.biz_type} onChange={(e) => setRuleForm({ ...ruleForm, biz_type: e.target.value })}>
            {Object.entries(BIZ_LABELS).map(([k, v]) => <MenuItem key={k} value={k}>{v}</MenuItem>)}
          </TextField>
          <TextField label="规则名称" fullWidth margin="normal" value={ruleForm.name} onChange={(e) => setRuleForm({ ...ruleForm, name: e.target.value })} />
          <TextField label="申请人角色(可选)" fullWidth margin="normal" value={ruleForm.applicant_role} onChange={(e) => setRuleForm({ ...ruleForm, applicant_role: e.target.value })} />
          <TextField label="申请人(可选)" fullWidth margin="normal" value={ruleForm.applicant} onChange={(e) => setRuleForm({ ...ruleForm, applicant: e.target.value })} />
          <TextField label="审批角色(可选)" fullWidth margin="normal" value={ruleForm.approver_role} onChange={(e) => setRuleForm({ ...ruleForm, approver_role: e.target.value })} />
          <TextField label="审批人(可选)" fullWidth margin="normal" value={ruleForm.approver} onChange={(e) => setRuleForm({ ...ruleForm, approver: e.target.value })} />
          <TextField label="优先级(越小越优先)" type="number" fullWidth margin="normal" value={ruleForm.priority} onChange={(e) => setRuleForm({ ...ruleForm, priority: Number(e.target.value) })} />
        </DialogContent>
        <DialogActions><Button onClick={() => setRuleOpen(false)}>取消</Button><Button variant="contained" onClick={saveRule}>保存</Button></DialogActions>
      </Dialog>

      <Snackbar open={!!msg} autoHideDuration={2500} onClose={() => setMsg('')} message={msg} />
      <Snackbar open={!!err} autoHideDuration={4000} onClose={() => setErr('')}><Alert severity="error" onClose={() => setErr('')}>{err}</Alert></Snackbar>
    </Box>
  );
};

export default ApprovalCenterPage;
