import React from 'react';
import { Tooltip, Box } from '@mui/material';

interface Props {
  value: string | number | null | undefined;
  maxWidth?: number;
  /** 悬停时显示的完整文字（默认取 value 的字符串） */
  title?: string;
}

const R = '2px';

const TruncatedCell: React.FC<Props> = ({ value, maxWidth = 140, title }) => {
  const str = value == null ? '' : String(value);
  if (!str) return <>-</>;
  // 短字符串直接显示，不加 tooltip（节省无意义 hover）
  if (str.length <= Math.floor(maxWidth / 8)) {
    return (
      <Box sx={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', maxWidth }}>
        {str}
      </Box>
    );
  }
  return (
    <Tooltip title={title || str} placement="top" arrow disableInteractive>
      <Box sx={{
        overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap',
        maxWidth, cursor: 'help', display: 'inline-block', verticalAlign: 'middle',
        borderRadius: R, px: 0.5,
      }}>
        {str}
      </Box>
    </Tooltip>
  );
};

export default TruncatedCell;
