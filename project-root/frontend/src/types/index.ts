export interface ProjectGroup {
  id: number;
  name: string;
  sort_order: number;
  description?: string;
  created_at: string;
}

// v0.2.17: 卡片独立 — Project 简化
export interface Project {
  id: number;
  name: string;
  notes: string;
  lab_ids: number[];
  lab_names: string[];
  method_ids: number[];
  method_names: string[];
  created_at: string;
}

// v0.2.17: 新增 Method 类型
export interface Method {
  id: number;
  name: string;
  full_name: string;
  coefficient: number;
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
  project_name?: string;
  group_name?: string;
  user_name: string;
  quantity: number;
  recorded_at: string;
  batch_no?: string;
  extra_info?: string;
  instrument?: string;
  instrument_type?: string;
  created_at: string;
}

export interface SampleRecord {
  id: number;
  group_id: number;
  group_name?: string;
  sample_name: string;
  sample_type?: string;
  quantity: number;
  user_name: string;
  recorded_at: string;
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
}

// --- API Response ---
export interface ApiResponse<T> {
  code: number;
  message: string;
  data: T | null;
}

// --- Sample Stats ---
export interface SampleStats {
  total_count: number;
  total_quantity: number;
  user_count: number;
  group_count: number;
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
  last_backup: string | null;
  backup_count: number;
  backup_files: { name: string; size: number; time: string }[];
  db_size: number;
  tables: { table: string; rows: number }[];
  backups_dir: string;
}
