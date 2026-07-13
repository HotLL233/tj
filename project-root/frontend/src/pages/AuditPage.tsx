import React, { useEffect, useState } from 'react';
import { Box, Paper, Typography, Table, TableHead, TableRow, TableCell, TableBody, TextField, MenuItem, Stack, Snackbar, Alert } from '@mui/material';
import { useAuth } from '../context/AuthContext';
import { getAuditLogsLims } from '../api/client';
import type { AuditLogResponse } from '../types/lims';

const MODULES = [
  { value: '', label: '全部' },
  { value: 'work', label: '分析检测' },
  { value: 'rd', label: '研发送样' },
  { value: 'instrument', label: '仪器' },
  { value: 'inventory', label: '库存' },
  { value: 'purchase', label: '采购' },
  { value: 'approval', label: '审批' },
  { value: 'notification', label: '通知' },
];

const AuditPage: React.FC = () => {
  const { hasPerm } = useAuth();
  const [rows, setRows] = useState<AuditLogResponse[]>([]);
  const [module, setModule] = useState('');
  const [page, setPage] = useState(1);
  const [total, setTotal] = useState(0);
  const [err, setErr] = useState('');

  const load = () => {
    getAuditLogsLims({ page, page_size: 50, module: module || undefined }).then((r) => {
      if (r.code === 0 && r.data) { setRows(r.data.items); setTotal(r.data.total); }
    }).catch((e) => setErr(e.message));
  };
  useEffect(load, [page, module]); // eslint-disable-line

  if (!hasPerm('audit:read')) return <Alert severity="error">无权限访问</Alert>;

  return (
    <Box>
      <Stack direction="row" justifyContent="space-between" alignItems="center" sx={{ mb: 2 }}>
        <Typography variant="h4" fontWeight={700}>审计日志</Typography>
        <TextField select label="模块" size="small" value={module} onChange={(e) => { setModule(e.target.value); setPage(1); }} sx={{ width: 160 }}>
          {MODULES.map((m) => <MenuItem key={m.value} value={m.value}>{m.label}</MenuItem>)}
        </TextField>
      </Stack>
      <Paper elevation={1} sx={{ p: 1 }}>
        <Table size="small">
          <TableHead><TableRow><TableCell>时间</TableCell><TableCell>用户</TableCell><TableCell>模块</TableCell><TableCell>操作</TableCell><TableCell>对象</TableCell><TableCell>详情</TableCell></TableRow></TableHead>
          <TableBody>
            {rows.map((r) => (
              <TableRow key={r.id}>
                <TableCell>{r.created_at}</TableCell><TableCell>{r.user_name}</TableCell><TableCell>{r.module}</TableCell>
                <TableCell>{r.action}</TableCell><TableCell>{r.table_name}{r.record_id != null ? ` #${r.record_id}` : ''}</TableCell>
                <TableCell sx={{ maxWidth: 320 }}>{r.detail}</TableCell>
              </TableRow>
            ))}
            {rows.length === 0 && <TableRow><TableCell colSpan={6} align="center" sx={{ py: 3, color: '#999' }}>暂无记录</TableCell></TableRow>}
          </TableBody>
        </Table>
        <Stack direction="row" justifyContent="space-between" alignItems="center" sx={{ mt: 1, px: 1 }}>
          <Typography variant="body2" color="text.secondary">共 {total} 条</Typography>
          <Stack direction="row" spacing={1}>
            <MenuItem disabled={page <= 1} onClick={() => setPage((p) => Math.max(1, p - 1))}>上一页</MenuItem>
            <MenuItem disabled={page * 50 >= total} onClick={() => setPage((p) => p + 1)}>下一页</MenuItem>
          </Stack>
        </Stack>
      </Paper>
      <Snackbar open={!!err} autoHideDuration={4000} onClose={() => setErr('')}><Alert severity="error" onClose={() => setErr('')}>{err}</Alert></Snackbar>
    </Box>
  );
};

export default AuditPage;
