import React, { useState, useEffect, useCallback } from 'react';
import { useSearchParams, useNavigate } from 'react-router-dom';
import {
  Box, Typography, Paper, TextField, Button, Grid, IconButton,
  Chip, Snackbar, Alert, Select, MenuItem, FormControl, InputLabel,
  TablePagination, Collapse, CircularProgress, Checkbox, Table,
  TableBody, TableCell, TableContainer, TableHead, TableRow,
  Switch,
} from '@mui/material';
import ArrowBackIcon from '@mui/icons-material/ArrowBack';
import ExpandMoreIcon from '@mui/icons-material/ExpandMore';
import ExpandLessIcon from '@mui/icons-material/ExpandLess';
import AddIcon from '@mui/icons-material/Add';
import DeleteIcon from '@mui/icons-material/Delete';
import {
  getSampleInfoRecords, createSampleInfo, updateSampleInfo, updateSampleInfoStatus,
  getSampleInfoTypes, getDivisions, getActiveSampleInfoColumns,
} from '../api/client';
import type { SampleInfoRecord, SampleInfoType, Division, SampleInfoColumn } from '../types';

const R = '2px';
const PAGE_SIZE = 20;

const STATUS_OPTIONS = ['全部', '待检测', '待取样', '已取样', '检测完成'] as const;
const STATUS_COLORS: Record<string, 'warning' | 'info' | 'success' | 'default'> = {
  '待检测': 'warning', '待取样': 'info', '已取样': 'success', '检测完成': 'default',
};
const NEXT_STATUS: Record<string, string | null> = {
  '待检测': '待取样', '待取样': '已取样', '已取样': '检测完成', '检测完成': null,
};
const NEXT_STATUS_LABEL: Record<string, string | null> = {
  '待检测': '取样', '待取样': '已取样', '已取样': '检测完成', '检测完成': null,
};

// 预置字段列表 — 在 extra_fields 中排除
const PREDEFINED_FIELDS = new Set([
  'seq_no', 'user_name', 'division_id', 'lab_name', 'project_name',
  'quantity', 'batch_no', 'main_components', 'notes', 'submitted_at',
  'detection_type', 'detection_date', 'type_key', 'status',
]);

type RowData = Record<string, any>;

const emptyRow = (columns: SampleInfoColumn[]): RowData => {
  const row: RowData = { checked: false, _extra: {} };
  for (const col of columns) {
    if (PREDEFINED_FIELDS.has(col.field_key)) {
      if (col.field_key === 'quantity') row[col.field_key] = 1;
      else if (col.field_key === 'division_id') row[col.field_key] = '';
      else row[col.field_key] = '';
    } else {
      // 自定义字段 → 存到 _extra
      row._extra[col.field_key] = '';
    }
  }
  return row;
};

