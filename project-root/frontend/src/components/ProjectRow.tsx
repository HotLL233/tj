import React, { useState } from 'react';
import {
  Box,
  Typography,
  TextField,
  IconButton,
  CircularProgress,
  Paper,
} from '@mui/material';
import CheckCircleIcon from '@mui/icons-material/CheckCircle';
import CheckCircleOutlineIcon from '@mui/icons-material/CheckCircleOutline';
import type { Project } from '../types';

interface ProjectRowProps {
  project: Project;
  onSubmit: (projectId: number, quantity: number) => Promise<boolean>;
}

/** Infer instrument type from project name or group name */
const getInstrumentColor = (project: Project): string => {
  const text = (project.group_name + project.name + (project.full_name || '')).toLowerCase();
  if (text.includes('液相')) return '#1e88e5';
  if (text.includes('气相')) return '#43a047';
  return '#9e9e9e';
};

/**
 * ProjectRow — uiverse.io card style.
 * Left colored stripe indicates instrument type (液相=blue, 气相=green).
 * Pill-style submit button with success animation.
 */
const ProjectRow: React.FC<ProjectRowProps> = ({ project, onSubmit }) => {
  const [quantity, setQuantity] = useState<number | ''>('');
  const [loading, setLoading] = useState(false);
  const [success, setSuccess] = useState(false);
  const accentColor = getInstrumentColor(project);

  const handleSubmit = async () => {
    if (quantity === '' || Number(quantity) < 1) return;
    setLoading(true);
    const ok = await onSubmit(project.id, Number(quantity));
    setLoading(false);
    if (ok) {
      setSuccess(true);
      setTimeout(() => {
        setSuccess(false);
        setQuantity('');
      }, 2000);
    }
  };

  return (
    <Paper
      elevation={0}
      sx={{
        display: 'flex',
        alignItems: 'center',
        gap: 1.5,
        py: 1.5,
        px: 2,
        mb: 1,
        borderRadius: 3,
        background: success
          ? 'linear-gradient(145deg, #e8f5e9, #f1f8e9)'
          : 'linear-gradient(145deg, #ffffff, #fafafa)',
        border: '1px solid',
        borderColor: success ? '#a5d6a7' : 'rgba(0,0,0,0.06)',
        borderLeft: `4px solid ${success ? '#43a047' : accentColor}`,
        boxShadow: '0 2px 12px rgba(0,0,0,0.04)',
        transition: 'all 0.3s cubic-bezier(0.4, 0, 0.2, 1)',
        '&:hover': {
          boxShadow: '0 4px 20px rgba(0,0,0,0.08)',
          transform: 'translateY(-1px)',
        },
      }}
    >
      {/* Project Info */}
      <Box sx={{ flex: 1, minWidth: 0 }}>
        <Typography
          variant="body1"
          fontWeight={600}
          sx={{
            overflow: 'hidden',
            textOverflow: 'ellipsis',
            whiteSpace: 'nowrap',
          }}
        >
          {project.name}
        </Typography>
        {project.full_name && (
          <Typography
            variant="caption"
            color="text.secondary"
            sx={{
              overflow: 'hidden',
              textOverflow: 'ellipsis',
              whiteSpace: 'nowrap',
              display: 'block',
              mt: 0.3,
            }}
          >
            {project.full_name}
          </Typography>
        )}
      </Box>

      {/* Quantity Input */}
      <TextField
        type="number"
        size="small"
        value={quantity}
        onChange={(e) =>
          setQuantity(e.target.value === '' ? '' : Number(e.target.value))
        }
        inputProps={{ min: 1, style: { textAlign: 'center', width: 60 } }}
        sx={{
          width: 80,
          '& .MuiOutlinedInput-root': {
            borderRadius: 3,
            '& fieldset': { borderColor: 'rgba(0,0,0,0.1)' },
          },
        }}
        disabled={loading || success}
        onKeyDown={(e) => {
          if (e.key === 'Enter') handleSubmit();
        }}
      />

      {/* Submit Button — pill style */}
      <IconButton
        onClick={handleSubmit}
        disabled={
          loading || success || quantity === '' || Number(quantity) < 1
        }
        sx={{
          borderRadius: '50%',
          bgcolor: success ? '#e8f5e9' : `${accentColor}14`,
          color: success ? '#43a047' : accentColor,
          transition: 'all 0.25s cubic-bezier(0.4, 0, 0.2, 1)',
          '&:hover': {
            bgcolor: `${accentColor}28`,
          },
          '&:disabled': {
            color: 'rgba(0,0,0,0.2)',
            bgcolor: 'transparent',
          },
        }}
        size="medium"
      >
        {loading ? (
          <CircularProgress size={24} sx={{ color: accentColor }} />
        ) : success ? (
          <CheckCircleIcon className="animate-checkmark" />
        ) : (
          <CheckCircleOutlineIcon />
        )}
      </IconButton>
    </Paper>
  );
};

export default ProjectRow;
