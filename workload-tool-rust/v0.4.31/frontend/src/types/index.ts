export interface ProjectGroup {
  id: number;
  name: string;
  sort_order: number;
  description?: string;
  created_at: string;
  project_count?: number;
  project_names?: string;
  rd_record_count?: number;
  show_in_work?: boolean;
  show_in_rd?: boolean;
  division_id?: number | null;
  division_name?: string | null;
}

// ========== v0.4.24: 事业部 ==========
export interface Division {
  id: number;
  name: string;
  sort_order: number;
  lab_count?: number;
  color?: string;
  is_active?: boolean;
}

// v0.2.17: 卡片独立 — Project 简化
export interface Project {
  id: number;
  name: string;
  full_name?: string;
  notes: string;
  sort_order?: number;
  is_active?: boolean;
  lab_ids: number[];
  lab_names: string[];
  method_ids: number[];
  method_names: string[];
  created_at: string;
  group_name?: string;
  coefficient?: number;
  method_type?: string;
}

// v0.2.17: 新增 Method 类型
export interface Method {
  id: number;
  name: string;
  full_name: string;
  coefficient: number;
  multiplier: number;
  amount?: number;
  notes: string;
  is_active: boolean;
  type_ids: number[];
  type_names: string[];
  created_at: string;
}

// v0.2.8: 方法类型
export interface MethodType {
  id: number;
  name: string;
  sort_order: number;
}

export interface WorkRecord {
  id: number;
  project_id: number;
  method_id?: number;
  project_name?: string;
  group_name?: string;
  user_name: string;
  quantity: number;
  recorded_at: string;
  batch_no?: string;
  extra_info?: string;
  instrument?: string;
  instrument_type?: string;
  method_name?: string;
  method_type?: string;
  multiplier?: number;
  division_id?: number | null;
  created_at: string;
  status?: string;
  sampler?: string;
  sampled_at?: string;
}

export interface SampleRecord {
  id: number;
  group_id: number;
  group_name?: string;
  sample_name: string;
  sample_type?: string;
  quantity: number;
  sample_count?: number;
  unit?: string;
  batch_no?: string;
  user_name: string;
  recorded_at: string;
  submitted_at?: string;
  project_id?: number;
  notes?: string;
  extra_info?: string;
  created_at: string;
}

export interface AuditLog {
  id: number;
  action: string;
  table_name: string;
  record_id: number;
  user_name: string;
  detail?: string;
  created_at: string;
}

export interface StatsDetail {
  period: string;
  total_quantity: number;
  record_count: number;
  coefficient_score: number;
}

export interface StatsSummary {
  total_quantity: number;
  total_records: number;
  user_count: number;
  project_count: number;
  coefficient_score: number;
  details: StatsDetail[];
}

export interface UserStats {
  user_name: string;
  total_quantity: number;
  record_count: number;
  coefficient_score: number;
}

export interface ProjectStats {
  project_id: number;
  project_name: string;
  group_name: string;
  total_quantity: number;
  record_count: number;
  coefficient_score: number;
}

export interface TypeStats {
  instrument_type: string;
  total_quantity: number;
  record_count: number;
  coefficient_score: number;
}

export interface InstrumentStats {
  instrument: string;
  instrument_type: string;
  total_quantity: number;
  record_count: number;
  user_count: number;
  coefficient_score: number;
}

// --- Record Update (user correction) ---
export interface RecordUpdate {
  user_name?: string;
  quantity?: number;
  recorded_at?: string;
  multiplier?: number;
}

// --- API Response ---
export interface ApiResponse<T> {
  code: number;
  message: string;
  data: T | null;
}

// --- Sample Stats ---
export interface GroupSampleStats {
  group_name: string;
  count: number;
  total_samples: number;
}

export interface ProjectSampleStats {
  project_name: string;
  group_name: string;
  count: number;
  total_samples: number;
}

export interface UserSampleStats {
  user_name: string;
  count: number;
  total_samples: number;
}

export interface SampleStats {
  total_count?: number;
  total_samples?: number;
  by_group?: GroupSampleStats[];
  by_project?: ProjectSampleStats[];
  by_user?: UserSampleStats[];
}

export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  page_size: number;
}

// --- Import Result (Excel导入返回结果) ---
export interface ImportResult {
  success: boolean;
  total_rows_read: number;
  inserted: number;
  updated: number;
  skipped: number;
  sheet_name: string;
  columns_found: string[];
  errors: string[];
  warnings: string[];
}

// v0.2.17: Method import summary
export interface ImportSummary {
  total_methods: number;
  total_projects: number;
  total_groups: number;
  by_type: { method_type: string; count: number }[];
}

export interface BackupStatus {
  auto_enabled: boolean;
  auto_interval_hours: number;
  max_backup_count: number;
  last_backup: string | null;
  backup_count: number;
  backup_files: { name: string; size: number; time: string }[];
  db_size: number;
  tables: { table: string; rows: number; label?: string }[];
  backups_dir: string;
}

// v0.3.0: 导入映射配置
export interface ImportMapping {
  id: number;
  header_pattern: string;
  match_mode: string;
  target_table: string;
  default_type: string;
  priority: number;
  is_active: boolean;
}

// ========== v0.3.7: 导出预览数据类型 ==========

