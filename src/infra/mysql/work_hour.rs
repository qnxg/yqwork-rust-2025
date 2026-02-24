use anyhow::anyhow;

use super::get_db_pool;
use crate::{result::AppResult, utils};

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WorkHour {
    pub id: u32,
    pub name: String,
    pub end_time: chrono::NaiveDateTime,
    pub status: WorkHourStatus,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkHourStatus {
    // 未开始
    Pending,
    // 申报中
    Ongoing,
    // 申报结束
    Ended,
    // 已发放
    Closed,
}
impl From<u32> for WorkHourStatus {
    fn from(value: u32) -> Self {
        match value {
            0 => WorkHourStatus::Pending,
            1 => WorkHourStatus::Ongoing,
            2 => WorkHourStatus::Ended,
            4 => WorkHourStatus::Closed,
            _ => WorkHourStatus::Closed,
        }
    }
}
impl From<WorkHourStatus> for u32 {
    fn from(value: WorkHourStatus) -> Self {
        match value {
            WorkHourStatus::Pending => 0,
            WorkHourStatus::Ongoing => 1,
            WorkHourStatus::Ended => 2,
            WorkHourStatus::Closed => 4,
        }
    }
}
impl serde::Serialize for WorkHourStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let v = u32::from(*self);
        serializer.serialize_u32(v)
    }
}

pub async fn get_work_hour_list(page: u32, page_size: u32) -> AppResult<(u32, Vec<WorkHour>)> {
    let res = sqlx::query!(
        r#"
        SELECT id, name, endTime, status, comment
        FROM yqwork_new.work_hours
        WHERE deletedAt IS NULL
        ORDER BY id DESC
        LIMIT ? OFFSET ?
        "#,
        page_size,
        (page - 1) * page_size,
    )
    .fetch_all(get_db_pool().await)
    .await?
    .into_iter()
    .map(|r| WorkHour {
        id: r.id,
        name: r.name,
        end_time: r.endTime,
        status: WorkHourStatus::from(r.status),
        comment: r.comment,
    })
    .collect::<Vec<_>>();

    let total: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as count FROM yqwork_new.work_hours
        WHERE deletedAt IS NULL
        "#
    )
    .fetch_one(get_db_pool().await)
    .await?;

    Ok((total as u32, res))
}

pub async fn get_work_hour(id: u32) -> AppResult<Option<WorkHour>> {
    let res = sqlx::query!(
        r#"
        SELECT id, name, endTime, status, comment
        FROM yqwork_new.work_hours
        WHERE id = ? AND deletedAt IS NULL
        "#,
        id
    )
    .fetch_optional(get_db_pool().await)
    .await?
    .map(|r| WorkHour {
        id: r.id,
        name: r.name,
        end_time: r.endTime,
        status: WorkHourStatus::from(r.status),
        comment: r.comment,
    });
    Ok(res)
}

