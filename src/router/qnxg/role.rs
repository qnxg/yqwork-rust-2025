use crate::result::{AppError, RouterResult};
use crate::service;
use crate::service::qnxg::permission::PermissionItem;
use anyhow::anyhow;
use salvo::handler;
use salvo::macros::Extractible;

const ROLE_PERMISSION_PREFIX: &str = "system:role";

pub fn routers() -> salvo::Router {
    salvo::Router::with_path("role")
        .get(get_role_list)
        .post(post_role)
        .push(
            salvo::Router::with_path("{id}")
                .put(put_role)
                .delete(delete_role),
        )
}

#[handler]
async fn get_role_list(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(crate::utils::auth::parse_token(req)?)
        .await?
        .has(&format!("{}:query", ROLE_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Serialize)]
    struct RoleWithPermission {
        id: u32,
        name: String,
        permissions: Vec<PermissionItem>,
    }
    let mut res: Vec<RoleWithPermission> = Vec::new();
    let list = service::qnxg::role::get_role_list().await?;
    for role in list {
        let permissions = service::qnxg::role::get_role_permission(&[role.id]).await?;
        res.push(RoleWithPermission {
            id: role.id,
            name: role.name,
            permissions: permissions.into_inner(),
        });
    }
    Ok(res.into())
}

#[handler]
async fn post_role(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(crate::utils::auth::parse_token(req)?)
        .await?
        .has(&format!("{}:query", ROLE_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, salvo::macros::Extractible, Debug)]
    #[salvo(extract(default_source(from = "body"), rename_all = "camelCase"))]
    struct PostRoleReq {
        name: String,
        permission_ids: Vec<u32>,
    }
    let PostRoleReq {
        name,
        permission_ids,
    } = req.extract().await?;
    let permission_list = service::qnxg::permission::get_permission_list()
        .await?
        .into_iter()
        .map(|v| v.id)
        .collect::<Vec<u32>>();
    if !permission_ids.iter().all(|v| permission_list.contains(v)) {
        return Err(anyhow!("权限不存在").into());
    }
    service::qnxg::role::add_role(&name, &permission_ids).await?;
    Ok(().into())
}

#[handler]
async fn put_role(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(crate::utils::auth::parse_token(req)?)
        .await?
        .has(&format!("{}:query", ROLE_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "body"), rename_all = "camelCase"))]
    struct PutRoleReq {
        #[salvo(source(from = "param"))]
        id: u32,
        name: String,
        permission_ids: Vec<u32>,
    }
    let PutRoleReq {
        id,
        name,
        permission_ids,
    } = req.extract().await?;
    let list = service::qnxg::role::get_role_list().await?;
    if !list.iter().any(|r| r.id == id) {
        return Err(anyhow!("角色不存在").into());
    }
    service::qnxg::role::update_role(id, &name, &permission_ids).await?;
    Ok(().into())
}

#[handler]
async fn delete_role(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(crate::utils::auth::parse_token(req)?)
        .await?
        .has(&format!("{}:query", ROLE_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "param")))]
    struct DeleteRoleReq {
        id: u32,
    }
    let DeleteRoleReq { id } = req.extract().await?;
    let list = service::qnxg::role::get_role_list().await?;
    if !list.iter().any(|r| r.id == id) {
        return Err(anyhow!("角色不存在").into());
    }
    service::qnxg::role::delete_role(id).await?;
    Ok(().into())
}
