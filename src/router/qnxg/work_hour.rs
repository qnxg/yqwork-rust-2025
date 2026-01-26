use crate::service::qnxg::work_hour::{
    WorkDesc, WorkHourRecordStatus, WorkHourStatus, WorkInclude,
};
use crate::{
    result::{AppError, RouterResult},
    service, utils,
};
use anyhow::anyhow;
use salvo::{handler, macros::Extractible};
use serde_json::json;

const WORK_HOUR_PERMISSION_PREFIX: &str = "yq:workHours";

pub fn routers() -> salvo::Router {
    salvo::Router::new()
        .push(
            salvo::Router::with_path("work-hours")
                .get(get_work_hour_list)
                .post(post_work_hour)
                .push(
                    salvo::Router::with_path("{id}")
                        .get(get_work_hour)
                        .put(put_work_hour)
                        .delete(delete_work_hour),
                ),
        )
        .push(
            salvo::Router::with_path("work-hours-record")
                .get(get_work_hour_record_list)
                .put(put_work_hour_record)
                .push(
                    salvo::Router::with_path("department")
                        .get(get_work_hour_record_department_list),
                )
                .push(
                    salvo::Router::with_path("my")
                        .get(get_my_work_hour_record)
                        .put(put_my_work_hour_record),
                )
                .push(salvo::Router::with_path("save").put(save_work_hour_table)),
        )
}

#[handler]
async fn get_work_hour_list(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req).await?.id)
        .await?
        .has(&format!("{}:query", WORK_HOUR_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source = "query", rename_all = "camelCase"))]
    struct GetWorkHourListReq {
        page: Option<u32>,
        page_size: Option<u32>,
    }
    let GetWorkHourListReq { page, page_size } = req.extract().await?;
    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(10);
    let (count, rows) = service::qnxg::work_hour::get_work_hour_list(page, page_size).await?;
    Ok(json!({
        "count": count,
        "rows": rows,
    })
    .into())
}

#[handler]
async fn get_work_hour(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req).await?.id)
        .await?
        .has(&format!("{}:query", WORK_HOUR_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source = "param"))]
    struct GetWorkHourReq {
        id: u32,
    }
    let GetWorkHourReq { id } = req.extract().await?;
    let work_hour = service::qnxg::work_hour::get_work_hour(id).await?;
    Ok(work_hour.into())
}

#[handler]
async fn post_work_hour(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req).await?.id)
        .await?
        .has(&format!("{}:add", WORK_HOUR_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source = "body", rename_all = "camelCase"))]
    struct PostWorkHourReq {
        name: String,
        end_time: String,
        status: u32,
        comment: Option<String>,
    }
    let PostWorkHourReq {
        name,
        end_time,
        status,
        comment,
    } = req.extract().await?;
    let status = WorkHourStatus::from(status);
    let end_time = chrono::NaiveDateTime::parse_from_str(&end_time, "%Y-%m-%dT%H:%M")
        .map_err(|_| AppError::ParamParseError)?;
    service::qnxg::work_hour::add_work_hour(&name, &end_time, status, comment.as_deref()).await?;
    Ok(().into())
}

#[handler]
async fn put_work_hour(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req).await?.id)
        .await?
        .has(&format!("{}:edit", WORK_HOUR_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source = "body", rename_all = "camelCase"))]
    struct PutWorkHourReq {
        #[salvo(extract(source(from = "param")))]
        id: u32,
        name: String,
        end_time: String,
        status: u32,
        comment: Option<String>,
    }
    let PutWorkHourReq {
        id,
        name,
        end_time,
        status,
        comment,
    } = req.extract().await?;
    let status = WorkHourStatus::from(status);
    let end_time = chrono::NaiveDateTime::parse_from_str(&end_time, "%Y-%m-%dT%H:%M")
        .map_err(|_| AppError::ParamParseError)?;
    if service::qnxg::work_hour::get_work_hour(id).await?.is_none() {
        return Err(anyhow!("工时记录不存在").into());
    }
    service::qnxg::work_hour::update_work_hour(id, &name, &end_time, status, comment.as_deref())
        .await?;
    Ok(().into())
}

