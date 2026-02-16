use sqlx::Row;

use super::get_db_pool;
use crate::{result::AppResult, utils};

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GoodsRecord {
    pub id: u32,
    pub stu_id: String,
    pub goods_id: u32,
    pub status: GoodsRecordStatus,
    pub receive_time: Option<chrono::NaiveDateTime>,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoodsRecordStatus {
    // 待后台确认
    Pending,
    // 已兑换，待领取
    Exchanged,
    // 已领取
    Received,
}
impl From<u32> for GoodsRecordStatus {
    fn from(value: u32) -> Self {
        match value {
            0 => GoodsRecordStatus::Pending,
            1 => GoodsRecordStatus::Exchanged,
            2 => GoodsRecordStatus::Received,
            _ => GoodsRecordStatus::Pending,
        }
    }
}
impl From<GoodsRecordStatus> for u32 {
    fn from(value: GoodsRecordStatus) -> Self {
        match value {
            GoodsRecordStatus::Pending => 0,
            GoodsRecordStatus::Exchanged => 1,
            GoodsRecordStatus::Received => 2,
        }
    }
}
impl serde::Serialize for GoodsRecordStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let v = u32::from(*self);
        serializer.serialize_u32(v)
    }
}

pub async fn get_goods_record_list(
    page: u32,
    page_size: u32,
    stu_id: Option<&str>,
    goods_id: Option<u32>,
    status: Option<GoodsRecordStatus>,
) -> AppResult<(u32, Vec<GoodsRecord>)> {
    let mut main_query = sqlx::QueryBuilder::new(
        r#"
        SELECT id, stuId, goodsId, status, receiveTime, createdAt
        FROM weihuda_new.jifen_exchange
        WHERE deletedAt IS NULL
        "#,
    );
    let mut count_query = sqlx::QueryBuilder::new(
        r#"
        SELECT COUNT(*) AS count
        FROM weihuda_new.jifen_exchange
        WHERE deletedAt IS NULL
        "#,
    );

    if let Some(stu_id) = stu_id {
        main_query
            .push(" AND stuId LIKE ")
            .push_bind(format!("%{}%", stu_id));
        count_query
            .push(" AND stuId LIKE ")
            .push_bind(format!("%{}%", stu_id));
    }

    if let Some(goods_id) = goods_id {
        main_query.push(" AND goodsId = ").push_bind(goods_id);
        count_query.push(" AND goodsId = ").push_bind(goods_id);
    }

    if let Some(status) = status {
        main_query
            .push(" AND status = ")
            .push_bind(u32::from(status));
        count_query
            .push(" AND status = ")
            .push_bind(u32::from(status));
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
        .map(|r| GoodsRecord {
            id: r.get("id"),
            stu_id: r.get("stuId"),
            goods_id: r.get("goodsId"),
            created_at: r.get("createdAt"),
            status: GoodsRecordStatus::from(r.get::<u32, _>("status")),
            receive_time: r.get("receiveTime"),
        })
        .collect::<Vec<_>>();

    let count: i64 = count_query
        .build_query_scalar()
        .fetch_one(get_db_pool().await)
        .await?;

    Ok((count as u32, res))
}

pub async fn get_goods_record(id: u32) -> AppResult<Option<GoodsRecord>> {
    let res = sqlx::query!(
        r#"
        SELECT id, stuId, goodsId, status, receiveTime, createdAt
        FROM weihuda_new.jifen_exchange
        WHERE id = ? AND deletedAt IS NULL
        "#,
        id
    )
    .fetch_optional(get_db_pool().await)
    .await?
    .map(|r| GoodsRecord {
        id: r.id,
        stu_id: r.stuId,
        goods_id: r.goodsId,
        created_at: r.createdAt,
        status: GoodsRecordStatus::from(r.status),
        receive_time: r.receiveTime,
    });
    Ok(res)
}

pub async fn update_goods_record(
    id: u32,
    status: GoodsRecordStatus,
    receive_time: Option<chrono::NaiveDateTime>,
) -> AppResult<()> {
    let now = utils::now_time();
    sqlx::query!(
        r#"
        UPDATE weihuda_new.jifen_exchange
        SET status = ?, receiveTime = ?, updatedAt = ?
        WHERE id = ? AND deletedAt IS NULL
        "#,
        u32::from(status),
        receive_time,
        now,
        id
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(())
}

pub async fn delete_goods_record(id: u32) -> AppResult<()> {
    let now = chrono::Utc::now().naive_utc();
    sqlx::query!(
        r#"
        UPDATE weihuda_new.jifen_exchange
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

#[derive(serde::Serialize, Debug)]
pub struct JifenGoods {
    pub id: u32,
    pub name: String,
    pub cover: String,
    pub count: u32,
    pub price: i32,
    pub description: Option<String>,
    pub enabled: bool,
}

pub async fn get_goods_list() -> AppResult<Vec<JifenGoods>> {
    let res = sqlx::query!(
        r#"
        SELECT id, name, cover, count, price, description, enabled
        FROM weihuda_new.jifen_goods
        WHERE deletedAt IS NULL
        ORDER BY id DESC
        "#,
    )
    .fetch_all(get_db_pool().await)
    .await?
    .into_iter()
    .map(|r| JifenGoods {
        id: r.id,
        name: r.name,
        cover: r.cover,
        count: r.count,
        price: r.price,
        description: r.description,
        enabled: r.enabled != 0,
    })
    .collect::<Vec<_>>();
    Ok(res)
}

pub async fn add_goods(
    name: &str,
    cover: &str,
    count: u32,
    price: i32,
    description: Option<&str>,
    enabled: bool,
) -> AppResult<u32> {
    let now = utils::now_time();
    let res = sqlx::query!(
        r#"
        INSERT INTO weihuda_new.jifen_goods (name, cover, count, price, description, enabled, createdAt, updatedAt)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        name,
        cover,
        count,
        price,
        description,
        enabled as u32,
        now,
        now
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(res.last_insert_id() as u32)
}

pub async fn update_goods(
    id: u32,
    name: &str,
    cover: &str,
    count: u32,
    price: i32,
    description: Option<&str>,
    enabled: bool,
) -> AppResult<()> {
    let now = utils::now_time();
    sqlx::query!(
        r#"
        UPDATE weihuda_new.jifen_goods
        SET name = ?, cover = ?, count = ?, price = ?, description = ?, enabled = ?, updatedAt = ?
        WHERE id = ?
        "#,
        name,
        cover,
        count,
        price,
        description,
        enabled as u32,
        now,
        id
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(())
}

pub async fn delete_goods(id: u32) -> AppResult<()> {
    let now = utils::now_time();
    sqlx::query!(
        r#"
        UPDATE weihuda_new.jifen_goods
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

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JifenRecord {
    pub id: u32,
    pub key: String,
    pub param: String,
    pub stu_id: String,
    pub desc: String,
    pub jifen: i32,
    pub created_at: chrono::NaiveDateTime,
}

pub async fn get_record_list(
    page: u32,
    page_size: u32,
    key: Option<&str>,
    param: Option<&str>,
    stu_id: Option<&str>,
) -> AppResult<(u32, Vec<JifenRecord>)> {
    let mut main_query = sqlx::QueryBuilder::new(
        r#"
        SELECT id, `key`, `param`, stuId, `desc`, jifen, createdAt
        FROM weihuda_new.jifen_records
        "#,
    );
    let mut count_query = sqlx::QueryBuilder::new(
        r#"
        SELECT COUNT(*) AS count
        FROM weihuda_new.jifen_records
        "#,
    );

    let mut first_condition = true;

    if let Some(key) = key {
        if first_condition {
            main_query.push(" WHERE ");
            count_query.push(" WHERE ");
            first_condition = false;
        } else {
            main_query.push(" AND ");
            count_query.push(" AND ");
        }
        main_query
            .push("`key` LIKE ")
            .push_bind(format!("%{}%", key));
        count_query
            .push("`key` LIKE ")
            .push_bind(format!("%{}%", key));
    }

    if let Some(param) = param {
        if first_condition {
            main_query.push(" WHERE ");
            count_query.push(" WHERE ");
            first_condition = false;
        } else {
            main_query.push(" AND ");
            count_query.push(" AND ");
        }
        main_query
            .push("`param` LIKE ")
            .push_bind(format!("%{}%", param));
        count_query
            .push("`param` LIKE ")
            .push_bind(format!("%{}%", param));
    }

    if let Some(stu_id) = stu_id {
        if first_condition {
            main_query.push(" WHERE ");
            count_query.push(" WHERE ");
        } else {
            main_query.push(" AND ");
            count_query.push(" AND ");
        }
        main_query
            .push("stuId LIKE ")
            .push_bind(format!("%{}%", stu_id));
        count_query
            .push("stuId LIKE ")
            .push_bind(format!("%{}%", stu_id));
    }

    main_query.push(" ORDER BY id DESC");
    main_query.push(" LIMIT ").push_bind(page_size);
    main_query
        .push(" OFFSET ")
        .push_bind((page - 1) * page_size);

    let res = main_query
        .build()
        .fetch_all(get_db_pool().await)
        .await?
        .into_iter()
        .map(|r| JifenRecord {
            id: r.get("id"),
            key: r.get("key"),
            param: r.get("param"),
            stu_id: r.get("stuId"),
            desc: r.get("desc"),
            jifen: r.get("jifen"),
            created_at: r.get("createdAt"),
        })
        .collect::<Vec<_>>();

    let count: i64 = count_query
        .build_query_scalar()
        .fetch_one(get_db_pool().await)
        .await?;

    Ok((count as u32, res))
}

pub async fn get_record(id: u32) -> AppResult<Option<JifenRecord>> {
    let res = sqlx::query!(
        r#"
        SELECT id, `key`, `param`, stuId, `desc`, jifen, createdAt
        FROM weihuda_new.jifen_records
        WHERE id = ?
        "#,
        id
    )
    .fetch_optional(get_db_pool().await)
    .await?
    .map(|r| JifenRecord {
        id: r.id,
        key: r.key,
        param: r.param,
        stu_id: r.stuId,
        desc: r.desc,
        jifen: r.jifen,
        created_at: r.createdAt,
    });
    Ok(res)
}

pub async fn add_record(
    key: &str,
    param: &str,
    stu_id: &str,
    desc: &str,
    jifen: i32,
) -> AppResult<u32> {
    let now = utils::now_time();
    let res = sqlx::query!(
        r#"
        INSERT INTO weihuda_new.jifen_records (`key`, `param`, stuId, `desc`, jifen, createdAt)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
        key,
        param,
        stu_id,
        desc,
        jifen,
        now
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(res.last_insert_id() as u32)
}

pub async fn update_jifen(stu_id: &str, delta: i32) -> AppResult<()> {
    let now = utils::now_time();
    sqlx::query!(
        r#"
        UPDATE weihuda_new.mini_bind
        SET jifen = jifen + ?, updatedAt = ?
        WHERE stuId = ?
        "#,
        delta,
        now,
        stu_id
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(())
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JifenRule {
    pub id: u32,
    pub key: String,
    pub name: String,
    pub jifen: i32,
    pub cycle: u32,
    pub max_count: u32,
    pub is_show: bool,
}

pub async fn get_rule_list() -> AppResult<Vec<JifenRule>> {
    let res = sqlx::query!(
        r#"
        SELECT id, `key`, name, jifen, cycle, maxCount, isShow
        FROM weihuda_new.jifen_rules
        WHERE deletedAt IS NULL
        ORDER BY id DESC
        "#,
    )
    .fetch_all(get_db_pool().await)
    .await?
    .into_iter()
    .map(|r| JifenRule {
        id: r.id,
        key: r.key,
        name: r.name,
        jifen: r.jifen,
        cycle: r.cycle,
        max_count: r.maxCount,
        is_show: r.isShow != 0,
    })
    .collect::<Vec<_>>();
    Ok(res)
}

pub async fn add_rule(
    key: &str,
    name: &str,
    jifen: i32,
    cycle: u32,
    max_count: u32,
    is_show: bool,
) -> AppResult<u32> {
    let now = utils::now_time();
    let res = sqlx::query!(
        r#"
        INSERT INTO weihuda_new.jifen_rules (`key`, name, jifen, cycle, maxCount, isShow, createdAt, updatedAt)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        key,
        name,
        jifen,
        cycle,
        max_count,
        is_show as u32,
        now,
        now
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(res.last_insert_id() as u32)
}

pub async fn update_rule(
    id: u32,
    key: &str,
    name: &str,
    jifen: i32,
    cycle: u32,
    max_count: u32,
    is_show: bool,
) -> AppResult<()> {
    let now = utils::now_time();
    sqlx::query!(
        r#"
        UPDATE weihuda_new.jifen_rules
        SET `key` = ?, name = ?, jifen = ?, cycle = ?, maxCount = ?, isShow = ?, updatedAt = ?
        WHERE id = ? AND deletedAt IS NULL
        "#,
        key,
        name,
        jifen,
        cycle,
        max_count,
        is_show as u32,
        now,
        id
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(())
}

pub async fn delete_rule(id: u32) -> AppResult<()> {
    let now = utils::now_time();
    sqlx::query!(
        r#"
        UPDATE weihuda_new.jifen_rules
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