// Sheet 1 预览数据（后端 FlatRow = (String, String, String, String, i64, bool, f64)）
// JSON 序列化后为含数字索引的 object
export interface Sheet1Row {
  0: string; // 实验室
  1: string; // 项目代号
  2: string; // 液相仪器
  3: string; // 检测方法
  4: number; // 检测数量
  5: boolean; // 是否气相
  6: number; // 系数
}
export type Sheet1Data = [string, string, string, string, number, boolean, number][];

// Sheet 2: 仪器-汇总
export interface Sheet2Row {
  date: string;
  instrument: string;
  lab: string;
  project: string;
  method: string;
  quantity: number;
}

// Sheet 3: 项目-汇总
export interface Sheet3Row {
  project: string;
  lab: string;
  instrument: string;
  method: string;
  quantity: number;
  unit_price: number;  // 单价（原 amount）
}

// Sheet 4: 实验室-汇总
export interface Sheet4Row {
  lab: string;
  project: string;
  instrument: string;
  method: string;
  quantity: number;
  unit_price: number;  // 单价（原 amount）
}

// Sheet 5: 人员-汇总（原始记录）
export interface Sheet5Row {
  recorded_at: string;
  lab: string;
  project: string;
  method: string;
  method_type: string;
  quantity: number;
  user_name: string;
}

// Sheet 6: 人员汇总表
export interface Sheet6Row {
  user_name: string;
  method_type: string;
  coefficient: number;
  quantity: number;
}

// Sheet 7: 实验室总表
export interface Sheet7Row {
  lab: string;
  project: string;
  method_type: string;
  unit_price: number;  // 单价（原 amount）
  quantity: number;
}

// Sheet 8: 项目总表
export interface Sheet8Row {
  project: string;
  method_type: string;
  unit_price: number;  // 单价（原 amount）
  quantity: number;
}

// Sheet 9: 仪器汇总表
export interface Sheet9Row {
  instrument: string;
  quantity: number;
  instrument_type: string;
}

// ========== v0.4.11: 帮助文档 ==========
export interface HelpDocument {
  id: number;
  title: string;
  filename: string;
  file_path: string;
  file_type: string;
  file_size: number;
  is_visible: boolean;
  sort_order: number;
  page_count: number;
  created_at: string;
  updated_at: string;
}

export interface TocItem {
  id: string;
  text: string;
  level: number;
  children: TocItem[];
}

export interface HelpArticle {
  id: number;
  title: string;
  content_html: string;
  toc_json: string | null;
  source_file: string | null;
  is_visible: boolean;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

// Sheet 10: 理化汇总表
export interface Sheet10Row {
  method: string;
  quantity: number;
}

// ========== v0.4.22: 样品信息登记 ==========
export interface SampleInfoRecord {
  id: number;
  status: string;
  seq_no: number;
  batch_no: string;
  user_name: string;
  lab_name: string;
  project_name: string;
  submitted_at: string;
  detection_date: string;
  main_components: string;
  detection_type: string;
  type_key: string;
  division_id?: number | null;
  division_name?: string | null;
  quantity: number;
  notes: string;
  extra_fields?: Record<string, any>;
  created_at: string;
  updated_at?: string;
}

// ========== v0.4.26: 列自定义 ==========
export interface SampleInfoColumn {
  id: number;
  field_key: string;
  label: string;
  data_type: 'text' | 'number' | 'select' | 'date' | 'attachment';
  is_predefined: boolean;
  is_required: boolean;
  is_active: boolean;
  width: number;
  sort_order: number;
  options: string | null;
  show_in_list: boolean;
  show_in_export: boolean;
  show_in_form: boolean;
  type_key?: string | null;
  created_at: string;
  updated_at: string | null;
}

// ========== v0.4.27-A: 列可见性 ==========
export interface SampleInfoColumnVisibility {
  id: number;
  type_key: string;
  column_id: number;
  is_visible: boolean;
}

// ========== v0.4.27-A: 附件 ==========
export interface SampleInfoAttachment {
  id: number;
  record_id: number;
  file_name: string;
  stored_name: string;
  file_size: number;
  file_type: string;
  created_at: string;
}

// ========== v0.4.27-A: 用户 ==========
export interface User {
  id: number;
  username: string;
  division_id?: number | null;
  division_name?: string | null;
  group_id?: number | null;
  group_name?: string | null;
  is_admin: boolean;
  is_active: boolean;
  created_at: string;
  updated_at?: string | null;
}

export interface LoginRequest {
  username: string;
  password: string;
}

export interface LoginResponse {
  token: string;
  user: User;
}

export interface UserUpdate {
  username?: string;
  password?: string;
  division_id?: number | null;
  group_id?: number | null;
  is_admin?: boolean;
  is_active?: boolean;
}

// v0.4.28: 事业部统计
export interface DivisionStats {
  division_id: number | null;
  division_name: string;
  total_quantity: number;
  record_count: number;
  coefficient_score: number;
  lab_count: number;
}

// v0.4.28: 事业部导出预览
export interface Sheet11Row {
  division_name: string;
  lab_count: number;
  total_quantity: number;
  record_count: number;
  coefficient_score: number;
}

export interface ColumnVisibilityItem {
  column_id: number;
  is_visible: boolean;
}

// ========== v0.4.23: 检测类型 ==========
export interface SampleInfoType {
  id: number;
  type_key: string;
  label: string;
  description: string;
  color: string;
  sort_order: number;
  is_active: number;
  created_at: string;
}
