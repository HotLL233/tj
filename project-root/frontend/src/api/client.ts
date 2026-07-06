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
  ImportMapping,
  Sheet1Data,
  Sheet2Row,
  Sheet3Row,
  Sheet4Row,
  Sheet5Row,
  Sheet6Row,
  Sheet7Row,
  Sheet8Row,
  Sheet9Row,
  Sheet10Row,
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
  data: { name?: string; full_name?: string; notes?: string; sort_order?: number; is_active?: boolean; lab_ids?: number[]; method_ids?: number[] }
): Promise<ApiResponse<Project>> =>
  client.put(`/projects/${id}`, data).then((r) => r.data);

export const deleteProject = (id: number): Promise<ApiResponse<null>> =>
  client.delete(`/projects/${id}`).then((r) => r.data);

export const batchProjectCoefficient = (data: { group_id: number; coefficient: number }): Promise<ApiResponse<number>> =>
  client.put('/projects/batch-coefficient', data).then((r) => r.data);

// --- Methods (v0.2.17 新增) ---
export const getMethods = (params?: { type_id?: number }): Promise<ApiResponse<Method[]>> =>
  client.get('/methods', { params }).then((r) => r.data);

export const createMethod = (data: { name: string; full_name?: string; coefficient?: number; amount?: number; notes?: string; type_ids?: number[] }): Promise<ApiResponse<Method>> =>
  client.post('/methods', data).then((r) => r.data);

export const updateMethod = (id: number, data: { name?: string; full_name?: string; coefficient?: number; amount?: number; notes?: string; is_active?: boolean; type_ids?: number[] }): Promise<ApiResponse<Method>> =>
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

// v0.3.0: 导入映射配置
export const getImportMappings = (): Promise<ApiResponse<ImportMapping[]>> =>
  client.get('/import/mappings').then(r => r.data);

// --- Records ---
export const getRecords = (params: { start?: string; end?: string; group_id?: number; page?: number; page_size?: number; include_deleted?: boolean }): Promise<ApiResponse<PaginatedResponse<WorkRecord>>> =>
  client.get('/records', { params }).then((r) => r.data);

export const createRecord = (data: { project_id: number; method_id?: number; user_name: string; quantity: number; recorded_at: string; group_id?: number }): Promise<ApiResponse<WorkRecord>> =>
  client.post('/records', data).then((r) => r.data);

export const deleteRecord = (id: number): Promise<ApiResponse<null>> =>
  client.delete(`/records/${id}`).then((r) => r.data);

export const restoreRecord = (id: number): Promise<ApiResponse<WorkRecord>> =>
  client.post(`/records/restore/${id}`).then((r) => r.data);

export const updateRecord = (id: number, data: { user_name?: string; quantity?: number; recorded_at?: string }): Promise<ApiResponse<WorkRecord>> =>
  client.put(`/records/${id}`, data).then((r) => r.data);

export const deleteRecordsByUser = (user_name: string, params: { start: string; end: string; group_id?: number }): Promise<ApiResponse<number>> =>
  client.delete('/records/by-user', { params: { ...params, user_name } }).then((r) => r.data);

// --- Stats ---
export const getStatsSummary = (params?: { start?: string; end?: string; group_id?: number; group_by?: string }): Promise<ApiResponse<StatsSummary>> =>
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
  client.get('/export/excel', { params, responseType: 'blob' })
    .then(async (r) => {
      // 检查 HTTP 状态码，非 200 说明是错误响应
      if (r.status !== 200) {
        const text = await r.data.text();
        try {
          const json = JSON.parse(text);
          throw new Error(json.message || '导出失败');
        } catch (e) {
          if (e instanceof Error) throw e;
          throw new Error(text || '导出失败');
        }
      }
      return r.data;
    });

// --- Audit ---
export const getAuditLogs = (params?: { page?: number; page_size?: number }): Promise<ApiResponse<PaginatedResponse<AuditLog>>> =>
  client.get('/audit-logs', { params }).then((r) => r.data);

// --- Samples ---
export const getSamples = (params?: { group_id?: number; user_name?: string; page?: number; page_size?: number }): Promise<ApiResponse<PaginatedResponse<SampleRecord>>> =>
  client.get('/samples', { params }).then((r) => r.data);

export const getSample = (id: number): Promise<ApiResponse<SampleRecord>> =>
  client.get(`/samples/${id}`).then((r) => r.data);

