use salvo::{handler, macros::Extractible};

use crate::{
    result::{AppError, RouterResult},
    service, utils,
};

const MINI_CONFIG_PERMISSION_PREFIX: &str = "hdwsh:miniConfig";

pub fn routers() -> salvo::Router {
    salvo::Router::with_path("mini-config")
        .get(get_mini_config)
        .put(put_mini_config)
}

#[handler]
async fn get_mini_config(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req)?)
        .await?
        .has(&format!("{}:query", MINI_CONFIG_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    let res = service::weihuda::config::get_mini_config().await?;
    Ok(res.into())
}

#[handler]
async fn put_mini_config(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req)?)
        .await?
        .has(&format!("{}:edit", MINI_CONFIG_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(from = "json"))]
    struct UpdateMiniConfigReq {
        key: String,
        value: String,
    }
    let UpdateMiniConfigReq { key, value } = req.extract().await?;
    service::weihuda::config::update_mini_config(&key, &value).await?;
    Ok(().into())
}
