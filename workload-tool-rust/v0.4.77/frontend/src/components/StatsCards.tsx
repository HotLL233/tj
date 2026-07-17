import React, { useState, useEffect } from 'react';
import { Box, Paper, Typography } from '@mui/material';
import BarChartIcon from '@mui/icons-material/BarChart';
import ReceiptLongIcon from '@mui/icons-material/ReceiptLong';
import PeopleIcon from '@mui/icons-material/People';
import FolderIcon from '@mui/icons-material/Folder';
import ScienceIcon from '@mui/icons-material/Science';
import MemoryIcon from '@mui/icons-material/Memory';
import type { StatsSummary, TypeStats, ProjectStats, InstrumentStats, UserStats, StatCard } from '../types';
import type { TabValue } from '../pages/StatsPage';
import StatsMiniChart, { type MiniChartPoint } from './StatsMiniChart';

interface StatsCardsProps {
  summary: StatsSummary;
  trendDetails?: StatsSummary['details'];
  byType?: TypeStats[];
  byProject?: ProjectStats[];
  byInstrument?: InstrumentStats[];
  byUser?: UserStats[];
  onCardClick?: (tab: TabValue) => void;
  themeColor?: string;
  activeTab?: TabValue | null;
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
  chartType: 'line' | 'bar' | 'pie';
  chartData: MiniChartPoint[];
}

const fallbackIconMap: Record<string, React.ReactNode> = {
  total: <BarChartIcon />,
  records: <ReceiptLongIcon />,
  users: <PeopleIcon />,
  project: <FolderIcon />,
  project_count: <FolderIcon />,
  lab: <ScienceIcon />,
  lab_count: <ScienceIcon />,
  type: <MemoryIcon />,
  type_count: <MemoryIcon />,
};

const defaultCardItems: StatCard[] = [
  { key: 'total', label: '总数量', color: '#667eea', gradient: 'linear-gradient(135deg,#667eea,#764ba2)' },
  { key: 'records', label: '总记录数', color: '#43a047', gradient: 'linear-gradient(135deg,#43a047,#1b5e20)' },
  { key: 'users', label: '参与人数', color: '#fb8c00', gradient: 'linear-gradient(135deg,#fb8c00,#e65100)' },
];

const tabMap: Record<string, TabValue> = {
  total: 'week',
  records: 'user-log',
  users: 'sheet6',
  project: 'sheet8',
  project_count: 'sheet8',
  lab: 'sheet7',
  lab_count: 'sheet7',
  type: 'sheet9',
  type_count: 'sheet9',
};

const valueMap = (summary: StatsSummary, key: string, labCount: number, typeCount: number, projectCount: number): number => {
  switch (key) {
    case 'total': return summary.total_quantity;
    case 'records': return summary.total_records;
    case 'users': return summary.user_count;
    case 'project': case 'project_count': return summary.project_count;
    case 'lab': case 'lab_count': return labCount;
    case 'type': case 'type_count': return typeCount;
    default: return 0;
  }
};

