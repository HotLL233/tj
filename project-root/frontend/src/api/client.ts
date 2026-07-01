import axios from 'axios';
import type {
  ApiResponse,
  PaginatedResponse,
  ProjectGroup,
  Project,
  Method,
  WorkRecord,
  SampleRecord,
  SampleStats,
  AuditLog,
  StatsSummary,
  UserStats,
  ProjectStats,
  TypeStats,
  InstrumentStats,
  BackupStatus,
  MethodType,
  ImportSummary,
} from '../types';

const client = axios.create({ baseURL: '/api' });

client.interceptors.response.use(
  (res) => res,
  (err) => {
    const msg = err.response?.data?.message || '网络错误';
    return Promise.reject(new Error(msg));
  }
);

// --- Groups ---
export const getGroups = (): Promise<ApiResponse<ProjectGroup[]>> =>
  client.get('/groups').then((r) => r.data);

export const createGroup = (data: { name: string; sort_order?: number }): Promise<ApiResponse<ProjectGroup>> =>
  client.post('/groups', data).then((r) => r.data);

export const updateGroup = (id: number, data: { name?: string; sort_order?: number }): Promise<ApiResponse<ProjectGroup>> =>
  client.put(`/groups/${id}`, data).then((r) => r.data);

export const deleteGroup = (id: number): Promise<ApiResponse<null>> =>
  client.delete(`/groups/${id}`).then((r) => r.data);

// --- Projects (v0.2.17 简化) ---
export const getProjects = (params?: { group_id?: number; active_only?: boolean; method_type?: string }): Promise<ApiResponse<Project[]>> =>
  client.get('/projects', { params }).then((r) => r.data);

export const createProject = (data: {
  name: string;
  notes?: string;
  lab_ids?: number[];
  method_ids?: number[];
}): Promise<ApiResponse<Project>> =>
  client.post('/projects', data).then((r) => r.data);

export const updateProject = (
  id: number,
  data: { name?: string; notes?: string; lab_ids?: number[]; method_ids?: number[] }
): Promise<ApiResponse<Project>> =>
  client.put(`/projects/${id}`, data).then((r) => r.data);

export const deleteProject = (id: number): Promise<ApiResponse<null>> =>
  client.delete(`/projects/${id}`).then((r) => r.data);

export const batchProjectCoefficient = (data: { group_id: number; coefficient: number }): Promise<ApiResponse<number>> =>
  client.put('/projects/batch-coefficient', data).then((r) => r.data);

// --- Methods (v0.2.17 新增) ---
export const getMethods = (params?: { type_id?: number }): Promise<ApiResponse<Method[]>> =>
  client.get('/methods', { params }).then((r) => r.data);

export const createMethod = (data: { name: string; full_name?: string; coefficient?: number; notes?: string; type_ids?: number[] }): Promise<ApiResponse<Method>> =>
  client.post('/methods', data).then((r) => r.data);

export const updateMethod = (id: number, data: { name?: string; full_name?: string; coefficient?: number; notes?: string; is_active?: boolean; type_ids?: number[] }): Promise<ApiResponse<Method>> =>
  client.put(`/methods/${id}`, data).then((r) => r.data);

export const deleteMethod = (id: number): Promise<ApiResponse<null>> =>
  client.delete(`/methods/${id}`).then((r) => r.data);

export const methodImport = (file: File): Promise<ApiResponse<ImportSummary>> => {
  const fd = new FormData();
  fd.append('file', file);
  return client.post('/methods/import', fd, { headers: { 'Content-Type': 'multipart/form-data' } }).then((r) => r.data);
};

// v0.2.8: 方法类型 (路由移到 /api/method-types)
export const getMethodTypes = (): Promise<ApiResponse<MethodType[]>> =>
  client.get('/method-types').then((r) => r.data);

export const createMethodType = (data: { name: string; sort_order?: number }): Promise<ApiResponse<MethodType>> =>
  client.post('/method-types', data).then((r) => r.data);

export const updateMethodType = (id: number, data: { name?: string; sort_order?: number }): Promise<ApiResponse<MethodType>> =>
  client.put(`/method-types/${id}`, data).then((r) => r.data);

export const deleteMethodType = (id: number): Promise<ApiResponse<null>> =>
  client.delete(`/method-types/${id}`).then((r) => r.data);

// --- Records ---
export const getRecords = (params: { start?: string; end?: string; group_id?: number; page?: number; page_size?: number }): Promise<ApiResponse<PaginatedResponse<WorkRecord>>> =>
  client.get('/records', { params }).then((r) => r.data);

export const createRecord = (data: { project_id: number; user_name: string; quantity: number; recorded_at: string }): Promise<ApiResponse<WorkRecord>> =>
  client.post('/records', data).then((r) => r.data);

