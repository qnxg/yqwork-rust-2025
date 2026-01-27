use sqlx::Row;

use super::get_db_pool;
use crate::result::AppResult;
use crate::service::qnxg::permission::{Permission, PermissionItem};

#[derive(serde::Serialize, Debug)]
pub struct Role {
    pub id: u32,
    pub name: String,
}

pub async fn get_user_roles(user_id: u32) -> AppResult<Vec<Role>> {
    let res = sqlx::query!(
        r#"
        SELECT id, name
        FROM yqwork.system_user_role ur
        INNER JOIN yqwork.roles r
        ON r.id = ur.roleId
        WHERE ur.userId = ? AND r.deletedAt IS NULL
        "#,
        user_id
    )
    .fetch_all(get_db_pool().await)
    .await?
    .into_iter()
    .map(|r| Role {
        id: r.id,
        name: r.name,
    })
    .collect::<Vec<_>>();
    Ok(res)
}

pub async fn update_user_roles(user_id: u32, role_id: &[u32]) -> AppResult<()> {
    let now = chrono::Utc::now().naive_utc();
    let pool = get_db_pool().await;

    let mut tx = pool.begin().await?;

    sqlx::query!(
        r#"
        DELETE FROM yqwork.system_user_role
        WHERE userId = ?
        "#,
        user_id
    )
    .execute(&mut *tx)
    .await?;

    for r_id in role_id {
        sqlx::query!(
            r#"
            INSERT INTO yqwork.system_user_role (userId, roleId, createdAt, updatedAt)
            VALUES (?, ?, ?, ?)
            "#,
            user_id,
            r_id,
            now,
            now
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn get_role_list() -> AppResult<Vec<Role>> {
    let res = sqlx::query!(
        r#"
        SELECT id, name
        FROM yqwork.roles
        WHERE deletedAt IS NULL
        "#
    )
    .fetch_all(get_db_pool().await)
    .await?
    .into_iter()
    .map(|r| Role {
        id: r.id,
        name: r.name,
    })
    .collect::<Vec<_>>();
    Ok(res)
}

pub async fn get_role_permission(role_id: &[u32]) -> AppResult<Permission> {
    let placeholders = vec!["?"; role_id.len()].join(",");
    let query_str = format!(
        r#"
            SELECT DISTINCT p.id, p.name, p.permission
            FROM yqwork.system_role_permission rp 
            INNER JOIN yqwork.permissions p 
            ON p.id = rp.permissionId 
            WHERE p.deletedAt IS NULL AND rp.roleId IN ({})
            "#,
        placeholders
    );
    let mut query = sqlx::query(&query_str);
    for id in role_id.iter() {
        query = query.bind(id);
    }
    let res = query
        .fetch_all(get_db_pool().await)
        .await?
        .into_iter()
        .map(|r| PermissionItem {
            id: r.get("id"),
            name: r.get("name"),
            permission: r.get("permission"),
        })
        .collect::<Vec<_>>();
    Ok(Permission::new(res))
}

pub async fn update_role(role_id: u32, name: &str, permission: &[u32]) -> AppResult<()> {
    let now = chrono::Utc::now().naive_utc();
    let pool = get_db_pool().await;

    let mut tx = pool.begin().await?;

    sqlx::query!(
        r#"
        UPDATE yqwork.roles
        SET name = ?, updatedAt = ?
        WHERE id = ? AND deletedAt IS NULL
        "#,
        name,
        now,
        role_id
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        r#"
        DELETE FROM yqwork.system_role_permission
        WHERE roleId = ?
        "#,
        role_id
    )
    .execute(&mut *tx)
    .await?;

    for perm_id in permission {
        sqlx::query!(
            r#"
            INSERT INTO yqwork.system_role_permission (roleId, permissionId, createdAt, updatedAt)
            VALUES (?, ?, ?, ?)
            "#,
            role_id,
            perm_id,
            now,
            now
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn add_role(name: &str, permission: &[u32]) -> AppResult<u32> {
    let now = chrono::Utc::now().naive_utc();
    let pool = get_db_pool().await;

    let mut tx = pool.begin().await?;

    let res = sqlx::query!(
        r#"
        INSERT INTO yqwork.roles (name, createdAt, updatedAt)
        VALUES (?, ?, ?)
        "#,
        name,
        now,
        now
    )
    .execute(&mut *tx)
    .await?;
    let role_id = res.last_insert_id() as u32;

    for perm_id in permission {
        sqlx::query!(
            r#"
            INSERT INTO yqwork.system_role_permission (roleId, permissionId, createdAt, updatedAt)
            VALUES (?, ?, ?, ?)
            "#,
            role_id,
            perm_id,
            now,
            now
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(role_id)
}

pub async fn delete_role(id: u32) -> AppResult<()> {
    let now = chrono::Utc::now().naive_utc();
    sqlx::query!(
        r#"
        UPDATE yqwork.roles
        SET deletedAt = ?
        WHERE id = ? AND deletedAt IS NULL
        "#,
        now,
        id
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(())
}
