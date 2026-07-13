import React from 'react';
import { Paper, Typography, Chip, Avatar, Box, useMediaQuery, useTheme } from '@mui/material';
import ScienceIcon from '@mui/icons-material/Science';
import type { ProjectGroup } from '../types';
interface GroupCardProps { group: ProjectGroup; onClick: () => void; }
const CARD_COLORS = ['#667eea','#764ba2','#1e88e5','#00acc1','#00897b','#43a047','#f4511e','#e53935','#7b1fa2','#f57c00'];
const GroupCard: React.FC<GroupCardProps> = ({ group, onClick }) => { 
  const theme = useTheme(); const isMobile = useMediaQuery(theme.breakpoints.down('sm'));
  const color = CARD_COLORS[group.id % CARD_COLORS.length]; 
  return (<Paper className="card-hover" onClick={onClick} sx={{ 
    p: isMobile ? 1.5 : 3, borderRadius: '2px', textAlign: 'center', cursor: 'pointer', 
    background: 'linear-gradient(145deg,#ffffff,#f5f5f5)', border: '1px solid rgba(0,0,0,0.06)', 
    boxShadow: '0 4px 20px rgba(0,0,0,0.06)', transition: 'all 0.3s', 
    '&:hover': { transform: 'translateY(-4px)', boxShadow: `0 12px 30px ${color}25`, borderColor: `${color}50` }, 
    '&:active': { transform: 'scale(0.98)' }
  }}><Avatar sx={{ width: isMobile ? 32 : 64, height: isMobile ? 32 : 64, mx: 'auto', mb: isMobile ? 1 : 2, bgcolor: `${color}18` }}>
    <ScienceIcon sx={{ fontSize: isMobile ? 20 : 36, color }} />
  </Avatar>
  <Typography variant={isMobile ? 'caption' : 'h6'} fontWeight={700} sx={{ 
    overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', mb: isMobile ? 0.5 : 1.5, lineHeight: 1.3 
  }}>{group.name}</Typography>
  {!isMobile && <Chip label={`${group.project_count} 个项目`} size="small" variant="outlined" sx={{ borderRadius: '2px', borderColor: `${color}40`, color, fontWeight: 500 }} />}
  </Paper>); };
export default GroupCard;
