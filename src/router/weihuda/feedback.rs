use anyhow::anyhow;
use salvo::{handler, macros::Extractible};
use serde_json::json;

use crate::{
    result::{AppError, RouterResult},
    service::{
        self,
        weihuda::feedback::{FeedbackMsgType, FeedbackStatus},
    },
    utils,
};

const PERMISSION_PREFIX: &str = "hdwsh:feedback";

pub fn routers() -> salvo::Router {
    salvo::Router::with_path("feedback")
        .get(get_feedback_list)
        .push(
            salvo::Router::with_path("{id}")
                .get(get_feedback)
                .put(put_feedback)
                .delete(delete_feedback)
                .push(
                    salvo::Router::with_path("msg")
                        .get(get_feedback_msg_list)
                        .post(add_feedback_msg)
                        .push(salvo::Router::with_path("{msg_id}").delete(delete_feedback_msg)),
                ),
        )
}

#[handler]
async fn get_feedback_list(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req).await?.id)
        .await?
        .has(&format!("{}:query", PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "query"), rename_all = "camelCase"))]
    struct GetFeedbackListReq {
        stu_id: Option<String>,
        page: Option<u32>,
        page_size: Option<u32>,
        status: Option<u32>,
        from: Option<String>,
        to: Option<String>,
    }
    let GetFeedbackListReq {
        stu_id,
        page,
        page_size,
        status,
        from,
        to,
    } = req.extract().await?;
    let status = status.map(FeedbackStatus::from);
    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(10);
    let (count, rows) =
        service::weihuda::feedback::get_feedback_list(page, page_size, stu_id, status, from, to)
            .await?;
    Ok(json!({
        "count": count,
        "rows": rows,
    })
    .into())
}

#[handler]
async fn get_feedback(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req).await?.id)
        .await?
        .has(&format!("{}:query", PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "param")))]
    struct GetFeedbackReq {
        id: u32,
    }
    let GetFeedbackReq { id } = req.extract().await?;
    let feedback = service::weihuda::feedback::get_feedback(id).await?;
    Ok(feedback.into())
}

#[handler]
async fn put_feedback(req: &mut salvo::Request) -> RouterResult {
    let user = utils::auth::parse_token(req).await?;
    if !service::qnxg::user::get_user_permission(user.id)
        .await?
        .has(&format!("{}:edit", PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "body")))]
    struct PutFeedbackReq {
        #[salvo(extract(source(from = "param")))]
        id: u32,
        status: u32,
    }
    let PutFeedbackReq { id, status } = req.extract().await?;
    if !matches!(status, 0..=3) {
        return Err(AppError::ParamParseError);
    }
    let feedback = service::weihuda::feedback::get_feedback(id).await?;
    if feedback.is_none() {
        return Err(anyhow!("反馈不存在").into());
    }
    service::weihuda::feedback::update_feedback(id, FeedbackStatus::from(status)).await?;
    let new_feedback = service::weihuda::feedback::get_feedback(id)
        .await?
        .ok_or(anyhow!("更新问题反馈失败"))?;
    Ok(new_feedback.into())
}

#[handler]
async fn delete_feedback(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req).await?.id)
        .await?
        .has(&format!("{}:delete", PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "param")))]
    struct DeleteFeedbackReq {
        id: u32,
    }
    let DeleteFeedbackReq { id } = req.extract().await?;
    let feedback = service::weihuda::feedback::get_feedback(id).await?;
    if feedback.is_none() {
        return Err(anyhow!("反馈不存在").into());
    }
    service::weihuda::feedback::delete_feedback(id).await?;
    Ok(().into())
}

#[handler]
async fn get_feedback_msg_list(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req).await?.id)
        .await?
        .has(&format!("{}:query", PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "param")))]
    struct GetFeedbackMsgListReq {
        id: u32,
    }
    let GetFeedbackMsgListReq { id } = req.extract().await?;
    let res = service::weihuda::feedback::get_feedback_msg_list(id).await?;
    Ok(res.into())
}

#[handler]
async fn add_feedback_msg(req: &mut salvo::Request) -> RouterResult {
    let user = utils::auth::parse_token(req).await?;
    if !service::qnxg::user::get_user_permission(user.id)
        .await?
        .has(&format!("{}:edit", PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "body"), rename_all = "camelCase"))]
    struct AddFeedbackMsgReq {
        #[salvo(extract(source(from = "param")))]
        id: u32,
        typ: String,
        msg: Option<String>,
    }
    let AddFeedbackMsgReq { id, typ, msg } = req.extract().await?;
    let Some(feedback) = service::weihuda::feedback::get_feedback(id).await? else {
        return Err(anyhow!("反馈不存在").into());
    };
    let feedback_msg_id = service::weihuda::feedback::add_feedback_msg(
        FeedbackMsgType::from(typ),
        msg.as_deref(),
        &feedback,
        &user,
    )
    .await?;
    Ok(feedback_msg_id.into())
}

#[handler]
async fn delete_feedback_msg(req: &mut salvo::Request) -> RouterResult {
    let user = utils::auth::parse_token(req).await?;
    if !service::qnxg::user::get_user_permission(user.id)
        .await?
        .has(&format!("{}:delete", PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "param")))]
    struct DeleteFeedbackMsgReq {
        id: u32,
        msg_id: u32,
    }
    let DeleteFeedbackMsgReq { id, msg_id } = req.extract().await?;
    if service::weihuda::feedback::get_feedback(id)
        .await?
        .is_none()
    {
        return Err(anyhow!("反馈不存在").into());
    }
    if !service::weihuda::feedback::get_feedback_msg_list(id)
        .await?
        .iter()
        .any(|msg| msg.id == msg_id)
    {
        return Err(anyhow!("反馈消息不存在").into());
    }
    service::weihuda::feedback::delete_feedback_msg(msg_id).await?;
    Ok(().into())
}
