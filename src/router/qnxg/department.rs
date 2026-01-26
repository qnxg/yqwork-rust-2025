use anyhow::anyhow;
use salvo::{handler, macros::Extractible};
use serde_json::json;

use crate::{
    result::{AppError, RouterResult},
    service::{self},
    utils,
};

const DEPARTMENT_PERMISSION_PREFIX: &str = "yq:department";

pub fn routers() -> salvo::Router {
    salvo::Router::new().push(
        salvo::Router::with_path("department")
            .get(get_department_list)
            .post(post_department)
            .push(
                salvo::Router::with_path("{id}")
                    .put(put_department)
                    .delete(delete_department),
            ),
    )
}

#[handler]
async fn get_department_list() -> RouterResult {
    let res = service::qnxg::department::get_department_list().await?;
    Ok(json!({
        "count": res.len(),
        "rows": res,
    })
    .into())
}

#[handler]
async fn post_department(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req).await?.id)
        .await?
        .has(&format!("{}:add", DEPARTMENT_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "body"), rename_all = "camelCase"))]
    struct PostDepartmentReq {
        name: String,
        desc: String,
    }
    let PostDepartmentReq { name, desc } = req.extract().await?;
    let id = service::qnxg::department::add_department(name.as_str(), desc.as_str()).await?;
    let new_department = service::qnxg::department::Department { id, name, desc };
    Ok(new_department.into())
}

#[handler]
async fn put_department(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req).await?.id)
        .await?
        .has(&format!("{}:edit", DEPARTMENT_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "body"), rename_all = "camelCase"))]
    struct PutDepartmentReq {
        #[salvo(extract(source(from = "param")))]
        id: u32,
        name: String,
        desc: String,
    }
    let PutDepartmentReq { id, name, desc } = req.extract().await?;
    // 判断部门是否存在
    let departments = service::qnxg::department::get_department_list().await?;
    if !departments.into_iter().any(|d| d.id == id) {
        return Err(anyhow!("部门不存在").into());
    }
    // 更新部门
    service::qnxg::department::update_department(id, name.as_str(), desc.as_str()).await?;
    Ok(().into())
}

#[handler]
async fn delete_department(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req).await?.id)
        .await?
        .has(&format!("{}:delete", DEPARTMENT_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "param"), rename_all = "camelCase"))]
    struct DeleteDepartmentReq {
        id: u32,
    }
    let DeleteDepartmentReq { id } = req.extract().await?;
    // 判断部门是否存在
    let departments = service::qnxg::department::get_department_list().await?;
    if !departments.into_iter().any(|d| d.id == id) {
        return Err(anyhow!("部门不存在").into());
    }
    // 删除部门
    service::qnxg::department::delete_department(id).await?;
    Ok(().into())
}
