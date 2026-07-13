import React, { useEffect, useState } from 'react';
import { Box, Typography, Paper, CircularProgress, useMediaQuery, useTheme, Grid, Dialog, DialogTitle, DialogContent, Button, IconButton, List, ListItemButton, ListItemText } from '@mui/material';
import PictureAsPdfIcon from '@mui/icons-material/PictureAsPdf';
import DescriptionIcon from '@mui/icons-material/Description';
import TableChartIcon from '@mui/icons-material/TableChart';
import ImageIcon from '@mui/icons-material/Image';
import ArticleIcon from '@mui/icons-material/Article';
import InsertDriveFileIcon from '@mui/icons-material/InsertDriveFile';
import MenuBookIcon from '@mui/icons-material/MenuBook';
import CloseIcon from '@mui/icons-material/Close';
import ArrowBackIosIcon from '@mui/icons-material/ArrowBackIos';
import ArrowForwardIosIcon from '@mui/icons-material/ArrowForwardIos';
import { getHelpDocuments, getHelpArticles, getHelpDocumentPageUrl, getHelpDocumentFileUrl } from '../api/client';
import type { HelpDocument, HelpArticle, TocItem } from '../types';

const R = '2px';

const fileIcon = (ft: string, isArticle?: boolean) => {
  const sx = { fontSize: 40, color: 'text.secondary' };
  if (isArticle) return <ArticleIcon sx={{ ...sx, color: '#7c4dff' }} />;
  switch (ft.toLowerCase()) {
    case 'pdf': return <PictureAsPdfIcon sx={{ ...sx, color: '#e53935' }} />;
    case 'doc':
    case 'docx': return <DescriptionIcon sx={{ ...sx, color: '#1976d2' }} />;
    case 'xls':
    case 'xlsx': return <TableChartIcon sx={{ ...sx, color: '#388e3c' }} />;
    case 'png': case 'jpg': case 'jpeg': case 'gif': case 'svg': return <ImageIcon sx={{ ...sx, color: '#f57c00' }} />;
    default: return <InsertDriveFileIcon sx={sx} />;
  }
};

const formatSize = (bytes: number): string => {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
};

