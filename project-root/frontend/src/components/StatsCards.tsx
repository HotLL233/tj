import React from 'react';
import { Box, Paper, Typography, Chip } from '@mui/material';
import BarChartIcon from '@mui/icons-material/BarChart';
import ReceiptLongIcon from '@mui/icons-material/ReceiptLong';
import PeopleIcon from '@mui/icons-material/People';
import FolderIcon from '@mui/icons-material/Folder';
import ScienceIcon from '@mui/icons-material/Science';
import MemoryIcon from '@mui/icons-material/Memory';
import type { StatsSummary, TypeStats, ProjectStats, InstrumentStats } from '../types';
import type { TabValue } from '../pages/StatsPage';

interface StatsCardsProps {
  summary: StatsSummary;
  byType?: TypeStats[];
  byProject?: ProjectStats[];
  byInstrument?: InstrumentStats[];
  onCardClick?: (tab: TabValue) => void;
}
interface CardItemDef {
  key: string;
  label: string;
  color: string;
  gradient: string;
  icon: React.ReactNode;
  tab: TabValue;
  value: number;
  subtitle?: string;
}
const StatsCards: React.FC<StatsCardsProps> = ({ summary, byType, byProject, byInstrument, onCardClick }) => {
  const typeCount = byType?.length || 0;
  const projectCount = byProject?.length || 0;
  const instrumentCount = byInstrument?.length || 0;
  const labCount = new Set(byProject?.map(p => p.group_name).filter(Boolean)).size;

  const CARD_ITEMS: CardItemDef[] = [
    { key: 'total_quantity', label: '总数量', color: '#667eea', gradient: 'linear-gradient(135deg, #667eea, #764ba2)', icon: <BarChartIcon />, tab: 'week', value: summary.total_quantity },
    { key: 'total_records', label: '总记录数', color: '#43a047', gradient: 'linear-gradient(135deg, #43a047, #66bb6a)', icon: <ReceiptLongIcon />, tab: 'user-log', value: summary.total_records },
    { key: 'user_count', label: '参与人数', color: '#f57c00', gradient: 'linear-gradient(135deg, #f57c00, #ff9800)', icon: <PeopleIcon />, tab: 'sheet6', value: summary.user_count },
    { key: 'project_count', label: '项目数', color: '#7b1fa2', gradient: 'linear-gradient(135deg, #7b1fa2, #ab47bc)', icon: <FolderIcon />, tab: 'sheet8', value: summary.project_count },
    { key: 'lab_count', label: '实验室数', color: '#00897b', gradient: 'linear-gradient(135deg, #00897b, #43a047)', icon: <ScienceIcon />, tab: 'sheet7', value: labCount, subtitle: `${projectCount} 个项目` },
    { key: 'type_count', label: '方法类型', color: '#e91e63', gradient: 'linear-gradient(135deg, #e91e63, #ff5722)', icon: <MemoryIcon />, tab: 'sheet9', value: typeCount, subtitle: byType?.map(t => `${t.instrument_type}:${t.total_quantity}`).join(', ') },
  ];
  return (<Box sx={{ display: 'grid', gridTemplateColumns: { xs: 'repeat(2, 1fr)', sm: 'repeat(4, 1fr)', md: 'repeat(6, 1fr)' }, gap: 2, mb: 3, px: 1 }}>{CARD_ITEMS.map(({ key, label, color, gradient, icon, tab, value, subtitle }) => (<Paper key={key} elevation={0} onClick={() => onCardClick?.(tab)} sx={{ p: 2.5, borderRadius: '2px', cursor: onCardClick ? 'pointer' : 'default', background: 'linear-gradient(145deg, #ffffff, #f5f5f5)', border: '1px solid rgba(0,0,0,0.06)', borderTop: `3px solid ${color}`, boxShadow: '0 4px 20px rgba(0,0,0,0.06)', transition: 'all 0.3s cubic-bezier(0.4,0,0.2,1)', '&:hover': { transform: 'translateY(-4px)', boxShadow: `0 12px 30px ${color}25` } }}><Box sx={{ position: 'absolute', top: -8, right: -8, width: 40, height: 40, borderRadius: '50%', background: gradient, opacity: 0.1 }} /><Box sx={{ display: 'flex', alignItems: 'center', gap: 1.5, mb: 1.5 }}><Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'center', width: 36, height: 36, borderRadius: '2px', background: `${color}16`, color }}>{icon}</Box><Typography variant="body2" color="text.secondary" fontWeight={500}>{label}</Typography></Box><Typography variant="h4" fontWeight={800} sx={{ background: gradient, WebkitBackgroundClip: 'text', WebkitTextFillColor: 'transparent', backgroundClip: 'text' }}>{String(value)}</Typography>{subtitle && <Typography variant="caption" color="text.secondary" sx={{ mt: 0.5, display: 'block', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{subtitle}</Typography>}</Paper>))}</Box>);
};
export default StatsCards;
