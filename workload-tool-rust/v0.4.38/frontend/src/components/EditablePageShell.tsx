import React, { useState, useEffect, useCallback } from 'react';
import { Box, Button, Snackbar, Alert } from '@mui/material';
import EditIcon from '@mui/icons-material/Edit';
import type { ReactNode } from 'react';
import type { FieldDef } from '../types/layout';
import { getSetting, updateSetting } from '../api/client';
import { useUser } from '../UserContext';
import LayoutDesigner from './LayoutDesigner';

interface EditablePageShellProps {
  pageKey: string; // 'sample-entry', 'home' 等
  children: ReactNode;
  defaultFields: FieldDef[];
  renderField: (
    field: FieldDef,
    isEditMode: boolean,
    onUpdate: (f: FieldDef) => void
  ) => ReactNode;
  onFieldsLoaded?: (fields: FieldDef[]) => void;
}

const R = '2px';

const EditablePageShell: React.FC<EditablePageShellProps> = ({
  pageKey,
  children,
  defaultFields,
  renderField,
  onFieldsLoaded,
}) => {
  const { user, hasPermission } = useUser();
  const isAdmin = user?.is_admin || hasPermission('manage:settings');

  const [fields, setFields] = useState<FieldDef[]>(defaultFields);
  const [editMode, setEditMode] = useState(false);
  const [snackMsg, setSnackMsg] = useState('');
  const [snackErr, setSnackErr] = useState(false);

  // Load layout from server
  useEffect(() => {
    const settingKey = `layout_${pageKey}`;
    getSetting(settingKey)
      .then((res) => {
        if (res.code === 0 && res.data) {
          try {
            const parsed = JSON.parse(res.data.value) as FieldDef[];
            if (Array.isArray(parsed) && parsed.length > 0) {
              setFields(parsed);
              onFieldsLoaded?.(parsed);
            }
          } catch {
            // fallback to default
          }
        }
      })
      .catch(() => {
        // fallback to default
      });
  }, [pageKey, onFieldsLoaded]);

  const handleSave = useCallback(
    async (newFields: FieldDef[]) => {
      try {
        const settingKey = `layout_${pageKey}`;
        await updateSetting(settingKey, newFields);
        setFields(newFields);
        onFieldsLoaded?.(newFields);
        setSnackMsg('布局已发布，3秒后刷新页面...');
        setSnackErr(false);
        setEditMode(false);
        // 关键：保存后强制刷新页面，避免今日记录表格读不到最新 layout
        setTimeout(() => { try { window.location.reload(); } catch {} }, 1500);
      } catch (e: any) {
        const msg = (e && e.response && e.response.data && e.response.data.message) || (e && e.message) || '保存失败';
        setSnackMsg('保存失败: ' + msg);
        setSnackErr(true);
        // 不自动关闭编辑模式，让用户看到错误
      }
    },
    [pageKey, onFieldsLoaded]
  );

  return (
    <>
      {/* 编辑模式开关（仅管理员可见） */}
      {isAdmin && !editMode && (
        <Box sx={{ display: 'flex', justifyContent: 'flex-end', mb: 1 }}>
          <Button
            variant="outlined"
            size="small"
            startIcon={<EditIcon />}
            onClick={() => setEditMode(true)}
            sx={{ borderRadius: R, fontSize: '0.8rem' }}
          >
            ✎ 编辑布局
          </Button>
        </Box>
      )}

      {/* 页面内容 */}
      {children}

      {/* 布局编辑器（全屏覆盖） */}
      {editMode && (
        <LayoutDesigner
          fields={fields}
          onChange={setFields}
          onSave={handleSave}
          onClose={() => setEditMode(false)}
          renderFieldPreview={(field) => renderField(field, true, () => {})}
        />
      )}

      {/* Snackbar */}
      <Snackbar
        open={!!snackMsg}
        autoHideDuration={6000}
        onClose={() => setSnackMsg('')}
        anchorOrigin={{ vertical: 'top', horizontal: 'center' }}
      >
        <Alert
          severity={snackErr ? 'error' : 'success'}
          sx={{ borderRadius: R }}
          onClose={() => setSnackMsg('')}
        >
          {snackMsg}
        </Alert>
      </Snackbar>
    </>
  );
};

export default EditablePageShell;
