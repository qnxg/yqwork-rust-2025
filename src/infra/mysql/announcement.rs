use super::get_db_pool;
use crate::result::AppResult;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Announcement {
    pub id: u32,
    pub title: String,
    pub content: String,
    pub url: Option<String>,
}

pub async fn get_announcement_list(
    page: u32,
    page_size: u32,
) -> AppResult<(u32, Vec<Announcement>)> {
    let res = sqlx::query_as!(
        Announcement,
        r#"
        SELECT id, title, content, url FROM weihuda.mini_message
        WHERE deleted_at IS NULL
        ORDER BY id DESC
        LIMIT ? OFFSET ?
        "#,
        page_size,
        (page - 1) * page_size,
    )
    .fetch_all(get_db_pool().await)
    .await?;

    let total = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as count FROM weihuda.mini_message
        WHERE deleted_at IS NULL
        "#
    )
    .fetch_one(get_db_pool().await)
    .await?;

    Ok((total as u32, res))
}

pub async fn get_announcement(id: u32) -> AppResult<Option<Announcement>> {
    let res = sqlx::query_as!(
        Announcement,
        r#"
        SELECT id, title, content, url FROM weihuda.mini_message
        WHERE id = ? AND deleted_at IS NULL
        "#,
        id,
    )
    .fetch_optional(get_db_pool().await)
    .await?;
    Ok(res)
}

pub async fn add_announcement(title: &str, content: &str, url: Option<&str>) -> AppResult<u32> {
    let now = chrono::Utc::now().naive_utc();
    let res = sqlx::query!(
        r#"
        INSERT INTO weihuda.mini_message (title, content, url, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?)
        "#,
        title,
        content,
        url,
        now,
        now,
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(res.last_insert_id() as u32)
}

pub async fn update_announcement(
    id: u32,
    title: &str,
    content: &str,
    url: Option<&str>,
) -> AppResult<()> {
    let now = chrono::Utc::now().naive_utc();
    sqlx::query!(
        r#"
        UPDATE weihuda.mini_message
        SET title = ?, content = ?, url = ?, updated_at = ?
        WHERE id = ? AND deleted_at IS NULL
        "#,
        title,
        content,
        url,
        now,
        id,
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(())
}

pub async fn delete_announcement(id: u32) -> AppResult<()> {
    let now = chrono::Utc::now().naive_utc();
    sqlx::query!(
        r#"
        UPDATE weihuda.mini_message
        SET deleted_at = ?
        WHERE id = ? AND deleted_at IS NULL
        "#,
        now,
        id,
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(())
}
