import React, { useEffect, useState, useCallback } from 'react';
import {
  Box, Typography, IconButton, TextField, CircularProgress, Snackbar, Alert, Chip,
  Table, TableBody, TableCell, TableContainer, TableHead, TableRow, Paper, TablePagination,
} from '@mui/material';
import ArrowBackIcon from '@mui/icons-material/ArrowBack';
import { useNavigate } from 'react-router-dom';
import type { WorkRecord } from '../types';
import { getRdRecords, sampleRdRecord, getGroups } from '../api/client';

const R = '2px';

// 从方法名中提取仪器标签（@符号后的[...]内容）
const extractInstrumentFromMethodName = (methodName: string): string | null => {
  if (!methodName) return null;
  const atIndex = methodName.indexOf('@');
  if (atIndex === -1) return null;
  const afterAt = methodName.substring(atIndex + 1);
  const bracketStart = afterAt.indexOf('[');
  if (bracketStart === -1) return null;
  const bracketEnd = afterAt.indexOf(']', bracketStart);
  if (bracketEnd === -1) return null;
  return afterAt.substring(bracketStart + 1, bracketEnd);
};

const RdRecordsPage: React.FC = () => {
  const navigate = useNavigate();
  const [records, setRecords] = useState<WorkRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(0);
  const [groups, setGroups] = useState<{ id: number; name: string }[]>([]);
  const [snackMsg, setSnackMsg] = useState('');
  const [snackErr, setSnackErr] = useState(false);
  const pageSize = 20;

  const loadRecords = useCallback(async (p: number) => {
    setLoading(true);
    try {
      const r = await getRdRecords({ page: p + 1, page_size: pageSize });
      if (r.code === 0 && r.data) {
        setRecords(r.data.items);
        setTotal(r.data.total);
      }
    } catch {} finally { setLoading(false); }
  }, []);

  const loadGroups = useCallback(async () => {
    try { const r = await getGroups(); if (r.code === 0 && r.data) setGroups(r.data); } catch {}
  }, []);

  useEffect(() => { loadRecords(0); loadGroups(); }, []);

  const handlePageChange = (_e: unknown, newPage: number) => {
    setPage(newPage);
    setLoading(true);
    getRdRecords({ page: newPage + 1, page_size: pageSize })
      .then(r => { if (r.code === 0 && r.data) { setRecords(r.data.items); setTotal(r.data.total); } })
      .catch(() => {})
      .finally(() => setLoading(false));
  };

  const getGroupName = (rec: WorkRecord) => rec.group_name || groups.find(g => g.name === rec.group_name)?.name || '-';

  return (<Box>
    <Box sx={{ display: 'flex', alignItems: 'center', gap: 1.5, mb: 2 }}>
      <IconButton onClick={() => navigate(-1)} sx={{ bgcolor: 'rgba(230,81,0,0.08)', '&:hover': { bgcolor: 'rgba(230,81,0,0.15)' } }}>
        <ArrowBackIcon />
      </IconButton>
      <Typography variant="h5" fontWeight={700}>研发送样记录</Typography>
    </Box>

    {loading && records.length === 0 ? (
      <Box sx={{ display: 'flex', justifyContent: 'center', py: 8 }}><CircularProgress /></Box>
    ) : records.length === 0 ? (
      <Typography color="text.secondary" textAlign="center" sx={{ py: 6, fontSize: '0.875rem' }}>暂无记录</Typography>
    ) : (
      <TableContainer component={Paper} variant="outlined" sx={{ borderRadius: R, boxShadow: 'none', '& .MuiPaper-root': { borderRadius: R } }}>
        <Table size="small" sx={{ minWidth: 960 }}>
          <TableHead>
            <TableRow sx={{ bgcolor: 'rgba(230,81,0,0.06)' }}>
              <TableCell sx={{ fontWeight: 700, fontSize: '0.8rem', whiteSpace: 'nowrap', width: 40, textAlign: 'center' }}>序号</TableCell>
              <TableCell sx={{ fontWeight: 700, fontSize: '0.8rem', whiteSpace: 'nowrap' }}>状态</TableCell>
              <TableCell sx={{ fontWeight: 700, fontSize: '0.8rem', whiteSpace: 'nowrap' }}>送样时间</TableCell>
              <TableCell sx={{ fontWeight: 700, fontSize: '0.8rem', whiteSpace: 'nowrap' }}>实验室</TableCell>
              <TableCell sx={{ fontWeight: 700, fontSize: '0.8rem', whiteSpace: 'nowrap' }}>研发项目</TableCell>
              <TableCell sx={{ fontWeight: 700, fontSize: '0.8rem', whiteSpace: 'nowrap' }}>送样人</TableCell>
              <TableCell sx={{ fontWeight: 700, fontSize: '0.8rem', whiteSpace: 'nowrap' }}>方法</TableCell>
              <TableCell sx={{ fontWeight: 700, fontSize: '0.8rem', whiteSpace: 'nowrap' }}>类型</TableCell>
              <TableCell sx={{ fontWeight: 700, fontSize: '0.8rem', whiteSpace: 'nowrap' }}>仪器</TableCell>
              <TableCell sx={{ fontWeight: 700, fontSize: '0.8rem', whiteSpace: 'nowrap' }}>数量</TableCell>
              <TableCell sx={{ fontWeight: 700, fontSize: '0.8rem', whiteSpace: 'nowrap' }}>取样人</TableCell>
              <TableCell sx={{ fontWeight: 700, fontSize: '0.8rem', whiteSpace: 'nowrap' }}>取样时间</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {records.map((rec, idx) => {
              const status = rec.status || '待取样';
              const isSampled = status === '已取样';
              return (
              <TableRow key={rec.id} hover sx={{ '&:last-child td': { borderBottom: 0 } }}>
                <TableCell sx={{ fontSize: '0.8rem', textAlign: 'center' }}>{page * pageSize + idx + 1}</TableCell>
                <TableCell sx={{ fontSize: '0.8rem', whiteSpace: 'nowrap' }}>
                  <Typography variant="body2" sx={{
                    display: 'inline-block', px: 1, py: 0.3, borderRadius: R, fontSize: '0.75rem', fontWeight: 600,
                    bgcolor: isSampled ? '#c8e6c9' : '#fff9c4',
                    color: isSampled ? '#2e7d32' : '#f57f17',
                  }}>{status}</Typography>
                </TableCell>
                <TableCell sx={{ fontSize: '0.8rem', whiteSpace: 'nowrap' }}>
                  {rec.recorded_at ? rec.recorded_at.replace('T', ' ').substring(0, 19) : '-'}
                </TableCell>
                <TableCell sx={{ fontSize: '0.8rem', maxWidth: 100, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{getGroupName(rec)}</TableCell>
                <TableCell sx={{ fontSize: '0.8rem', maxWidth: 120, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{rec.project_name || '-'}</TableCell>
                <TableCell sx={{ fontSize: '0.8rem', whiteSpace: 'nowrap' }}>{rec.user_name || '-'}</TableCell>
                <TableCell sx={{ fontSize: '0.8rem', maxWidth: 140, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{rec.method_name || '-'}</TableCell>
                <TableCell sx={{ fontSize: '0.8rem', whiteSpace: 'nowrap' }}>{rec.method_type || '-'}</TableCell>
                <TableCell sx={{ fontSize: '0.8rem', whiteSpace: 'nowrap' }}>
                  {extractInstrumentFromMethodName(rec.method_name || '') ? (
                    <Chip label={extractInstrumentFromMethodName(rec.method_name || '')} size="small" sx={{ bgcolor: '#00897b', color: '#fff', borderRadius: R, height: 20, fontSize: '0.7rem' }} />
                  ) : '-'}
                </TableCell>
                <TableCell sx={{ fontSize: '0.8rem', fontWeight: 600, whiteSpace: 'nowrap' }}>{rec.quantity}</TableCell>
                <TableCell sx={{ fontSize: '0.8rem', whiteSpace: 'nowrap' }}>
                  {isSampled ? (
                    <Typography variant="body2" sx={{ color: '#2e7d32', fontWeight: 600 }}>{rec.sampler || '-'}</Typography>
                  ) : (
                    <TextField
                      size="small"
                      placeholder="取样人"
                      onBlur={async (e) => {
                        const val = e.target.value.trim();
                        if (val) {
                          try {
                            await sampleRdRecord(rec.id, val);
                            setSnackMsg('取样成功'); setSnackErr(false);
                            loadRecords(page);
                          } catch { setSnackMsg('取样失败'); setSnackErr(true); }
                        }
                      }}
                      onKeyDown={async (e) => {
                        if (e.key === 'Enter') {
                          const val = (e.target as HTMLInputElement).value.trim();
                          if (val) {
                            try {
                              await sampleRdRecord(rec.id, val);
                              setSnackMsg('取样成功'); setSnackErr(false);
                              loadRecords(page);
                            } catch { setSnackMsg('取样失败'); setSnackErr(true); }
                          }
                        }
                      }}
                      sx={{ '& .MuiOutlinedInput-root': { borderRadius: R, fontSize: '0.75rem' }, width: 80 }}
                      inputProps={{ style: { padding: '2px 8px' } }}
                    />
                  )}
                </TableCell>
                <TableCell sx={{ fontSize: '0.8rem', whiteSpace: 'nowrap' }}>
                  {rec.sampled_at ? rec.sampled_at.replace('T', ' ').substring(0, 19) : '-'}
                </TableCell>
              </TableRow>
              );
            })}
          </TableBody>
        </Table>
        {total > pageSize && (
          <TablePagination
            component="div"
            count={total}
            page={page}
            onPageChange={handlePageChange}
            rowsPerPage={pageSize}
            rowsPerPageOptions={[pageSize]}
            labelDisplayedRows={({ from, to, count }) => `${from}-${to} / ${count}`}
            sx={{ '& .MuiTablePagination-toolbar': { minHeight: 40 }, '& .MuiTablePagination-selectLabel': { fontSize: '0.75rem' }, '& .MuiTablePagination-displayedRows': { fontSize: '0.75rem' } }}
          />
        )}
      </TableContainer>
    )}

    <Snackbar open={!!snackMsg} autoHideDuration={3000} onClose={() => setSnackMsg('')} anchorOrigin={{ vertical: 'bottom', horizontal: 'center' }}>
      <Alert severity={snackErr ? 'error' : 'success'} sx={{ borderRadius: R }} onClose={() => setSnackMsg('')}>{snackMsg}</Alert>
    </Snackbar>
  </Box>);
};
export default RdRecordsPage;
