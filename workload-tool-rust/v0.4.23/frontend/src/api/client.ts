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

// 导出预览查询参数（工作量统计预览表）
type PreviewParams = { start?: string; end?: string; group_id?: number };
import type {
  LoginData,
  MeData,
  UserPublic,
  Role,
  RoleWithPermissions,
  InstrumentResponse,
  BookingResponse,
  MaintenanceResponse,
  InventoryCategory,
  ItemResponse,
  InventoryBatch,
  TransactionResponse,
  Supplier,
  PurchaseRequisition,
  OrderResponse,
  ApprovalRule,
  ApprovalTask,
  NotificationResponse,
  AuditLogResponse,
} from '../types/lims';

export const TOKEN_KEY = 'limsc_token';

const client = axios.create({ baseURL: '/api' });

// 请求拦截：自动附加 Bearer Token
client.interceptors.request.use((config) => {
  const token = localStorage.getItem(TOKEN_KEY);
  if (token) {
    config.headers = config.headers ?? {};
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

// 响应拦截：统一错误提示 + 401 跳登录
client.interceptors.response.use(
  (res) => res,
  (err) => {
    const url = (err.config?.url as string) || '';
    const status = err.response?.status;
    const code = err.response?.data?.code;
    // 精准化错误提示：本地同源架构下无"网络"概念，避免把连接失败/业务错误误标为"网络错误"
    let msg: string;
    const body = err.response?.data as { message?: string } | undefined;
    if (!err.response) {
      // 请求未到达本地服务（连接被拒 / 服务未启动）
      msg = '无法连接本地服务，请确认程序是否正在运行';
    } else if (body && body.message) {
      // 业务错误原样透出（后端 AppError 规范返回 {code,message,data}）
      msg = body.message;
    } else if (status) {
      // 有响应但无 message（如 400/500 非标准体）
      msg = `请求失败（HTTP ${status}）`;
    } else {
      msg = '请求未到达服务，请稍后重试';
    }
    // 登录接口自身的 401 不跳转，交由登录页提示
    if ((status === 401 || code === 401) && !url.includes('/auth/login')) {
      localStorage.removeItem(TOKEN_KEY);
      if (window.location.pathname !== '/login') {
        window.location.href = '/login';
      }
    }
    return Promise.reject(new Error(msg));
  }
);

// ===== 鉴权 / RBAC =====
export const login = (data: { username: string; password: string }): Promise<ApiResponse<LoginData>> =>
  client.post('/auth/login', data).then((r) => r.data);

export const fetchMe = (): Promise<ApiResponse<MeData>> => client.get('/auth/me').then((r) => r.data);

export const changePassword = (data: { old_password?: string; new_password: string }): Promise<ApiResponse<null>> =>
  client.post('/auth/change-password', data).then((r) => r.data);

export const getUsers = (): Promise<ApiResponse<UserPublic[]>> => client.get('/users').then((r) => r.data);
export const createUser = (data: { username: string; display_name?: string; password: string; role_id: number; lab_id?: number | null; is_active?: number }): Promise<ApiResponse<UserPublic>> =>
  client.post('/users', data).then((r) => r.data);
export const updateUser = (id: number, data: { display_name?: string; role_id?: number; lab_id?: number | null; is_active?: number }): Promise<ApiResponse<UserPublic>> =>
  client.put(`/users/${id}`, data).then((r) => r.data);
export const deleteUser = (id: number): Promise<ApiResponse<null>> => client.delete(`/users/${id}`).then((r) => r.data);
export const resetPassword = (id: number, data: { new_password: string }): Promise<ApiResponse<null>> =>
  client.post(`/users/${id}/reset-password`, data).then((r) => r.data);

export const getRoles = (): Promise<ApiResponse<RoleWithPermissions[]>> => client.get('/roles').then((r) => r.data);
export const createRole = (data: { name: string; description?: string; permissions?: string[] }): Promise<ApiResponse<RoleWithPermissions>> =>
  client.post('/roles', data).then((r) => r.data);
export const updateRole = (id: number, data: { name?: string; description?: string; sort_order?: number }): Promise<ApiResponse<RoleWithPermissions>> =>
  client.put(`/roles/${id}`, data).then((r) => r.data);
export const deleteRole = (id: number): Promise<ApiResponse<null>> => client.delete(`/roles/${id}`).then((r) => r.data);
export const getRolePermissions = (id: number): Promise<ApiResponse<string[]>> => client.get(`/roles/${id}/permissions`).then((r) => r.data);
export const setRolePermissions = (id: number, permissions: string[]): Promise<ApiResponse<RoleWithPermissions>> =>
  client.put(`/roles/${id}/permissions`, permissions).then((r) => r.data);

// ===== 仪器 =====
export const getInstruments = (): Promise<ApiResponse<InstrumentResponse[]>> => client.get('/instruments').then((r) => r.data);
export const getInstrument = (id: number): Promise<ApiResponse<InstrumentResponse>> => client.get(`/instruments/${id}`).then((r) => r.data);
export const createInstrument = (data: { name: string; model?: string; location?: string; manager?: string; status?: string; notes?: string }): Promise<ApiResponse<InstrumentResponse>> =>
  client.post('/instruments', data).then((r) => r.data);
export const updateInstrument = (id: number, data: { name?: string; model?: string; location?: string; manager?: string; status?: string; notes?: string }): Promise<ApiResponse<InstrumentResponse>> =>
  client.put(`/instruments/${id}`, data).then((r) => r.data);
export const deleteInstrument = (id: number): Promise<ApiResponse<null>> => client.delete(`/instruments/${id}`).then((r) => r.data);
export const generateInstrumentQr = (id: number): Promise<ApiResponse<{ qr_data_url: string; qr_code_path: string }>> =>
  client.post(`/instruments/${id}/qrcode`).then((r) => r.data);
export const getBookings = (params?: { instrument_id?: number; status?: string; applicant?: string }): Promise<ApiResponse<BookingResponse[]>> =>
  client.get('/instrument-bookings', { params }).then((r) => r.data);
export const submitBooking = (data: { instrument_id: number; applicant: string; start_time: string; end_time: string; purpose?: string }): Promise<ApiResponse<BookingResponse>> =>
  client.post('/instrument-bookings', data).then((r) => r.data);
export const getMaintenances = (params?: { instrument_id?: number }): Promise<ApiResponse<MaintenanceResponse[]>> =>
  client.get('/instrument-maintenances', { params }).then((r) => r.data);
export const addMaintenance = (data: { instrument_id: number; maintainer: string; maintained_at: string; content?: string; cost?: number }): Promise<ApiResponse<null>> =>
  client.post('/instrument-maintenances', data).then((r) => r.data);

// ===== 库存 =====
export const getCategories = (): Promise<ApiResponse<InventoryCategory[]>> => client.get('/inventory/categories').then((r) => r.data);
export const createCategory = (data: { name: string; parent_id?: number | null; sort_order?: number }): Promise<ApiResponse<InventoryCategory>> =>
  client.post('/inventory/categories', data).then((r) => r.data);
export const updateCategory = (id: number, data: { name?: string; parent_id?: number | null; sort_order?: number }): Promise<ApiResponse<InventoryCategory>> =>
  client.put(`/inventory/categories/${id}`, data).then((r) => r.data);
export const deleteCategory = (id: number): Promise<ApiResponse<null>> => client.delete(`/inventory/categories/${id}`).then((r) => r.data);
export const getItems = (params?: { category_id?: number; low_stock?: boolean }): Promise<ApiResponse<ItemResponse[]>> =>
  client.get('/inventory/items', { params }).then((r) => r.data);
export const getItem = (id: number): Promise<ApiResponse<ItemResponse>> => client.get(`/inventory/items/${id}`).then((r) => r.data);
export const createItem = (data: { name: string; brand?: string; unit?: string; category_id?: number | null; tags?: string; location?: string; spec?: string; safety_stock?: number; expiry_threshold_days?: number }): Promise<ApiResponse<ItemResponse>> =>
  client.post('/inventory/items', data).then((r) => r.data);
export const updateItem = (id: number, data: { name?: string; brand?: string; unit?: string; category_id?: number | null; tags?: string; location?: string; spec?: string; safety_stock?: number; expiry_threshold_days?: number }): Promise<ApiResponse<ItemResponse>> =>
  client.put(`/inventory/items/${id}`, data).then((r) => r.data);
export const deleteItem = (id: number): Promise<ApiResponse<null>> => client.delete(`/inventory/items/${id}`).then((r) => r.data);
export const getBatches = (itemId: number): Promise<ApiResponse<InventoryBatch[]>> => client.get(`/inventory/items/${itemId}/batches`).then((r) => r.data);
export const createBatch = (itemId: number, data: { batch_no?: string; quantity: number; unit_price?: number; produced_at?: string; expiry_date?: string; source_type?: string; source_id?: number }): Promise<ApiResponse<InventoryBatch>> =>
  client.post(`/inventory/items/${itemId}/batches`, data).then((r) => r.data);
export const getTransactions = (params: { item_id?: number; page?: number; page_size?: number }): Promise<ApiResponse<PaginatedResponse<TransactionResponse>>> =>
  client.get('/inventory/transactions', { params }).then((r) => r.data);
export const createOutTransaction = (data: { item_id: number; tx_type: string; quantity: number; note?: string; related_id?: number }): Promise<ApiResponse<null>> =>
  client.post('/inventory/transactions', data).then((r) => r.data);

// ===== 供应商 / 采购 =====
export const getSuppliers = (): Promise<ApiResponse<Supplier[]>> => client.get('/suppliers').then((r) => r.data);
export const createSupplier = (data: { name: string; contact?: string; phone?: string; email?: string; qualification?: string; notes?: string }): Promise<ApiResponse<Supplier>> =>
  client.post('/suppliers', data).then((r) => r.data);
export const updateSupplier = (id: number, data: { name?: string; contact?: string; phone?: string; email?: string; qualification?: string; status?: string; notes?: string }): Promise<ApiResponse<Supplier>> =>
  client.put(`/suppliers/${id}`, data).then((r) => r.data);
export const deleteSupplier = (id: number): Promise<ApiResponse<null>> => client.delete(`/suppliers/${id}`).then((r) => r.data);

export const getRequisitions = (params?: { status?: string; applicant?: string }): Promise<ApiResponse<PurchaseRequisition[]>> =>
  client.get('/purchase/requisitions', { params }).then((r) => r.data);
export const submitRequisition = (data: { item_name: string; spec?: string; quantity: number; unit?: string; purpose?: string; expected_supplier?: string }): Promise<ApiResponse<null>> =>
  client.post('/purchase/requisitions', data).then((r) => r.data);
export const getOrders = (params?: { status?: string }): Promise<ApiResponse<OrderResponse[]>> =>
  client.get('/purchase/orders', { params }).then((r) => r.data);
export const createOrder = (data: { supplier_id?: number | null; requisition_ids?: number[]; items: { item_name: string; spec?: string; quantity: number; unit_price: number }[]; note?: string }): Promise<ApiResponse<null>> =>
  client.post('/purchase/orders', data).then((r) => r.data);
export const receiveOrder = (id: number): Promise<ApiResponse<null>> => client.post(`/purchase/orders/${id}/receive`).then((r) => r.data);

// ===== 审批 =====
export const getApprovalRules = (params?: { biz_type?: string }): Promise<ApiResponse<ApprovalRule[]>> =>
  client.get('/approval/rules', { params }).then((r) => r.data);
export const createApprovalRule = (data: { biz_type: string; name?: string; applicant_role?: string | null; applicant?: string | null; object_type?: string | null; object_value?: string | null; approver_role?: string | null; approver?: string | null; priority?: number }): Promise<ApiResponse<ApprovalRule>> =>
  client.post('/approval/rules', data).then((r) => r.data);
export const updateApprovalRule = (id: number, data: { biz_type?: string; name?: string; applicant_role?: string | null; applicant?: string | null; object_type?: string | null; object_value?: string | null; approver_role?: string | null; approver?: string | null; priority?: number; is_active?: number }): Promise<ApiResponse<ApprovalRule>> =>
  client.put(`/approval/rules/${id}`, data).then((r) => r.data);
export const deleteApprovalRule = (id: number): Promise<ApiResponse<null>> => client.delete(`/approval/rules/${id}`).then((r) => r.data);
export const getApprovalTasks = (params?: { view?: string }): Promise<ApiResponse<ApprovalTask[]>> =>
  client.get('/approval/tasks', { params }).then((r) => r.data);
export const decideTask = (id: number, data: { decision: string; note?: string }): Promise<ApiResponse<null>> =>
  client.post(`/approval/tasks/${id}/decide`, data).then((r) => r.data);

// ===== 通知 =====
export const getNotifications = (params?: { unread_only?: boolean }): Promise<ApiResponse<NotificationResponse[]>> =>
  client.get('/notifications', { params }).then((r) => r.data);
export const getUnreadCount = (): Promise<ApiResponse<number>> => client.get('/notifications/unread-count').then((r) => r.data);
export const markNotificationRead = (id: number): Promise<ApiResponse<null>> => client.post(`/notifications/${id}/read`).then((r) => r.data);
export const markAllRead = (): Promise<ApiResponse<number>> => client.post('/notifications/read-all').then((r) => r.data);
export const sendNotification = (data: { recipient: string; title: string; content?: string; link?: string; module?: string }): Promise<ApiResponse<null>> =>
  client.post('/notifications', data).then((r) => r.data);

// ===== 审计 =====
export const getAuditLogsLims = (params?: { page?: number; page_size?: number; module?: string }): Promise<ApiResponse<PaginatedResponse<AuditLogResponse>>> =>
  client.get('/audit-logs', { params }).then((r) => r.data);

// ===================== 以下为既有工作量统计 API（兼容） =====================
// --- Groups ---
export const getGroups = (): Promise<ApiResponse<ProjectGroup[]>> => client.get('/groups').then((r) => r.data);
export const createGroup = (data: { name: string; sort_order?: number }): Promise<ApiResponse<ProjectGroup>> => client.post('/groups', data).then((r) => r.data);
export const updateGroup = (id: number, data: { name?: string; sort_order?: number }): Promise<ApiResponse<ProjectGroup>> => client.put(`/groups/${id}`, data).then((r) => r.data);
export const deleteGroup = (id: number): Promise<ApiResponse<null>> => client.delete(`/groups/${id}`).then((r) => r.data);

// --- Projects ---
export const getProjects = (params?: { group_id?: number; active_only?: boolean; method_type?: string }): Promise<ApiResponse<Project[]>> => client.get('/projects', { params }).then((r) => r.data);
export const createProject = (data: { name: string; notes?: string; lab_ids?: number[]; method_ids?: number[] }): Promise<ApiResponse<Project>> => client.post('/projects', data).then((r) => r.data);
export const updateProject = (id: number, data: { name?: string; full_name?: string; notes?: string; sort_order?: number; is_active?: boolean; lab_ids?: number[]; method_ids?: number[] }): Promise<ApiResponse<Project>> => client.put(`/projects/${id}`, data).then((r) => r.data);
export const deleteProject = (id: number): Promise<ApiResponse<null>> => client.delete(`/projects/${id}`).then((r) => r.data);
export const batchProjectCoefficient = (data: { group_id: number; coefficient: number }): Promise<ApiResponse<number>> => client.put('/projects/batch-coefficient', data).then((r) => r.data);

// --- Methods ---
export const getMethods = (params?: { type_id?: number }): Promise<ApiResponse<Method[]>> => client.get('/methods', { params }).then((r) => r.data);
export const createMethod = (data: { name: string; full_name?: string; coefficient?: number; amount?: number; notes?: string; type_ids?: number[] }): Promise<ApiResponse<Method>> => client.post('/methods', data).then((r) => r.data);
export const updateMethod = (id: number, data: { name?: string; full_name?: string; coefficient?: number; amount?: number; notes?: string; is_active?: boolean; type_ids?: number[] }): Promise<ApiResponse<Method>> => client.put(`/methods/${id}`, data).then((r) => r.data);
export const deleteMethod = (id: number): Promise<ApiResponse<null>> => client.delete(`/methods/${id}`).then((r) => r.data);
export const methodImport = (file: File): Promise<ApiResponse<ImportSummary>> => { const fd = new FormData(); fd.append('file', file); return client.post('/methods/import', fd, { headers: { 'Content-Type': 'multipart/form-data' } }).then((r) => r.data); };

export const getMethodTypes = (): Promise<ApiResponse<MethodType[]>> => client.get('/method-types').then((r) => r.data);
export const createMethodType = (data: { name: string; sort_order?: number }): Promise<ApiResponse<MethodType>> => client.post('/method-types', data).then((r) => r.data);
export const updateMethodType = (id: number, data: { name?: string; sort_order?: number }): Promise<ApiResponse<MethodType>> => client.put(`/method-types/${id}`, data).then((r) => r.data);
export const deleteMethodType = (id: number): Promise<ApiResponse<null>> => client.delete(`/method-types/${id}`).then((r) => r.data);

export const getImportMappings = (): Promise<ApiResponse<ImportMapping[]>> => client.get('/import/mappings').then((r) => r.data);

// --- Records ---
export const getRecords = (params: { start?: string; end?: string; group_id?: number; page?: number; page_size?: number; include_deleted?: boolean }): Promise<ApiResponse<PaginatedResponse<WorkRecord>>> => client.get('/records', { params }).then((r) => r.data);
export const createRecord = (data: { project_id: number; method_id?: number; user_name: string; quantity: number; recorded_at: string; group_id?: number }): Promise<ApiResponse<WorkRecord>> => client.post('/records', data).then((r) => r.data);
export const deleteRecord = (id: number): Promise<ApiResponse<null>> => client.delete(`/records/${id}`).then((r) => r.data);
export const restoreRecord = (id: number): Promise<ApiResponse<WorkRecord>> => client.post(`/records/restore/${id}`).then((r) => r.data);
export const updateRecord = (id: number, data: { user_name?: string; quantity?: number; recorded_at?: string }): Promise<ApiResponse<WorkRecord>> => client.put(`/records/${id}`, data).then((r) => r.data);
export const deleteRecordsByUser = (user_name: string, params: { start: string; end: string; group_id?: number }): Promise<ApiResponse<number>> => client.delete('/records/by-user', { params: { ...params, user_name } }).then((r) => r.data);

// --- Stats ---
export const getStatsSummary = (params?: { start?: string; end?: string; group_id?: number; group_by?: string }): Promise<ApiResponse<StatsSummary>> => client.get('/stats/summary', { params }).then((r) => r.data);
export const getStatsByUser = (params?: { start?: string; end?: string; group_id?: number }): Promise<ApiResponse<UserStats[]>> => client.get('/stats/by-user', { params }).then((r) => r.data);
export const getStatsByProject = (params?: { start?: string; end?: string; group_id?: number }): Promise<ApiResponse<ProjectStats[]>> => client.get('/stats/by-project', { params }).then((r) => r.data);
export const getStatsByType = (params?: { start?: string; end?: string; group_id?: number }): Promise<ApiResponse<TypeStats[]>> => client.get('/stats/by-type', { params }).then((r) => r.data);
export const getStatsByInstrument = (params?: { start?: string; end?: string; group_id?: number }): Promise<ApiResponse<InstrumentStats[]>> => client.get('/stats/by-instrument', { params }).then((r) => r.data);

// --- Export ---
export const exportExcel = (params: { start?: string; end?: string; group_id?: number }): Promise<Blob> =>
  client.get('/export/excel', { params, responseType: 'blob' }).then(async (r) => { if (r.status !== 200) { const text = await r.data.text(); try { const json = JSON.parse(text); throw new Error(json.message || '导出失败'); } catch (e) { if (e instanceof Error) throw e; throw new Error(text || '导出失败'); } } return r.data; });

// --- Audit (兼容旧类型) ---
export const getAuditLogs = (params?: { page?: number; page_size?: number }): Promise<ApiResponse<PaginatedResponse<AuditLog>>> => client.get('/audit-logs', { params }).then((r) => r.data);

// --- Samples ---
export const getSamples = (params?: { group_id?: number; user_name?: string; page?: number; page_size?: number }): Promise<ApiResponse<PaginatedResponse<SampleRecord>>> => client.get('/samples', { params }).then((r) => r.data);
export const getSample = (id: number): Promise<ApiResponse<SampleRecord>> => client.get(`/samples/${id}`).then((r) => r.data);
export const createSample = (data: { project_id: number; user_name: string; sample_name: string; sample_count: number; submitted_at: string; unit?: string; batch_no?: string; notes?: string }): Promise<ApiResponse<SampleRecord>> => client.post('/samples', data).then((r) => r.data);
export const updateSample = (id: number, data: { sample_name?: string; sample_count?: number; unit?: string; batch_no?: string; notes?: string; submitted_at?: string }): Promise<ApiResponse<SampleRecord>> => client.put(`/samples/${id}`, data).then((r) => r.data);
export const deleteSample = (id: number): Promise<ApiResponse<null>> => client.delete(`/samples/${id}`).then((r) => r.data);
export const restoreSample = (id: number): Promise<ApiResponse<null>> => client.post(`/samples/${id}/restore`).then((r) => r.data);
export const getSampleStats = (params?: { start?: string; end?: string }): Promise<ApiResponse<SampleStats>> => client.get('/samples/stats', { params }).then((r) => r.data);

// --- Backup ---
export const getBackupStatus = (): Promise<ApiResponse<BackupStatus>> => client.get('/backup/status').then((r) => r.data);
export const backupNow = (): Promise<ApiResponse<string>> => client.post('/backup/now').then((r) => r.data);
export const getBackupConfig = (): Promise<ApiResponse<{ enabled: boolean; interval_hours: number }>> => client.get('/backup/config').then((r) => r.data);
export const updateBackupConfig = (data: { enabled: boolean; interval_hours: number; max_backup_count?: number }): Promise<ApiResponse<string>> => client.put('/backup/config', data).then((r) => r.data);
export const deleteBackup = (filename: string): Promise<ApiResponse<string>> => client.delete(`/backup/file/${encodeURIComponent(filename)}`).then((r) => r.data);
export const restoreBackup = (file: File): Promise<ApiResponse<string>> => { const fd = new FormData(); fd.append('file', file); return client.post('/backup/restore', fd, { headers: { 'Content-Type': 'multipart/form-data' } }).then((r) => r.data); };
export const restoreBackupFile = (filename: string): Promise<ApiResponse<string>> => client.post(`/backup/restore/${encodeURIComponent(filename)}`).then((r) => r.data);

// --- 研发送样 Stats/Export (节选) ---
export const getRdStatsSummary = (params?: { start?: string; end?: string; group_id?: number; group_by?: string }): Promise<ApiResponse<StatsSummary>> => client.get('/rd-stats/summary', { params }).then((r) => r.data);
export const exportRdExcel = (params: { start?: string; end?: string; group_id?: number }): Promise<Blob> =>
  client.get('/rd-export/excel', { params, responseType: 'blob' }).then(async (r) => { if (r.status !== 200) { const text = await r.data.text(); try { const json = JSON.parse(text); throw new Error(json.message || '导出失败'); } catch (e) { if (e instanceof Error) throw e; throw new Error(text || '导出失败'); } } return r.data; });

// ===================== 导出预览（工作量统计预览表，复用于 StatsPage） =====================
export const getPreviewSheet1 = (params: PreviewParams): Promise<ApiResponse<Sheet1Data>> => client.get('/export/preview/sheet1', { params }).then((r) => r.data);
export const getPreviewSheet2 = (params: PreviewParams): Promise<ApiResponse<Sheet2Row[]>> => client.get('/export/preview/sheet2', { params }).then((r) => r.data);
export const getPreviewSheet3 = (params: PreviewParams): Promise<ApiResponse<Sheet3Row[]>> => client.get('/export/preview/sheet3', { params }).then((r) => r.data);
export const getPreviewSheet4 = (params: PreviewParams): Promise<ApiResponse<Sheet4Row[]>> => client.get('/export/preview/sheet4', { params }).then((r) => r.data);
export const getPreviewSheet5 = (params: PreviewParams): Promise<ApiResponse<Sheet5Row[]>> => client.get('/export/preview/sheet5', { params }).then((r) => r.data);
export const getPreviewSheet6 = (params: PreviewParams): Promise<ApiResponse<Sheet6Row[]>> => client.get('/export/preview/sheet6', { params }).then((r) => r.data);
export const getPreviewSheet7 = (params: PreviewParams): Promise<ApiResponse<Sheet7Row[]>> => client.get('/export/preview/sheet7', { params }).then((r) => r.data);
export const getPreviewSheet8 = (params: PreviewParams): Promise<ApiResponse<Sheet8Row[]>> => client.get('/export/preview/sheet8', { params }).then((r) => r.data);
export const getPreviewSheet9 = (params: PreviewParams): Promise<ApiResponse<Sheet9Row[]>> => client.get('/export/preview/sheet9', { params }).then((r) => r.data);
export const getPreviewSheet10 = (params: PreviewParams): Promise<ApiResponse<Sheet10Row[]>> => client.get('/export/preview/sheet10', { params }).then((r) => r.data);

export default client;
