import React, { useState, useEffect, useCallback } from 'react';
import {
  Box, Typography, Paper, Button, CircularProgress, Alert, Snackbar,
  FormControl, InputLabel, Select, MenuItem, Chip, IconButton, Tooltip,
} from '@mui/material';
import AddLinkIcon from '@mui/icons-material/AddLink';
import LinkOffIcon from '@mui/icons-material/LinkOff';
import SaveIcon from '@mui/icons-material/Save';
import RefreshIcon from '@mui/icons-material/Refresh';
import {
  DndContext,
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  DragEndEvent,
  DragStartEvent,
  DragOverlay,
} from '@dnd-kit/core';
import {
  arrayMove,
  SortableContext,
  sortableKeyboardCoordinates,
  verticalListSortingStrategy,
} from '@dnd-kit/sortable';
import { useSortable } from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';

import {
  getGroups,
  getProjects,
  getMethods,
  updateProject,
} from '../api/client';
import type { ProjectGroup, Project, Method } from '../types';

const R = '2px';

type AssociationMode = 'lab' | 'method';

interface DraggableItem {
  id: number;
  name: string;
  type: 'project' | 'lab' | 'method';
}

interface AssociationState {
  unassociated: DraggableItem[];
  associated: DraggableItem[];
}

const SortableItem: React.FC<{
  item: DraggableItem;
  onRemove?: (id: number) => void;
}> = ({ item, onRemove }) => {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: item.id });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  };

  return (
    <Paper
      ref={setNodeRef}
      style={style}
      elevation={0}
      sx={{
        p: 1.5,
        mb: 1,
        borderRadius: R,
        border: '1px solid',
        borderColor: isDragging ? '#f4511e' : 'rgba(0,0,0,0.08)',
        backgroundColor: isDragging ? 'rgba(244,81,30,0.04)' : 'white',
        cursor: 'grab',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        transition: 'all 0.2s',
        '&:hover': {
          borderColor: '#f4511e',
          boxShadow: '0 2px 8px rgba(244,81,30,0.12)',
        },
      }}
      {...attributes}
      {...listeners}
    >
      <Typography variant="body2" sx={{ fontWeight: 500 }}>
        {item.name}
      </Typography>
      {onRemove && (
        <IconButton
          size="small"
          onClick={(e) => {
            e.stopPropagation();
            onRemove(item.id);
          }}
          sx={{ ml: 1, color: 'error.main' }}
        >
          <LinkOffIcon fontSize="small" />
        </IconButton>
      )}
    </Paper>
  );
};

