import React from 'react';
import {
  Paper,
  Typography,
  Chip,
  Avatar,
  Box,
} from '@mui/material';
import ScienceIcon from '@mui/icons-material/Science';
import type { ProjectGroup } from '../types';

interface GroupCardProps {
  group: ProjectGroup;
  onClick: () => void;
}

/** Color palette for rotating card accents */
const CARD_COLORS = [
  '#667eea', '#764ba2', '#1e88e5', '#00acc1',
  '#00897b', '#43a047', '#f4511e', '#e53935',
  '#7b1fa2', '#f57c00',
];

/**
 * GroupCard — uiverse.io style.
 * Displays lab name with ScienceIcon avatar, project count chip,
 * hover lift + colored border glow effect.
 */
const GroupCard: React.FC<GroupCardProps> = ({ group, onClick }) => {
  const color = CARD_COLORS[group.id % CARD_COLORS.length];

  return (
    <Paper
      className="card-hover"
      onClick={onClick}
      sx={{
        p: 3,
        borderRadius: 4,
        textAlign: 'center',
        cursor: 'pointer',
        background: 'linear-gradient(145deg, #ffffff, #f5f5f5)',
        border: '1px solid rgba(0,0,0,0.06)',
        boxShadow: '0 4px 20px rgba(0,0,0,0.06), 0 1px 3px rgba(0,0,0,0.04)',
        transition: 'all 0.3s cubic-bezier(0.4, 0, 0.2, 1)',
        '&:hover': {
          transform: 'translateY(-4px)',
          boxShadow: `0 12px 30px ${color}25, 0 4px 10px rgba(0,0,0,0.06)`,
          borderColor: `${color}50`,
        },
        '&:active': {
          transform: 'scale(0.98)',
        },
      }}
    >
      <Avatar
        sx={{
          width: 64,
          height: 64,
          mx: 'auto',
          mb: 2,
          bgcolor: `${color}18`,
        }}
      >
        <ScienceIcon sx={{ fontSize: 36, color }} />
      </Avatar>
      <Typography
        variant="h6"
        fontWeight={700}
        sx={{
          overflow: 'hidden',
          textOverflow: 'ellipsis',
          display: '-webkit-box',
          WebkitLineClamp: 2,
          WebkitBoxOrient: 'vertical',
          mb: 1.5,
          lineHeight: 1.3,
        }}
      >
        {group.name}
      </Typography>
      <Chip
        label={`${group.project_count} 个项目`}
        size="small"
        variant="outlined"
        sx={{
          borderRadius: 2,
          borderColor: `${color}40`,
          color,
          fontWeight: 500,
        }}
      />
    </Paper>
  );
};

export default GroupCard;
