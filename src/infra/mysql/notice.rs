use sqlx::Row;

use crate::{result::AppResult, utils};

use super::get_db_pool;

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Notice {
    pub id: u32,
    pub content: String,
    pub stu_id: String,
    pub is_show: bool,
    pub status: NoticeStatus,
    pub url: Option<String>,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoticeStatus {
    // 已读
    Read,
    // 未读
    Unread,
}
impl serde::Serialize for NoticeStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let v = u32::from(*self);
        serializer.serialize_u32(v)
    }
}
impl From<u32> for NoticeStatus {
    fn from(value: u32) -> Self {
        match value {
            0 => NoticeStatus::Unread,
            1 => NoticeStatus::Read,
            _ => NoticeStatus::Unread,
        }
    }
}
impl From<NoticeStatus> for u32 {
    fn from(value: NoticeStatus) -> Self {
        match value {
            NoticeStatus::Unread => 0,
            NoticeStatus::Read => 1,
        }
    }
}

pub async fn get_notice_list(
    page: u32,
    page_size: u32,
    stu_id: Option<String>,
    status: Option<NoticeStatus>,
    from: Option<String>,
    to: Option<String>,
) -> AppResult<(u32, Vec<Notice>)> {
    let mut main_query: sqlx::QueryBuilder<sqlx::MySql> = sqlx::QueryBuilder::new(
        r#"
        SELECT id, content, createdAt, url, stuId, status, isShow
        FROM weihuda_new.notices
        WHERE deletedAt IS NULL
    "#,
    );
    let mut count_query: sqlx::QueryBuilder<sqlx::MySql> = sqlx::QueryBuilder::new(
        r#"
        SELECT COUNT(*) AS count
        FROM weihuda_new.notices
        WHERE deletedAt IS NULL
    "#,
    );

    if let Some(stu_id) = stu_id {
        main_query.push(" AND stuId LIKE ");
        count_query.push(" AND stuId LIKE ");
        main_query.push_bind(format!("%{}%", stu_id));
        count_query.push_bind(format!("%{}%", stu_id));
    }
    if let Some(status) = status {
        main_query.push(" AND status = ");
        count_query.push(" AND status = ");
        main_query.push_bind(u32::from(status));
        count_query.push_bind(u32::from(status));
    }
    if let Some(from) = from {
        main_query.push(" AND createdAt >= ");
        count_query.push(" AND createdAt >= ");
        main_query.push_bind(from.clone());
        count_query.push_bind(from);
    }
    if let Some(to) = to {
        main_query.push(" AND createdAt <= ");
        count_query.push(" AND createdAt <= ");
        main_query.push_bind(to.clone());
        count_query.push_bind(to);
    }

    main_query.push(" ORDER BY id DESC");
    main_query.push(" LIMIT ");
    main_query.push_bind(page_size);
    main_query.push(" OFFSET ");
    main_query.push_bind((page - 1) * page_size);

    let res = main_query
        .build()
        .fetch_all(get_db_pool().await)
        .await?
        .into_iter()
        .map(|r| Notice {
            id: r.get("id"),
            content: r.get("content"),
            stu_id: r.get("stuId"),
            is_show: r.get::<u32, _>("isShow") != 0,
            status: NoticeStatus::from(r.get::<u32, _>("status")),
            url: r.get("url"),
            created_at: r.get("createdAt"),
        })
        .collect::<Vec<_>>();
    let count: i64 = count_query
        .build_query_scalar()
        .fetch_one(get_db_pool().await)
        .await?;
    Ok((count as u32, res))
}

pub async fn get_notice(id: u32) -> AppResult<Option<Notice>> {
    let res = sqlx::query!(
        r#"
        SELECT id, content, createdAt, url, stuId, status, isShow
        FROM weihuda_new.notices
        WHERE id = ? AND deletedAt IS NULL
        "#,
        id
    )
    .fetch_optional(get_db_pool().await)
    .await?
    .map(|r| Notice {
        id: r.id,
        content: r.content,
        stu_id: r.stuId,
        is_show: r.isShow != 0,
        status: NoticeStatus::from(r.status),
        url: r.url,
        created_at: r.createdAt,
    });
    Ok(res)
}

pub async fn add_notice(
    stu_id: &str,
    content: &str,
    is_show: bool,
    url: Option<&str>,
) -> AppResult<u32> {
    let now = utils::now_time();
    let res = sqlx::query!(
        r#"
        INSERT INTO weihuda_new.notices
        (stuId, content, isShow, status, url, createdAt, updatedAt)
        VALUES
        (?, ?, ?, ?, ?, ?, ?)
        "#,
        stu_id,
        content,
        is_show as u32,
        u32::from(NoticeStatus::Unread),
        url,
        now,
        now
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(res.last_insert_id() as u32)
}

pub async fn delete_notice(id: u32) -> AppResult<()> {
    let now = utils::now_time();
    sqlx::query!(
        r#"
        UPDATE weihuda_new.notices SET deletedAt = ?
        WHERE id = ? AND deletedAt IS NULL
        "#,
        now,
        id
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(())
}
