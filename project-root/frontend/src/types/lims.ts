// 本地化 LIMS（v0.4.21）前端类型定义

export interface LoginData {
  token: string;
  must_change_password: boolean;
  username: string;
  role: string;
  permissions: string[];
}

export interface MeData {
  id: number;
  username: string;
  display_name: string;
  role: string;
  permissions: string[];
  must_change_password: boolean;
}

export interface UserPublic {
  id: number;
  username: string;
  display_name: string;
  role_id: number;
  role_name: string;
  lab_id: number | null;
  is_active: number;
  must_change_password: number;
  created_at: string;
  updated_at: string;
}

export interface Role {
  id: number;
  name: string;
  description: string;
  is_system: number;
  sort_order: number;
}

export interface RoleWithPermissions extends Role {
  permissions: string[];
}

export interface InstrumentResponse {
  id: number;
  name: string;
  model: string;
  location: string;
  manager: string;
  status: string;
  photo_path: string;
  qr_code_path: string;
  notes: string;
  created_by: string;
  created_at: string;
}

export interface BookingResponse {
  id: number;
  instrument_id: number;
  instrument_name: string;
  applicant: string;
  start_time: string;
  end_time: string;
  purpose: string;
  status: string;
  approver: string | null;
  approved_at: string | null;
  approver_note: string;
  created_at: string;
}

export interface MaintenanceResponse {
  id: number;
  instrument_id: number;
  instrument_name: string;
  maintainer: string;
  maintained_at: string;
  content: string;
  cost: number;
  created_at: string;
}

export interface InventoryCategory {
  id: number;
  name: string;
  parent_id: number | null;
  sort_order: number;
}

export interface ItemResponse {
  id: number;
  name: string;
  brand: string;
  unit: string;
  category_id: number | null;
  category_name: string;
  tags: string;
  location: string;
  spec: string;
  safety_stock: number;
  expiry_threshold_days: number;
  current_quantity: number;
  created_by: string;
  created_at: string;
}

export interface InventoryBatch {
  id: number;
  item_id: number;
  batch_no: string;
  quantity: number;
  unit_price: number;
  produced_at: string | null;
  expiry_date: string | null;
  source_type: string;
  source_id: number | null;
  created_at: string;
  deleted_at: string | null;
}

export interface TransactionResponse {
  id: number;
  item_id: number;
  item_name: string;
  batch_id: number | null;
  tx_type: string;
  quantity: number;
  applicant: string;
  approver: string;
  approval_task_id: number | null;
  related_id: number | null;
  note: string;
  created_by: string;
  created_at: string;
}

export interface Supplier {
  id: number;
  name: string;
  contact: string;
  phone: string;
  email: string;
  qualification: string;
  status: string;
  notes: string;
  created_at: string;
  deleted_at: string | null;
}

export interface PurchaseRequisition {
  id: number;
  requester: string;
  item_name: string;
  spec: string;
  quantity: number;
  unit: string;
  purpose: string;
  expected_supplier: string;
  status: string;
  approval_task_id: number | null;
  created_by: string;
  created_at: string;
  deleted_at: string | null;
}

export interface PurchaseOrderItem {
  id: number;
  order_id: number;
  item_name: string;
  spec: string;
  quantity: number;
  unit_price: number;
  amount: number;
  requisition_id: number | null;
}

export interface OrderResponse {
  id: number;
  order_no: string;
  supplier_id: number | null;
  supplier_name: string;
  requisition_ids: string;
  total_amount: number;
  status: string;
  approval_task_id: number | null;
  sent_at: string | null;
  received_at: string | null;
  note: string;
  created_by: string;
  created_at: string;
  items: PurchaseOrderItem[];
}

export interface ApprovalRule {
  id: number;
  biz_type: string;
  name: string;
  applicant_role: string | null;
  applicant: string | null;
  object_type: string | null;
  object_value: string | null;
  approver_role: string | null;
  approver: string | null;
  priority: number;
  is_active: number;
  created_at: string;
}

export interface ApprovalTask {
  id: number;
  biz_type: string;
  biz_id: number;
  title: string;
  applicant: string;
  approver: string | null;
  approver_role: string | null;
  status: string;
  rule_id: number | null;
  decision_note: string;
  decided_at: string | null;
  created_at: string;
}

export interface NotificationResponse {
  id: number;
  recipient: string;
  sender: string;
  title: string;
  content: string;
  link: string;
  module: string;
  is_read: number;
  created_at: string;
}

export interface AuditLogResponse {
  id: number;
  action: string;
  table_name: string;
  record_id: number | null;
  user_name: string;
  detail: string;
  module: string;
  before_json: string | null;
  after_json: string | null;
  created_at: string;
}
