use sqlx::Row;

use super::get_db_pool;
use crate::{result::AppResult, utils};

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Feedback {
    pub id: u32,
    pub contact: Option<String>,
    pub desc: String,
    pub img_url: Option<String>,
    pub stu_id: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub status: FeedbackStatus,
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
    status: Option<FeedbackStatus>,
    from: Option<String>,
    to: Option<String>,
) -> AppResult<(u32, Vec<Feedback>)> {
    let mut main_query: sqlx::QueryBuilder<sqlx::MySql> = sqlx::QueryBuilder::new(
        r#"
        SELECT id, contact, createdAt, `desc`, imgUrl, stuId, updatedAt, status
        FROM weihuda_new.feedbacks
    "#,
    );
    let mut count_query: sqlx::QueryBuilder<sqlx::MySql> = sqlx::QueryBuilder::new(
        r#"
        SELECT COUNT(*) AS count
        FROM weihuda_new.feedbacks
    "#,
    );

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
        main_query.push_bind(u32::from(status));
        count_query.push_bind(u32::from(status));
        first_condition = false;
    }
    if let Some(from) = from {
        if !first_condition {
            main_query.push(" AND ");
            count_query.push(" AND ");
        }
        main_query.push("createdAt >= ");
        count_query.push("createdAt >= ");
        main_query.push_bind(from.clone());
        count_query.push_bind(from);
        first_condition = false;
    }
    if let Some(to) = to {
        if !first_condition {
            main_query.push(" AND ");
            count_query.push(" AND ");
        }
        main_query.push("createdAt <= ");
        count_query.push("createdAt <= ");
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
        .map(|r| Feedback {
            id: r.get("id"),
            contact: r.get("contact"),
            created_at: r.get("createdAt"),
            desc: r.get("desc"),
            img_url: r.get("imgUrl"),
            stu_id: r.get("stuId"),
            updated_at: r.get("updatedAt"),
            status: FeedbackStatus::from(r.get::<u32, _>("status")),
        })
        .collect::<Vec<_>>();
    let count: i64 = count_query
        .build_query_scalar()
        .fetch_one(get_db_pool().await)
        .await?;
    Ok((count as u32, res))
}

pub async fn get_feedback(id: u32) -> AppResult<Option<Feedback>> {
    let r = sqlx::query!(
        r#"
        SELECT id, contact, createdAt, `desc`, imgUrl, stuId, updatedAt, status
        FROM weihuda_new.feedbacks
        WHERE id = ?
        "#,
        id
    )
    .fetch_optional(get_db_pool().await)
    .await?
    .map(|r| Feedback {
        id: r.id,
        contact: r.contact,
        created_at: r.createdAt,
        desc: r.desc,
        img_url: r.imgUrl,
        stu_id: r.stuId,
        updated_at: r.updatedAt,
        status: FeedbackStatus::from(r.status),
    });
    Ok(r)
}

pub async fn update_feedback(id: u32, status: FeedbackStatus) -> AppResult<()> {
    let now = utils::now_time();
    sqlx::query!(
        r#"
        UPDATE weihuda_new.feedbacks
        SET status = ?, updatedAt = ?
        WHERE id = ?
        "#,
        u32::from(status),
        now,
        id
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(())
}

// weihuda_new.feedbacks 并没有设计伪删除
pub async fn delete_feedback(id: u32) -> AppResult<()> {
    sqlx::query!(
        r#"
        DELETE FROM weihuda_new.feedbacks
        WHERE id = ?
        "#,
        id
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(())
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FeedbackMsg {
    pub id: u32,
    pub typ: FeedbackMsgType,
    pub msg: Option<String>,
    pub stu_id: String,
    pub feedback_id: u32,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedbackMsgType {
    // 正常后台回复
    Comment,
}

impl serde::Serialize for FeedbackMsgType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = String::from(*self);
        serializer.serialize_str(&s)
    }
}
impl From<FeedbackMsgType> for String {
    fn from(value: FeedbackMsgType) -> Self {
        let s = match value {
            FeedbackMsgType::Comment => "comment",
        };
        s.to_string()
    }
}
impl From<String> for FeedbackMsgType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "comment" => FeedbackMsgType::Comment,
            _ => FeedbackMsgType::Comment,
        }
    }
}

pub async fn get_feedback_msg_list(id: u32) -> AppResult<Vec<FeedbackMsg>> {
    let res = sqlx::query!(
        r#"
        SELECT id, typ, msg, stuId, feedbackId, createdAt
        FROM weihuda_new.feedback_msg
        WHERE feedbackId = ? AND deletedAt IS NULL
        ORDER BY id DESC
        "#,
        id
    )
    .fetch_all(get_db_pool().await)
    .await?
    .into_iter()
    .map(|r| FeedbackMsg {
        id: r.id,
        typ: FeedbackMsgType::from(r.typ),
        msg: r.msg,
        stu_id: r.stuId,
        feedback_id: r.feedbackId,
        created_at: r.createdAt,
    })
    .collect::<Vec<_>>();
    Ok(res)
}

pub async fn add_feedback_msg(
    typ: FeedbackMsgType,
    msg: Option<&str>,
    stu_id: &str,
    feedback_id: u32,
) -> AppResult<u32> {
    let now = utils::now_time();
    let res = sqlx::query!(
        r#"
        INSERT INTO weihuda_new.feedback_msg (typ, msg, stuId, feedbackId, createdAt)
        VALUES (?, ?, ?, ?, ?)
        "#,
        String::from(typ),
        msg,
        stu_id,
        feedback_id,
        now
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(res.last_insert_id() as u32)
}

pub async fn delete_feedback_msg(id: u32) -> AppResult<()> {
    let now = utils::now_time();
    sqlx::query!(
        r#"
        UPDATE weihuda_new.feedback_msg
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
