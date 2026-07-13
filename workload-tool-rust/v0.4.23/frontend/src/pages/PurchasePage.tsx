import React, { useEffect, useState } from 'react';
import {
  Box, Paper, Typography, Button, Table, TableHead, TableRow, TableCell, TableBody, IconButton,
  Dialog, DialogTitle, DialogContent, DialogActions, TextField, Tabs, Tab, Chip, Stack, Alert, Snackbar, MenuItem,
} from '@mui/material';
import EditIcon from '@mui/icons-material/Edit';
import DeleteIcon from '@mui/icons-material/Delete';
import { useAuth } from '../context/AuthContext';
import { getSuppliers, createSupplier, updateSupplier, deleteSupplier, getRequisitions, submitRequisition, getOrders, createOrder, receiveOrder } from '../api/client';
import type { Supplier, PurchaseRequisition, OrderResponse } from '../types/lims';

type TabVal = 'req' | 'order' | 'sup';

const PurchasePage: React.FC = () => {
  const { hasPerm, user } = useAuth();
  const [tab, setTab] = useState<TabVal>('req');
  const [suppliers, setSuppliers] = useState<Supplier[]>([]);
  const [reqs, setReqs] = useState<PurchaseRequisition[]>([]);
  const [orders, setOrders] = useState<OrderResponse[]>([]);
  const [err, setErr] = useState('');
  const [msg, setMsg] = useState('');

  const load = () => {
    getRequisitions().then((r) => { if (r.code === 0) setReqs(r.data); }).catch((e) => setErr(e.message));
    getOrders().then((r) => { if (r.code === 0) setOrders(r.data); }).catch((e) => setErr(e.message));
    getSuppliers().then((r) => { if (r.code === 0) setSuppliers(r.data); }).catch(() => {});
  };
  useEffect(load, []); // eslint-disable-line

  // 供应商
  const [supOpen, setSupOpen] = useState(false);
  const [supEditing, setSupEditing] = useState<Supplier | null>(null);
  const [supForm, setSupForm] = useState({ name: '', contact: '', phone: '', email: '', qualification: '', notes: '' });
  const openSup = (s?: Supplier) => { setSupEditing(s || null); setSupForm({ name: s?.name || '', contact: s?.contact || '', phone: s?.phone || '', email: s?.email || '', qualification: s?.qualification || '', notes: s?.notes || '' }); setSupOpen(true); };
  const saveSup = async () => {
    try {
      if (supEditing) { const r = await updateSupplier(supEditing.id, supForm); if (r.code !== 0) throw new Error(r.message); }
      else { const r = await createSupplier(supForm); if (r.code !== 0) throw new Error(r.message); }
      setSupOpen(false); setMsg('已保存'); load();
    } catch (e) { setErr(e instanceof Error ? e.message : '保存失败'); }
  };
  const delSup = async (s: Supplier) => { if (!confirm(`删除供应商「${s.name}」？`)) return; try { const r = await deleteSupplier(s.id); if (r.code !== 0) throw new Error(r.message); setMsg('已删除'); load(); } catch (e) { setErr(e instanceof Error ? e.message : '删除失败'); } };

  // 采购申请
  const [reqOpen, setReqOpen] = useState(false);
  const [reqForm, setReqForm] = useState({ item_name: '', spec: '', quantity: 0, unit: '', purpose: '', expected_supplier: '' });
  const submitReq = async () => {
    try { const r = await submitRequisition(reqForm); if (r.code !== 0) throw new Error(r.message); setReqOpen(false); setMsg('申请已提交'); load(); } catch (e) { setErr(e instanceof Error ? e.message : '提交失败'); }
  };

  // 采购单
  const [orderOpen, setOrderOpen] = useState(false);
  const [orderForm, setOrderForm] = useState({ supplier_id: '', note: '', items: [{ item_name: '', spec: '', quantity: 0, unit_price: 0 }] });
  const addOrderItem = () => setOrderForm({ ...orderForm, items: [...orderForm.items, { item_name: '', spec: '', quantity: 0, unit_price: 0 }] });
  const updateOrderItem = (idx: number, f: Partial<{ item_name: string; spec: string; quantity: number; unit_price: number }>) => setOrderForm({ ...orderForm, items: orderForm.items.map((it, i) => i === idx ? { ...it, ...f } : it) });
  const removeOrderItem = (idx: number) => setOrderForm({ ...orderForm, items: orderForm.items.filter((_, i) => i !== idx) });
  const submitOrder = async () => {
    try {
      const payload = { supplier_id: orderForm.supplier_id ? Number(orderForm.supplier_id) : null, requisition_ids: [], items: orderForm.items, note: orderForm.note };
      const r = await createOrder(payload); if (r.code !== 0) throw new Error(r.message); setOrderOpen(false); setMsg('采购单已创建'); load();
    } catch (e) { setErr(e instanceof Error ? e.message : '创建失败'); }
  };
  const onReceive = async (o: OrderResponse) => { if (!confirm(`登记采购单「${o.order_no}」到货并入库？`)) return; try { const r = await receiveOrder(o.id); if (r.code !== 0) throw new Error(r.message); setMsg('已到货入库'); load(); } catch (e) { setErr(e instanceof Error ? e.message : '失败'); } };

  return (
    <Box>
      <Typography variant="h4" fontWeight={700} gutterBottom>采购管理</Typography>
      <Tabs value={tab} onChange={(_, v) => setTab(v)} sx={{ mb: 2 }}>
        <Tab label="采购申请" value="req" /><Tab label="采购单" value="order" /><Tab label="供应商" value="sup" />
      </Tabs>

      {tab === 'req' && (
        <Paper elevation={1} sx={{ p: 1 }}>
          <Stack direction="row" justifyContent="flex-end" sx={{ mb: 1 }}>
            {hasPerm('purchase:request') && <Button variant="contained" onClick={() => { setReqForm({ item_name: '', spec: '', quantity: 0, unit: '', purpose: '', expected_supplier: '' }); setReqOpen(true); }}>提交申请</Button>}
          </Stack>
          <Table size="small">
            <TableHead><TableRow><TableCell>物料</TableCell><TableCell>规格</TableCell><TableCell>数量</TableCell><TableCell>单位</TableCell><TableCell>申请人</TableCell><TableCell>状态</TableCell></TableRow></TableHead>
            <TableBody>
              {reqs.map((r) => (<TableRow key={r.id}><TableCell>{r.item_name}</TableCell><TableCell>{r.spec}</TableCell><TableCell>{r.quantity}</TableCell><TableCell>{r.unit}</TableCell><TableCell>{r.requester}</TableCell><TableCell><Chip size="small" label={r.status} color={r.status === '已通过' ? 'success' : r.status === '已拒绝' ? 'error' : 'warning'} /></TableCell></TableRow>))}
              {reqs.length === 0 && <TableRow><TableCell colSpan={6} align="center" sx={{ py: 3, color: '#999' }}>暂无申请</TableCell></TableRow>}
            </TableBody>
          </Table>
        </Paper>
      )}

      {tab === 'order' && (
        <Paper elevation={1} sx={{ p: 1 }}>
          <Stack direction="row" justifyContent="flex-end" sx={{ mb: 1 }}>
            {hasPerm('purchase:write') && <Button variant="contained" onClick={() => { setOrderForm({ supplier_id: '', note: '', items: [{ item_name: '', spec: '', quantity: 0, unit_price: 0 }] }); setOrderOpen(true); }}>创建采购单</Button>}
          </Stack>
          <Table size="small">
            <TableHead><TableRow><TableCell>单号</TableCell><TableCell>供应商</TableCell><TableCell>金额</TableCell><TableCell>状态</TableCell><TableCell>操作</TableCell></TableRow></TableHead>
            <TableBody>
              {orders.map((o) => (
                <TableRow key={o.id}><TableCell>{o.order_no}</TableCell><TableCell>{o.supplier_name}</TableCell><TableCell>{o.total_amount}</TableCell><TableCell><Chip size="small" label={o.status} color={o.status === '已通过' ? 'success' : o.status === '已收货' ? 'info' : 'warning'} /></TableCell>
                  <TableCell>{hasPerm('purchase:approve') && o.status === '待审批' && <Button size="small" variant="outlined" onClick={() => onReceive(o)}>到货登记</Button>}</TableCell></TableRow>
              ))}
              {orders.length === 0 && <TableRow><TableCell colSpan={5} align="center" sx={{ py: 3, color: '#999' }}>暂无采购单</TableCell></TableRow>}
            </TableBody>
          </Table>
        </Paper>
      )}

      {tab === 'sup' && (
        <Paper elevation={1} sx={{ p: 1 }}>
          <Stack direction="row" justifyContent="flex-end" sx={{ mb: 1 }}>
            {hasPerm('supplier:write') && <Button variant="contained" onClick={() => openSup()}>新增供应商</Button>}
          </Stack>
          <Table size="small">
            <TableHead><TableRow><TableCell>名称</TableCell><TableCell>联系人</TableCell><TableCell>电话</TableCell><TableCell>资质</TableCell><TableCell>操作</TableCell></TableRow></TableHead>
            <TableBody>
              {suppliers.map((s) => (
                <TableRow key={s.id}><TableCell>{s.name}</TableCell><TableCell>{s.contact}</TableCell><TableCell>{s.phone}</TableCell><TableCell>{s.qualification}</TableCell>
                  <TableCell>{hasPerm('supplier:write') && <><IconButton size="small" onClick={() => openSup(s)}><EditIcon fontSize="small" /></IconButton><IconButton size="small" color="error" onClick={() => delSup(s)}><DeleteIcon fontSize="small" /></IconButton></>}</TableCell></TableRow>
              ))}
              {suppliers.length === 0 && <TableRow><TableCell colSpan={5} align="center" sx={{ py: 3, color: '#999' }}>暂无供应商</TableCell></TableRow>}
            </TableBody>
          </Table>
        </Paper>
      )}

      {/* 申请 */}
      <Dialog open={reqOpen} onClose={() => setReqOpen(false)} maxWidth="xs" fullWidth>
        <DialogTitle>提交采购申请</DialogTitle>
        <DialogContent>
          <TextField label="物料名称" fullWidth margin="normal" value={reqForm.item_name} onChange={(e) => setReqForm({ ...reqForm, item_name: e.target.value })} />
          <TextField label="规格" fullWidth margin="normal" value={reqForm.spec} onChange={(e) => setReqForm({ ...reqForm, spec: e.target.value })} />
          <TextField label="数量" type="number" fullWidth margin="normal" value={reqForm.quantity} onChange={(e) => setReqForm({ ...reqForm, quantity: Number(e.target.value) })} />
          <TextField label="单位" fullWidth margin="normal" value={reqForm.unit} onChange={(e) => setReqForm({ ...reqForm, unit: e.target.value })} />
          <TextField label="用途" fullWidth margin="normal" value={reqForm.purpose} onChange={(e) => setReqForm({ ...reqForm, purpose: e.target.value })} />
          <TextField label="期望供应商" fullWidth margin="normal" value={reqForm.expected_supplier} onChange={(e) => setReqForm({ ...reqForm, expected_supplier: e.target.value })} />
        </DialogContent>
        <DialogActions><Button onClick={() => setReqOpen(false)}>取消</Button><Button variant="contained" onClick={submitReq}>提交</Button></DialogActions>
      </Dialog>

      {/* 采购单 */}
      <Dialog open={orderOpen} onClose={() => setOrderOpen(false)} maxWidth="md" fullWidth>
        <DialogTitle>创建采购单</DialogTitle>
        <DialogContent>
          <TextField select label="供应商" fullWidth margin="normal" value={orderForm.supplier_id} onChange={(e) => setOrderForm({ ...orderForm, supplier_id: e.target.value })}>
            <MenuItem value="">（无）</MenuItem>
            {suppliers.map((s) => <MenuItem key={s.id} value={String(s.id)}>{s.name}</MenuItem>)}
          </TextField>
          {orderForm.items.map((it, idx) => (
            <Stack direction="row" spacing={1} key={idx} sx={{ mt: 1 }} alignItems="center">
              <TextField label="物料" value={it.item_name} onChange={(e) => updateOrderItem(idx, { item_name: e.target.value })} />
              <TextField label="规格" value={it.spec} onChange={(e) => updateOrderItem(idx, { spec: e.target.value })} />
              <TextField label="数量" type="number" value={it.quantity} onChange={(e) => updateOrderItem(idx, { quantity: Number(e.target.value) })} sx={{ width: 90 }} />
              <TextField label="单价" type="number" value={it.unit_price} onChange={(e) => updateOrderItem(idx, { unit_price: Number(e.target.value) })} sx={{ width: 90 }} />
              {orderForm.items.length > 1 && <IconButton color="error" onClick={() => removeOrderItem(idx)}><DeleteIcon /></IconButton>}
            </Stack>
          ))}
          <Button sx={{ mt: 1 }} onClick={addOrderItem}>＋ 添加明细</Button>
          <TextField label="备注" fullWidth margin="normal" value={orderForm.note} onChange={(e) => setOrderForm({ ...orderForm, note: e.target.value })} />
        </DialogContent>
        <DialogActions><Button onClick={() => setOrderOpen(false)}>取消</Button><Button variant="contained" onClick={submitOrder}>创建</Button></DialogActions>
      </Dialog>

      {/* 供应商 */}
      <Dialog open={supOpen} onClose={() => setSupOpen(false)} maxWidth="xs" fullWidth>
        <DialogTitle>{supEditing ? '编辑供应商' : '新增供应商'}</DialogTitle>
        <DialogContent>
          <TextField label="名称" fullWidth margin="normal" value={supForm.name} onChange={(e) => setSupForm({ ...supForm, name: e.target.value })} />
          <TextField label="联系人" fullWidth margin="normal" value={supForm.contact} onChange={(e) => setSupForm({ ...supForm, contact: e.target.value })} />
          <TextField label="电话" fullWidth margin="normal" value={supForm.phone} onChange={(e) => setSupForm({ ...supForm, phone: e.target.value })} />
          <TextField label="邮箱" fullWidth margin="normal" value={supForm.email} onChange={(e) => setSupForm({ ...supForm, email: e.target.value })} />
          <TextField label="资质" fullWidth margin="normal" value={supForm.qualification} onChange={(e) => setSupForm({ ...supForm, qualification: e.target.value })} />
          <TextField label="备注" fullWidth margin="normal" value={supForm.notes} onChange={(e) => setSupForm({ ...supForm, notes: e.target.value })} />
        </DialogContent>
        <DialogActions><Button onClick={() => setSupOpen(false)}>取消</Button><Button variant="contained" onClick={saveSup}>保存</Button></DialogActions>
      </Dialog>

      <Snackbar open={!!msg} autoHideDuration={2500} onClose={() => setMsg('')} message={msg} />
      <Snackbar open={!!err} autoHideDuration={4000} onClose={() => setErr('')}><Alert severity="error" onClose={() => setErr('')}>{err}</Alert></Snackbar>
    </Box>
  );
};

export default PurchasePage;
