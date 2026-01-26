use sqlx::Row;

use super::get_db_pool;
use crate::result::AppResult;

#[derive(Debug, serde::Serialize)]
pub struct Zhihu {
    // TODO 数据库中的是 i64
    pub id: u32,
    pub info: ZhihuBasicInfo,
}
#[derive(Debug, serde::Serialize)]
pub struct ZhihuBasicInfo {
    pub title: String,
    pub typ: ZhihuType,
    pub content: String,
    // TODO 应该要为 NOT NULL
    pub tags: String,
    pub cover: Option<String>,
    pub status: ZhihuStatus,
    // TODO 应该要为 NOT NULL
    pub publish_time: chrono::NaiveDateTime,
    // TODO 应该要为 NOT NULL
    pub stu_id: String,
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
        SELECT id, title, `type` AS typ, content, tags, cover, status, publishTime, stuId
        FROM weihuda.zhihus
        ORDER BY id DESC
        WHERE deletedAt IS NULL
        "#,
    );
    let mut count_query = sqlx::QueryBuilder::new(
        r#"
        SELECT COUNT(*) AS count
        FROM weihuda.zhihus
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
            id: r.get::<i32, _>("id") as u32,
            info: ZhihuBasicInfo {
                title: r.get("title"),
                typ: ZhihuType::from(r.get::<String, _>("typ").as_str()),
                content: r.get("content"),
                tags: r.get::<Option<String>, _>("tags").unwrap_or_default(),
                cover: r.get("cover"),
                status: ZhihuStatus::from(
                    r.get::<Option<i32>, _>("status").unwrap_or_default() as u32
                ),
                publish_time: r
                    .get::<Option<chrono::NaiveDateTime>, _>("publishTime")
                    .unwrap(),
                stu_id: r.get::<Option<String>, _>("stuId").unwrap(),
            },
        })
        .collect::<Vec<_>>();
    let count = count_query
        .build_query_scalar()
        .fetch_one(get_db_pool().await)
        .await?;

    Ok((count, res))
}

pub async fn get_zhihu(id: u32) -> AppResult<Option<Zhihu>> {
    let res = sqlx::query!(
        r#"
        SELECT id, title, `type` AS typ, content, tags, cover, status, publishTime, stuId
        FROM weihuda.zhihus
        WHERE id = ? AND deletedAt IS NULL
        "#,
        id
    )
    .fetch_optional(get_db_pool().await)
    .await?;

    if let Some(r) = res {
        Ok(Some(Zhihu {
            id: r.id as u32,
            info: ZhihuBasicInfo {
                title: r.title,
                typ: ZhihuType::from(r.typ.as_str()),
                content: r.content,
                tags: r.tags.unwrap(),
                cover: r.cover,
                status: ZhihuStatus::from(r.status.unwrap_or_default() as u32),
                publish_time: r.publishTime.unwrap(),
                stu_id: r.stuId.unwrap(),
            },
        }))
    } else {
        Ok(None)
    }
}

pub async fn add_zhihu(info: &ZhihuBasicInfo) -> AppResult<u32> {
    let now = chrono::Utc::now().naive_utc();
    let res = sqlx::query!(
        r#"
        INSERT INTO weihuda.zhihus (title, `type`, content, tags, cover, status, publishTime, stuId, createdAt, updatedAt)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        info.title,
        <&str>::from(info.typ),
        info.content,
        info.tags,
        info.cover,
        u32::from(info.status) as i32,
        info.publish_time,
        info.stu_id,
        now,
        now
    )
    .execute(get_db_pool().await)
    .await?;

    Ok(res.last_insert_id() as u32)
}

pub async fn update_zhihu(id: u32, info: &ZhihuBasicInfo) -> AppResult<()> {
    let now = chrono::Utc::now().naive_utc();
    sqlx::query!(
        r#"
        UPDATE weihuda.zhihus
        SET title = ?, `type` = ?, content = ?, tags = ?, cover = ?, status = ?, publishTime = ?, stuId = ?, updatedAt = ?
        WHERE id = ? AND deletedAt IS NULL
        "#,
        info.title,
        <&str>::from(info.typ),
        info.content,
        info.tags,
        info.cover,
        u32::from(info.status) as i32,
        info.publish_time,
        info.stu_id,
        now,
        id
    )
    .execute(get_db_pool().await)
    .await?;

    Ok(())
}

pub async fn delete_zhihu(id: u32) -> AppResult<()> {
    let now = chrono::Utc::now().naive_utc();
    sqlx::query!(
        r#"
        UPDATE weihuda.zhihus
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
