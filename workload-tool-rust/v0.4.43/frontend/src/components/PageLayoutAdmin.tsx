import React, { useState, useEffect, useCallback } from 'react';
import {
  Box, Paper, Typography, List, ListItemButton, ListItemIcon, ListItemText,
  Switch, TextField, Button, Snackbar, Alert, Tooltip, IconButton, Divider,
} from '@mui/material';
import HomeIcon from '@mui/icons-material/Home';
import AssessmentIcon from '@mui/icons-material/Assessment';
import ScienceIcon from '@mui/icons-material/Science';
import EditNoteIcon from '@mui/icons-material/EditNote';
import ListAltIcon from '@mui/icons-material/ListAlt';
import FolderIcon from '@mui/icons-material/Folder';
import WorkIcon from '@mui/icons-material/Work';
import DashboardIcon from '@mui/icons-material/Dashboard';
import SettingsIcon from '@mui/icons-material/Settings';
import SaveIcon from '@mui/icons-material/Save';
import RefreshIcon from '@mui/icons-material/Refresh';
import type { PageLayout, SectionConfig } from './PageSectionEditor';
import { getSetting, updateSetting } from '../api/client';

const R = '2px';

interface PageDef {
  key: string;
  name: string;
  icon: React.ReactNode;
  sections: {
    key: string;
    label: string;
    hasLabel?: boolean;
    hasColor?: boolean;
    hasChipLabel?: boolean;
    hasButtonLabel?: boolean;
    hasActionBtns?: boolean;
  }[];
}

const PAGES: PageDef[] = [
  {
    key: 'home', name: '家页（知微）', icon: <HomeIcon />,
    sections: [
      { key: 'brand-title', label: '品牌标题', hasLabel: true, hasColor: true },
      { key: 'entry-cards', label: '入口卡片区', hasButtonLabel: true },
      { key: 'footer-text', label: '底部文字', hasLabel: true },
    ],
  },
  {
    key: 'stats', name: '分析检测统计', icon: <AssessmentIcon />,
    sections: [
      { key: 'page-title', label: '页面标题', hasLabel: true },
      { key: 'stat-cards', label: '统计卡片', hasButtonLabel: true },
      { key: 'charts', label: '图表区', hasButtonLabel: true },
    ],
  },
  {
    key: 'sample_portal', name: '研发送样入口', icon: <ScienceIcon />,
    sections: [
      { key: 'page-title', label: '页面标题', hasLabel: true },
      { key: 'division-chips', label: '部门筛选 Chip', hasChipLabel: true },
      { key: 'group-cards', label: '实验室卡片', hasButtonLabel: true },
    ],
  },
  {
    key: 'sample_entry', name: '研发送样录入', icon: <EditNoteIcon />,
    sections: [
      { key: 'page-title', label: '页面标题', hasLabel: true },
      { key: 'sample-time', label: '送样时间', hasLabel: true },
      { key: 'action-btns', label: '操作按钮栏', hasActionBtns: true },
      { key: 'entry-table', label: '录入表格', hasButtonLabel: true },
      { key: 'submit-btn', label: '提交按钮', hasLabel: true },
      { key: 'today-records', label: '今日记录', hasButtonLabel: true },
    ],
  },
  {
    key: 'rd_records', name: '研发送样记录', icon: <ListAltIcon />,
    sections: [
      { key: 'page-title', label: '页面标题', hasLabel: true },
      { key: 'table-columns', label: '表格列配置', hasButtonLabel: true },
      { key: 'filters', label: '筛选区', hasButtonLabel: true },
    ],
  },
  {
    key: 'sample_info_home', name: '样品信息入口', icon: <FolderIcon />,
    sections: [
      { key: 'page-title', label: '页面标题', hasLabel: true },
      { key: 'type-cards', label: '类型卡片', hasButtonLabel: true },
    ],
  },
  {
    key: 'sample_info_entry', name: '样品信息登记', icon: <EditNoteIcon />,
    sections: [
      { key: 'page-title', label: '页面标题', hasLabel: true },
      { key: 'form-fields', label: '表单字段区', hasButtonLabel: true },
      { key: 'detection-type-chip', label: '检测类型 Chip', hasChipLabel: true },
      { key: 'seq-no-chip', label: '序号 Chip', hasLabel: true },
      { key: 'sample-time', label: '送样时间', hasLabel: true },
      { key: 'action-btns', label: '操作按钮栏', hasActionBtns: true },
      { key: 'submit-btn', label: '提交按钮', hasLabel: true },
      { key: 'records-table', label: '记录表格', hasButtonLabel: true },
    ],
  },
  {
    key: 'sample_info_stats', name: '样品信息统计', icon: <AssessmentIcon />,
    sections: [
      { key: 'page-title', label: '页面标题', hasLabel: true },
      { key: 'stat-cards', label: '统计卡片', hasButtonLabel: true },
      { key: 'charts', label: '图表区', hasButtonLabel: true },
    ],
  },
  {
    key: 'workload_portal', name: '分析检测入口', icon: <WorkIcon />,
    sections: [
      { key: 'page-title', label: '页面标题', hasLabel: true },
      { key: 'division-chips', label: '部门筛选 Chip', hasChipLabel: true },
      { key: 'group-cards', label: '实验室卡片', hasButtonLabel: true },
    ],
  },
  {
    key: 'sample_stats', name: '研发送样统计', icon: <AssessmentIcon />,
    sections: [
      { key: 'page-title', label: '页面标题', hasLabel: true },
      { key: 'stat-cards', label: '统计卡片', hasButtonLabel: true },
      { key: 'charts', label: '图表区', hasButtonLabel: true },
    ],
  },
  {
    key: 'manage', name: '管理页', icon: <SettingsIcon />,
    sections: [
      { key: 'page-title', label: '页面标题', hasLabel: true },
      { key: 'tabs', label: '标签页区域', hasButtonLabel: true },
    ],
  },
  {
    key: 'entry', name: '分析检测录入', icon: <DashboardIcon />,
    sections: [
      { key: 'page-title', label: '页面标题', hasLabel: true },
      { key: 'form', label: '表单区域', hasButtonLabel: true },
      { key: 'action-btns', label: '操作按钮栏', hasActionBtns: true },
      { key: 'submit-btn', label: '提交按钮', hasLabel: true },
    ],
  },
];

