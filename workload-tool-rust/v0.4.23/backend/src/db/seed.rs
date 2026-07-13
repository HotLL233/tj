//! 首次启动种子数据：创建 6 个系统角色 + 默认权限点 + admin 管理员账号。
//!
//! 仅在 `roles` 表为空时执行，保证幂等（重复启动不会重复写入）。
//! admin 账号默认密码 `admin123`，并置 `must_change_password=1`，首登强制改密。

use argon2::password_hash::{PasswordHasher, SaltString};
use rand_core::OsRng;
use argon2::Argon2;

use crate::error::Result;
use crate::models::role::ROLE_DEFAULT_PERMISSIONS;

/// v0.4.20 起种子数据仅保留系统角色与 admin 账号；实验室/项目/方法由导入或手动创建。
pub fn ensure_seeded(conn: &rusqlite::Connection) -> Result<()> {
    let role_count: i64 = conn.query_row("SELECT COUNT(*) FROM roles", [], |r| r.get(0))?;
    if role_count > 0 {
        return Ok(());
    }

    // 1) 系统角色 + 默认权限点（is_system=1，禁止改名/删除）
    let mut order: i32 = 1;
    for &(name, perms) in ROLE_DEFAULT_PERMISSIONS {
        let description = role_description_for(name);
        conn.execute(
            "INSERT INTO roles (name, description, is_system, sort_order) VALUES (?1,?2,1,?3)",
            rusqlite::params!(name, description, order),
        )?;
        let role_id = conn.last_insert_rowid();
        for p in perms {
            conn.execute(
                "INSERT INTO role_permissions (role_id, permission) VALUES (?1,?2)",
                rusqlite::params!(role_id, p),
            )?;
        }
        order += 1;
    }

    // 2) admin 账号：argon2id 哈希 'admin123'，首登强制改密
    let admin_role_id: i64 = conn.query_row(
        "SELECT id FROM roles WHERE name='系统管理员' LIMIT 1",
        [],
        |r| r.get(0),
    )?;
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(b"admin123", &salt)
        .map_err(|e| crate::error::AppError::Internal(format!("种子密码哈希失败: {}", e)))?
        .to_string();
    conn.execute(
        "INSERT INTO users (username, display_name, password_hash, role_id, must_change_password, is_active) \
         VALUES ('admin','系统管理员',?1,?2,1,1)",
        rusqlite::params!(hash, admin_role_id),
    )?;

    Ok(())
}

/// 角色默认中文描述（种子用，便于管理员理解）。
fn role_description_for(name: &str) -> String {
    match name {
        "系统管理员" => "拥有系统全部权限，可管理用户/角色/审批规则等",
        "主管" => "审批仪器预约与采购申请，查看运营统计与审计",
        "库管员" => "维护库存与物料，处理出入库",
        "采购员" => "创建采购单、管理供应商、推进采购审批",
        "实验员" => "预约仪器、提交采购申请、查看通知",
        "仪器管理员" => "维护仪器档案、审批仪器预约、登记保养",
        _ => "",
    }
    .to_string()
}
