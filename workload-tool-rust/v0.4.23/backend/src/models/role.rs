use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub is_system: i32,
    pub sort_order: i32,
}

/// 角色及其权限点（聚合返回）
#[derive(Debug, Serialize)]
pub struct RoleWithPermissions {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub is_system: i32,
    pub sort_order: i32,
    pub permissions: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct RoleCreate {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)] pub permissions: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct RoleUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
}

/// 全部权限点常量（接口级权限点白名单）
pub const PERMISSIONS: &[&str] = &[
    "user:manage",
    "role:manage",
    "inventory:read",
    "inventory:write",
    "instrument:read",
    "instrument:write",
    "instrument:approve",
    "instrument:book",
    "instrument:book_manage",
    "purchase:read",
    "purchase:write",
    "purchase:request",
    "purchase:approve",
    "approval:read",
    "approval:approve",
    "approval_rule:read",
    "approval_rule:manage",
    "supplier:read",
    "supplier:write",
    "audit:read",
    "notification:read",
    "notification:manage",
    "ops_stats:read",
];

/// 角色 → 默认权限点映射（种子）。`*` 表示全部权限。
pub const ROLE_DEFAULT_PERMISSIONS: &[(&str, &[&str])] = &[
    ("系统管理员", &["*"]),
    ("主管", &[
        "inventory:read", "instrument:read", "purchase:read",
        "approval:read", "approval:approve", "ops_stats:read", "audit:read", "notification:read",
    ]),
    ("库管员", &[
        "inventory:read", "inventory:write", "instrument:read",
        "purchase:read", "approval:read", "notification:read",
    ]),
    ("采购员", &[
        "purchase:read", "purchase:write", "purchase:approve",
        "supplier:read", "supplier:write", "inventory:read", "approval:read", "notification:read",
    ]),
    ("实验员", &[
        "inventory:read", "instrument:read", "instrument:book",
        "purchase:read", "purchase:request", "approval:read", "notification:read",
    ]),
    ("仪器管理员", &[
        "instrument:read", "instrument:write", "instrument:approve", "instrument:book_manage",
        "inventory:read", "approval:read", "notification:read",
    ]),
];