const PageLayoutAdmin: React.FC = () => {
  const [sel, setSel] = useState<string>('');
  const [layout, setLayout] = useState<PageLayout | null>(null);
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [snackMsg, setSnackMsg] = useState('');
  const [snackErr, setSnackErr] = useState(false);
  const [dirty, setDirty] = useState(false);

  const pageDef = PAGES.find(p => p.key === sel);

  const loadLayout = useCallback(async (pageKey: string) => {
    if (!pageKey) return;
    setLoading(true);
    try {
      const r = await getSetting(`layout_${pageKey}`);
      if (r.code === 0 && r.data) {
        const parsed = JSON.parse(r.data.value) as PageLayout;
        if (parsed?.sections) setLayout(parsed);
        else setLayout(null);
      } else setLayout(null);
    } catch { setLayout(null); }
    finally { setLoading(false); setDirty(false); }
  }, []);

  useEffect(() => { if (sel) loadLayout(sel); }, [sel, loadLayout]);

  const updateSection = (sectionKey: string, patch: Partial<SectionConfig>) => {
    if (!layout) return;
    const updated = {
      ...layout,
      sections: {
        ...layout.sections,
        [sectionKey]: { ...(layout.sections[sectionKey] || { visible: true }), ...patch },
      },
    };
    setLayout(updated);
    setDirty(true);
  };

  const saveLayout = async () => {
    if (!sel || !layout) return;
    setSaving(true);
    try {
      const r = await updateSetting(`layout_${sel}`, layout);
      if (r.code !== 0) throw new Error(r.message || '保存失败');
      setSnackMsg('✅ 布局已保存');
      setSnackErr(false);
      setDirty(false);
    } catch (e: any) {
      setSnackMsg('❌ 保存失败: ' + (e.message || ''));
      setSnackErr(true);
    } finally { setSaving(false); }
  };

  const resetDefault = () => {
    if (!sel || !layout) return;
    const reset: PageLayout = { sections: {} };
    const def = PAGES.find(p => p.key === sel);
    if (def) {
      def.sections.forEach(s => {
        reset.sections[s.key] = { visible: true };
        if (s.hasLabel) reset.sections[s.key].label = s.label;
      });
    }
    setLayout(reset);
    setDirty(true);
  };

  const renderSectionEditor = (sec: PageDef['sections'][0]) => {
    if (!layout) return null;
    const cfg = layout.sections[sec.key] || { visible: true };
    return (
      <Paper
        key={sec.key}
        elevation={0}
        sx={{
          p: 2, mb: 1.5, border: '1px solid #e0e0e0', borderRadius: R,
          ...((!cfg.visible) && { opacity: 0.5, bgcolor: '#fafafa' }),
        }}
      >
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 1, flexWrap: 'wrap' }}>
          <Switch
            size="small"
            checked={cfg.visible !== false}
            onChange={(_, v) => updateSection(sec.key, { visible: v })}
          />
          <Typography variant="subtitle2" sx={{ fontWeight: 700, flex: 1 }}>
            {cfg.label || sec.label}
          </Typography>
          <Typography variant="caption" color="text.secondary">
            section: {sec.key}
          </Typography>
        </Box>

        {(sec.hasLabel || sec.hasChipLabel || sec.hasButtonLabel) && (
          <Box sx={{ display: 'flex', gap: 1, flexWrap: 'wrap', mb: 1 }}>
            {sec.hasLabel && (
              <TextField
                size="small" label="显示文本"
                value={cfg.label ?? ''}
                onChange={e => updateSection(sec.key, { label: e.target.value || undefined })}
                sx={{ minWidth: 200, '& .MuiOutlinedInput-root': { borderRadius: R } }}
              />
            )}
            {sec.hasColor && (
              <TextField
                size="small" label="颜色" type="color"
                value={cfg.color ?? '#1976d2'}
                onChange={e => updateSection(sec.key, { color: e.target.value })}
                sx={{ width: 80, '& .MuiOutlinedInput-root': { borderRadius: R } }}
                inputProps={{ style: { padding: '2px 4px', height: 32 } }}
              />
            )}
            {sec.hasChipLabel && (
              <TextField
                size="small" label="Chip 文字"
                value={cfg.chipLabel ?? ''}
                onChange={e => updateSection(sec.key, { chipLabel: e.target.value || undefined })}
                sx={{ minWidth: 150, '& .MuiOutlinedInput-root': { borderRadius: R } }}
              />
            )}
            {sec.hasButtonLabel && (
              <TextField
                size="small" label="按钮文字"
                value={cfg.buttonLabel ?? ''}
                onChange={e => updateSection(sec.key, { buttonLabel: e.target.value || undefined })}
                sx={{ minWidth: 150, '& .MuiOutlinedInput-root': { borderRadius: R } }}
              />
            )}
          </Box>
        )}

        {sec.hasActionBtns && (
          <Box>
            <Typography variant="caption" color="text.secondary" sx={{ mb: 0.5, display: 'block' }}>
              操作按钮文字
            </Typography>
            <Box sx={{ display: 'flex', gap: 1, flexWrap: 'wrap' }}>
              <TextField
                size="small" label="添加行按钮"
                value={cfg.actionButtons?.addLabel ?? ''}
                onChange={e => updateSection(sec.key, {
                  actionButtons: { ...(cfg.actionButtons || {}), addLabel: e.target.value || undefined },
                })}
                sx={{ minWidth: 150, '& .MuiOutlinedInput-root': { borderRadius: R } }}
              />
              <TextField
                size="small" label="删除按钮"
                value={cfg.actionButtons?.deleteLabel ?? ''}
                onChange={e => updateSection(sec.key, {
                  actionButtons: { ...(cfg.actionButtons || {}), deleteLabel: e.target.value || undefined },
                })}
                sx={{ minWidth: 150, '& .MuiOutlinedInput-root': { borderRadius: R } }}
              />
              <TextField
                size="small" label="重置按钮"
                value={cfg.actionButtons?.resetLabel ?? ''}
                onChange={e => updateSection(sec.key, {
                  actionButtons: { ...(cfg.actionButtons || {}), resetLabel: e.target.value || undefined },
                })}
                sx={{ minWidth: 150, '& .MuiOutlinedInput-root': { borderRadius: R } }}
              />
            </Box>
          </Box>
        )}
      </Paper>
    );
  };

  return (
    <Box sx={{ display: 'flex', gap: 2, height: 'calc(100vh - 200px)', minHeight: 500 }}>
      {/* 左侧：页面列表 */}
      <Paper elevation={0} sx={{ width: 220, flexShrink: 0, border: '1px solid #e0e0e0', borderRadius: R, overflow: 'auto' }}>
        <Typography variant="subtitle2" sx={{ p: 1.5, fontWeight: 700, bgcolor: '#f5f5f5', borderBottom: '1px solid #e0e0e0' }}>
          选择页面
        </Typography>
        <List dense disablePadding>
          {PAGES.map(p => (
            <ListItemButton
              key={p.key}
              selected={sel === p.key}
              onClick={() => setSel(p.key)}
              sx={{ borderRadius: 0, '&.Mui-selected': { bgcolor: 'rgba(25,118,210,0.08)', '&:hover': { bgcolor: 'rgba(25,118,210,0.12)' } } }}
            >
              <ListItemIcon sx={{ minWidth: 36 }}>{p.icon}</ListItemIcon>
              <ListItemText primary={p.name} primaryTypographyProps={{ variant: 'body2', noWrap: true }} />
            </ListItemButton>
          ))}
        </List>
      </Paper>

      {/* 右侧：编辑区 */}
      <Paper elevation={0} sx={{ flex: 1, border: '1px solid #e0e0e0', borderRadius: R, p: 2, overflow: 'auto' }}>
        {!sel && (
          <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%', color: '#999', flexDirection: 'column', gap: 1 }}>
            <SettingsIcon sx={{ fontSize: 48, opacity: 0.3 }} />
            <Typography>← 选择左侧页面开始编辑布局</Typography>
          </Box>
        )}

        {sel && loading && (
          <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'center', py: 4, color: '#999' }}>
            <Typography>加载中...</Typography>
          </Box>
        )}

        {sel && !loading && (
          <>
            <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', mb: 2 }}>
              <Typography variant="h6" fontWeight={700}>{pageDef?.name || sel}</Typography>
              <Box sx={{ display: 'flex', gap: 1 }}>
                <Tooltip title="重置为默认">
                  <Button size="small" variant="outlined" startIcon={<RefreshIcon />} onClick={resetDefault} sx={{ borderRadius: R }}>
                    重置默认
                  </Button>
                </Tooltip>
                <Tooltip title={dirty ? '有未保存的更改' : ''}>
                  <Button size="small" variant="contained" startIcon={<SaveIcon />} onClick={saveLayout}
                    disabled={saving || !dirty} sx={{ borderRadius: R, bgcolor: dirty ? '#1976d2' : '#ccc' }}>
                    {saving ? '保存中...' : '保存布局'}
                  </Button>
                </Tooltip>
              </Box>
            </Box>
            <Divider sx={{ mb: 2 }} />
            {pageDef?.sections.map(renderSectionEditor)}
          </>
        )}
      </Paper>

      <Snackbar open={!!snackMsg} autoHideDuration={4000} onClose={() => setSnackMsg('')}
        anchorOrigin={{ vertical: 'bottom', horizontal: 'center' }}>
        <Alert severity={snackErr ? 'error' : 'success'} onClose={() => setSnackMsg('')} sx={{ borderRadius: R }}>
          {snackMsg}
        </Alert>
      </Snackbar>
    </Box>
  );
};

export default PageLayoutAdmin;
