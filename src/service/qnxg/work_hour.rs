use std::collections::HashMap;

use anyhow::anyhow;

pub use crate::infra::mysql::work_hour::{
    WorkDesc, WorkHourRecordStatus, WorkHourStatus, add_work_hour, delete_work_hour, get_work_hour,
    get_work_hour_list, update_work_hour,
};
use crate::service::qnxg::department::Department;
use crate::service::qnxg::user::User;
use crate::{infra, result::AppResult};
use crate::service;

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WorkHourRecord {
    #[serde(flatten)]
    pub info: infra::mysql::work_hour::WorkHourRecord,
    pub includes: Vec<WorkInclude>,
    pub user_info: User,
}

#[derive(serde::Serialize, Debug)]
pub struct WorkInclude {
    pub id: u32,
    pub hour: u32,
    pub user: User,
}

#[derive(serde::Deserialize, Debug)]
pub struct WorkHourTableItem {
    id: u32,
    includes: Vec<infra::mysql::work_hour::WorkInclude>,
}

impl WorkInclude {
    pub fn po(&self) -> infra::mysql::work_hour::WorkInclude {
        infra::mysql::work_hour::WorkInclude {
            id: self.id,
            hour: self.hour,
        }
    }
}

async fn get_work_hour_record_includes(
    record: &infra::mysql::work_hour::WorkHourRecord,
) -> AppResult<Vec<WorkInclude>> {
    let mut res = Vec::new();
    for include in infra::mysql::work_hour::get_work_hour_record_includes(record.id).await? {
        let Some(item) = infra::mysql::work_hour::get_work_hour_record_by_id(include.id).await?
        else {
            return Err(anyhow!("工时记录关联信息不存在").into());
        };
        let Some(user) = service::qnxg::user::get_user(item.user_id).await? else {
            return Err(anyhow!("工时记录关联用户不存在").into());
        };
        res.push(WorkInclude {
            id: include.id,
            hour: include.hour,
            user,
        });
    }
    Ok(res)
}

pub async fn get_work_hour_record(
    work_hour_id: u32,
    user_id: u32,
) -> AppResult<Option<WorkHourRecord>> {
    let Some(record) = infra::mysql::work_hour::get_work_hour_record(work_hour_id, user_id).await?
    else {
        return Ok(None);
    };
    let includes = get_work_hour_record_includes(&record).await?;
    let Some(user) = service::qnxg::user::get_user(user_id).await? else {
        return Err(anyhow!("工时记录关联用户不存在").into());
    };
    Ok(Some(WorkHourRecord {
        info: record,
        includes,
        user_info: user,
    }))
}

pub async fn get_my_work_hour_record(
    work_hour_id: u32,
    user_id: u32,
) -> AppResult<Option<WorkHourRecord>> {
    let Some(record) =
        infra::mysql::work_hour::get_my_work_hour_record(work_hour_id, user_id).await?
    else {
        return Ok(None);
    };
    let includes = get_work_hour_record_includes(&record).await?;
    let Some(user) = service::qnxg::user::get_user(user_id).await? else {
        return Err(anyhow!("工时记录关联用户不存在").into());
    };
    Ok(Some(WorkHourRecord {
        info: record,
        includes,
        user_info: user,
    }))
}

pub async fn submit_work_hour_record(
    work_hour_id: u32,
    user_id: u32,
    descs: &Vec<WorkDesc>,
) -> AppResult<u32> {
    let res = infra::mysql::work_hour::update_work_hour_record(
        work_hour_id,
        user_id,
        descs,
        None,
        None,
        WorkHourRecordStatus::PendingApproval,
    )
    .await?;
    Ok(res)
}

pub async fn get_work_hour_record_department_list(
    work_hour_id: u32,
    department_id: u32,
) -> AppResult<Vec<WorkHourRecord>> {
    let mut res = Vec::new();
    let records =
        infra::mysql::work_hour::get_work_hour_record_department_list(work_hour_id, department_id)
            .await?;
    for record in records {
        let includes = get_work_hour_record_includes(&record).await?;
        let Some(user) = service::qnxg::user::get_user(record.user_id).await? else {
            return Err(anyhow!("工时记录关联用户不存在").into());
        };
        res.push(WorkHourRecord {
            info: record,
            includes,
            user_info: user,
        });
    }
    Ok(res)
}

pub async fn get_work_hour_record_list(work_hour_id: u32) -> AppResult<Vec<WorkHourRecord>> {
    let mut res = Vec::new();
    let records = infra::mysql::work_hour::get_work_hour_record_list(work_hour_id).await?;
    for record in records {
        let includes = get_work_hour_record_includes(&record).await?;
        let Some(user) = service::qnxg::user::get_user(record.user_id).await? else {
            return Err(anyhow!("工时记录关联用户不存在").into());
        };
        res.push(WorkHourRecord {
            info: record,
            includes,
            user_info: user,
        });
    }
    Ok(res)
}

