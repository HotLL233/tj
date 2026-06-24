import React from 'react';
import { Box, Paper, Typography } from '@mui/material';
import BarChartIcon from '@mui/icons-material/BarChart';
import ReceiptLongIcon from '@mui/icons-material/ReceiptLong';
import PeopleIcon from '@mui/icons-material/People';
import FolderIcon from '@mui/icons-material/Folder';
import type { StatsSummary } from '../types';
import type { TabValue } from '../pages/StatsPage';

interface StatsCardsProps { summary: StatsSummary; onCardClick?: (tab: TabValue) => void; }
interface CardItemDef { key: keyof StatsSummary; label: string; color: string; gradient: string; icon: React.ReactNode; tab: TabValue; }
const CARD_ITEMS: CardItemDef[] = [
  { key: 'total_quantity', label: '总数量', color: '#667eea', gradient: 'linear-gradient(135deg, #667eea, #764ba2)', icon: <BarChartIcon />, tab: 'week' },
  { key: 'total_records', label: '总记录数', color: '#43a047', gradient: 'linear-gradient(135deg, #43a047, #66bb6a)', icon: <ReceiptLongIcon />, tab: 'user-log' },
  { key: 'user_count', label: '参与人数', color: '#f57c00', gradient: 'linear-gradient(135deg, #f57c00, #ff9800)', icon: <PeopleIcon />, tab: 'user' },
  { key: 'project_count', label: '项目数', color: '#7b1fa2', gradient: 'linear-gradient(135deg, #7b1fa2, #ab47bc)', icon: <FolderIcon />, tab: 'project' },
];
const StatsCards: React.FC<StatsCardsProps> = ({ summary, onCardClick }) => (<Box sx={{ display: 'grid', gridTemplateColumns: { xs: 'repeat(2, 1fr)', sm: 'repeat(4, 1fr)' }, gap: 2, mb: 3, px: 1 }}>{CARD_ITEMS.map(({ key, label, color, gradient, icon, tab }) => (<Paper key={key} elevation={0} onClick={() => onCardClick?.(tab)} sx={{ p: 2.5, borderRadius: '2px', cursor: onCardClick ? 'pointer' : 'default', background: 'linear-gradient(145deg, #ffffff, #f5f5f5)', border: '1px solid rgba(0,0,0,0.06)', borderTop: `3px solid ${color}`, boxShadow: '0 4px 20px rgba(0,0,0,0.06)', transition: 'all 0.3s cubic-bezier(0.4,0,0.2,1)', '&:hover': { transform: 'translateY(-4px)', boxShadow: `0 12px 30px ${color}25` } }}><Box sx={{ position: 'absolute', top: -8, right: -8, width: 40, height: 40, borderRadius: '50%', background: gradient, opacity: 0.1 }} /><Box sx={{ display: 'flex', alignItems: 'center', gap: 1.5, mb: 1.5 }}><Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'center', width: 36, height: 36, borderRadius: '2px', background: `${color}16`, color }}>{icon}</Box><Typography variant="body2" color="text.secondary" fontWeight={500}>{label}</Typography></Box><Typography variant="h4" fontWeight={800} sx={{ background: gradient, WebkitBackgroundClip: 'text', WebkitTextFillColor: 'transparent', backgroundClip: 'text' }}>{String(summary[key] ?? 0)}</Typography></Paper>))}</Box>);
export default StatsCards;