pub async fn add_work_hour(
    name: &str,
    end_time: &chrono::NaiveDateTime,
    status: WorkHourStatus,
    comment: Option<&str>,
) -> AppResult<u32> {
    let now = utils::now_time();
    let res = sqlx::query!(
        r#"
        INSERT INTO yqwork_new.work_hours (name, endTime, status, comment, createdAt, updatedAt)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
        name,
        end_time,
        u32::from(status),
        comment,
        now,
        now
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(res.last_insert_id() as u32)
}

pub async fn update_work_hour(
    work_hour_id: u32,
    name: &str,
    end_time: &chrono::NaiveDateTime,
    status: WorkHourStatus,
    comment: Option<&str>,
) -> AppResult<()> {
    let now = utils::now_time();
    sqlx::query!(
        r#"
        UPDATE yqwork_new.work_hours
        SET name = ?, endTime = ?, status = ?, comment = ?, updatedAt = ?
        WHERE id = ? AND deletedAt IS NULL
        "#,
        name,
        end_time,
        u32::from(status),
        comment,
        now,
        work_hour_id
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(())
}

pub async fn delete_work_hour(work_hour_id: u32) -> AppResult<()> {
    let now = utils::now_time();
    sqlx::query!(
        r#"
        UPDATE yqwork_new.work_hours
        SET deletedAt = ?
        WHERE id = ? AND deletedAt IS NULL
        "#,
        now,
        work_hour_id
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(())
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WorkHourRecord {
    pub id: u32,
    pub work_hour_id: u32,
    pub user_id: u32,
    pub work_descs: Vec<WorkDesc>,
    pub comment: Option<String>,
    pub status: WorkHourRecordStatus,
}
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct WorkDesc {
    pub desc: String,
    pub hour: u32,
}
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct WorkInclude {
    pub id: u32,
    pub hour: u32,
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum WorkHourRecordStatus {
    // 未提交
    Unsubmitted,
    // 等待部门负责人审核
    PendingApproval,
    // 等待财务审核
    PendingFinance,
    // 待发放
    PendingDistribution,
    // 已发放
    Closed,
}
impl From<u32> for WorkHourRecordStatus {
    fn from(value: u32) -> Self {
        match value {
            0 => WorkHourRecordStatus::Unsubmitted,
            1 => WorkHourRecordStatus::PendingApproval,
            2 => WorkHourRecordStatus::PendingFinance,
            3 => WorkHourRecordStatus::PendingDistribution,
            4 => WorkHourRecordStatus::Closed,
            _ => WorkHourRecordStatus::Closed,
        }
    }
}
impl From<WorkHourRecordStatus> for u32 {
    fn from(value: WorkHourRecordStatus) -> Self {
        match value {
            WorkHourRecordStatus::Unsubmitted => 0,
            WorkHourRecordStatus::PendingApproval => 1,
            WorkHourRecordStatus::PendingFinance => 2,
            WorkHourRecordStatus::PendingDistribution => 3,
            WorkHourRecordStatus::Closed => 4,
        }
    }
}
impl serde::Serialize for WorkHourRecordStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let v = u32::from(*self);
        serializer.serialize_u32(v)
    }
}

pub async fn get_work_hour_record(
    work_hour_id: u32,
    user_id: u32,
) -> AppResult<Option<WorkHourRecord>> {
    let res = sqlx::query!(
        r#"
        SELECT id, workHourId, userId, workDescs, includes, comment, status
        FROM yqwork_new.work_hours_records
        WHERE workHourId = ? AND userId = ? AND deletedAt IS NULL
        "#,
        work_hour_id,
        user_id
    )
    .fetch_optional(get_db_pool().await)
    .await?
    .map(|r| WorkHourRecord {
        id: r.id,
        work_hour_id: r.workHourId,
        user_id: r.userId,
        work_descs: serde_json::from_str::<Vec<WorkDesc>>(&r.workDescs).unwrap_or_default(),
        comment: r.comment,
        status: WorkHourRecordStatus::from(r.status),
    });
    Ok(res)
}

pub async fn get_work_hour_record_by_id(id: u32) -> AppResult<Option<WorkHourRecord>> {
    let res = sqlx::query!(
        r#"
        SELECT id, workHourId, userId, workDescs, includes, comment, status
        FROM yqwork_new.work_hours_records
        WHERE id = ? AND deletedAt IS NULL
        "#,
        id
    )
    .fetch_optional(get_db_pool().await)
    .await?
    .map(|r| WorkHourRecord {
        id: r.id,
        work_hour_id: r.workHourId,
        user_id: r.userId,
        work_descs: serde_json::from_str::<Vec<WorkDesc>>(&r.workDescs).unwrap_or_default(),
        comment: r.comment,
        status: WorkHourRecordStatus::from(r.status),
    });
    Ok(res)
}

/// 不校验 id 是否存在，需要确保 id 存在后再调用此函数
pub async fn get_work_hour_record_includes(id: u32) -> AppResult<Vec<WorkInclude>> {
    let res = sqlx::query!(
        r#"
        SELECT includes
        FROM yqwork_new.work_hours_records
        WHERE id = ? AND deletedAt IS NULL
        "#,
        id
    )
    .fetch_one(get_db_pool().await)
    .await?
    .includes;
    let includes = if let Some(inc) = res {
        serde_json::from_str::<Vec<WorkInclude>>(&inc).unwrap_or_default()
    } else {
        vec![]
    };
    Ok(includes)
}

/// 返回财务视角的工时记录列表
/// 只显示通过部门负责人审核的记录
/// 不提供 page_size 的话将不会开启分页，page 参数无效
pub async fn get_work_hour_record_list(work_hour_id: u32) -> AppResult<Vec<WorkHourRecord>> {
    let res = sqlx::query!(
        r#"
        SELECT id, workHourId, userId, workDescs, includes, comment, status
        FROM yqwork_new.work_hours_records
        WHERE workHourId = ? AND deletedAt IS NULL AND status >= 2
        ORDER BY status ASC, id DESC
        "#,
        work_hour_id,
    )
    .fetch_all(get_db_pool().await)
    .await?
    .into_iter()
    .map(|r| WorkHourRecord {
        id: r.id,
        work_hour_id: r.workHourId,
        user_id: r.userId,
        work_descs: serde_json::from_str::<Vec<WorkDesc>>(&r.workDescs).unwrap_or_default(),
        comment: r.comment,
        status: WorkHourRecordStatus::from(r.status),
    })
    .collect::<Vec<_>>();
    Ok(res)
}

/// 只显示已经提交，等待部门负责人审核的记录
pub async fn get_work_hour_record_department_list(
    work_hour_id: u32,
    department_id: u32,
) -> AppResult<Vec<WorkHourRecord>> {
    let res = sqlx::query!(
        r#"
        SELECT whr.id, whr.workHourId, whr.userId, whr.workDescs, whr.includes, whr.comment, whr.status
        FROM yqwork_new.work_hours_records whr
        INNER JOIN yqwork_new.users u
        ON whr.userId = u.id
        WHERE whr.workHourId = ? AND u.departmentId = ? AND whr.deletedAt IS NULL AND whr.status >= 1
        ORDER BY whr.status ASC, whr.id DESC
        "#,
        work_hour_id,
        department_id,
    ).fetch_all(get_db_pool().await).await?.into_iter().map(|r| WorkHourRecord {
        id: r.id,
        work_hour_id: r.workHourId,
        user_id: r.userId,
        work_descs: serde_json::from_str::<Vec<WorkDesc>>(&r.workDescs).unwrap_or_default(),
        comment: r.comment,
        status: WorkHourRecordStatus::from(r.status),
    }).collect::<Vec<_>>();
    Ok(res)
}

pub async fn get_my_work_hour_record(
    work_hour_id: u32,
    user_id: u32,
) -> AppResult<Option<WorkHourRecord>> {
    let res = sqlx::query!(
        r#"
        SELECT id, workHourId, userId, workDescs, includes, comment, status
        FROM yqwork_new.work_hours_records
        WHERE workHourId = ? AND userId = ? AND deletedAt IS NULL
        "#,
        work_hour_id,
        user_id
    )
    .fetch_optional(get_db_pool().await)
    .await?
    .map(|r| WorkHourRecord {
        id: r.id,
        work_hour_id: r.workHourId,
        user_id: r.userId,
        work_descs: serde_json::from_str::<Vec<WorkDesc>>(&r.workDescs).unwrap_or_default(),
        comment: r.comment,
        status: WorkHourRecordStatus::from(r.status),
    });
    Ok(res)
}

/// 如果对应的 user_id 和 work_hour_id 的记录已存在，则更新记录，否则新增记录
pub async fn update_work_hour_record(
    work_hour_id: u32,
    user_id: u32,
    work_descs: &Vec<WorkDesc>,
    includes: Option<&Vec<WorkInclude>>,
    comment: Option<&str>,
    status: WorkHourRecordStatus,
) -> AppResult<u32> {
    let now = utils::now_time();
    let res = if let Some(id) = sqlx::query_scalar!(
        r#"
        SELECT id
        FROM yqwork_new.work_hours_records
        WHERE workHourId = ? AND userId = ? AND deletedAt IS NULL
        "#,
        work_hour_id,
        user_id
    )
    .fetch_optional(get_db_pool().await)
    .await?
    {
        sqlx::query!(
            r#"
            UPDATE yqwork_new.work_hours_records
            SET workDescs = ?, includes = ?, comment = ?, status = ?, updatedAt = ?
            WHERE id = ? AND deletedAt IS NULL
            "#,
            serde_json::to_string(work_descs)
                .map_err(|err| anyhow!("更新工时记录时失败：序列化工时明细错误 {:?}", err))?,
            includes
                .as_ref()
                .map(|inc| serde_json::to_string(inc))
                .transpose()
                .map_err(|err| anyhow!("更新工时记录时失败：序列化包含错误 {:?}", err))?,
            comment,
            u32::from(status),
            now,
            id
        )
        .execute(get_db_pool().await)
        .await?;
        id
    } else {
        let res = sqlx::query!(
            r#"
            INSERT INTO yqwork_new.work_hours_records (workHourId, userId, workDescs, includes, comment, status, createdAt, updatedAt)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            work_hour_id,
            user_id,
            serde_json::to_string(work_descs).map_err(|err| anyhow!("新增工时记录时失败：序列化工时明细错误 {:?}", err))?,
            includes.as_ref().map(|inc| serde_json::to_string(inc)).transpose().map_err(|err| anyhow!("新增工时记录时失败：序列化包含错误 {:?}", err))?,
            comment,
            u32::from(status),
            now,
            now
        ).execute(get_db_pool().await).await?;
        res.last_insert_id() as u32
    };
    Ok(res)
}
