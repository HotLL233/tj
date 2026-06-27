// --- Project Group ---
export interface ProjectGroup {
  id: number;
  name: string;
  sort_order: number;
  project_count: number;
  created_at: string;
}

// --- Project ---
export interface Project {
  id: number;
  group_id: number;
  group_name: string;
  name: string;
  full_name?: string;
  notes?: string;
  sort_order: number;
  is_active: number;
  created_at: string;
}

// --- Work Record ---
export interface WorkRecord {
  id: number;
  project_id: number;
  project_name: string;
  group_name: string;
  user_name: string;
  quantity: number;
  recorded_at: string;
  created_at: string;
  deleted_at: string | null;
}

// --- Sample Record ---
export interface SampleRecord {
  id: number;
  project_id: number;
  project_name: string;
  group_id: number;
  group_name: string;
  user_name: string;
  sample_name: string;
  sample_count: number;
  unit: string;
  batch_no: string;
  notes: string;
  submitted_at: string;
  created_at: string;
  deleted_at: string | null;
}

// --- Audit Log ---
export interface AuditLog {
  id: number;
  action: string;
  table_name: string;
  record_id: number;
  user_name: string;
  detail: string;
  created_at: string;
}

// --- Stats ---
export interface StatsDetail {
  period: string;
  total_quantity: number;
  record_count: number;
}

export interface StatsSummary {
  total_quantity: number;
  total_records: number;
  user_count: number;
  project_count: number;
  details: StatsDetail[];
}

export interface UserStats {
  user_name: string;
  total_quantity: number;
  record_count: number;
}

export interface ProjectStats {
  project_id: number;
  project_name: string;
  group_name: string;
  total_quantity: number;
  record_count: number;
}

export interface TypeStats {
  instrument_type: string;
  total_quantity: number;
  record_count: number;
}

export interface InstrumentStats {
  instrument: string;
  instrument_type: string;
  total_quantity: number;
  record_count: number;
  user_count: number;
}

// --- Record Update (user correction) ---
export interface RecordUpdate {
  user_name?: string;
  quantity?: number;
  recorded_at?: string;
}

// --- API Response ---
export interface ApiResponse<T = unknown> {
  code: number;
  data: T;
  message: string;
}

export interface SampleStats {
  total_count: number;
  total_samples: number;
  by_group: { group_name: string; count: number; total_samples: number }[];
  by_project: { project_name: string; group_name: string; count: number; total_samples: number }[];
  by_user: { user_name: string; count: number; total_samples: number }[];
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