export const createSample = (data: { project_id: number; user_name: string; sample_name: string; sample_count: number; submitted_at: string; unit?: string; batch_no?: string; notes?: string }): Promise<ApiResponse<SampleRecord>> =>
  client.post('/samples', data).then((r) => r.data);

export const updateSample = (id: number, data: { sample_name?: string; sample_count?: number; unit?: string; batch_no?: string; notes?: string; submitted_at?: string }): Promise<ApiResponse<SampleRecord>> =>
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
export const updateBackupConfig = (data: { enabled: boolean; interval_hours: number; max_backup_count?: number }): Promise<ApiResponse<string>> => client.put('/backup/config', data).then((r) => r.data);
export const deleteBackup = (filename: string): Promise<ApiResponse<string>> => client.delete(`/backup/file/${encodeURIComponent(filename)}`).then((r) => r.data);
export const restoreBackup = (file: File): Promise<ApiResponse<string>> => { const fd = new FormData(); fd.append('file', file); return client.post('/backup/restore', fd, { headers: { 'Content-Type': 'multipart/form-data' } }).then((r) => r.data); };
export const restoreBackupFile = (filename: string): Promise<ApiResponse<string>> => client.post(`/backup/restore/${encodeURIComponent(filename)}`).then((r) => r.data);

// ========== v0.3.7: 导出预览 API ==========

// Sheet 1 需要 group_id（可选）
export const getPreviewSheet1 = (params: { start: string; end: string; group_id?: number }): Promise<ApiResponse<Sheet1Data>> =>
  client.get('/export/preview/sheet1', { params }).then((r) => r.data);

export const getPreviewSheet2 = (params: { start: string; end: string }): Promise<ApiResponse<Sheet2Row[]>> =>
  client.get('/export/preview/sheet2', { params }).then((r) => r.data);

export const getPreviewSheet3 = (params: { start: string; end: string }): Promise<ApiResponse<Sheet3Row[]>> =>
  client.get('/export/preview/sheet3', { params }).then((r) => r.data);

export const getPreviewSheet4 = (params: { start: string; end: string }): Promise<ApiResponse<Sheet4Row[]>> =>
  client.get('/export/preview/sheet4', { params }).then((r) => r.data);

export const getPreviewSheet5 = (params: { start: string; end: string }): Promise<ApiResponse<Sheet5Row[]>> =>
  client.get('/export/preview/sheet5', { params }).then((r) => r.data);

export const getPreviewSheet6 = (params: { start: string; end: string }): Promise<ApiResponse<Sheet6Row[]>> =>
  client.get('/export/preview/sheet6', { params }).then((r) => r.data);

export const getPreviewSheet7 = (params: { start: string; end: string }): Promise<ApiResponse<Sheet7Row[]>> =>
  client.get('/export/preview/sheet7', { params }).then((r) => r.data);

export const getPreviewSheet8 = (params: { start: string; end: string }): Promise<ApiResponse<Sheet8Row[]>> =>
  client.get('/export/preview/sheet8', { params }).then((r) => r.data);

export const getPreviewSheet9 = (params: { start: string; end: string }): Promise<ApiResponse<Sheet9Row[]>> =>
  client.get('/export/preview/sheet9', { params }).then((r) => r.data);

export const getPreviewSheet10 = (params: { start: string; end: string }): Promise<ApiResponse<Sheet10Row[]>> =>
  client.get('/export/preview/sheet10', { params }).then((r) => r.data);

// ========== 研发送样 (rd) — 与分析检测完全独立存储，共用主数据 ==========

// --- RD Records ---
export const getRdRecords = (params: { start?: string; end?: string; group_id?: number; page?: number; page_size?: number; include_deleted?: boolean }): Promise<ApiResponse<PaginatedResponse<WorkRecord>>> =>
  client.get('/rd-records', { params }).then((r) => r.data);

export const createRdRecord = (data: { project_id: number; method_id?: number; user_name: string; quantity: number; recorded_at: string; group_id?: number }): Promise<ApiResponse<WorkRecord>> =>
  client.post('/rd-records', data).then((r) => r.data);

export const deleteRdRecord = (id: number): Promise<ApiResponse<null>> =>
  client.delete(`/rd-records/${id}`).then((r) => r.data);

export const restoreRdRecord = (id: number): Promise<ApiResponse<WorkRecord>> =>
  client.post(`/rd-records/restore/${id}`).then((r) => r.data);