const HelpPage: React.FC = () => {
  const [docs, setDocs] = useState<HelpDocument[]>([]);
  const [articles, setArticles] = useState<HelpArticle[]>([]);
  const [loading, setLoading] = useState(true);
  const [viewing, setViewing] = useState<{ type: 'article'; data: HelpArticle; toc: TocItem[] } | { type: 'doc'; data: HelpDocument } | null>(null);
  const [pageNum, setPageNum] = useState(1);
  const theme = useTheme();
  const isMobile = useMediaQuery(theme.breakpoints.down('sm'));

  useEffect(() => {
    setLoading(true);
    Promise.all([
      getHelpDocuments(true).then(r => (r.code === 0 && r.data) ? r.data : []).catch(() => []),
      getHelpArticles(true).then(r => (r.code === 0 && r.data) ? r.data : []).catch(() => []),
    ]).then(([d, a]) => { setDocs(d); setArticles(a); }).finally(() => setLoading(false));
  }, []);

  const handleOpenArticle = (a: HelpArticle) => {
    const toc: TocItem[] = a.toc_json ? JSON.parse(a.toc_json) : [];
    setViewing({ type: 'article', data: a, toc });
  };

  const handleOpenDoc = (d: HelpDocument) => { setViewing({ type: 'doc', data: d }); setPageNum(1); };
  const handleClose = () => setViewing(null);
  const totalPages = (viewing?.type === 'doc' ? viewing.data?.page_count : 0) ?? 1;

  if (loading) return <Box sx={{ display: 'flex', justifyContent: 'center', mt: 8 }}><CircularProgress /></Box>;

  return (
    <Box sx={{ maxWidth: 1100, mx: 'auto', mt: { xs: 2, md: 3 } }}>
      <Typography variant="h5" fontWeight={700} sx={{ mb: 3, px: 1 }}>教程与帮助</Typography>

      {docs.length === 0 && articles.length === 0 ? (
        <Box sx={{ textAlign: 'center', py: 8 }}>
          <MenuBookIcon sx={{ fontSize: 64, color: '#ccc', mb: 2 }} />
          <Typography variant="h6" color="text.secondary" gutterBottom>暂无帮助内容</Typography>
        </Box>
      ) : (
        <Grid container spacing={2}>
          {articles.map(a => (
            <Grid key={`a_${a.id}`} item xs={12} sm={6} md={4}>
              <Paper elevation={0} sx={{ p: 2.5, borderRadius: R, border: '1px solid rgba(124,77,255,0.2)', transition: 'all 0.2s', '&:hover': { boxShadow: '0 4px 24px rgba(124,77,255,0.12)', transform: 'translateY(-2px)' }, display: 'flex', flexDirection: 'column', height: '100%', borderLeft: '3px solid #7c4dff' }}>
                <Box sx={{ display: 'flex', alignItems: 'flex-start', gap: 1.5, mb: 1.5 }}>
                  {fileIcon('', true)}
                  <Box sx={{ flex: 1, minWidth: 0 }}>
                    <Typography variant="subtitle1" fontWeight={600} sx={{ overflow: 'hidden', textOverflow: 'ellipsis', display: '-webkit-box', WebkitLineClamp: 2, WebkitBoxOrient: 'vertical', lineHeight: 1.3 }}>{a.title}</Typography>
                    <Typography variant="caption" color="text.secondary" sx={{ mt: 0.3, display: 'block' }}>{a.created_at?.substring(0, 10)} · 文章</Typography>
                  </Box>
                </Box>
                <Box sx={{ mt: 'auto', pt: 1 }}>
                  <Box component="button" onClick={() => handleOpenArticle(a)} sx={{ width: '100%', py: 0.75, px: 2, borderRadius: R, border: '1px solid #7c4dff', background: 'linear-gradient(135deg,#7c4dff,#651fff)', color: '#fff', fontWeight: 600, fontSize: '0.85rem', cursor: 'pointer', transition: 'all 0.2s', '&:hover': { opacity: 0.9 } }}>阅读文章</Box>
                </Box>
              </Paper>
            </Grid>
          ))}
          {docs.map(doc => (
            <Grid key={`d_${doc.id}`} item xs={12} sm={6} md={4}>
              <Paper elevation={0} sx={{ p: 2.5, borderRadius: R, border: '1px solid rgba(0,0,0,0.08)', transition: 'all 0.2s', '&:hover': { boxShadow: '0 4px 24px rgba(0,0,0,0.08)', transform: 'translateY(-2px)' }, display: 'flex', flexDirection: 'column', height: '100%' }}>
                <Box sx={{ display: 'flex', alignItems: 'flex-start', gap: 1.5, mb: 1.5 }}>
                  {fileIcon(doc.file_type)}
                  <Box sx={{ flex: 1, minWidth: 0 }}>
                    <Typography variant="subtitle1" fontWeight={600} sx={{ overflow: 'hidden', textOverflow: 'ellipsis', display: '-webkit-box', WebkitLineClamp: 2, WebkitBoxOrient: 'vertical', lineHeight: 1.3 }}>{doc.title}</Typography>
                    <Typography variant="caption" color="text.secondary" sx={{ mt: 0.3, display: 'block' }}>{doc.created_at?.substring(0, 10)} · {formatSize(doc.file_size)}</Typography>
                  </Box>
                </Box>
                <Box sx={{ mt: 'auto', pt: 1 }}>
                  <Box component="button" onClick={() => handleOpenDoc(doc)} sx={{ width: '100%', py: 0.75, px: 2, borderRadius: R, border: '1px solid #667eea', background: 'linear-gradient(135deg,#667eea,#764ba2)', color: '#fff', fontWeight: 600, fontSize: '0.85rem', cursor: 'pointer', transition: 'all 0.2s', '&:hover': { opacity: 0.9, boxShadow: '0 4px 14px rgba(102,126,234,0.35)' } }}>查看文档</Box>
                </Box>
              </Paper>
            </Grid>
          ))}
        </Grid>
      )}

      {/* 文章查看弹窗 — 左右分栏 */}
      {viewing?.type === 'article' && (
        <Dialog open fullWidth maxWidth="xl" onClose={handleClose} PaperProps={{ sx: { borderRadius: R, height: '90vh' } }}>
          <DialogTitle sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', pr: 1, borderBottom: '1px solid #eee' }}>
            <Typography variant="h6" fontWeight={600} noWrap sx={{ flex: 1, mr: 2 }}>{viewing.data.title}</Typography>
            <IconButton size="small" onClick={handleClose}><CloseIcon /></IconButton>
          </DialogTitle>
          <DialogContent sx={{ p: 0, display: 'flex', overflow: 'hidden' }}>
            {/* 左侧目录 */}
            <Box sx={{ width: 240, borderRight: '1px solid #eee', overflow: 'auto', flexShrink: 0, bgcolor: '#fafafa', display: { xs: 'none', md: 'block' } }}>
              <List dense>
                {viewing.toc.map(item => (
                  <ListItemButton key={item.id} sx={{ pl: 1 + item.level * 2, py: 0.5 }} onClick={() => document.getElementById(item.id)?.scrollIntoView({ behavior: 'smooth' })}>
                    <ListItemText primary={item.text} primaryTypographyProps={{ fontSize: item.level < 3 ? '0.85rem' : '0.78rem', fontWeight: item.level === 1 ? 600 : 400, noWrap: true }} />
                  </ListItemButton>
                ))}
              </List>
              {viewing.toc.length === 0 && <Typography variant="body2" color="text.secondary" sx={{ p: 2 }}>无目录</Typography>}
            </Box>
            {/* 右侧内容 */}
            <Box sx={{ flex: 1, overflow: 'auto', p: { xs: 2, md: 4 } }}>
              <style>{`#help-content h1{font-size:1.5em;margin:0.5em 0}#help-content h2{font-size:1.25em;margin:0.5em 0}#help-content h3{font-size:1.1em;margin:0.4em 0}#help-content h4{font-size:1em;margin:0.3em 0}#help-content p{line-height:1.8;margin:0.5em 0}#help-content b{color:#333}`}</style>
              <div id="help-content" dangerouslySetInnerHTML={{ __html: viewing.data.content_html }} />
            </Box>
          </DialogContent>
        </Dialog>
      )}

      {/* 文档查看弹窗 */}
      {viewing?.type === 'doc' && (
        <Dialog open fullWidth maxWidth="lg" onClose={handleClose} PaperProps={{ sx: { borderRadius: R, height: '90vh' } }}>
          <DialogTitle sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', pr: 1 }}>
            <Typography variant="h6" fontWeight={600} noWrap sx={{ flex: 1, mr: 2 }}>{viewing.data.title}</Typography>
            <IconButton size="small" onClick={handleClose}><CloseIcon /></IconButton>
          </DialogTitle>
          <DialogContent sx={{ p: 0, flex: 1, overflow: 'auto', display: 'flex', flexDirection: 'column', alignItems: 'center', bgcolor: '#f5f5f5' }}>
            {viewing.data.page_count > 0 ? (<>
              <Box sx={{ flex: 1, display: 'flex', alignItems: 'flex-start', justifyContent: 'center', width: '100%' }}>
                <img src={getHelpDocumentPageUrl(viewing.data.id, pageNum)} alt={`第 ${pageNum} 页`} style={{ maxWidth: '100%', height: 'auto' }} />
              </Box>
              <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, py: 1.5 }}>
                <IconButton size="small" disabled={pageNum <= 1} onClick={() => setPageNum(p => p - 1)}><ArrowBackIosIcon fontSize="small" /></IconButton>
                <Typography variant="body2" color="text.secondary">{pageNum} / {totalPages}</Typography>
                <IconButton size="small" disabled={pageNum >= totalPages} onClick={() => setPageNum(p => p + 1)}><ArrowForwardIosIcon fontSize="small" /></IconButton>
              </Box>
            </>) : (
              <Box sx={{ p: 4, textAlign: 'center', width: '100%', bgcolor: '#fff' }}>
                <Typography color="text.secondary">不支持在线预览（{viewing.data.file_type.toUpperCase()}）</Typography>
                <Button variant="outlined" href={getHelpDocumentFileUrl(viewing.data.id)} target="_blank" sx={{ mt: 2, borderRadius: R }}>下载查看</Button>
              </Box>
            )}
          </DialogContent>
        </Dialog>
      )}
    </Box>
  );
};

export default HelpPage;