/// 工时审核通过
pub async fn accept_work_hour_record(record: &WorkHourRecord) -> AppResult<()> {
    let next_status = match record.info.status {
        WorkHourRecordStatus::PendingApproval => WorkHourRecordStatus::PendingFinance,
        WorkHourRecordStatus::PendingFinance => WorkHourRecordStatus::PendingDistribution,
        _ => {
            unreachable!()
        }
    };
    infra::mysql::work_hour::update_work_hour_record(
        record.info.work_hour_id,
        record.info.user_id,
        &record.info.work_descs,
        Some(
            record
                .includes
                .iter()
                .map(|i| i.po())
                .collect::<Vec<_>>()
                .as_ref(),
        ),
        None,
        next_status,
    )
    .await?;
    Ok(())
}

/// 用于财务一键通过所有待审核工时记录
pub async fn accept_all(work_hour_id: u32) -> AppResult<()> {
    let records = get_work_hour_record_list(work_hour_id).await?;
    for record in records {
        if record.info.status == WorkHourRecordStatus::PendingFinance {
            accept_work_hour_record(&record).await?;
        }
    }
    Ok(())
}

/// 用于财务一键设置所有待发放工时记录为已发放、
pub async fn close_all(work_hour_id: u32) -> AppResult<()> {
    let records = get_work_hour_record_list(work_hour_id).await?;
    for record in records {
        if record.info.status == WorkHourRecordStatus::PendingDistribution {
            close_work_hour_record(&record).await?;
        }
    }
    Ok(())
}

/// 打回工时记录
pub async fn reject_work_hour_record(record: &WorkHourRecord, comment: &str) -> AppResult<()> {
    infra::mysql::work_hour::update_work_hour_record(
        record.info.work_hour_id,
        record.info.user_id,
        &record.info.work_descs,
        Some(
            record
                .includes
                .iter()
                .map(|i| i.po())
                .collect::<Vec<_>>()
                .as_ref(),
        ),
        Some(comment),
        WorkHourRecordStatus::Unsubmitted,
    )
    .await?;
    Ok(())
}

/// 设置工时记录已经发放
pub async fn close_work_hour_record(record: &WorkHourRecord) -> AppResult<()> {
    infra::mysql::work_hour::update_work_hour_record(
        record.info.work_hour_id,
        record.info.user_id,
        &record.info.work_descs,
        Some(
            record
                .includes
                .iter()
                .map(|i| i.po())
                .collect::<Vec<_>>()
                .as_ref(),
        ),
        None,
        WorkHourRecordStatus::Closed,
    )
    .await?;
    Ok(())
}

/// 保存工时表，delta 为变更的工时记录
pub async fn save_work_hour_table(delta: &Vec<WorkHourTableItem>) -> AppResult<()> {
    for item in delta {
        let Some(record) = infra::mysql::work_hour::get_work_hour_record_by_id(item.id).await?
        else {
            return Err(anyhow!("工时记录不存在").into());
        };
        infra::mysql::work_hour::update_work_hour_record(
            record.work_hour_id,
            record.user_id,
            &record.work_descs,
            Some(item.includes.as_ref()),
            record.comment.as_deref(),
            record.status,
        )
        .await?;
    }
    Ok(())
}

#[derive(serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct WorkHourStatisticsItem {
    pub count: u32,
    pub total_hours: u32,
}

#[derive(serde::Serialize, Debug)]
pub struct WorkHourStatistics {
    pub department: Department,
    pub stats: WorkHourStatisticsItem,
}

/// 生成工时统计数据
pub async fn gen_work_hour_statistics(work_hour_id: u32) -> AppResult<Vec<WorkHourStatistics>> {
    let mut res = HashMap::new();
    let data = infra::mysql::work_hour::get_work_hour_record_list(work_hour_id).await?;
    for stat in data {
        let Some(user) = service::qnxg::user::get_user(stat.user_id).await? else {
            return Err(anyhow!("工时记录关联用户不存在").into());
        };
        let entry: &mut WorkHourStatisticsItem = res.entry(user.info.department_id).or_default();
        entry.count += 1;
        entry.total_hours += stat.work_descs.iter().map(|v| v.hour).sum::<u32>();
    }
    let mut final_res = Vec::new();
    for (dept_id, stats) in res {
        let Some(department) = service::qnxg::department::get_department_list()
            .await?
            .into_iter()
            .find(|d| d.id == dept_id)
        else {
            return Err(anyhow!("工时记录关联部门不存在").into());
        };
        final_res.push(WorkHourStatistics { department, stats });
    }
    Ok(final_res)
}
