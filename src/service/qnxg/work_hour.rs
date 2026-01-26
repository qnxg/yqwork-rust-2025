pub use crate::infra::mysql::work_hour::{
    WorkDesc, WorkHourRecord, WorkHourRecordStatus, WorkHourStatus, WorkInclude, add_work_hour,
    delete_work_hour, get_my_work_hour_record, get_work_hour, get_work_hour_list,
    get_work_hour_record, get_work_hour_record_by_id, get_work_hour_record_department_list,
    get_work_hour_record_list, update_work_hour,
};
use crate::{infra, result::AppResult};

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

/// 工时审核通过
pub async fn accept_work_hour_record(record: &WorkHourRecord) -> AppResult<()> {
    let next_status = match record.status {
        WorkHourRecordStatus::PendingApproval => WorkHourRecordStatus::PendingFinance,
        WorkHourRecordStatus::PendingFinance => WorkHourRecordStatus::PendingDistribution,
        _ => {
            unreachable!()
        }
    };
    infra::mysql::work_hour::update_work_hour_record(
        record.work_hour_id,
        record.user_id,
        &record.work_descs,
        record.includes.as_ref(),
        None,
        next_status,
    )
    .await?;
    Ok(())
}

/// 打回工时记录
pub async fn reject_work_hour_record(record: &WorkHourRecord, comment: &str) -> AppResult<()> {
    infra::mysql::work_hour::update_work_hour_record(
        record.work_hour_id,
        record.user_id,
        &record.work_descs,
        record.includes.as_ref(),
        Some(comment),
        WorkHourRecordStatus::Unsubmitted,
    )
    .await?;
    Ok(())
}

/// 设置工时记录已经发放
pub async fn close_work_hour_record(record: &WorkHourRecord) -> AppResult<()> {
    infra::mysql::work_hour::update_work_hour_record(
        record.work_hour_id,
        record.user_id,
        &record.work_descs,
        record.includes.as_ref(),
        None,
        WorkHourRecordStatus::Closed,
    )
    .await?;
    Ok(())
}

/// 保存工时表，delta 为变更的工时记录
pub async fn save_work_hour_table(
    delta: &Vec<(WorkHourRecord, Vec<WorkInclude>)>,
) -> AppResult<()> {
    for (record, includes) in delta {
        infra::mysql::work_hour::update_work_hour_record(
            record.work_hour_id,
            record.user_id,
            &record.work_descs,
            Some(includes),
            record.comment.as_deref(),
            record.status,
        )
        .await?;
    }
    Ok(())
}
