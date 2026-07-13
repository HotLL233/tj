import React, { useEffect, useState } from 'react';
import { Box, Typography, Paper, CircularProgress, useMediaQuery, useTheme, Grid } from '@mui/material';
import PictureAsPdfIcon from '@mui/icons-material/PictureAsPdf';
import DescriptionIcon from '@mui/icons-material/Description';
import TableChartIcon from '@mui/icons-material/TableChart';
import ImageIcon from '@mui/icons-material/Image';
import InsertDriveFileIcon from '@mui/icons-material/InsertDriveFile';
import MenuBookIcon from '@mui/icons-material/MenuBook';
import { getHelpDocuments, getHelpDocumentFileUrl } from '../api/client';
import type { HelpDocument } from '../types';

const R = '2px';

/** 根据 file_type 返回对应图标 */
const fileIcon = (ft: string): React.ReactNode => {
  const sx = { fontSize: 40, color: 'text.secondary' };
  switch (ft.toLowerCase()) {
    case 'pdf': return <PictureAsPdfIcon sx={{ ...sx, color: '#e53935' }} />;
    case 'doc':
    case 'docx': return <DescriptionIcon sx={{ ...sx, color: '#1976d2' }} />;
    case 'xls':
    case 'xlsx': return <TableChartIcon sx={{ ...sx, color: '#388e3c' }} />;
    case 'png':
    case 'jpg':
    case 'jpeg':
    case 'gif':
    case 'svg': return <ImageIcon sx={{ ...sx, color: '#f57c00' }} />;
    default: return <InsertDriveFileIcon sx={sx} />;
  }
};

/** 格式化文件大小 */
const formatSize = (bytes: number): string => {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
};

const HelpPage: React.FC = () => {
  const [docs, setDocs] = useState<HelpDocument[]>([]);
  const [loading, setLoading] = useState(true);
  const theme = useTheme();
  const isMobile = useMediaQuery(theme.breakpoints.down('sm'));

  useEffect(() => {
    setLoading(true);
    getHelpDocuments(true)
      .then((r) => { if (r.code === 0 && r.data) setDocs(r.data); })
      .catch(() => {})
      .finally(() => setLoading(false));
  }, []);

  if (loading) {
    return <Box sx={{ display: 'flex', justifyContent: 'center', mt: 8 }}><CircularProgress /></Box>;
  }

  return (
    <Box sx={{ maxWidth: 1100, mx: 'auto', mt: { xs: 2, md: 3 } }}>
      <Typography variant="h5" fontWeight={700} sx={{ mb: 3, px: 1 }}>
        教程与帮助
      </Typography>

      {docs.length === 0 ? (
        <Box sx={{ textAlign: 'center', py: 8 }}>
          <MenuBookIcon sx={{ fontSize: 64, color: '#ccc', mb: 2 }} />
          <Typography variant="h6" color="text.secondary" gutterBottom>
            暂无帮助文档
          </Typography>
          <Typography variant="body2" color="text.disabled">
            管理员可在管理页面中上传和编辑帮助文档
          </Typography>
        </Box>
      ) : (
        <Grid container spacing={2}>
          {docs.map((doc) => (
            <Grid key={doc.id} item xs={12} sm={6} md={4}>
              <Paper
                elevation={0}
                sx={{
                  p: { xs: 2, md: 2.5 },
                  borderRadius: R,
                  border: '1px solid rgba(0,0,0,0.08)',
                  transition: 'all 0.2s',
                  '&:hover': { boxShadow: '0 4px 24px rgba(0,0,0,0.08)', transform: 'translateY(-2px)' },
                  display: 'flex',
                  flexDirection: 'column',
                  height: '100%',
                }}
              >
                <Box sx={{ display: 'flex', alignItems: 'flex-start', gap: 1.5, mb: 1.5 }}>
                  {fileIcon(doc.file_type)}
                  <Box sx={{ flex: 1, minWidth: 0 }}>
                    <Typography
                      variant="subtitle1"
                      fontWeight={600}
                      sx={{
                        overflow: 'hidden',
                        textOverflow: 'ellipsis',
                        display: '-webkit-box',
                        WebkitLineClamp: 2,
                        WebkitBoxOrient: 'vertical',
                        lineHeight: 1.3,
                      }}
                    >
                      {doc.title}
                    </Typography>
                    <Typography variant="caption" color="text.secondary" sx={{ mt: 0.3, display: 'block' }}>
                      {doc.created_at?.substring(0, 10)} · {formatSize(doc.file_size)}
                    </Typography>
                  </Box>
                </Box>
                <Box sx={{ mt: 'auto', pt: 1 }}>
                  <Box
                    component="button"
                    onClick={() => window.open(getHelpDocumentFileUrl(doc.id), '_blank')}
                    sx={{
                      width: '100%',
                      py: 0.75,
                      px: 2,
                      borderRadius: R,
                      border: '1px solid #667eea',
                      background: 'linear-gradient(135deg,#667eea,#764ba2)',
                      color: '#fff',
                      fontWeight: 600,
                      fontSize: '0.85rem',
                      cursor: 'pointer',
                      transition: 'all 0.2s',
                      '&:hover': { opacity: 0.9, boxShadow: '0 4px 14px rgba(102,126,234,0.35)' },
                    }}
                  >
                    查看文档
                  </Box>
                </Box>
              </Paper>
            </Grid>
          ))}
        </Grid>
      )}
    </Box>
  );
};

export default HelpPage;