export const deleteRecord = (id: number): Promise<ApiResponse<null>> =>
  client.delete(`/records/${id}`).then((r) => r.data);

export const restoreRecord = (id: number): Promise<ApiResponse<null>> =>
  client.post(`/records/${id}/restore`).then((r) => r.data);

export const updateRecord = (id: number, data: { user_name?: string; quantity?: number; recorded_at?: string }): Promise<ApiResponse<WorkRecord>> =>
  client.put(`/records/${id}`, data).then((r) => r.data);

export const deleteRecordsByUser = (params: { start: string; end: string; group_id?: number; user_name: string }): Promise<ApiResponse<number>> =>
  client.delete('/records/by-user', { params }).then((r) => r.data);

// --- Stats ---
export const getStatsSummary = (params?: { start?: string; end?: string; group_id?: number }): Promise<ApiResponse<StatsSummary>> =>
  client.get('/stats/summary', { params }).then((r) => r.data);

export const getStatsByUser = (params?: { start?: string; end?: string; group_id?: number }): Promise<ApiResponse<UserStats[]>> =>
  client.get('/stats/by-user', { params }).then((r) => r.data);

export const getStatsByProject = (params?: { start?: string; end?: string; group_id?: number }): Promise<ApiResponse<ProjectStats[]>> =>
  client.get('/stats/by-project', { params }).then((r) => r.data);

export const getStatsByType = (params?: { start?: string; end?: string; group_id?: number }): Promise<ApiResponse<TypeStats[]>> =>
  client.get('/stats/by-type', { params }).then((r) => r.data);

export const getStatsByInstrument = (params?: { start?: string; end?: string; group_id?: number }): Promise<ApiResponse<InstrumentStats[]>> =>
  client.get('/stats/by-instrument', { params }).then((r) => r.data);

// --- Export ---
export const exportExcel = (params: { start?: string; end?: string; group_id?: number }): Promise<Blob> =>
  client.get('/export/excel', { params, responseType: 'blob' }).then((r) => r.data);

// --- Audit ---
export const getAuditLogs = (params?: { page?: number; page_size?: number }): Promise<ApiResponse<PaginatedResponse<AuditLog>>> =>
  client.get('/audit', { params }).then((r) => r.data);

// --- Samples ---
export const getSamples = (params?: { group_id?: number; page?: number; page_size?: number }): Promise<ApiResponse<PaginatedResponse<SampleRecord>>> =>
  client.get('/samples', { params }).then((r) => r.data);

export const getSample = (id: number): Promise<ApiResponse<SampleRecord>> =>
  client.get(`/samples/${id}`).then((r) => r.data);

export const createSample = (data: { group_id: number; sample_name: string; sample_type?: string; quantity: number; user_name: string; notes?: string }): Promise<ApiResponse<SampleRecord>> =>
  client.post('/samples', data).then((r) => r.data);

export const updateSample = (id: number, data: { sample_name?: string; sample_type?: string; quantity?: number; notes?: string }): Promise<ApiResponse<SampleRecord>> =>
  client.put(`/samples/${id}`, data).then((r) => r.data);

export const deleteSample = (id: number): Promise<ApiResponse<null>> =>
  client.delete(`/samples/${id}`).then((r) => r.data);

export const restoreSample = (id: number): Promise<ApiResponse<null>> =>
  client.post(`/samples/${id}/restore`).then((r) => r.data);

export const getSampleStats = (params?: { start?: string; end?: string }): Promise<ApiResponse<SampleStats>> =>
  client.get('/samples/stats', { params }).then((r) => r.data);

// --- Auth ---
export const adminLogin = (data: { username: string; password: string }): Promise<ApiResponse<{ token: string }>> =>
  client.post('/auth/login', data).then((r) => r.data);

// --- Backup ---
export const getBackupStatus = (): Promise<ApiResponse<BackupStatus>> => client.get('/backup/status').then((r) => r.data);
export const backupNow = (): Promise<ApiResponse<string>> => client.post('/backup/now').then((r) => r.data);
export const getBackupConfig = (): Promise<ApiResponse<{ enabled: boolean; interval_hours: number }>> => client.get('/backup/config').then((r) => r.data);
export const updateBackupConfig = (data: { enabled: boolean; interval_hours: number }): Promise<ApiResponse<string>> => client.put('/backup/config', data).then((r) => r.data);
export const deleteBackup = (filename: string): Promise<ApiResponse<string>> => client.delete(`/backup/file/${encodeURIComponent(filename)}`).then((r) => r.data);
export const restoreBackup = (file: File): Promise<ApiResponse<string>> => { const fd = new FormData(); fd.append('file', file); return client.post('/backup/restore', fd, { headers: { 'Content-Type': 'multipart/form-data' } }).then((r) => r.data); };

export default client;