const StatsCards: React.FC<StatsCardsProps> = ({ summary, trendDetails, byType, byProject, byInstrument, byUser, onCardClick, themeColor, activeTab }) => {
  const [statCards, setStatCards] = useState<StatCard[]>(defaultCardItems);
  const typeCount = byType?.length || 0;
  const projectCount = byProject?.length || 0;
  const labCount = new Set(byProject?.map(p => p.group_name).filter(Boolean)).size;
  const labTotals = [...(byProject || []).reduce((map, item) => {
    const name = item.group_name || '未分组';
    map.set(name, (map.get(name) || 0) + item.total_quantity);
    return map;
  }, new Map<string, number>())].map(([name, value]) => ({ name, value }));

  const miniChart = (key: string): { type: 'line' | 'bar' | 'pie'; data: MiniChartPoint[] } => {
    if (key === 'total') return { type: 'line', data: (trendDetails || summary.details || []).map(item => ({ name: item.period, value: item.total_quantity })) };
    if (key === 'records') return { type: 'bar', data: (trendDetails || summary.details || []).map(item => ({ name: item.period, value: item.record_count })) };
    if (key === 'users') return { type: 'bar', data: (byUser || []).slice(0, 8).map(item => ({ name: item.user_name, value: item.total_quantity })) };
    if (key === 'project' || key === 'project_count') return { type: 'bar', data: (byProject || []).slice(0, 8).map(item => ({ name: item.project_name, value: item.total_quantity })) };
    if (key === 'lab' || key === 'lab_count') return { type: 'pie', data: labTotals };
    if (key === 'type' || key === 'type_count') return { type: 'pie', data: (byType || []).map(item => ({ name: item.instrument_type, value: item.total_quantity })) };
    return { type: 'line', data: (byInstrument || []).slice(0, 8).map(item => ({ name: item.instrument, value: item.total_quantity })) };
  };

  useEffect(() => {
    fetch('/api/settings/stats-cards')
      .then(r => r.json())
      .then(d => {
        if (d.data?.value) {
          try { setStatCards(JSON.parse(d.data.value)); } catch {}
        }
      })
      .catch(() => {});
  }, []);

  const CARD_ITEMS: CardItemDef[] = statCards.map(sc => {
    const chart = miniChart(sc.key);
    return {
      key: sc.key, label: sc.label, color: sc.color, gradient: sc.gradient,
      icon: fallbackIconMap[sc.key] || <BarChartIcon />, tab: tabMap[sc.key] || 'week',
      value: valueMap(summary, sc.key, labCount, typeCount, projectCount),
      chartType: chart.type, chartData: chart.data,
    };
  });

  // 兼容旧模式：当 statCards 只有 3 条默认时，补全旧有的额外卡片
  if (statCards.length <= 3) {
    const extraBuiltins: CardItemDef[] = [
      { key: 'project_count', label: '项目数', color: themeColor || '#7b1fa2', gradient: 'linear-gradient(135deg,#7b1fa2,#ab47bc)', icon: <FolderIcon />, tab: 'sheet8', value: summary.project_count, chartType: 'bar', chartData: miniChart('project_count').data },
      { key: 'lab_count', label: '实验室数', color: themeColor || '#00897b', gradient: 'linear-gradient(135deg,#00897b,#43a047)', icon: <ScienceIcon />, tab: 'sheet7', value: labCount, subtitle: `${projectCount} 个项目`, chartType: 'pie', chartData: miniChart('lab_count').data },
      { key: 'type_count', label: '方法类型', color: themeColor || '#e91e63', gradient: 'linear-gradient(135deg,#e91e63,#ff5722)', icon: <MemoryIcon />, tab: 'sheet9', value: typeCount, subtitle: byType?.map(t => `${t.instrument_type}:${t.total_quantity}`).join(', '), chartType: 'pie', chartData: miniChart('type_count').data },
    ];
    extraBuiltins.forEach(eb => CARD_ITEMS.push(eb));
  }

  return (<Box sx={{ display: 'grid', gridTemplateColumns: { xs: 'repeat(2, 1fr)', sm: 'repeat(4, 1fr)', md: 'repeat(6, 1fr)' }, gap: 2, mb: 3, px: 1 }}>
    {CARD_ITEMS.map(({ key, label, color, gradient, icon, tab, value, subtitle, chartType, chartData }) => (
      <Paper key={key} elevation={0} onClick={() => onCardClick?.(tab)} sx={{
        p: 2, minHeight: 178, borderRadius: '2px', cursor: onCardClick ? 'pointer' : 'default', position: 'relative', overflow: 'hidden',
        background: 'linear-gradient(145deg, #ffffff, #f5f5f5)',
        border: activeTab === tab ? `2px solid ${color}` : '1px solid rgba(0,0,0,0.06)', borderTop: `3px solid ${color}`,
        boxShadow: activeTab === tab ? `0 0 0 2px ${color}18, 0 8px 24px ${color}20` : '0 4px 20px rgba(0,0,0,0.06)',
        transition: 'all 0.3s cubic-bezier(0.4,0,0.2,1)',
        '&:hover': { transform: 'translateY(-4px)', boxShadow: `0 12px 30px ${color}25` },
      }}>
        <Box sx={{ position: 'absolute', top: -8, right: -8, width: 40, height: 40, borderRadius: '50%', background: gradient, opacity: 0.1 }} />
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 1.25, mb: 1 }}>
          <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'center', width: 36, height: 36, borderRadius: '2px', background: `${color}16`, color }}>{icon}</Box>
          <Typography variant="body2" color="text.secondary" fontWeight={500}>{label}</Typography>
        </Box>
        <Typography variant="h4" fontWeight={800} sx={{ background: gradient, WebkitBackgroundClip: 'text', WebkitTextFillColor: 'transparent', backgroundClip: 'text' }}>{String(value)}</Typography>
        {subtitle && <Typography variant="caption" color="text.secondary" sx={{ mt: 0.25, display: 'block', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{subtitle}</Typography>}
        <Box sx={{ position: 'absolute', left: 14, right: 14, bottom: 7 }}><StatsMiniChart type={chartType} data={chartData} color={color} height={40} /></Box>
      </Paper>
    ))}
  </Box>);
};
export default StatsCards;