#[handler]
async fn delete_work_hour(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req).await?.id)
        .await?
        .has(&format!("{}:delete", WORK_HOUR_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source = "param"))]
    struct DeleteWorkHourReq {
        id: u32,
    }
    let DeleteWorkHourReq { id } = req.extract().await?;
    if service::qnxg::work_hour::get_work_hour(id).await?.is_none() {
        return Err(anyhow!("工时记录不存在").into());
    }
    service::qnxg::work_hour::delete_work_hour(id).await?;
    Ok(().into())
}

#[handler]
async fn get_work_hour_record_list(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req).await?.id)
        .await?
        .has(&format!("{}:generateTable", WORK_HOUR_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source = "query", rename_all = "camelCase"))]
    struct GetWorkHourRecordListReq {
        page: Option<u32>,
        page_size: Option<u32>,
        work_hour_id: u32,
    }
    let GetWorkHourRecordListReq {
        page,
        page_size,
        work_hour_id,
    } = req.extract().await?;
    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(10);
    let (count, rows) =
        service::qnxg::work_hour::get_work_hour_record_list(page, page_size, work_hour_id).await?;
    Ok(json!({
        "count": count,
        "rows": rows,
    })
    .into())
}

#[handler]
async fn put_work_hour_record(req: &mut salvo::Request) -> RouterResult {
    // 主要是进行打回和批准
    let user = utils::auth::parse_token(req).await?;
    let permission = service::qnxg::user::get_user_permission(user.id).await?;
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source = "body", rename_all = "camelCase"))]
    struct PutWorkHourRecordReq {
        #[salvo(extract(source(from = "query")))]
        work_hour_id: u32,
        #[salvo(extract(source(from = "query")))]
        user_id: u32,
        status: u32,
        // 打回的时候必须为 Some
        comment: Option<String>,
    }
    let PutWorkHourRecordReq {
        work_hour_id,
        user_id,
        status,
        comment,
    } = req.extract().await?;
    let status = WorkHourRecordStatus::from(status);
    let Some(record) =
        service::qnxg::work_hour::get_work_hour_record(work_hour_id, user_id).await?
    else {
        return Err(anyhow!("工时记录不存在").into());
    };
    let Some(target_user) = service::qnxg::user::get_user(user_id).await? else {
        return Err(anyhow!("工时记录不存在").into());
    };
    match status {
        WorkHourRecordStatus::Unsubmitted => {
            // 打回
            match record.status {
                WorkHourRecordStatus::PendingApproval => {
                    // 需要是部门负责人打回
                    if !permission.has(&format!("{}:checkDepartment", WORK_HOUR_PERMISSION_PREFIX))
                        || target_user.info.department_id != user.info.department_id
                    {
                        return Err(AppError::PermissionDenied);
                    }
                }
                WorkHourRecordStatus::PendingFinance => {
                    // 需要是财务负责人打回
                    if !permission.has(&format!("{}:generateTable", WORK_HOUR_PERMISSION_PREFIX)) {
                        return Err(AppError::PermissionDenied);
                    }
                }
                _ => return Err(AppError::PermissionDenied),
            }
            if let Some(comment) = comment {
                service::qnxg::work_hour::reject_work_hour_record(&record, &comment).await?;
            } else {
                return Err(AppError::ParamParseError);
            }
        }
        WorkHourRecordStatus::PendingFinance => {
            // 部门负责人批准
            if record.status != WorkHourRecordStatus::PendingApproval
                || !permission.has(&format!("{}:checkDepartment", WORK_HOUR_PERMISSION_PREFIX))
                || target_user.info.department_id != user.info.department_id
            {
                return Err(AppError::PermissionDenied);
            }
            service::qnxg::work_hour::accept_work_hour_record(&record).await?;
        }
        WorkHourRecordStatus::PendingDistribution => {
            // 财务负责人批准
            if record.status != WorkHourRecordStatus::PendingFinance
                || !permission.has(&format!("{}:generateTable", WORK_HOUR_PERMISSION_PREFIX))
            {
                return Err(AppError::PermissionDenied);
            }
            service::qnxg::work_hour::accept_work_hour_record(&record).await?;
        }
        WorkHourRecordStatus::Closed => {
            // 财务负责人设置已发放
            if record.status != WorkHourRecordStatus::PendingDistribution
                || !permission.has(&format!("{}:generateTable", WORK_HOUR_PERMISSION_PREFIX))
            {
                return Err(AppError::PermissionDenied);
            }
            service::qnxg::work_hour::close_work_hour_record(&record).await?;
        }
        _ => {
            return Err(AppError::ParamParseError);
        }
    }
    Ok(().into())
}

