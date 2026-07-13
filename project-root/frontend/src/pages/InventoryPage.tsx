import React, { useEffect, useState } from 'react';
import {
  Box, Paper, Typography, Button, Table, TableHead, TableRow, TableCell, TableBody, IconButton,
  Dialog, DialogTitle, DialogContent, DialogActions, TextField, Tabs, Tab, Chip, Stack, Alert, Snackbar, MenuItem,
} from '@mui/material';
import EditIcon from '@mui/icons-material/Edit';
import DeleteIcon from '@mui/icons-material/Delete';
import InventoryIcon from '@mui/icons-material/Inventory';
import AddIcon from '@mui/icons-material/Add';
import { useAuth } from '../context/AuthContext';
import { getCategories, createCategory, updateCategory, deleteCategory, getItems, createItem, updateItem, deleteItem, getBatches, createBatch, getTransactions, createOutTransaction } from '../api/client';
import type { InventoryCategory, ItemResponse, InventoryBatch, TransactionResponse } from '../types/lims';

type TabVal = 'items' | 'cats' | 'tx';

const InventoryPage: React.FC = () => {
  const { hasPerm } = useAuth();
  const [tab, setTab] = useState<TabVal>('items');
  const [cats, setCats] = useState<InventoryCategory[]>([]);
  const [items, setItems] = useState<ItemResponse[]>([]);
  const [txs, setTxs] = useState<TransactionResponse[]>([]);
  const [err, setErr] = useState('');
  const [msg, setMsg] = useState('');

  const load = () => {
    getItems().then((r) => { if (r.code === 0) setItems(r.data); }).catch((e) => setErr(e.message));
    getCategories().then((r) => { if (r.code === 0) setCats(r.data); }).catch(() => {});
    getTransactions({ page: 1, page_size: 100 }).then((r) => { if (r.code === 0) setTxs(r.data.items); }).catch(() => {});
  };
  useEffect(load, []); // eslint-disable-line

  // 分类
  const [catOpen, setCatOpen] = useState(false);
  const [catEditing, setCatEditing] = useState<InventoryCategory | null>(null);
  const [catForm, setCatForm] = useState({ name: '', parent_id: '', sort_order: 0 });
  const openCat = (c?: InventoryCategory) => { setCatEditing(c || null); setCatForm({ name: c?.name || '', parent_id: c?.parent_id != null ? String(c.parent_id) : '', sort_order: c?.sort_order || 0 }); setCatOpen(true); };
  const saveCat = async () => {
    try {
      const payload = { name: catForm.name, parent_id: catForm.parent_id ? Number(catForm.parent_id) : null, sort_order: catForm.sort_order };
      if (catEditing) { const r = await updateCategory(catEditing.id, payload); if (r.code !== 0) throw new Error(r.message); }
      else { const r = await createCategory(payload); if (r.code !== 0) throw new Error(r.message); }
      setCatOpen(false); setMsg('已保存'); load();
    } catch (e) { setErr(e instanceof Error ? e.message : '保存失败'); }
  };
  const delCat = async (c: InventoryCategory) => { if (!confirm(`删除分类「${c.name}」？`)) return; try { const r = await deleteCategory(c.id); if (r.code !== 0) throw new Error(r.message); setMsg('已删除'); load(); } catch (e) { setErr(e instanceof Error ? e.message : '删除失败'); } };

  // 物料
  const [itemOpen, setItemOpen] = useState(false);
  const [itemEditing, setItemEditing] = useState<ItemResponse | null>(null);
  const [itemForm, setItemForm] = useState({ name: '', brand: '', unit: '个', category_id: '', tags: '', location: '', spec: '', safety_stock: 0 });
  const openItem = (i?: ItemResponse) => { setItemEditing(i || null); setItemForm({ name: i?.name || '', brand: i?.brand || '', unit: i?.unit || '个', category_id: i?.category_id != null ? String(i.category_id) : '', tags: i?.tags || '', location: i?.location || '', spec: i?.spec || '', safety_stock: i?.safety_stock || 0 }); setItemOpen(true); };
  const saveItem = async () => {
    try {
      const payload = { ...itemForm, category_id: itemForm.category_id ? Number(itemForm.category_id) : null };
      if (itemEditing) { const r = await updateItem(itemEditing.id, payload); if (r.code !== 0) throw new Error(r.message); }
      else { const r = await createItem(payload); if (r.code !== 0) throw new Error(r.message); }
      setItemOpen(false); setMsg('已保存'); load();
    } catch (e) { setErr(e instanceof Error ? e.message : '保存失败'); }
  };
  const delItem = async (i: ItemResponse) => { if (!confirm(`删除物料「${i.name}」？`)) return; try { const r = await deleteItem(i.id); if (r.code !== 0) throw new Error(r.message); setMsg('已删除'); load(); } catch (e) { setErr(e instanceof Error ? e.message : '删除失败'); } };

  // 批次 / 入库
  const [batchOpen, setBatchOpen] = useState(false);
  const [batchItem, setBatchItem] = useState<ItemResponse | null>(null);
  const [batches, setBatches] = useState<InventoryBatch[]>([]);
  const [batchForm, setBatchForm] = useState({ batch_no: '', quantity: 0, unit_price: 0 });
  const openBatch = async (i: ItemResponse) => { setBatchItem(i); const r = await getBatches(i.id).catch(() => null); setBatches(r && r.code === 0 ? r.data : []); setBatchForm({ batch_no: '', quantity: 0, unit_price: 0 }); setBatchOpen(true); };
  const saveBatch = async () => {
    if (!batchItem) return;
    try { const r = await createBatch(batchItem.id, batchForm); if (r.code !== 0) throw new Error(r.message); setMsg('入库成功'); const rr = await getBatches(batchItem.id).catch(() => null); setBatches(rr && rr.code === 0 ? rr.data : []); load(); } catch (e) { setErr(e instanceof Error ? e.message : '入库失败'); }
  };

  // 出库
  const [outOpen, setOutOpen] = useState(false);
  const [outItem, setOutItem] = useState<ItemResponse | null>(null);
  const [outForm, setOutForm] = useState({ tx_type: 'out', quantity: 0, note: '' });
  const openOut = (i: ItemResponse) => { setOutItem(i); setOutForm({ tx_type: 'out', quantity: 0, note: '' }); setOutOpen(true); };
  const saveOut = async () => {
    if (!outItem) return;
    try { const r = await createOutTransaction({ item_id: outItem.id, ...outForm }); if (r.code !== 0) throw new Error(r.message); setOutOpen(false); setMsg('出库申请已提交'); load(); } catch (e) { setErr(e instanceof Error ? e.message : '出库失败'); }
  };

  return (
    <Box>
      <Typography variant="h4" fontWeight={700} gutterBottom>库存管理</Typography>
      <Tabs value={tab} onChange={(_, v) => setTab(v)} sx={{ mb: 2 }}>
        <Tab label="物料" value="items" /><Tab label="分类" value="cats" /><Tab label="流水" value="tx" />
      </Tabs>

      {tab === 'items' && (
        <Paper elevation={1} sx={{ p: 1 }}>
          <Stack direction="row" justifyContent="flex-end" sx={{ mb: 1 }}>
            {hasPerm('inventory:write') && <Button variant="contained" startIcon={<AddIcon />} onClick={() => openItem()}>新增物料</Button>}
          </Stack>
          <Table size="small">
            <TableHead><TableRow><TableCell>名称</TableCell><TableCell>品牌</TableCell><TableCell>分类</TableCell><TableCell>单位</TableCell><TableCell>当前库存</TableCell><TableCell>安全库存</TableCell><TableCell>操作</TableCell></TableRow></TableHead>
            <TableBody>
              {items.map((i) => (
                <TableRow key={i.id}>
                  <TableCell>{i.name}</TableCell><TableCell>{i.brand}</TableCell><TableCell>{i.category_name}</TableCell><TableCell>{i.unit}</TableCell>
                  <TableCell><b>{i.current_quantity}</b> {i.current_quantity <= i.safety_stock && <Chip size="small" label="偏低" color="warning" />}</TableCell>
                  <TableCell>{i.safety_stock}</TableCell>
                  <TableCell>
                    <IconButton size="small" title="批次/入库" onClick={() => openBatch(i)}><InventoryIcon fontSize="small" /></IconButton>
                    {hasPerm('inventory:write') && <IconButton size="small" title="出库" onClick={() => openOut(i)}><AddIcon fontSize="small" /></IconButton>}
                    {hasPerm('inventory:write') && <IconButton size="small" onClick={() => openItem(i)}><EditIcon fontSize="small" /></IconButton>}
                    {hasPerm('inventory:write') && <IconButton size="small" color="error" onClick={() => delItem(i)}><DeleteIcon fontSize="small" /></IconButton>}
                  </TableCell>
                </TableRow>
              ))}
              {items.length === 0 && <TableRow><TableCell colSpan={7} align="center" sx={{ py: 3, color: '#999' }}>暂无物料</TableCell></TableRow>}
            </TableBody>
          </Table>
        </Paper>
      )}

      {tab === 'cats' && (
        <Paper elevation={1} sx={{ p: 1 }}>
          <Stack direction="row" justifyContent="flex-end" sx={{ mb: 1 }}>
            {hasPerm('inventory:write') && <Button variant="contained" onClick={() => openCat()}>新增分类</Button>}
          </Stack>
          <Table size="small">
            <TableHead><TableRow><TableCell>名称</TableCell><TableCell>上级</TableCell><TableCell>排序</TableCell><TableCell>操作</TableCell></TableRow></TableHead>
            <TableBody>
              {cats.map((c) => (
                <TableRow key={c.id}><TableCell>{c.name}</TableCell><TableCell>{cats.find((x) => x.id === c.parent_id)?.name || '—'}</TableCell><TableCell>{c.sort_order}</TableCell>
                  <TableCell>{hasPerm('inventory:write') && <><IconButton size="small" onClick={() => openCat(c)}><EditIcon fontSize="small" /></IconButton><IconButton size="small" color="error" onClick={() => delCat(c)}><DeleteIcon fontSize="small" /></IconButton></>}</TableCell></TableRow>
              ))}
              {cats.length === 0 && <TableRow><TableCell colSpan={4} align="center" sx={{ py: 3, color: '#999' }}>暂无分类</TableCell></TableRow>}
            </TableBody>
          </Table>
        </Paper>
      )}

      {tab === 'tx' && (
        <Paper elevation={1} sx={{ p: 1 }}>
          <Table size="small">
            <TableHead><TableRow><TableCell>物料</TableCell><TableCell>类型</TableCell><TableCell>数量</TableCell><TableCell>申请人</TableCell><TableCell>审批人</TableCell><TableCell>备注</TableCell><TableCell>时间</TableCell></TableRow></TableHead>
            <TableBody>
              {txs.map((t) => (<TableRow key={t.id}><TableCell>{t.item_name}</TableCell><TableCell><Chip size="small" label={t.tx_type} color={t.tx_type === 'in' ? 'success' : 'warning'} /></TableCell><TableCell>{t.quantity}</TableCell><TableCell>{t.applicant}</TableCell><TableCell>{t.approver}</TableCell><TableCell>{t.note}</TableCell><TableCell>{t.created_at}</TableCell></TableRow>))}
              {txs.length === 0 && <TableRow><TableCell colSpan={7} align="center" sx={{ py: 3, color: '#999' }}>暂无流水</TableCell></TableRow>}
            </TableBody>
          </Table>
        </Paper>
      )}

      {/* 分类 */}
      <Dialog open={catOpen} onClose={() => setCatOpen(false)} maxWidth="xs" fullWidth>
        <DialogTitle>{catEditing ? '编辑分类' : '新增分类'}</DialogTitle>
        <DialogContent>
          <TextField label="名称" fullWidth margin="normal" value={catForm.name} onChange={(e) => setCatForm({ ...catForm, name: e.target.value })} />
          <TextField select label="上级分类" fullWidth margin="normal" value={catForm.parent_id} onChange={(e) => setCatForm({ ...catForm, parent_id: e.target.value })}>
            <MenuItem value="">（无）</MenuItem>
            {cats.filter((c) => c.id !== catEditing?.id).map((c) => <MenuItem key={c.id} value={String(c.id)}>{c.name}</MenuItem>)}
          </TextField>
          <TextField label="排序" type="number" fullWidth margin="normal" value={catForm.sort_order} onChange={(e) => setCatForm({ ...catForm, sort_order: Number(e.target.value) })} />
        </DialogContent>
        <DialogActions><Button onClick={() => setCatOpen(false)}>取消</Button><Button variant="contained" onClick={saveCat}>保存</Button></DialogActions>
      </Dialog>

      {/* 物料 */}
      <Dialog open={itemOpen} onClose={() => setItemOpen(false)} maxWidth="sm" fullWidth>
        <DialogTitle>{itemEditing ? '编辑物料' : '新增物料'}</DialogTitle>
        <DialogContent>
          <TextField label="名称" fullWidth margin="normal" value={itemForm.name} onChange={(e) => setItemForm({ ...itemForm, name: e.target.value })} />
          <TextField label="品牌" fullWidth margin="normal" value={itemForm.brand} onChange={(e) => setItemForm({ ...itemForm, brand: e.target.value })} />
          <TextField label="单位" fullWidth margin="normal" value={itemForm.unit} onChange={(e) => setItemForm({ ...itemForm, unit: e.target.value })} />
          <TextField select label="分类" fullWidth margin="normal" value={itemForm.category_id} onChange={(e) => setItemForm({ ...itemForm, category_id: e.target.value })}>
            <MenuItem value="">（无）</MenuItem>
            {cats.map((c) => <MenuItem key={c.id} value={String(c.id)}>{c.name}</MenuItem>)}
          </TextField>
          <TextField label="规格" fullWidth margin="normal" value={itemForm.spec} onChange={(e) => setItemForm({ ...itemForm, spec: e.target.value })} />
          <TextField label="位置" fullWidth margin="normal" value={itemForm.location} onChange={(e) => setItemForm({ ...itemForm, location: e.target.value })} />
          <TextField label="标签" fullWidth margin="normal" value={itemForm.tags} onChange={(e) => setItemForm({ ...itemForm, tags: e.target.value })} />
          <TextField label="安全库存" type="number" fullWidth margin="normal" value={itemForm.safety_stock} onChange={(e) => setItemForm({ ...itemForm, safety_stock: Number(e.target.value) })} />
        </DialogContent>
        <DialogActions><Button onClick={() => setItemOpen(false)}>取消</Button><Button variant="contained" onClick={saveItem}>保存</Button></DialogActions>
      </Dialog>

      {/* 批次 / 入库 */}
      <Dialog open={batchOpen} onClose={() => setBatchOpen(false)} maxWidth="sm" fullWidth>
        <DialogTitle>批次管理：{batchItem?.name}</DialogTitle>
        <DialogContent>
          <Table size="small"><TableHead><TableRow><TableCell>批次号</TableCell><TableCell>数量</TableCell><TableCell>单价</TableCell><TableCell>来源</TableCell></TableRow></TableHead>
            <TableBody>{batches.map((b) => (<TableRow key={b.id}><TableCell>{b.batch_no || '—'}</TableCell><TableCell>{b.quantity}</TableCell><TableCell>{b.unit_price}</TableCell><TableCell>{b.source_type}</TableCell></TableRow>))}</TableBody>
          </Table>
          <Typography variant="subtitle2" sx={{ mt: 2 }}>新增入库批次</Typography>
          <TextField label="批次号" fullWidth margin="normal" value={batchForm.batch_no} onChange={(e) => setBatchForm({ ...batchForm, batch_no: e.target.value })} />
          <TextField label="数量" type="number" fullWidth margin="normal" value={batchForm.quantity} onChange={(e) => setBatchForm({ ...batchForm, quantity: Number(e.target.value) })} />
          <TextField label="单价" type="number" fullWidth margin="normal" value={batchForm.unit_price} onChange={(e) => setBatchForm({ ...batchForm, unit_price: Number(e.target.value) })} />
        </DialogContent>
        <DialogActions><Button onClick={() => setBatchOpen(false)}>关闭</Button><Button variant="contained" onClick={saveBatch}>入库</Button></DialogActions>
      </Dialog>

      {/* 出库 */}
      <Dialog open={outOpen} onClose={() => setOutOpen(false)} maxWidth="xs" fullWidth>
        <DialogTitle>出库申请：{outItem?.name}</DialogTitle>
        <DialogContent>
          <TextField select label="类型" fullWidth margin="normal" value={outForm.tx_type} onChange={(e) => setOutForm({ ...outForm, tx_type: e.target.value })}>
            <MenuItem value="out">出库</MenuItem><MenuItem value="scrap">报废</MenuItem>
          </TextField>
          <TextField label="数量" type="number" fullWidth margin="normal" value={outForm.quantity} onChange={(e) => setOutForm({ ...outForm, quantity: Number(e.target.value) })} />
          <TextField label="备注" fullWidth margin="normal" value={outForm.note} onChange={(e) => setOutForm({ ...outForm, note: e.target.value })} />
        </DialogContent>
        <DialogActions><Button onClick={() => setOutOpen(false)}>取消</Button><Button variant="contained" onClick={saveOut}>提交</Button></DialogActions>
      </Dialog>

      <Snackbar open={!!msg} autoHideDuration={2500} onClose={() => setMsg('')} message={msg} />
      <Snackbar open={!!err} autoHideDuration={4000} onClose={() => setErr('')}><Alert severity="error" onClose={() => setErr('')}>{err}</Alert></Snackbar>
    </Box>
  );
};

export default InventoryPage;
