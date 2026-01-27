use crate::utils;
use crate::{result::RouterResult, service};
use anyhow::anyhow;
use salvo::{handler, macros::Extractible};

const PERMISSION_PERMISSION_PREFIX: &str = "system:permission";

pub fn routers() -> salvo::Router {
    salvo::Router::with_path("permission")
        .get(get_permission_list)
        .post(post_permission)
        .push(
            salvo::Router::with_path("{id}")
                .put(put_permission)
                .delete(delete_permission),
        )
}

#[handler]
async fn get_permission_list(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req).await?.id)
        .await?
        .has(&format!("{}:query", PERMISSION_PERMISSION_PREFIX))
    {
        return Err(crate::result::AppError::PermissionDenied);
    }
    let res = service::qnxg::permission::get_permission_list().await?;
    Ok(res.into())
}

#[handler]
async fn post_permission(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req).await?.id)
        .await?
        .has(&format!("{}:add", PERMISSION_PERMISSION_PREFIX))
    {
        return Err(crate::result::AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "body"), rename_all = "camelCase"))]
    struct PostPermissionReq {
        name: String,
        permission: String,
    }
    let PostPermissionReq { name, permission } = req.extract().await?;
    let permission_list = service::qnxg::permission::get_permission_list().await?;
    if permission_list.iter().any(|p| p.permission == permission) {
        return Err(anyhow!("权限标识已存在").into());
    }
    let res = service::qnxg::permission::add_permission(&name, &permission).await?;
    let new_permission = service::qnxg::permission::get_permission_list()
        .await?
        .into_iter()
        .find(|p| p.id == res)
        .ok_or(anyhow!("新增权限失败"))?;
    Ok(new_permission.into())
}

#[handler]
async fn put_permission(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req).await?.id)
        .await?
        .has(&format!("{}:edit", PERMISSION_PERMISSION_PREFIX))
    {
        return Err(crate::result::AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "body"), rename_all = "camelCase"))]
    struct PutPermissionReq {
        #[salvo(source(from = "param"))]
        id: u32,
        name: String,
        permission: String,
    }
    let PutPermissionReq {
        id,
        name,
        permission,
    } = req.extract().await?;
    let list = service::qnxg::permission::get_permission_list().await?;
    if !list.iter().any(|p| p.id == id) {
        return Err(anyhow!("权限不存在").into());
    }
    service::qnxg::permission::update_permission(id, &name, &permission).await?;
    let new_permission = service::qnxg::permission::get_permission_list()
        .await?
        .into_iter()
        .find(|p| p.id == id)
        .ok_or(anyhow!("更新权限失败"))?;
    Ok(new_permission.into())
}

#[handler]
async fn delete_permission(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req).await?.id)
        .await?
        .has(&format!("{}:delete", PERMISSION_PERMISSION_PREFIX))
    {
        return Err(crate::result::AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "param")))]
    struct DeletePermissionReq {
        id: u32,
    }
    let DeletePermissionReq { id } = req.extract().await?;
    let list = service::qnxg::permission::get_permission_list().await?;
    if !list.iter().any(|p| p.id == id) {
        return Err(anyhow!("权限不存在").into());
    }
    service::qnxg::permission::delete_permission(id).await?;
    Ok(().into())
}