const AssociationConfig: React.FC = () => {
  const [mode, setMode] = useState<AssociationMode>('lab');
  const [selectedProjectId, setSelectedProjectId] = useState<number | ''>('');
  const [projects, setProjects] = useState<Project[]>([]);
  const [groups, setGroups] = useState<ProjectGroup[]>([]);
  const [methods, setMethods] = useState<Method[]>([]);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState('');
  const [isError, setIsError] = useState(false);
  const [activeId, setActiveId] = useState<number | null>(null);

  // Association state
  const [state, setState] = useState<AssociationState>({
    unassociated: [],
    associated: [],
  });

  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: {
        distance: 8,
      },
    }),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    })
  );

  const showMessage = useCallback((msg: string, isErr?: boolean) => {
    setMessage(msg);
    setIsError(!!isErr);
  }, []);

  const loadData = useCallback(async () => {
    setLoading(true);
    try {
      const [projectsRes, groupsRes, methodsRes] = await Promise.all([
        getProjects(),
        getGroups(),
        getMethods(),
      ]);

      if (projectsRes.code === 0 && projectsRes.data) {
        setProjects(projectsRes.data);
      }
      if (groupsRes.code === 0 && groupsRes.data) {
        setGroups(groupsRes.data.filter((g: ProjectGroup) => g.name !== '研发项目'));
      }
      if (methodsRes.code === 0 && methodsRes.data) {
        setMethods(methodsRes.data);
      }
    } catch (error) {
      showMessage('加载数据失败', true);
    } finally {
      setLoading(false);
    }
  }, [showMessage]);

  useEffect(() => {
    loadData();
  }, [loadData]);

  // Update association state when selected project or mode changes
  useEffect(() => {
    if (!selectedProjectId) {
      setState({ unassociated: [], associated: [] });
      return;
    }

    const project = projects.find((p) => p.id === selectedProjectId);
    if (!project) return;

    if (mode === 'lab') {
      const associatedLabIds = project.lab_ids || [];
      const associated = groups
        .filter((g) => associatedLabIds.includes(g.id))
        .map((g) => ({ id: g.id, name: g.name, type: 'lab' as const }));
      const unassociated = groups
        .filter((g) => !associatedLabIds.includes(g.id))
        .map((g) => ({ id: g.id, name: g.name, type: 'lab' as const }));

      setState({ unassociated, associated });
    } else {
      const associatedMethodIds = project.method_ids || [];
      const associated = methods
        .filter((m) => associatedMethodIds.includes(m.id))
        .map((m) => ({ id: m.id, name: m.name, type: 'method' as const }));
      const unassociated = methods
        .filter((m) => !associatedMethodIds.includes(m.id))
        .map((m) => ({ id: m.id, name: m.name, type: 'method' as const }));

      setState({ unassociated, associated });
    }
  }, [selectedProjectId, mode, projects, groups, methods]);

  const handleDragStart = (event: DragStartEvent) => {
    setActiveId(event.active.id as number);
  };

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;
    setActiveId(null);

    if (!over) return;

    const activeId = active.id as number;
    const overId = over.id as string;

    // Determine which list the item is moving to
    const isMovingToAssociated = overId === 'associated-list';
    const isMovingToUnassociated = overId === 'unassociated-list';

    if (isMovingToAssociated) {
      // Move from unassociated to associated
      const item = state.unassociated.find((i) => i.id === activeId);
      if (item) {
        setState({
          unassociated: state.unassociated.filter((i) => i.id !== activeId),
          associated: [...state.associated, item],
        });
      }
    } else if (isMovingToUnassociated) {
      // Move from associated to unassociated
      const item = state.associated.find((i) => i.id === activeId);
      if (item) {
        setState({
          unassociated: [...state.unassociated, item],
          associated: state.associated.filter((i) => i.id !== activeId),
        });
      }
    }
  };

  const handleSave = async () => {
    if (!selectedProjectId) {
      showMessage('请先选择项目', true);
      return;
    }

    setSaving(true);
    try {
      const project = projects.find((p) => p.id === selectedProjectId);
      if (!project) return;

      const updateData: any = {
        name: project.name,
        full_name: project.full_name,
        notes: project.notes,
        sort_order: project.sort_order,
        is_active: project.is_active,
      };

      if (mode === 'lab') {
        updateData.lab_ids = state.associated.map((item) => item.id);
        updateData.method_ids = project.method_ids;
      } else {
        updateData.lab_ids = project.lab_ids;
        updateData.method_ids = state.associated.map((item) => item.id);
      }

      const result = await updateProject(selectedProjectId as number, updateData);
      if (result.code === 0) {
        showMessage('保存成功');
        // Reload projects to get updated data
        const projectsRes = await getProjects();
        if (projectsRes.code === 0 && projectsRes.data) {
          setProjects(projectsRes.data);
        }
      } else {
        showMessage(result.message || '保存失败', true);
      }
    } catch (error) {
      showMessage('保存失败', true);
    } finally {
      setSaving(false);
    }
  };

  const handleAddAll = () => {
    setState({
      unassociated: [],
      associated: [...state.associated, ...state.unassociated],
    });
  };

  const handleRemoveAll = () => {
    setState({
      unassociated: [...state.unassociated, ...state.associated],
      associated: [],
    });
  };

  const activeItem = state.unassociated.find((i) => i.id === activeId) ||
    state.associated.find((i) => i.id === activeId);

  if (loading) {
    return (
      <Box sx={{ display: 'flex', justifyContent: 'center', mt: 8 }}>
        <CircularProgress />
      </Box>
    );
  }

  return (
    <Box>
      <Typography variant="h6" fontWeight={700} sx={{ mb: 2 }}>
        拖拽式关联配置
      </Typography>
      <Typography variant="body2" color="text.secondary" sx={{ mb: 3 }}>
        通过拖拽方式配置项目与实验室或检测方法的关联关系
      </Typography>

      {message && (
        <Alert
          severity={isError ? 'error' : 'success'}
          sx={{ mb: 2, borderRadius: R }}
          onClose={() => setMessage('')}
        >
          {message}
        </Alert>
      )}

      {/* Controls */}
      <Paper elevation={0} sx={{ p: 2, mb: 3, borderRadius: R, border: '1px solid rgba(0,0,0,0.06)' }}>
        <Box sx={{ display: 'flex', gap: 2, flexWrap: 'wrap', alignItems: 'center' }}>
          <FormControl size="small" sx={{ minWidth: 200 }}>
            <InputLabel>关联模式</InputLabel>
            <Select
              value={mode}
              label="关联模式"
              onChange={(e) => setMode(e.target.value as AssociationMode)}
              sx={{ borderRadius: R }}
            >
              <MenuItem value="lab">项目 - 实验室关联</MenuItem>
              <MenuItem value="method">项目 - 检测方法关联</MenuItem>
            </Select>
          </FormControl>

          <FormControl size="small" sx={{ minWidth: 300 }}>
            <InputLabel>选择项目</InputLabel>
            <Select
              value={selectedProjectId}
              label="选择项目"
              onChange={(e) => setSelectedProjectId(e.target.value as number)}
              sx={{ borderRadius: R }}
            >
              {projects
                .filter((p) => p.is_active !== false)
                .map((p) => (
                  <MenuItem key={p.id} value={p.id}>
                    {p.name}
                  </MenuItem>
                ))}
            </Select>
          </FormControl>

          <Tooltip title="刷新数据">
            <IconButton onClick={loadData} sx={{ color: '#f4511e' }}>
              <RefreshIcon />
            </IconButton>
          </Tooltip>
        </Box>
      </Paper>

      {!selectedProjectId ? (
        <Paper
          elevation={0}
          sx={{
            p: 4,
            borderRadius: R,
            border: '2px dashed rgba(0,0,0,0.12)',
            textAlign: 'center',
          }}
        >
          <Typography color="text.secondary">请先选择一个项目以配置关联</Typography>
        </Paper>
      ) : (
        <DndContext
          sensors={sensors}
          collisionDetection={closestCenter}
          onDragStart={handleDragStart}
          onDragEnd={handleDragEnd}
        >
          <Box sx={{ display: 'grid', gridTemplateColumns: '1fr auto 1fr', gap: 2, alignItems: 'start' }}>
            {/* Unassociated List */}
            <Paper
              elevation={0}
              sx={{
                p: 2,
                borderRadius: R,
                border: '1px solid rgba(0,0,0,0.08)',
                minHeight: 400,
              }}
            >
              <Typography variant="subtitle2" fontWeight={700} sx={{ mb: 2, color: 'text.secondary' }}>
                未关联 ({state.unassociated.length})
              </Typography>
              <SortableContext
                id="unassociated-list"
                items={state.unassociated.map((i) => i.id)}
                strategy={verticalListSortingStrategy}
              >
                <Box
                  id="unassociated-list"
                  sx={{
                    minHeight: 300,
                    borderRadius: R,
                    border: '2px dashed rgba(0,0,0,0.08)',
                    p: 1,
                    backgroundColor: 'rgba(0,0,0,0.01)',
                  }}
                >
                  {state.unassociated.length === 0 ? (
                    <Typography variant="caption" color="text.secondary" sx={{ display: 'block', textAlign: 'center', mt: 4 }}>
                      全部已关联
                    </Typography>
                  ) : (
                    state.unassociated.map((item) => (
                      <SortableItem key={item.id} item={item} />
                    ))
                  )}
                </Box>
              </SortableContext>
            </Paper>

            {/* Action Buttons */}
            <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1, pt: 8 }}>
              <Tooltip title="全部关联">
                <Button
                  variant="contained"
                  size="small"
                  onClick={handleAddAll}
                  disabled={state.unassociated.length === 0}
                  sx={{
                    borderRadius: R,
                    minWidth: 'auto',
                    width: 40,
                    height: 40,
                    p: 0,
                    background: 'linear-gradient(135deg,#f4511e,#e53935)',
                  }}
                >
                  <AddLinkIcon />
                </Button>
              </Tooltip>
              <Tooltip title="全部取消关联">
                <Button
                  variant="outlined"
                  size="small"
                  onClick={handleRemoveAll}
                  disabled={state.associated.length === 0}
                  sx={{
                    borderRadius: R,
                    minWidth: 'auto',
                    width: 40,
                    height: 40,
                    p: 0,
                    borderColor: '#f4511e',
                    color: '#f4511e',
                  }}
                >
                  <LinkOffIcon />
                </Button>
              </Tooltip>
            </Box>

            {/* Associated List */}
            <Paper
              elevation={0}
              sx={{
                p: 2,
                borderRadius: R,
                border: '1px solid rgba(0,0,0,0.08)',
                minHeight: 400,
              }}
            >
              <Typography variant="subtitle2" fontWeight={700} sx={{ mb: 2, color: 'success.main' }}>
                已关联 ({state.associated.length})
              </Typography>
              <SortableContext
                id="associated-list"
                items={state.associated.map((i) => i.id)}
                strategy={verticalListSortingStrategy}
              >
                <Box
                  id="associated-list"
                  sx={{
                    minHeight: 300,
                    borderRadius: R,
                    border: '2px dashed rgba(76,175,80,0.3)',
                    p: 1,
                    backgroundColor: 'rgba(76,175,80,0.02)',
                  }}
                >
                  {state.associated.length === 0 ? (
                    <Typography variant="caption" color="text.secondary" sx={{ display: 'block', textAlign: 'center', mt: 4 }}>
                      拖拽项目到此处进行关联
                    </Typography>
                  ) : (
                    state.associated.map((item) => (
                      <SortableItem
                        key={item.id}
                        item={item}
                        onRemove={(id) => {
                          const itemToRemove = state.associated.find((i) => i.id === id);
                          if (itemToRemove) {
                            setState({
                              unassociated: [...state.unassociated, itemToRemove],
                              associated: state.associated.filter((i) => i.id !== id),
                            });
                          }
                        }}
                      />
                    ))
                  )}
                </Box>
              </SortableContext>
            </Paper>
          </Box>

          <DragOverlay>
            {activeItem ? (
              <Paper
                elevation={4}
                sx={{
                  p: 1.5,
                  borderRadius: R,
                  border: '2px solid #f4511e',
                  backgroundColor: 'white',
                  opacity: 0.9,
                }}
              >
                <Typography variant="body2" sx={{ fontWeight: 500 }}>
                  {activeItem.name}
                </Typography>
              </Paper>
            ) : null}
          </DragOverlay>
        </DndContext>
      )}

      {/* Save Button */}
      {selectedProjectId && (
        <Box sx={{ display: 'flex', justifyContent: 'flex-end', mt: 3, gap: 1 }}>
          <Button
            variant="outlined"
            onClick={() => {
              // Reset to original state
              const project = projects.find((p) => p.id === selectedProjectId);
              if (!project) return;

              if (mode === 'lab') {
                const associatedLabIds = project.lab_ids || [];
                const associated = groups
                  .filter((g) => associatedLabIds.includes(g.id))
                  .map((g) => ({ id: g.id, name: g.name, type: 'lab' as const }));
                const unassociated = groups
                  .filter((g) => !associatedLabIds.includes(g.id))
                  .map((g) => ({ id: g.id, name: g.name, type: 'lab' as const }));
                setState({ unassociated, associated });
              } else {
                const associatedMethodIds = project.method_ids || [];
                const associated = methods
                  .filter((m) => associatedMethodIds.includes(m.id))
                  .map((m) => ({ id: m.id, name: m.name, type: 'method' as const }));
                const unassociated = methods
                  .filter((m) => !associatedMethodIds.includes(m.id))
                  .map((m) => ({ id: m.id, name: m.name, type: 'method' as const }));
                setState({ unassociated, associated });
              }
              showMessage('已重置');
            }}
            sx={{ borderRadius: R }}
          >
            重置
          </Button>
          <Button
            variant="contained"
            startIcon={saving ? <CircularProgress size={16} /> : <SaveIcon />}
            onClick={handleSave}
            disabled={saving}
            sx={{
              borderRadius: R,
              background: 'linear-gradient(135deg,#f4511e,#e53935)',
              boxShadow: '0 4px 14px rgba(244,81,30,0.3)',
            }}
          >
            {saving ? '保存中...' : '保存关联'}
          </Button>
        </Box>
      )}
    </Box>
  );
};

export default AssociationConfig;
