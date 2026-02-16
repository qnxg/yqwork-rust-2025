use sqlx::Row;

use super::get_db_pool;
use crate::{result::AppResult, utils};

#[derive(Debug, serde::Serialize)]
pub struct Zhihu {
    pub id: u32,
    pub info: ZhihuBasicInfo,
}
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZhihuBasicInfo {
    pub title: String,
    pub typ: ZhihuType,
    pub content: String,
    pub tags: String,
    pub cover: Option<String>,
    pub status: ZhihuStatus,
    pub stu_id: String,
    pub top: bool,
    pub created_at: chrono::NaiveDateTime,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZhihuType {
    Article,
    Link,
}
impl From<&str> for ZhihuType {
    fn from(s: &str) -> Self {
        match s {
            "article" => ZhihuType::Article,
            "link" => ZhihuType::Link,
            _ => ZhihuType::Article,
        }
    }
}
impl From<ZhihuType> for &str {
    fn from(t: ZhihuType) -> Self {
        match t {
            ZhihuType::Article => "article",
            ZhihuType::Link => "link",
        }
    }
}
impl serde::Serialize for ZhihuType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s: &str = (*self).into();
        serializer.serialize_str(s)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZhihuStatus {
    Pending,
    Accepted,
    Rejected,
}
impl From<u32> for ZhihuStatus {
    fn from(s: u32) -> Self {
        match s {
            0 => ZhihuStatus::Pending,
            1 => ZhihuStatus::Accepted,
            2 => ZhihuStatus::Rejected,
            _ => ZhihuStatus::Pending,
        }
    }
}
impl From<ZhihuStatus> for u32 {
    fn from(t: ZhihuStatus) -> Self {
        match t {
            ZhihuStatus::Pending => 0,
            ZhihuStatus::Accepted => 1,
            ZhihuStatus::Rejected => 2,
        }
    }
}
impl serde::Serialize for ZhihuStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = u32::from(*self);
        serializer.serialize_u32(s)
    }
}

pub async fn get_zhihu_list(
    page: u32,
    page_size: u32,
    title: Option<&str>,
    tags: Option<&str>,
    status: Option<ZhihuStatus>,
    stu_id: Option<&str>,
) -> AppResult<(u32, Vec<Zhihu>)> {
    let mut main_query = sqlx::QueryBuilder::new(
        r#"
        SELECT id, title, content, tags, cover, status, stuId, createdAt, top, typ
        FROM weihuda_new.zhihus
        WHERE deletedAt IS NULL
        "#,
    );
    let mut count_query = sqlx::QueryBuilder::new(
        r#"
        SELECT COUNT(*) AS count
        FROM weihuda_new.zhihus
        WHERE deletedAt IS NULL
        "#,
    );

    if let Some(title) = title {
        main_query
            .push(" AND title LIKE ")
            .push_bind(format!("%{}%", title));
        count_query
            .push(" AND title LIKE ")
            .push_bind(format!("%{}%", title));
    }
    if let Some(tags) = tags {
        main_query
            .push(" AND tags LIKE ")
            .push_bind(format!("%{}%", tags));
        count_query
            .push(" AND tags LIKE ")
            .push_bind(format!("%{}%", tags));
    }
    if let Some(status) = status {
        main_query
            .push(" AND status = ")
            .push_bind(u32::from(status));
        count_query
            .push(" AND status = ")
            .push_bind(u32::from(status));
    }
    if let Some(stu_id) = stu_id {
        main_query
            .push(" AND stuId LIKE ")
            .push_bind(format!("%{}%", stu_id));
        count_query
            .push(" AND stuId LIKE ")
            .push_bind(format!("%{}%", stu_id));
    }

    main_query.push(" ORDER BY top DESC, createdAt DESC");
    main_query.push(" LIMIT ").push_bind(page_size);
    main_query
        .push(" OFFSET ")
        .push_bind((page - 1) * page_size);

    let res = main_query
        .build()
        .fetch_all(get_db_pool().await)
        .await?
        .into_iter()
        .map(|r| Zhihu {
            id: r.get("id"),
            info: ZhihuBasicInfo {
                title: r.get("title"),
                typ: ZhihuType::from(r.get::<String, _>("typ").as_str()),
                content: r.get("content"),
                tags: r.get("tags"),
                cover: r.get("cover"),
                status: ZhihuStatus::from(r.get::<Option<u32>, _>("status").unwrap_or_default()),
                created_at: r.get("createdAt"),
                stu_id: r.get("stuId"),
                top: r.get::<u32, _>("top") != 0,
            },
        })
        .collect::<Vec<_>>();
    let count: i64 = count_query
        .build_query_scalar()
        .fetch_one(get_db_pool().await)
        .await?;

    Ok((count as u32, res))
}

pub async fn get_zhihu(id: u32) -> AppResult<Option<Zhihu>> {
    let res = sqlx::query!(
        r#"
        SELECT id, title, content, tags, cover, status, stuId, createdAt, top, typ
        FROM weihuda_new.zhihus
        WHERE id = ? AND deletedAt IS NULL
        "#,
        id
    )
    .fetch_optional(get_db_pool().await)
    .await?
    .map(|r| Zhihu {
        id: r.id,
        info: ZhihuBasicInfo {
            title: r.title,
            typ: ZhihuType::from(r.typ.as_str()),
            content: r.content,
            tags: r.tags,
            cover: r.cover,
            status: ZhihuStatus::from(r.status),
            stu_id: r.stuId,
            top: r.top != 0,
            created_at: r.createdAt,
        },
    });

    Ok(res)
}

pub async fn add_zhihu(info: &ZhihuBasicInfo) -> AppResult<u32> {
    let now = utils::now_time();
    let res = sqlx::query!(
        r#"
        INSERT INTO weihuda_new.zhihus (title, content, tags, cover, status, stuId, top, typ, createdAt, updatedAt)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        info.title,
        info.content,
        info.tags,
        info.cover,
        u32::from(info.status),
        info.stu_id,
        info.top as u32,
        <&str>::from(info.typ),
        now,
        now
    )
    .execute(get_db_pool().await)
    .await?;

    Ok(res.last_insert_id() as u32)
}

pub async fn update_zhihu(id: u32, info: &ZhihuBasicInfo) -> AppResult<()> {
    let now = utils::now_time();
    sqlx::query!(
        r#"
        UPDATE weihuda_new.zhihus
        SET title = ?, content = ?, tags = ?, cover = ?, status = ?, stuId = ?, top = ?, typ = ?, updatedAt = ?
        WHERE id = ? AND deletedAt IS NULL
        "#,
        info.title,
        info.content,
        info.tags,
        info.cover,
        u32::from(info.status),
        info.stu_id,
        info.top as u32,
        <&str>::from(info.typ),
        now,
        id
    )
    .execute(get_db_pool().await)
    .await?;

    Ok(())
}

pub async fn delete_zhihu(id: u32) -> AppResult<()> {
    let now = utils::now_time();
    sqlx::query!(
        r#"
        UPDATE weihuda_new.zhihus
        SET deletedAt = ?
        WHERE id = ?
        "#,
        now,
        id
    )
    .execute(get_db_pool().await)
    .await?;

    Ok(())
}
