use sqlx::Row;

use super::get_db_pool;
use crate::result::AppResult;

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Feedback {
    pub id: u32,
    pub contact: Option<String>,
    pub create_time: chrono::NaiveDateTime,
    pub desc: String,
    pub img_url: Option<String>,
    // TODO 数据库中是可为 NULL 的字段，但应该不是这样
    pub stu_id: String,
    pub typ: FeedbackType,
    pub updated_at: chrono::NaiveDateTime,
    // TODO 数据库中是可为 NULL 的字段，但应该不是这样
    pub status: FeedbackStatus,
    pub comment: Option<String>,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedbackType {
    Suggestion,
    Bug,
    Other,
}
impl serde::Serialize for FeedbackType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match self {
            FeedbackType::Suggestion => "suggestion",
            FeedbackType::Bug => "bug",
            FeedbackType::Other => "other",
        };
        serializer.serialize_str(s)
    }
}
impl From<String> for FeedbackType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "suggestion" => FeedbackType::Suggestion,
            "bug" => FeedbackType::Bug,
            _ => FeedbackType::Other,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedbackStatus {
    // 待确认
    Pending,
    // 已确认，等待处理
    InProgress,
    // 正在处理
    Resolving,
    // 已处理并关闭
    Closed,
}
impl serde::Serialize for FeedbackStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let v = u32::from(*self);
        serializer.serialize_u32(v)
    }
}
impl From<u32> for FeedbackStatus {
    fn from(value: u32) -> Self {
        match value {
            0 => FeedbackStatus::Pending,
            1 => FeedbackStatus::InProgress,
            2 => FeedbackStatus::Resolving,
            3 => FeedbackStatus::Closed,
            _ => FeedbackStatus::Pending,
        }
    }
}
impl From<FeedbackStatus> for u32 {
    fn from(value: FeedbackStatus) -> Self {
        match value {
            FeedbackStatus::Pending => 0,
            FeedbackStatus::InProgress => 1,
            FeedbackStatus::Resolving => 2,
            FeedbackStatus::Closed => 3,
        }
    }
}

pub async fn get_feedback_list(
    page: u32,
    page_size: u32,
    stu_id: Option<String>,
    status: Option<u32>,
    from: Option<String>,
    to: Option<String>,
) -> AppResult<(u32, Vec<Feedback>)> {
    let mut main_query: sqlx::QueryBuilder<sqlx::MySql> = sqlx::QueryBuilder::new(
        r#"
        SELECT id, contact, createTime, `desc`, imgUrl, stuId, `type` AS typ, updatedAt, `status`, `comment`, updateBy
        FROM weihuda.feedbacks
        ORDER BY id DESC
    "#,
    );
    let mut count_query: sqlx::QueryBuilder<sqlx::MySql> = sqlx::QueryBuilder::new(
        r#"
        SELECT COUNT(*) AS count
        FROM weihuda.feedbacks
    "#,
    );

    main_query.push(" LIMIT ");
    main_query.push_bind(page_size);
    main_query.push(" OFFSET ");
    main_query.push_bind((page - 1) * page_size);
    if stu_id.is_some() || status.is_some() || from.is_some() || to.is_some() {
        main_query.push(" WHERE ");
        count_query.push(" WHERE ");
    }
    let mut first_condition = true;
    if let Some(stu_id) = stu_id {
        if !first_condition {
            main_query.push(" AND ");
            count_query.push(" AND ");
        }
        main_query.push("stuId LIKE ");
        count_query.push("stuId LIKE ");
        main_query.push_bind(format!("%{}%", stu_id));
        count_query.push_bind(format!("%{}%", stu_id));
        first_condition = false;
    }
    if let Some(status) = status {
        if !first_condition {
            main_query.push(" AND ");
            count_query.push(" AND ");
        }
        main_query.push("status = ");
        count_query.push("status = ");
        main_query.push_bind(status);
        count_query.push_bind(status);
        first_condition = false;
    }
    if let Some(from) = from {
        if !first_condition {
            main_query.push(" AND ");
            count_query.push(" AND ");
        }
        main_query.push("createTime >= ");
        count_query.push("createTime >= ");
        main_query.push_bind(from.clone());
        count_query.push_bind(from);
        first_condition = false;
    }
    if let Some(to) = to {
        if !first_condition {
            main_query.push(" AND ");
            count_query.push(" AND ");
        }
        main_query.push("createTime <= ");
        count_query.push("createTime <= ");
        main_query.push_bind(to.clone());
        count_query.push_bind(to);
    }
    let res = main_query
        .build()
        .fetch_all(get_db_pool().await)
        .await?
        .into_iter()
        .map(|r| Feedback {
            id: r.get::<i32, _>("id") as u32,
            contact: r.get("contact"),
            create_time: r.get("createTime"),
            desc: r.get("desc"),
            img_url: r.get("imgUrl"),
            stu_id: r.get::<Option<String>, _>("stuId").unwrap_or_default(),
            typ: FeedbackType::from(r.get::<String, _>("typ")),
            updated_at: r.get("updatedAt"),
            status: FeedbackStatus::from(r.get::<Option<i8>, _>("status").unwrap_or(0) as u32),
            comment: r.get("comment"),
            updated_by: r.get("updateBy"),
        })
        .collect::<Vec<_>>();
    let count = count_query
        .build_query_scalar()
        .fetch_one(get_db_pool().await)
        .await?;
    Ok((count, res))
}

pub async fn get_feedback(id: u32) -> AppResult<Option<Feedback>> {
    let r = sqlx::query!(
        r#"
        SELECT id, contact, createTime, `desc`, imgUrl, stuId, `type` AS typ, updatedAt, `status`, `comment`, updateBy
        FROM weihuda.feedbacks
        WHERE id = ?
        "#,
        id
    )
    .fetch_optional(get_db_pool().await)
    .await?
    .map(|r|{
        Feedback {
            id: r.id as u32,
            contact: r.contact,
            create_time: r.createTime,
            desc: r.desc,
            img_url: r.imgUrl,
            stu_id: r.stuId.unwrap_or_default(),
            typ: FeedbackType::from(r.typ),
            updated_at: r.updatedAt,
            status: FeedbackStatus::from(r.status.unwrap_or(0) as u32),
            comment: r.comment,
            updated_by: r.updateBy,
        }
    });
    Ok(r)
}

pub async fn update_feedback(
    id: u32,
    status: FeedbackStatus,
    comment: &str,
    updated_by: &str,
) -> AppResult<()> {
    let now = chrono::Utc::now().naive_utc();
    sqlx::query!(
        r#"
        UPDATE weihuda.feedbacks
        SET status = ?, comment = ?, updateBy = ?, updatedAt = ?
        WHERE id = ?
        "#,
        status as u32,
        comment,
        updated_by,
        now,
        id
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(())
}

// weihuda.feedbacks 并没有设计伪删除
pub async fn delete_feedback(id: u32) -> AppResult<()> {
    sqlx::query!(
        r#"
        DELETE FROM weihuda.feedbacks
        WHERE id = ?
        "#,
        id
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(())
}