#[handler]
async fn get_work_hour_record_department_list(req: &mut salvo::Request) -> RouterResult {
    let user = utils::auth::parse_token(req).await?;
    if !service::qnxg::user::get_user_permission(user.id)
        .await?
        .has(&format!("{}:checkDepartment", WORK_HOUR_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source = "query", rename_all = "camelCase"))]
    struct GetWorkHourRecordDepartmentListReq {
        page: Option<u32>,
        page_size: Option<u32>,
        work_hour_id: u32,
    }
    let GetWorkHourRecordDepartmentListReq {
        page,
        page_size,
        work_hour_id,
    } = req.extract().await?;
    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(10);
    let (count, rows) = service::qnxg::work_hour::get_work_hour_record_department_list(
        page,
        page_size,
        work_hour_id,
        user.info.department_id,
    )
    .await?;
    Ok(json!({
        "count": count,
        "rows": rows,
    })
    .into())
}

#[handler]
async fn get_my_work_hour_record(req: &mut salvo::Request) -> RouterResult {
    let user_id = utils::auth::parse_token(req).await?.id;
    if !service::qnxg::user::get_user_permission(user_id)
        .await?
        .has(&format!("{}:query", WORK_HOUR_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source = "query", rename_all = "camelCase"))]
    struct GetMyWorkHourRecordReq {
        work_hour_id: u32,
    }
    let GetMyWorkHourRecordReq { work_hour_id } = req.extract().await?;
    let res = service::qnxg::work_hour::get_my_work_hour_record(work_hour_id, user_id).await?;
    Ok(res.into())
}

#[handler]
async fn put_my_work_hour_record(req: &mut salvo::Request) -> RouterResult {
    let user_id = utils::auth::parse_token(req).await?.id;
    if !service::qnxg::user::get_user_permission(user_id)
        .await?
        .has(&format!("{}:query", WORK_HOUR_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source = "body", rename_all = "camelCase"))]
    struct PutMyWorkHourRecordReq {
        #[salvo(extract(source(from = "query")))]
        work_hour_id: u32,
        work_descs: Vec<WorkDesc>,
    }
    let PutMyWorkHourRecordReq {
        work_hour_id,
        work_descs,
    } = req.extract().await?;
    if work_descs.is_empty() {
        return Err(AppError::ParamParseError);
    }
    // 已经提交的就不能改了
    let last_record =
        service::qnxg::work_hour::get_my_work_hour_record(work_hour_id, user_id).await?;
    if last_record.is_some_and(|r| r.status != WorkHourRecordStatus::Unsubmitted) {
        return Err(anyhow!("已提交的工时记录不能修改").into());
    }
    service::qnxg::work_hour::submit_work_hour_record(work_hour_id, user_id, &work_descs).await?;
    Ok(().into())
}

#[handler]
async fn save_work_hour_table(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req).await?.id)
        .await?
        .has(&format!("{}:generateTable", WORK_HOUR_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Debug)]
    struct WorkHourTableItem {
        id: u32,
        includes: Vec<WorkInclude>,
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source = "body"))]
    struct SaveWorkHourTableReq {
        data: Vec<WorkHourTableItem>,
    }
    let SaveWorkHourTableReq { data } = req.extract().await?;
    let mut table = Vec::new();
    for item in data {
        let Some(record) = service::qnxg::work_hour::get_work_hour_record_by_id(item.id).await?
        else {
            return Err(anyhow!("工时记录不存在").into());
        };
        table.push((record, item.includes));
    }
    service::qnxg::work_hour::save_work_hour_table(&table).await?;
    Ok(().into())
}
