use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct ProjectResponse {
    pub id: i64,
    pub group_id: i64,
    pub group_name: String,
    pub name: String,
    pub full_name: String,
    pub notes: String,
    pub sort_order: i64,
    pub is_active: bool,
    pub coefficient: f64,
    pub method_type: String,
    pub parent_id: i64,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub methods: Vec<ProjectResponse>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ProjectCreate {
    pub group_id: i64,
    pub name: String,
    pub sort_order: Option<i64>,
    pub coefficient: Option<f64>,
    pub method_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ProjectUpdate {
    pub name: Option<String>,
    pub full_name: Option<String>,
    pub notes: Option<String>,
    pub sort_order: Option<i64>,
    pub is_active: Option<bool>,
    pub coefficient: Option<f64>,
    pub method_type: Option<String>,
}

// 扁平导入项: 实验室+研发项目+方法名+类型+系数
#[derive(Debug, Clone, Deserialize)]
pub struct MethodImportItem {
    pub group_name: String,
    pub project_name: String,
    pub method_name: String,
    pub method_type: String,
    pub coefficient: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportSummary {
    pub total_methods: usize,
    pub total_projects: usize,
    pub total_groups: usize,
    pub by_type: Vec<TypeCount>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TypeCount { pub method_type: String, pub count: usize }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodType {
    pub id: i64,
    pub name: String,
    pub sort_order: i64,
}

#[derive(Debug, Deserialize)]
pub struct MethodTypeCreate {
    pub name: String,
    pub sort_order: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct MethodTypeUpdate {
    pub name: Option<String>,
    pub sort_order: Option<i64>,
}