const SampleInfoEntry: React.FC = () => {
  const [sp] = useSearchParams();
  const n = useNavigate();
  const dt = sp.get('type') || '';

  // 列配置
  const [columns, setColumns] = useState<SampleInfoColumn[]>([]);

  // 多行表格
  const [rows, setRows] = useState<RowData[]>([emptyRow([])]);
  const [submittedAt, setSubmittedAt] = useState(new Date().toISOString().slice(0, 16));
  const [snack, setSnack] = useState<{ open: boolean; msg: string; sev: 'success' | 'error' }>({ open: false, msg: '', sev: 'success' });

  // 列表
  const [records, setRecords] = useState<SampleInfoRecord[]>([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(0);
  const [statusFilter, setStatusFilter] = useState('全部');
  const [ld, setLd] = useState(false);
  const [expandedId, setExpandedId] = useState<number | null>(null);

  // 编辑
  const [editingId, setEditingId] = useState<number | null>(null);
  const [editForm, setEditForm] = useState<Record<string, string>>({});

  // 检测类型
  const [types, setTypes] = useState<SampleInfoType[]>([]);
  useEffect(() => {
    getSampleInfoTypes().then(r => { if (r.code === 0 && r.data) setTypes(r.data); }).catch(() => {});
  }, []);

  // 部门列表
  const [divs, setDivs] = useState<Division[]>([]);
  useEffect(() => {
    getDivisions().then(r => { if (r.code === 0 && r.data) setDivs(r.data); }).catch(() => {});
  }, []);

  // 加载列配置
  useEffect(() => {
    getActiveSampleInfoColumns().then(r => {
      if (r.code === 0 && r.data) {
        const cols = r.data;
        setColumns(cols);
        setRows([emptyRow(cols)]);
      }
    }).catch(() => {});
  }, []);

  const load = useCallback(async () => {
    setLd(true);
    try {
      const r = await getSampleInfoRecords({
        type_key: dt || undefined,
        status: statusFilter === '全部' ? undefined : statusFilter,
        page: page + 1,
        page_size: PAGE_SIZE,
      });
      if (r.data) {
        setRecords(r.data.items);
        setTotal(r.data.total);
      }
    } catch (e: any) { setSnack({ open: true, msg: e.message || '加载失败', sev: 'error' }); }
    setLd(false);
  }, [dt, statusFilter, page]);

  useEffect(() => { load(); }, [load]);

  // 表单列（show_in_form=true）
  const formColumns = columns.filter(c => c.show_in_form);
  // 列表列（show_in_list=true）
  const listColumns = columns.filter(c => c.show_in_list);

  const updateRow = (idx: number, key: string, val: any) => {
    setRows(prev => prev.map((r, i) => {
      if (i !== idx) return r;
      if (PREDEFINED_FIELDS.has(key)) return { ...r, [key]: val };
      return { ...r, _extra: { ...r._extra, [key]: val } };
    }));
  };

  const getRowValue = (row: RowData, fieldKey: string) => {
    if (PREDEFINED_FIELDS.has(fieldKey)) return row[fieldKey];
    return row._extra?.[fieldKey] ?? '';
  };

  const addRow = () => setRows(prev => [...prev, emptyRow(columns)]);

  const deleteSelected = () => {
    const remaining = rows.filter(r => !r.checked);
    if (remaining.length === 0) remaining.push(emptyRow(columns));
    setRows(remaining);
  };

  const resetRows = () => setRows([emptyRow(columns)]);

  const doSubmit = async () => {
    const typeLabel = types.find(t => t.type_key === dt)?.label || dt;
    let submitted = 0;
    let errors: string[] = [];

    for (let i = 0; i < rows.length; i++) {
      const row = rows[i];
      if (!row.batch_no?.trim() || !row.main_components?.trim()) {
        errors.push(`第 ${i + 1} 行缺少批号或主要成分`);
        continue;
      }
      try {
        // 分离预置字段和自定义字段
        const extra_fields: Record<string, any> = {};
        const presetData: any = {
          batch_no: row.batch_no,
          user_name: row.user_name || '未知',
          lab_name: row.lab_name || '',
          project_name: row.project_name || '',
          submitted_at: submittedAt,
          main_components: row.main_components,
          detection_type: typeLabel,
          type_key: dt,
          division_id: row.division_id || null,
          quantity: row.quantity || 1,
          notes: row.notes || undefined,
        };
        // 自定义字段
        if (row._extra) {
          for (const [k, v] of Object.entries(row._extra)) {
            if (v !== '' && v !== null && v !== undefined) {
              extra_fields[k] = v;
            }
          }
        }
        if (Object.keys(extra_fields).length > 0) {
          presetData.extra_fields = extra_fields;
        }
        await createSampleInfo(presetData);
        submitted++;
      } catch (e: any) {
        errors.push(`第 ${i + 1} 行: ${e.message || '提交失败'}`);
      }
    }

    if (submitted > 0) {
      setSnack({ open: true, msg: `成功登记 ${submitted} 条` + (errors.length > 0 ? `，${errors.length} 条失败` : ''), sev: 'success' });
      setRows([emptyRow(columns)]);
      setSubmittedAt(new Date().toISOString().slice(0, 16));
      setPage(0); load();
    } else if (errors.length > 0) {
      setSnack({ open: true, msg: errors.join('；'), sev: 'error' });
    } else {
      setSnack({ open: true, msg: '请填写数据', sev: 'error' });
    }
  };

  const doExpand = (id: number) => setExpandedId(p => p === id ? null : id);

  const doEdit = (rec: SampleInfoRecord) => {
    setEditingId(rec.id);
    setEditForm({
      batch_no: rec.batch_no, user_name: rec.user_name, lab_name: rec.lab_name,
      project_name: rec.project_name, submitted_at: rec.submitted_at,
      main_components: rec.main_components, notes: rec.notes,
    });
  };

  const doCancelEdit = () => { setEditingId(null); setEditForm({}); };

  const doSaveEdit = async (id: number) => {
    try {
      await updateSampleInfo(id, {
        batch_no: editForm.batch_no, user_name: editForm.user_name, lab_name: editForm.lab_name,
        project_name: editForm.project_name, submitted_at: editForm.submitted_at,
        main_components: editForm.main_components, notes: editForm.notes,
      });
      setSnack({ open: true, msg: '保存成功', sev: 'success' });
      setEditingId(null); setEditForm({}); load();
    } catch (e: any) { setSnack({ open: true, msg: e.message || '保存失败', sev: 'error' }); }
  };

  const doStatusFlow = async (id: number, curStatus: string) => {
    const nxt = NEXT_STATUS[curStatus];
    if (!nxt) return;
    try {
      await updateSampleInfoStatus(id, nxt);
      setSnack({ open: true, msg: `状态已流转: ${curStatus} → ${nxt}`, sev: 'success' });
      if (expandedId === id) setExpandedId(null);
      load();
    } catch (e: any) { setSnack({ open: true, msg: e.message || '状态流转失败', sev: 'error' }); }
  };

  const fmtDate = (s: string) => s ? s.slice(0, 16).replace('T', ' ') : '';

  /** 根据 data_type 渲染对应的输入控件 */
  const renderCellInput = (col: SampleInfoColumn, idx: number) => {
    const val = getRowValue(rows[idx], col.field_key);
    switch (col.data_type) {
      case 'select':
        return (
          <FormControl fullWidth size="small">
            <Select
              value={val}
              displayEmpty
              onChange={e => updateRow(idx, col.field_key, e.target.value)}
              sx={{ fontSize: '0.8rem' }}
            >
              <MenuItem value=""><em>请选择</em></MenuItem>
              {col.field_key === 'division_id' && divs.map(d => (
                <MenuItem key={d.id} value={d.id}>{d.name}</MenuItem>
              ))}
              {col.options?.split(',').map(opt => (
                <MenuItem key={opt} value={opt}>{opt}</MenuItem>
              ))}
            </Select>
          </FormControl>
        );
      case 'number':
        return (
          <TextField
            size="small" type="number"
            value={val}
            onChange={e => updateRow(idx, col.field_key, Math.max(1, Number(e.target.value) || 1))}
            inputProps={{ min: 1 }}
            sx={{ width: 70, '& .MuiOutlinedInput-root': { fontSize: '0.8rem' } }}
          />
        );
      case 'date':
        return (
          <TextField
            size="small" type="date"
            value={val}
            onChange={e => updateRow(idx, col.field_key, e.target.value)}
            InputLabelProps={{ shrink: true }}
            sx={{ width: 140, '& .MuiOutlinedInput-root': { fontSize: '0.8rem' } }}
          />
        );
      default:
        return (
          <TextField
            size="small" fullWidth
            value={val}
            onChange={e => updateRow(idx, col.field_key, e.target.value)}
            placeholder={col.label}
            required={col.is_required}
            sx={{ '& .MuiOutlinedInput-root': { fontSize: '0.8rem' } }}
          />
        );
    }
  };

  /** 根据 data_type 渲染只读显示值 */
  const renderCellValue = (col: SampleInfoColumn, rec: SampleInfoRecord) => {
    let val: any;
    if (PREDEFINED_FIELDS.has(col.field_key)) {
      val = (rec as any)[col.field_key];
    } else if (rec.extra_fields) {
      val = rec.extra_fields[col.field_key];
    }
    if (val === null || val === undefined || val === '') return '-';
    if (col.data_type === 'date') return String(val).slice(0, 10);
    return String(val);
  };

  return (
    <Box sx={{ maxWidth: 1100, mx: 'auto', mt: { xs: 1, md: 3 }, px: { xs: 1, md: 2 } }}>
      {/* 顶部 */}
      <Box sx={{ display: 'flex', alignItems: 'center', mb: 2, gap: 1 }}>
        <IconButton onClick={() => n('/sample-info')} size="small"><ArrowBackIcon /></IconButton>
        <Typography variant="h5" fontWeight={700} color="#2e7d32">样品信息登记</Typography>
      </Box>

      {/* === 部分 A：登记表单 === */}
      <Paper elevation={0} sx={{ p: 3, mb: 3, borderRadius: R, border: '2px solid #2e7d32', background: 'linear-gradient(145deg,#ffffff,#f1f8e9)' }}>
        <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2, flexWrap: 'wrap', gap: 1 }}>
          <Box>
            <Typography variant="h6" fontWeight={700} color="#2e7d32">样品信息登记</Typography>
            <Typography variant="body2" color="text.secondary">
              检测类型: {dt || '全部'} · 序号: <Box component="span" sx={{ color: '#999' }}>自动生成</Box>
            </Typography>
          </Box>
          <Box sx={{ display: 'flex', gap: 1, alignItems: 'center' }}>
            <Chip label="待检测" color="warning" size="small" />
          </Box>
        </Box>

        {/* 公共时间 */}
        <Box sx={{ mb: 2 }}>
          <TextField
            label="送样时间（整单公共）"
            type="datetime-local"
            required
            size="small"
            value={submittedAt}
            onChange={e => setSubmittedAt(e.target.value)}
            InputLabelProps={{ shrink: true }}
            sx={{ maxWidth: 280 }}
          />
        </Box>

        {/* 动态多行表格 */}
        <TableContainer component={Paper} sx={{ mb: 2, borderRadius: R, border: '1px solid rgba(0,0,0,0.08)' }}>
          <Table size="small" stickyHeader>
            <TableHead>
              <TableRow>
                <TableCell sx={{ fontWeight: 700, p: 1, width: 40 }}>
                  <Checkbox
                    size="small"
                    checked={rows.length > 0 && rows.every(r => r.checked)}
                    indeterminate={rows.some(r => r.checked) && !rows.every(r => r.checked)}
                    onChange={() => {
                      const all = rows.every(r => r.checked);
                      setRows(prev => prev.map(r => ({ ...r, checked: !all })));
                    }}
                  />
                </TableCell>
                <TableCell sx={{ fontWeight: 700, p: 1, width: 50 }}>序号</TableCell>
                {formColumns.map(col => (
                  <TableCell key={col.field_key} sx={{ fontWeight: 700, p: 1, minWidth: col.width || 100 }}>
                    {col.label}{col.is_required ? ' *' : ''}
                  </TableCell>
                ))}
              </TableRow>
            </TableHead>
            <TableBody>
              {rows.map((row, idx) => (
                <TableRow key={idx} hover selected={row.checked}>
                  <TableCell sx={{ p: 0.5 }}>
                    <Checkbox
                      size="small"
                      checked={row.checked}
                      onChange={() => updateRow(idx, 'checked', !row.checked)}
                    />
                  </TableCell>
                  <TableCell sx={{ p: 0.5, fontSize: '0.8rem', color: '#999' }}>{idx + 1}</TableCell>
                  {formColumns.map(col => (
                    <TableCell key={col.field_key} sx={{ p: 0.5 }}>
                      {renderCellInput(col, idx)}
                    </TableCell>
                  ))}
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>

        <Box sx={{ display: 'flex', gap: 1, justifyContent: 'space-between', flexWrap: 'wrap' }}>
          <Box sx={{ display: 'flex', gap: 1 }}>
            <Button variant="outlined" size="small" startIcon={<AddIcon />} onClick={addRow} sx={{ borderRadius: R }}>
              添加行
            </Button>
            <Button variant="outlined" size="small" color="error" startIcon={<DeleteIcon />} onClick={deleteSelected} disabled={!rows.some(r => r.checked)} sx={{ borderRadius: R }}>
              删除选中行
            </Button>
            <Button variant="outlined" size="small" onClick={resetRows} sx={{ borderRadius: R }}>
              重置
            </Button>
          </Box>
          <Button variant="contained" onClick={doSubmit} sx={{ borderRadius: R, bgcolor: '#2e7d32', '&:hover': { bgcolor: '#1b5e20' } }}>
            提交登记（{rows.length} 行）
          </Button>
        </Box>
      </Paper>

      {/* === 部分 B：记录列表 === */}
      <Paper elevation={0} sx={{ p: { xs: 1, md: 2 }, borderRadius: R, border: '1px solid #e0e0e0' }}>
        <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2, flexWrap: 'wrap', gap: 1 }}>
          <Typography variant="h6" fontWeight={700}>登记记录</Typography>
          <FormControl size="small" sx={{ minWidth: 120 }}>
            <InputLabel>状态</InputLabel>
            <Select value={statusFilter} label="状态" onChange={e => { setStatusFilter(e.target.value); setPage(0); }}>
              {STATUS_OPTIONS.map(s => <MenuItem key={s} value={s}>{s}</MenuItem>)}
            </Select>
          </FormControl>
        </Box>

        {ld ? <Box sx={{ textAlign: 'center', py: 4 }}><CircularProgress size={32} /></Box> : (
          <>
            {/* 动态列表头 */}
            <Box sx={{ display: 'flex', px: 2, py: 1, bgcolor: '#f5f5f5', borderRadius: R, fontWeight: 600, fontSize: '0.875rem', color: '#666', flexWrap: 'wrap' }}>
              <Box sx={{ width: 70 }}>状态</Box>
              <Box sx={{ width: 50 }}>序号</Box>
              {/* 当前固定显示几个关键列 + 动态列 */}
              {listColumns.filter(c => !['status', 'seq_no'].includes(c.field_key)).map(col => (
                <Box key={col.field_key} sx={{ flex: 1, minWidth: col.width || 80 }}>{col.label}</Box>
              ))}
              <Box sx={{ width: 30 }} />
            </Box>

            {records.map(r => {
              const isDone = r.status === '检测完成';
              return (
                <Box key={r.id}>
                  <Box
                    onClick={() => doExpand(r.id)}
                    sx={{
                      display: 'flex', px: 2, py: 1.5, borderBottom: '1px solid #f0f0f0', cursor: 'pointer',
                      alignItems: 'center', flexWrap: 'wrap',
                      opacity: isDone ? 0.5 : 1,
                      '&:hover': { bgcolor: '#fafafa' },
                    }}
                  >
                    <Box sx={{ width: 70 }}>
                      <Chip label={r.status} color={STATUS_COLORS[r.status] || 'default'} size="small" variant={isDone ? 'outlined' : 'filled'} />
                    </Box>
                    <Box sx={{ width: 50, fontSize: '0.875rem', color: '#999' }}>#{r.seq_no}</Box>
                    {listColumns.filter(c => !['status', 'seq_no'].includes(c.field_key)).map(col => (
                      <Box key={col.field_key} sx={{ flex: 1, minWidth: col.width || 80, fontSize: '0.875rem', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                        {renderCellValue(col, r)}
                      </Box>
                    ))}
                    <Box sx={{ width: 30 }}>{expandedId === r.id ? <ExpandLessIcon fontSize="small" /> : <ExpandMoreIcon fontSize="small" />}</Box>
                  </Box>

                  {/* 展开详情 */}
                  <Collapse in={expandedId === r.id}>
                    <Paper elevation={0} sx={{ mx: 2, mb: 1, p: 2, border: '1px solid #e8e8e8', borderRadius: R, bgcolor: '#fafafa' }}>
                      <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
                        <Typography variant="subtitle2" fontWeight={700} color="#2e7d32">登记详情 · 序号 #{r.seq_no}</Typography>
                        <Chip label={r.status} color={STATUS_COLORS[r.status] || 'default'} size="small" />
                      </Box>

                      {editingId === r.id ? (
                        <Grid container spacing={2}>
                          <Grid item xs={12} sm={6}>
                            <TextField label="样品批号" fullWidth size="small" value={editForm.batch_no || ''} onChange={e => setEditForm(p => ({ ...p, batch_no: e.target.value }))} />
                          </Grid>
                          <Grid item xs={12} sm={6}>
                            <TextField label="送样人" fullWidth size="small" value={editForm.user_name || ''} onChange={e => setEditForm(p => ({ ...p, user_name: e.target.value }))} />
                          </Grid>
                          <Grid item xs={12} sm={6}>
                            <TextField label="实验室/车间" fullWidth size="small" value={editForm.lab_name || ''} onChange={e => setEditForm(p => ({ ...p, lab_name: e.target.value }))} />
                          </Grid>
                          <Grid item xs={12} sm={6}>
                            <TextField label="所属项目" fullWidth size="small" value={editForm.project_name || ''} onChange={e => setEditForm(p => ({ ...p, project_name: e.target.value }))} />
                          </Grid>
                          <Grid item xs={12} sm={6}>
                            <TextField label="送样时间" type="datetime-local" fullWidth size="small" value={editForm.submitted_at || ''} onChange={e => setEditForm(p => ({ ...p, submitted_at: e.target.value }))} InputLabelProps={{ shrink: true }} />
                          </Grid>
                          <Grid item xs={12}>
                            <TextField label="样品主要成分" fullWidth size="small" value={editForm.main_components || ''} onChange={e => setEditForm(p => ({ ...p, main_components: e.target.value }))} />
                          </Grid>
                          <Grid item xs={12}>
                            <TextField label="注意事项" fullWidth size="small" multiline rows={2} value={editForm.notes || ''} onChange={e => setEditForm(p => ({ ...p, notes: e.target.value }))} />
                          </Grid>
                          <Grid item xs={12}>
                            <Box sx={{ display: 'flex', gap: 1, justifyContent: 'flex-end' }}>
                              <Button variant="outlined" size="small" onClick={doCancelEdit} sx={{ borderRadius: R }}>取消</Button>
                              <Button variant="contained" size="small" onClick={() => doSaveEdit(r.id)} sx={{ borderRadius: R, bgcolor: '#2e7d32', '&:hover': { bgcolor: '#1b5e20' } }}>保存</Button>
                            </Box>
                          </Grid>
                        </Grid>
                      ) : (
                        <>
                          <Grid container spacing={2} sx={{ mb: 2 }}>
                            <Grid item xs={12} sm={6}><Typography variant="caption" color="text.secondary">样品批号</Typography><Typography variant="body2">{r.batch_no}</Typography></Grid>
                            <Grid item xs={12} sm={6}><Typography variant="caption" color="text.secondary">送样人</Typography><Typography variant="body2">{r.user_name}</Typography></Grid>
                            <Grid item xs={12} sm={6}><Typography variant="caption" color="text.secondary">实验室/车间</Typography><Typography variant="body2">{r.lab_name}</Typography></Grid>
                            <Grid item xs={12} sm={6}><Typography variant="caption" color="text.secondary">所属项目</Typography><Typography variant="body2">{r.project_name}</Typography></Grid>
                            <Grid item xs={12} sm={6}><Typography variant="caption" color="text.secondary">送样时间</Typography><Typography variant="body2">{fmtDate(r.submitted_at)}</Typography></Grid>
                            <Grid item xs={12} sm={6}><Typography variant="caption" color="text.secondary">检测时间</Typography><Typography variant="body2">{fmtDate(r.detection_date)}</Typography></Grid>
                            <Grid item xs={12}><Typography variant="caption" color="text.secondary">样品主要成分</Typography><Typography variant="body2">{r.main_components}</Typography></Grid>
                            {r.notes && <Grid item xs={12}><Typography variant="caption" color="text.secondary">注意事项</Typography><Typography variant="body2">{r.notes}</Typography></Grid>}
                          </Grid>
                          <Box sx={{ display: 'flex', gap: 1, justifyContent: 'flex-end' }}>
                            {!isDone && <Button variant="outlined" size="small" onClick={() => doEdit(r)} sx={{ borderRadius: R }}>编辑</Button>}
                            {NEXT_STATUS[r.status] && (
                              <Button variant="contained" size="small" onClick={() => doStatusFlow(r.id, r.status)}
                                sx={{ borderRadius: R, bgcolor: '#2e7d32', '&:hover': { bgcolor: '#1b5e20' } }}>
                                {NEXT_STATUS_LABEL[r.status]}
                              </Button>
                            )}
                          </Box>
                        </>
                      )}
                    </Paper>
                  </Collapse>
                </Box>
              );
            })}

            {records.length === 0 && <Box sx={{ textAlign: 'center', py: 6, color: '#999' }}>暂无登记记录</Box>}

            <TablePagination
              component="div" count={total} page={page} onPageChange={(_, p) => setPage(p)}
              rowsPerPage={PAGE_SIZE} rowsPerPageOptions={[PAGE_SIZE]}
              onRowsPerPageChange={e => { setPage(0); }}
              labelRowsPerPage="每页"
            />
          </>
        )}
      </Paper>

      <Snackbar open={snack.open} autoHideDuration={3000} onClose={() => setSnack(s => ({ ...s, open: false }))} anchorOrigin={{ vertical: 'top', horizontal: 'center' }}>
        <Alert severity={snack.sev} sx={{ width: '100%' }} onClose={() => setSnack(s => ({ ...s, open: false }))}>{snack.msg}</Alert>
      </Snackbar>
    </Box>
  );
};
export default SampleInfoEntry;
