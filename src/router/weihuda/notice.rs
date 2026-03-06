use crate::{
    infra::mysql::notice::NoticeStatus,
    result::{AppError, RouterResult},
    service, utils,
};
use anyhow::anyhow;
use salvo::{handler, macros::Extractible};
use serde_json::json;

const NOTICE_PERMISSION_PREFIX: &str = "hdwsh:notice";

pub fn routers() -> salvo::Router {
    salvo::Router::with_path("notice")
        .get(get_list)
        .post(post)
        .push(salvo::Router::with_path("{id}").get(get).delete(delete))
}

#[handler]
async fn get_list(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req).await?.id)
        .await?
        .has(&format!("{}:query", NOTICE_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "query"), rename_all = "camelCase"))]
    struct GetNoticeListReq {
        page: Option<u32>,
        page_size: Option<u32>,
        stu_id: Option<String>,
        status: Option<u32>,
        from: Option<String>,
        to: Option<String>,
    }
    let GetNoticeListReq {
        page,
        page_size,
        stu_id,
        status,
        from,
        to,
    } = req.extract().await?;
    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(10);
    let status = status.map(NoticeStatus::from);
    dbg!(&status);
    let (count, rows) =
        service::weihuda::notice::get_notice_list(page, page_size, stu_id, status, from, to)
            .await?;
    Ok(json!({
        "rows": rows,
        "count": count
    })
    .into())
}

#[handler]
async fn get(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req).await?.id)
        .await?
        .has(&format!("{}:query", NOTICE_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "param")))]
    struct GetNoticeReq {
        id: u32,
    }
    let GetNoticeReq { id } = req.extract().await?;
    let notice = service::weihuda::notice::get_notice(id).await?;
    Ok(notice.into())
}

#[handler]
async fn post(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req).await?.id)
        .await?
        .has(&format!("{}:add", NOTICE_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "body"), rename_all = "camelCase"))]
    struct PostNoticeReq {
        stu_id: String,
        content: String,
        is_show: bool,
        url: Option<String>,
    }
    let PostNoticeReq {
        stu_id,
        content,
        is_show,
        url,
    } = req.extract().await?;
    let notice_id =
        service::weihuda::notice::add_notice(&stu_id, &content, is_show, url.as_deref()).await?;
    let new_notice = service::weihuda::notice::get_notice(notice_id)
        .await?
        .ok_or(anyhow!("添加消息失败"))?;
    Ok(new_notice.into())
}

#[handler]
async fn delete(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req).await?.id)
        .await?
        .has(&format!("{}:delete", NOTICE_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "param")))]
    struct DeleteNoticeReq {
        id: u32,
    }
    let DeleteNoticeReq { id } = req.extract().await?;
    service::weihuda::notice::delete_notice(id).await?;
    Ok(().into())
}
