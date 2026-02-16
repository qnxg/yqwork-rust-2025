use super::get_db_pool;
use crate::{result::AppResult, utils};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Announcement {
    pub id: u32,
    pub title: String,
    pub content: String,
    pub url: Option<String>,
    pub deleted_at: Option<chrono::NaiveDateTime>,
}

/// 已经删除的公告也会获取
pub async fn get_announcement_list(
    page: u32,
    page_size: u32,
) -> AppResult<(u32, Vec<Announcement>)> {
    let res = sqlx::query_as!(
        Announcement,
        r#"
        SELECT id, title, content, url, deletedAt as deleted_at FROM weihuda_new.announcement
        ORDER BY 
            CASE WHEN deletedAt IS NULL THEN 0 ELSE 1 END, 
            id DESC
        LIMIT ? OFFSET ?
        "#,
        page_size,
        (page - 1) * page_size,
    )
    .fetch_all(get_db_pool().await)
    .await?;

    let total: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as count FROM weihuda_new.announcement
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
        SELECT id, title, content, url, deletedAt as deleted_at FROM weihuda_new.announcement
        WHERE id = ? AND deletedAt IS NULL
        "#,
        id,
    )
    .fetch_optional(get_db_pool().await)
    .await?;
    Ok(res)
}

pub async fn add_announcement(title: &str, content: &str, url: Option<&str>) -> AppResult<u32> {
    let now = utils::now_time();
    let res = sqlx::query!(
        r#"
        INSERT INTO weihuda_new.announcement (title, content, url, createdAt, updatedAt)
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
    let now = utils::now_time();
    sqlx::query!(
        r#"
        UPDATE weihuda_new.announcement
        SET title = ?, content = ?, url = ?, updatedAt = ?
        WHERE id = ? AND deletedAt IS NULL
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
    let now = utils::now_time();
    sqlx::query!(
        r#"
        UPDATE weihuda_new.announcement
        SET deletedAt = ?
        WHERE id = ? AND deletedAt IS NULL
        "#,
        now,
        id,
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(())
}
