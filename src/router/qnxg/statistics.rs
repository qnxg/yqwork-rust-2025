use salvo::handler;

use crate::{
    result::{AppError, RouterResult},
    service, utils,
};

pub fn routers() -> salvo::Router {
    salvo::Router::with_path("statistics").get(get_statistics)
}

#[handler]
async fn get_statistics(req: &mut salvo::Request) -> RouterResult {
    // 微生活统计
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req).await?.id)
        .await?
        .has("hdwsh:statistics:query")
    {
        return Err(AppError::PermissionDenied);
    }
    let res = service::qnxg::statistics::get_statistics().await?;
    Ok(res.into())
}
