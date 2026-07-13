import React, { useEffect, useState } from 'react';
import {
  Box, Paper, Typography, Button, Table, TableHead, TableRow, TableCell, TableBody, IconButton,
  Dialog, DialogTitle, DialogContent, DialogActions, TextField, Tabs, Tab, Chip, Stack, Alert, Snackbar,
  MenuItem,
} from '@mui/material';
import EditIcon from '@mui/icons-material/Edit';
import DeleteIcon from '@mui/icons-material/Delete';
import EventIcon from '@mui/icons-material/Event';
import BuildIcon from '@mui/icons-material/Build';
import QrCodeIcon from '@mui/icons-material/QrCode';
import { useAuth } from '../context/AuthContext';
import {
  getInstruments, createInstrument, updateInstrument, deleteInstrument, generateInstrumentQr,
  getBookings, submitBooking, getMaintenances, addMaintenance,
} from '../api/client';
import type { InstrumentResponse, BookingResponse, MaintenanceResponse } from '../types/lims';

type TabVal = 'inst' | 'book' | 'maint';

const InstrumentPage: React.FC = () => {
  const { user, hasPerm } = useAuth();
  const [tab, setTab] = useState<TabVal>('inst');
  const [insts, setInsts] = useState<InstrumentResponse[]>([]);
  const [bookings, setBookings] = useState<BookingResponse[]>([]);
  const [maints, setMaints] = useState<MaintenanceResponse[]>([]);
  const [err, setErr] = useState('');
  const [msg, setMsg] = useState('');

  const load = () => {
    if (hasPerm('instrument:read')) getInstruments().then((r) => { if (r.code === 0) setInsts(r.data); }).catch((e) => setErr(e.message));
    if (hasPerm('instrument:read')) getBookings().then((r) => { if (r.code === 0) setBookings(r.data); }).catch(() => {});
    if (hasPerm('instrument:read')) getMaintenances().then((r) => { if (r.code === 0) setMaints(r.data); }).catch(() => {});
  };
  useEffect(load, []); // eslint-disable-line

  // 仪器增改
  const [editing, setEditing] = useState<InstrumentResponse | null>(null);
  const [open, setOpen] = useState(false);
  const [form, setForm] = useState({ name: '', model: '', location: '', manager: '', status: '正常', notes: '' });

  const openNew = () => { setEditing(null); setForm({ name: '', model: '', location: '', manager: '', status: '正常', notes: '' }); setOpen(true); };
  const openEdit = (i: InstrumentResponse) => { setEditing(i); setForm({ name: i.name, model: i.model, location: i.location, manager: i.manager, status: i.status, notes: i.notes }); setOpen(true); };
  const saveInst = async () => {
    try {
      if (editing) { const r = await updateInstrument(editing.id, form); if (r.code !== 0) throw new Error(r.message); }
      else { const r = await createInstrument(form); if (r.code !== 0) throw new Error(r.message); }
      setOpen(false); setMsg('已保存'); load();
    } catch (e) { setErr(e instanceof Error ? e.message : '保存失败'); }
  };
  const onDelete = async (i: InstrumentResponse) => {
    if (!confirm(`确认删除仪器「${i.name}」？`)) return;
    try { const r = await deleteInstrument(i.id); if (r.code !== 0) throw new Error(r.message); setMsg('已删除'); load(); } catch (e) { setErr(e instanceof Error ? e.message : '删除失败'); }
  };

  // 预约
  const [bkOpen, setBkOpen] = useState(false);
  const [bkInst, setBkInst] = useState<InstrumentResponse | null>(null);
  const [bkForm, setBkForm] = useState({ start_time: '', end_time: '', purpose: '' });
  const openBk = (i: InstrumentResponse) => { setBkInst(i); setBkForm({ start_time: '', end_time: '', purpose: '' }); setBkOpen(true); };
  const submitBk = async () => {
    if (!bkInst) return;
    try {
      const r = await submitBooking({ instrument_id: bkInst.id, applicant: user?.username || '', start_time: bkForm.start_time, end_time: bkForm.end_time, purpose: bkForm.purpose });
      if (r.code !== 0) throw new Error(r.message);
      setBkOpen(false); setMsg('预约已提交，等待审批'); load();
    } catch (e) { setErr(e instanceof Error ? e.message : '提交失败'); }
  };

  // 保养
  const [mtOpen, setMtOpen] = useState(false);
  const [mtInst, setMtInst] = useState<InstrumentResponse | null>(null);
  const [mtForm, setMtForm] = useState({ maintainer: '', maintained_at: '', content: '', cost: 0 });
  const openMt = (i: InstrumentResponse) => { setMtInst(i); setMtForm({ maintainer: '', maintained_at: new Date().toISOString().slice(0, 10), content: '', cost: 0 }); setMtOpen(true); };
  const submitMt = async () => {
    if (!mtInst) return;
    try { const r = await addMaintenance({ instrument_id: mtInst.id, ...mtForm }); if (r.code !== 0) throw new Error(r.message); setMtOpen(false); setMsg('保养已登记'); load(); } catch (e) { setErr(e instanceof Error ? e.message : '登记失败'); }
  };

  // 二维码
  const [qr, setQr] = useState<{ name: string; url: string } | null>(null);
  const onQr = async (i: InstrumentResponse) => {
    try { const r = await generateInstrumentQr(i.id); if (r.code === 0 && r.data) setQr({ name: i.name, url: r.data.qr_data_url }); } catch (e) { setErr(e instanceof Error ? e.message : '生成失败'); }
  };

  return (
    <Box>
      <Stack direction="row" justifyContent="space-between" alignItems="center" sx={{ mb: 2 }}>
        <Typography variant="h4" fontWeight={700}>仪器管理</Typography>
        {hasPerm('instrument:write') && <Button variant="contained" onClick={openNew}>新增仪器</Button>}
      </Stack>
      <Tabs value={tab} onChange={(_, v) => setTab(v)} sx={{ mb: 2 }}>
        <Tab label="仪器列表" value="inst" />
        <Tab label="预约记录" value="book" />
        <Tab label="保养记录" value="maint" />
      </Tabs>

      {tab === 'inst' && (
        <Paper elevation={1} sx={{ p: 1 }}>
          <Table size="small">
            <TableHead><TableRow><TableCell>名称</TableCell><TableCell>型号</TableCell><TableCell>位置</TableCell><TableCell>负责人</TableCell><TableCell>状态</TableCell><TableCell>操作</TableCell></TableRow></TableHead>
            <TableBody>
              {insts.map((i) => (
                <TableRow key={i.id}>
                  <TableCell>{i.name}</TableCell><TableCell>{i.model}</TableCell><TableCell>{i.location}</TableCell><TableCell>{i.manager}</TableCell>
                  <TableCell><Chip size="small" label={i.status} color={i.status === '正常' ? 'success' : 'warning'} /></TableCell>
                  <TableCell>
                    <IconButton size="small" title="预约" onClick={() => openBk(i)}><EventIcon fontSize="small" /></IconButton>
                    <IconButton size="small" title="保养" onClick={() => openMt(i)}><BuildIcon fontSize="small" /></IconButton>
                    <IconButton size="small" title="二维码" onClick={() => onQr(i)}><QrCodeIcon fontSize="small" /></IconButton>
                    {hasPerm('instrument:write') && <IconButton size="small" onClick={() => openEdit(i)}><EditIcon fontSize="small" /></IconButton>}
                    {hasPerm('instrument:write') && <IconButton size="small" color="error" onClick={() => onDelete(i)}><DeleteIcon fontSize="small" /></IconButton>}
                  </TableCell>
                </TableRow>
              ))}
              {insts.length === 0 && <TableRow><TableCell colSpan={6} align="center" sx={{ py: 3, color: '#999' }}>暂无仪器</TableCell></TableRow>}
            </TableBody>
          </Table>
        </Paper>
      )}

      {tab === 'book' && (
        <Paper elevation={1} sx={{ p: 1 }}>
          <Table size="small">
            <TableHead><TableRow><TableCell>仪器</TableCell><TableCell>申请人</TableCell><TableCell>开始</TableCell><TableCell>结束</TableCell><TableCell>目的</TableCell><TableCell>状态</TableCell></TableRow></TableHead>
            <TableBody>
              {bookings.map((b) => (
                <TableRow key={b.id}><TableCell>{b.instrument_name}</TableCell><TableCell>{b.applicant}</TableCell><TableCell>{b.start_time}</TableCell><TableCell>{b.end_time}</TableCell><TableCell>{b.purpose}</TableCell><TableCell><Chip size="small" label={b.status} color={b.status === '已通过' ? 'success' : b.status === '已拒绝' ? 'error' : 'warning'} /></TableCell></TableRow>
              ))}
              {bookings.length === 0 && <TableRow><TableCell colSpan={6} align="center" sx={{ py: 3, color: '#999' }}>暂无预约</TableCell></TableRow>}
            </TableBody>
          </Table>
        </Paper>
      )}

      {tab === 'maint' && (
        <Paper elevation={1} sx={{ p: 1 }}>
          <Table size="small">
            <TableHead><TableRow><TableCell>仪器</TableCell><TableCell>保养人</TableCell><TableCell>日期</TableCell><TableCell>内容</TableCell><TableCell>费用</TableCell></TableRow></TableHead>
            <TableBody>
              {maints.map((m) => (<TableRow key={m.id}><TableCell>{m.instrument_name}</TableCell><TableCell>{m.maintainer}</TableCell><TableCell>{m.maintained_at}</TableCell><TableCell>{m.content}</TableCell><TableCell>{m.cost}</TableCell></TableRow>))}
              {maints.length === 0 && <TableRow><TableCell colSpan={5} align="center" sx={{ py: 3, color: '#999' }}>暂无保养记录</TableCell></TableRow>}
            </TableBody>
          </Table>
        </Paper>
      )}

      {/* 仪器增改 */}
      <Dialog open={open} onClose={() => setOpen(false)} maxWidth="xs" fullWidth>
        <DialogTitle>{editing ? '编辑仪器' : '新增仪器'}</DialogTitle>
        <DialogContent>
          <TextField label="名称" fullWidth margin="normal" value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} />
          <TextField label="型号" fullWidth margin="normal" value={form.model} onChange={(e) => setForm({ ...form, model: e.target.value })} />
          <TextField label="位置" fullWidth margin="normal" value={form.location} onChange={(e) => setForm({ ...form, location: e.target.value })} />
          <TextField label="负责人" fullWidth margin="normal" value={form.manager} onChange={(e) => setForm({ ...form, manager: e.target.value })} />
          <TextField select label="状态" fullWidth margin="normal" value={form.status} onChange={(e) => setForm({ ...form, status: e.target.value })}>
            {['正常', '维修中', '停用'].map((s) => <MenuItem key={s} value={s}>{s}</MenuItem>)}
          </TextField>
          <TextField label="备注" fullWidth margin="normal" multiline minRows={2} value={form.notes} onChange={(e) => setForm({ ...form, notes: e.target.value })} />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setOpen(false)}>取消</Button>
          <Button variant="contained" onClick={saveInst}>保存</Button>
        </DialogActions>
      </Dialog>

      {/* 预约 */}
      <Dialog open={bkOpen} onClose={() => setBkOpen(false)} maxWidth="xs" fullWidth>
        <DialogTitle>预约仪器：{bkInst?.name}</DialogTitle>
        <DialogContent>
          <TextField label="开始时间" type="datetime-local" fullWidth margin="normal" InputLabelProps={{ shrink: true }} value={bkForm.start_time} onChange={(e) => setBkForm({ ...bkForm, start_time: e.target.value })} />
          <TextField label="结束时间" type="datetime-local" fullWidth margin="normal" InputLabelProps={{ shrink: true }} value={bkForm.end_time} onChange={(e) => setBkForm({ ...bkForm, end_time: e.target.value })} />
          <TextField label="用途" fullWidth margin="normal" value={bkForm.purpose} onChange={(e) => setBkForm({ ...bkForm, purpose: e.target.value })} />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setBkOpen(false)}>取消</Button>
          <Button variant="contained" onClick={submitBk}>提交</Button>
        </DialogActions>
      </Dialog>

      {/* 保养 */}
      <Dialog open={mtOpen} onClose={() => setMtOpen(false)} maxWidth="xs" fullWidth>
        <DialogTitle>登记保养：{mtInst?.name}</DialogTitle>
        <DialogContent>
          <TextField label="保养人" fullWidth margin="normal" value={mtForm.maintainer} onChange={(e) => setMtForm({ ...mtForm, maintainer: e.target.value })} />
          <TextField label="保养日期" type="date" fullWidth margin="normal" InputLabelProps={{ shrink: true }} value={mtForm.maintained_at} onChange={(e) => setMtForm({ ...mtForm, maintained_at: e.target.value })} />
          <TextField label="内容" fullWidth margin="normal" value={mtForm.content} onChange={(e) => setMtForm({ ...mtForm, content: e.target.value })} />
          <TextField label="费用" type="number" fullWidth margin="normal" value={mtForm.cost} onChange={(e) => setMtForm({ ...mtForm, cost: Number(e.target.value) })} />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setMtOpen(false)}>取消</Button>
          <Button variant="contained" onClick={submitMt}>登记</Button>
        </DialogActions>
      </Dialog>

      {/* 二维码 */}
      <Dialog open={!!qr} onClose={() => setQr(null)} maxWidth="xs" fullWidth>
        <DialogTitle>仪器二维码：{qr?.name}</DialogTitle>
        <DialogContent sx={{ textAlign: 'center' }}>
          {qr && <img src={qr.url} alt="qr" style={{ width: 220, height: 220 }} />}
        </DialogContent>
      </Dialog>

      <Snackbar open={!!msg} autoHideDuration={2500} onClose={() => setMsg('')} message={msg} />
      <Snackbar open={!!err} autoHideDuration={4000} onClose={() => setErr('')}><Alert severity="error" onClose={() => setErr('')}>{err}</Alert></Snackbar>
    </Box>
  );
};

export default InstrumentPage;
