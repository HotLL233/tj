import axios, { type AxiosInstance } from 'axios';
import type {
  ApiResponse,
  PaginatedResponse,
  ProjectGroup,
  Project,
  WorkRecord,
  AuditLog,
  StatsSummary,
  UserStats,
  ProjectStats,
  TypeStats,
  InstrumentStats,
} from '../types';

const client: AxiosInstance = axios.create({
  baseURL: '/api',
  timeout: 10000,
  headers: { 'Content-Type': 'application/json' },
});

// Response interceptor for error handling
client.interceptors.response.use(
  (response) => {
    const data = response.data as ApiResponse;
    if (data.code !== 0) {
      console.warn('API business error:', data.message);
    }
    return response;
  },
  (error) => {
    console.error('Network error:', error);
    return Promise.reject(error);
  }
);

// --- Groups ---
export const getGroups = (): Promise<ApiResponse<ProjectGroup[]>> =>
  client.get('/groups').then((r) => r.data);

export const createGroup = (data: {
  name: string;
  sort_order?: number;
}): Promise<ApiResponse<ProjectGroup>> =>
  client.post('/groups', data).then((r) => r.data);

export const updateGroup = (
  id: number,
  data: { name?: string; sort_order?: number }
): Promise<ApiResponse<ProjectGroup>> =>
  client.put(`/groups/${id}`, data).then((r) => r.data);

export const deleteGroup = (id: number): Promise<ApiResponse<null>> =>
  client.delete(`/groups/${id}`).then((r) => r.data);

// --- Projects ---
export const getProjects = (params?: {
  group_id?: number;
  active_only?: boolean;
}): Promise<ApiResponse<Project[]>> =>
  client.get('/projects', { params }).then((r) => r.data);

export const createProject = (data: {
  group_id: number;
  name: string;
  sort_order?: number;
}): Promise<ApiResponse<Project>> =>
  client.post('/projects', data).then((r) => r.data);

export const updateProject = (
  id: number,
  data: { name?: string; full_name?: string; notes?: string; sort_order?: number; is_active?: number }
): Promise<ApiResponse<Project>> =>
  client.put(`/projects/${id}`, data).then((r) => r.data);

export const deleteProject = (id: number): Promise<ApiResponse<null>> =>
  client.delete(`/projects/${id}`).then((r) => r.data);

// --- Records ---
export const getRecords = (params?: {
  project_id?: number;
  user_name?: string;
  start?: string;
  end?: string;
  page?: number;
  page_size?: number;
  include_deleted?: boolean;
}): Promise<ApiResponse<PaginatedResponse<WorkRecord>>> =>
  client.get('/records', { params }).then((r) => r.data);

export const createRecord = (data: {
  project_id: number;
  user_name: string;
  quantity: number;
  recorded_at: string;
}): Promise<ApiResponse<WorkRecord>> =>
  client.post('/records', data).then((r) => r.data);

export const deleteRecord = (id: number): Promise<ApiResponse<null>> =>
  client.delete(`/records/${id}`).then((r) => r.data);

export const restoreRecord = (id: number): Promise<ApiResponse<WorkRecord>> =>
  client.post(`/records/${id}/restore`).then((r) => r.data);

export const updateRecord = (
  id: number,
  data: { user_name?: string; quantity?: number; recorded_at?: string }
): Promise<ApiResponse<WorkRecord>> =>
  client.put(`/records/${id}`, data).then((r) => r.data);

export const deleteRecordsByUser = (
  user_name: string,
  params?: { start?: string; end?: string }
): Promise<ApiResponse<{ deleted_count: number }>> =>
  client.delete(`/records/by-user/${encodeURIComponent(user_name)}`, { params }).then((r) => r.data);

// --- Stats ---
export const getStatsSummary = (params?: {
  start?: string;
  end?: string;
  group_by?: string;
}): Promise<ApiResponse<StatsSummary>> =>
  client.get('/stats/summary', { params }).then((r) => r.data);

export const getStatsByUser = (params?: {
  start?: string;
  end?: string;
}): Promise<ApiResponse<UserStats[]>> =>
  client.get('/stats/by-user', { params }).then((r) => r.data);

export const getStatsByProject = (params?: {
  start?: string;
  end?: string;
  group_id?: number;
}): Promise<ApiResponse<ProjectStats[]>> =>
  client.get('/stats/by-project', { params }).then((r) => r.data);

export const getStatsByType = (params?: {
  start?: string;
  end?: string;
}): Promise<ApiResponse<TypeStats[]>> =>
  client.get('/stats/by-type', { params }).then((r) => r.data);

export const getStatsByInstrument = (params?: {
  start?: string;
  end?: string;
}): Promise<ApiResponse<InstrumentStats[]>> =>
  client.get('/stats/by-instrument', { params }).then((r) => r.data);

// --- Export ---
export const exportExcel = (params?: {
  start?: string;
  end?: string;
  group_id?: number;
}): Promise<Blob> =>
  client
    .get('/export/excel', { params, responseType: 'blob' })
    .then((r) => r.data);

// --- Audit ---
export const getAuditLogs = (params?: {
  page?: number;
  page_size?: number;
}): Promise<ApiResponse<PaginatedResponse<AuditLog>>> =>
  client.get('/audit-logs', { params }).then((r) => r.data);

export default client;