export const updateRdRecord = (id: number, data: { user_name?: string; quantity?: number; recorded_at?: string }): Promise<ApiResponse<WorkRecord>> =>
  client.put(`/rd-records/${id}`, data).then((r) => r.data);

export const deleteRdRecordsByUser = (user_name: string, params: { start: string; end: string; group_id?: number }): Promise<ApiResponse<number>> =>
  client.delete('/rd-records/by-user', { params: { ...params, user_name } }).then((r) => r.data);

// --- RD Stats ---
export const getRdStatsSummary = (params?: { start?: string; end?: string; group_id?: number; group_by?: string }): Promise<ApiResponse<StatsSummary>> =>
  client.get('/rd-stats/summary', { params }).then((r) => r.data);

export const getRdStatsByUser = (params?: { start?: string; end?: string; group_id?: number }): Promise<ApiResponse<UserStats[]>> =>
  client.get('/rd-stats/by-user', { params }).then((r) => r.data);

export const getRdStatsByProject = (params?: { start?: string; end?: string; group_id?: number }): Promise<ApiResponse<ProjectStats[]>> =>
  client.get('/rd-stats/by-project', { params }).then((r) => r.data);

export const getRdStatsByType = (params?: { start?: string; end?: string; group_id?: number }): Promise<ApiResponse<TypeStats[]>> =>
  client.get('/rd-stats/by-type', { params }).then((r) => r.data);

export const getRdStatsByInstrument = (params?: { start?: string; end?: string; group_id?: number }): Promise<ApiResponse<InstrumentStats[]>> =>
  client.get('/rd-stats/by-instrument', { params }).then((r) => r.data);

// --- RD Export ---
export const exportRdExcel = (params: { start?: string; end?: string; group_id?: number }): Promise<Blob> =>
  client.get('/rd-export/excel', { params, responseType: 'blob' })
    .then(async (r) => {
      if (r.status !== 200) {
        const text = await r.data.text();
        try {
          const json = JSON.parse(text);
          throw new Error(json.message || '导出失败');
        } catch (e) {
          if (e instanceof Error) throw e;
          throw new Error(text || '导出失败');
        }
      }
      return r.data;
    });

// --- RD Export Preview ---
export const getRdPreviewSheet1 = (params: { start: string; end: string; group_id?: number }): Promise<ApiResponse<Sheet1Data>> =>
  client.get('/rd-export/preview/sheet1', { params }).then((r) => r.data);

export const getRdPreviewSheet2 = (params: { start: string; end: string }): Promise<ApiResponse<Sheet2Row[]>> =>
  client.get('/rd-export/preview/sheet2', { params }).then((r) => r.data);

export const getRdPreviewSheet3 = (params: { start: string; end: string }): Promise<ApiResponse<Sheet3Row[]>> =>
  client.get('/rd-export/preview/sheet3', { params }).then((r) => r.data);

export const getRdPreviewSheet4 = (params: { start: string; end: string }): Promise<ApiResponse<Sheet4Row[]>> =>
  client.get('/rd-export/preview/sheet4', { params }).then((r) => r.data);

export const getRdPreviewSheet5 = (params: { start: string; end: string }): Promise<ApiResponse<Sheet5Row[]>> =>
  client.get('/rd-export/preview/sheet5', { params }).then((r) => r.data);

export const getRdPreviewSheet6 = (params: { start: string; end: string }): Promise<ApiResponse<Sheet6Row[]>> =>
  client.get('/rd-export/preview/sheet6', { params }).then((r) => r.data);

export const getRdPreviewSheet7 = (params: { start: string; end: string }): Promise<ApiResponse<Sheet7Row[]>> =>
  client.get('/rd-export/preview/sheet7', { params }).then((r) => r.data);

export const getRdPreviewSheet8 = (params: { start: string; end: string }): Promise<ApiResponse<Sheet8Row[]>> =>
  client.get('/rd-export/preview/sheet8', { params }).then((r) => r.data);

export const getRdPreviewSheet9 = (params: { start: string; end: string }): Promise<ApiResponse<Sheet9Row[]>> =>
  client.get('/rd-export/preview/sheet9', { params }).then((r) => r.data);

export const getRdPreviewSheet10 = (params: { start: string; end: string }): Promise<ApiResponse<Sheet10Row[]>> =>
  client.get('/rd-export/preview/sheet10', { params }).then((r) => r.data);

export default client;
