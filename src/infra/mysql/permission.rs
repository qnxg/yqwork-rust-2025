use super::get_db_pool;
use crate::result::AppResult;

#[derive(serde::Serialize, Debug)]
pub struct PermissionItem {
    pub id: u32,
    // 权限/菜单名称
    pub name: String,
    // 权限标识
    pub permission: String,
}

pub struct Permission {
    items: Vec<PermissionItem>,
}

impl Permission {
    // 判断是否拥有某个权限
    pub fn has(&self, permission: &str) -> bool {
        // 判断 * 权限是否拥有时，永远为 true
        // 如果某个人拥有了 * 权限，则拥有所有权限
        // 其他情况下，我们需要逐级判断权限。例如对于 a:b:c 权限，需要判断是否拥有 a、a:b、a:b:c 其中之一
        permission == "*"
            || self.items.iter().any(|item| {
                item.permission == "*" || permission.starts_with(item.permission.as_str())
            })
    }
    pub fn is_admin(&self) -> bool {
        self.has("*") || ["system", "yq", "hdwsh"].iter().all(|v| self.has(v))
    }
    pub fn into_inner(self) -> Vec<PermissionItem> {
        self.items
    }
    pub fn new(items: Vec<PermissionItem>) -> Self {
        Self { items }
    }
}

pub async fn get_permission_list() -> AppResult<Vec<PermissionItem>> {
    let res = sqlx::query!(
        r#"
        SELECT id, name, permission
        FROM yqwork.permissions
        WHERE deletedAt IS NULL
        "#
    )
    .fetch_all(get_db_pool().await)
    .await?
    .into_iter()
    .map(|r| PermissionItem {
        id: r.id,
        name: r.name,
        permission: r.permission,
    })
    .collect::<Vec<_>>();
    Ok(res)
}

pub async fn update_permission(id: u32, name: &str, permission: &str) -> AppResult<()> {
    let now = chrono::Utc::now().naive_utc();
    sqlx::query!(
        r#"
        UPDATE yqwork.permissions
        SET name = ?, permission = ?, updatedAt = ?
        WHERE id = ? AND deletedAt IS NULL
        "#,
        name,
        permission,
        now,
        id
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(())
}

pub async fn add_permission(name: &str, permission: &str) -> AppResult<u32> {
    let now = chrono::Utc::now().naive_utc();
    let res = sqlx::query!(
        r#"
        INSERT INTO yqwork.permissions (name, permission, createdAt, updatedAt)
        VALUES (?, ?, ?, ?)
        "#,
        name,
        permission,
        now,
        now
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(res.last_insert_id() as u32)
}

pub async fn delete_permission(id: u32) -> AppResult<()> {
    let now = chrono::Utc::now().naive_utc();
    sqlx::query!(
        r#"
        UPDATE yqwork.permissions
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
