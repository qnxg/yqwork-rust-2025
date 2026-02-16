use crate::result::AppError;
use crate::utils;
use anyhow::anyhow;
use salvo::{handler, macros::Extractible};
use serde_json::json;

use crate::{result::RouterResult, service};

const ANNOUNCEMENT_PERMISSION_PREFIX: &str = "hdwsh:announcement";

pub fn routers() -> salvo::Router {
    salvo::Router::with_path("announcement")
        .get(get_announcement_list)
        .post(post_announcement)
        .push(
            salvo::Router::with_path("{id}")
                .get(get_announcement)
                .put(put_announcement)
                .delete(delete_announcement),
        )
}

#[handler]
async fn get_announcement_list(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req).await?.id)
        .await?
        .has(&format!("{}:query", ANNOUNCEMENT_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "query"), rename_all = "camelCase"))]
    struct GetAnnouncementListReq {
        page: Option<u32>,
        page_size: Option<u32>,
    }
    let GetAnnouncementListReq { page, page_size } = req.extract().await?;
    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(10);
    let (count, rows) =
        service::weihuda::announcement::get_announcement_list(page, page_size).await?;
    Ok(json!({
        "count": count,
        "rows": rows,
    })
    .into())
}

#[handler]
async fn get_announcement(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req).await?.id)
        .await?
        .has(&format!("{}:query", ANNOUNCEMENT_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "body"), rename_all = "camelCase"))]
    struct GetAnnouncementReq {
        id: u32,
    }
    let GetAnnouncementReq { id } = req.extract().await?;
    let announcement = service::weihuda::announcement::get_announcement(id).await?;
    Ok(announcement.into())
}

#[handler]
async fn post_announcement(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req).await?.id)
        .await?
        .has(&format!("{}:add", ANNOUNCEMENT_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "body"), rename_all = "camelCase"))]
    struct PostAnnouncementReq {
        title: String,
        content: String,
        url: Option<String>,
    }
    let PostAnnouncementReq {
        title,
        content,
        url,
    } = req.extract().await?;
    let res =
        service::weihuda::announcement::add_announcement(&title, &content, url.as_deref()).await?;
    let new_announcement = service::weihuda::announcement::get_announcement(res).await?;
    Ok(new_announcement.into())
}

#[handler]
pub async fn put_announcement(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req).await?.id)
        .await?
        .has(&format!("{}:edit", ANNOUNCEMENT_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "body"), rename_all = "camelCase"))]
    struct PutAnnouncementReq {
        #[salvo(extract(source(from = "param")))]
        id: u32,
        title: String,
        content: String,
        url: Option<String>,
    }
    let PutAnnouncementReq {
        id,
        title,
        content,
        url,
    } = req.extract().await?;
    if service::weihuda::announcement::get_announcement(id)
        .await?
        .is_none()
    {
        return Err(anyhow!("公告不存在").into());
    }
    service::weihuda::announcement::update_announcement(id, &title, &content, url.as_deref())
        .await?;
    Ok(().into())
}

#[handler]
pub async fn delete_announcement(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req).await?.id)
        .await?
        .has(&format!("{}:delete", ANNOUNCEMENT_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "param")))]
    struct DeleteAnnouncementReq {
        id: u32,
    }
    let DeleteAnnouncementReq { id } = req.extract().await?;
    if service::weihuda::announcement::get_announcement(id)
        .await?
        .is_none()
    {
        return Err(anyhow!("公告不存在").into());
    }
    service::weihuda::announcement::delete_announcement(id).await?;
    Ok(().into())
}
