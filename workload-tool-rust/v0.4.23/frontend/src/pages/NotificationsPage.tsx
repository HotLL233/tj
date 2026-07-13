import React, { useEffect, useState } from 'react';
import { Box, Paper, Typography, List, ListItem, ListItemText, Badge, Button, Chip, Stack, Snackbar, Alert } from '@mui/material';
import DoneAllIcon from '@mui/icons-material/DoneAll';
import { useAuth } from '../context/AuthContext';
import { getNotifications, markNotificationRead, markAllRead } from '../api/client';
import type { NotificationResponse } from '../types/lims';

const NotificationsPage: React.FC = () => {
  const { hasPerm } = useAuth();
  const [list, setList] = useState<NotificationResponse[]>([]);
  const [err, setErr] = useState('');

  const load = () => getNotifications().then((r) => { if (r.code === 0) setList(r.data); }).catch((e) => setErr(e.message));
  useEffect(() => { void load(); }, []); // eslint-disable-line

  const read = async (id: number) => { try { await markNotificationRead(id); load(); } catch (e) { setErr(e instanceof Error ? e.message : '操作失败'); } };
  const readAll = async () => { try { await markAllRead(); load(); } catch (e) { setErr(e instanceof Error ? e.message : '操作失败'); } };

  if (!hasPerm('notification:read')) return <Alert severity="error">无权限访问</Alert>;

  return (
    <Box>
      <Stack direction="row" justifyContent="space-between" alignItems="center" sx={{ mb: 2 }}>
        <Typography variant="h4" fontWeight={700}>通知</Typography>
        <Button variant="outlined" startIcon={<DoneAllIcon />} onClick={readAll}>全部已读</Button>
      </Stack>
      <Paper elevation={1}>
        <List>
          {list.map((n) => (
            <ListItem key={n.id} secondaryAction={!n.is_read ? <Button size="small" onClick={() => read(n.id)}>标记已读</Button> : null} divider>
              <ListItemText
                primary={<><Badge color="error" variant="dot" invisible={n.is_read === 1}><Typography fontWeight={n.is_read ? 400 : 700}>{n.title}</Typography></Badge> <Chip size="small" label={n.module} sx={{ ml: 1 }} /></>}
                secondary={`${n.content || ''}  ·  ${n.created_at}`}
              />
            </ListItem>
          ))}
          {list.length === 0 && <ListItem><ListItemText primary={<Typography color="text.secondary">暂无通知</Typography>} /></ListItem>}
        </List>
      </Paper>
      <Snackbar open={!!err} autoHideDuration={4000} onClose={() => setErr('')}><Alert severity="error" onClose={() => setErr('')}>{err}</Alert></Snackbar>
    </Box>
  );
};

export default NotificationsPage;
