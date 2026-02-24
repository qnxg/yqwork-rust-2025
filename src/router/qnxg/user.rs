use std::collections::HashMap;

use crate::result::{AppError, RouterResult};
use crate::service::qnxg::permission::PermissionItem;
use crate::service::qnxg::user::{User, UserBasicInfo, UserStatus};
use crate::{service, utils};
use anyhow::anyhow;
use salvo::handler;
use salvo::macros::Extractible;
use serde_json::json;

const USER_PERMISSION_PREFIX: &str = "yq:user";

pub fn routers() -> salvo::Router {
    salvo::Router::with_path("user")
        .get(get_user_list)
        .post(post_user)
        .push(salvo::Router::with_path("pwd").put(put_pwd))
        .push(salvo::Router::with_path("whoami").get(get_whoami))
        .push(
            salvo::Router::with_path("{id}")
                .get(get_user)
                .put(put_user)
                .delete(delete_user),
        )
}

#[handler]
async fn get_user_list(req: &mut salvo::Request) -> RouterResult {
    let user = utils::auth::parse_token(req).await?;
    let permission = service::qnxg::user::get_user_permission(user.id).await?;
    if !permission.has(&format!("{}:query", USER_PERMISSION_PREFIX)) {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "query"), rename_all = "camelCase"))]
    struct GetUserListReq {
        page: Option<u32>,
        page_size: Option<u32>,
        stu_id: Option<String>,
        name: Option<String>,
        department_id: Option<u32>,
        status: Option<u32>,
    }
    let GetUserListReq {
        page,
        page_size,
        stu_id,
        name,
        department_id,
        status,
    } = req.extract().await?;
    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(10);
    // 管理员才能查看所有用户，非管理员只能看看自己所在部门的用户
    let (count, rows) = if permission.is_admin() {
        service::qnxg::user::get_user_list(
            page,
            page_size,
            stu_id.as_deref(),
            name.as_deref(),
            department_id,
            status,
        )
        .await?
    } else {
        let department_id = department_id.unwrap_or(user.info.department_id);
        if department_id != user.info.department_id {
            (0, Vec::new())
        } else {
            service::qnxg::user::get_user_list(
                page,
                page_size,
                stu_id.as_deref(),
                name.as_deref(),
                Some(department_id),
                status,
            )
            .await?
        }
    };
    // 生成一个用户id及其角色列表的 map
    let user_roles_map = {
        let mut map = HashMap::new();
        for user in &rows {
            let roles = service::qnxg::role::get_user_roles(user.id).await?;
            map.insert(user.id, roles);
        }
        map
    };
    Ok(json!({
        "count": count,
        "rows": rows,
        "userRolesMap": user_roles_map,
    })
    .into())
}

#[handler]
async fn get_user(req: &mut salvo::Request) -> RouterResult {
    let user = utils::auth::parse_token(req).await?;
    let permission = service::qnxg::user::get_user_permission(user.id).await?;
    if !permission.has(&format!("{}:query", USER_PERMISSION_PREFIX)) {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "param")))]
    struct GetUserReq {
        id: u32,
    }
    let GetUserReq { id } = req.extract().await?;
    let Some(user_res) = service::qnxg::user::get_user(id).await? else {
        return Ok(().into());
    };
    if !permission.is_admin() && user.info.department_id != user_res.info.department_id {
        return Ok(().into());
    }
    Ok(user_res.into())
}

#[handler]
async fn post_user(req: &mut salvo::Request) -> RouterResult {
    let user = utils::auth::parse_token(req).await?;
    let permission = service::qnxg::user::get_user_permission(user.id).await?;
    if !permission.has(&format!("{}:add", USER_PERMISSION_PREFIX)) {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "body"), rename_all = "camelCase"))]
    struct PostUserReq {
        username: Option<String>,
        password: String,
        name: String,
        stu_id: String,
        email: Option<String>,
        xueyuan: u32,
        gangwei: Option<String>,
        zaiku: bool,
        qingonggang: bool,
        department_id: u32,
        status: u32,
        role_id: Vec<u32>,
    }
    let param: PostUserReq = req.extract().await?;
    let status = UserStatus::from(param.status);
    // 学号是唯一的
    if service::qnxg::user::get_user_by_stu_id(&param.stu_id)
        .await?
        .is_some()
    {
        return Err(anyhow!("学号已存在").into());
    }
    // 部门必须存在
    if !service::qnxg::department::get_department_list()
        .await?
        .iter()
        .any(|v| v.id == param.department_id)
    {
        return Err(anyhow!("部门不存在").into());
    }
    // 只能创建自己部门的用户，非管理员不能创建其他部门的用户
    if !permission.is_admin() && user.info.department_id != param.department_id {
        return Err(AppError::PermissionDenied);
    }
    // 创建的用户的角色比如是创建者的子集
    let user_roles = service::qnxg::role::get_user_roles(user.id)
        .await?
        .into_iter()
        .map(|r| r.id)
        .collect::<Vec<_>>();
    if !permission.is_admin() && !param.role_id.iter().all(|id| user_roles.contains(id)) {
        return Err(AppError::PermissionDenied);
    }
    let info = UserBasicInfo {
        username: param.username,
        name: param.name,
        stu_id: param.stu_id,
        email: param.email,
        xueyuan: param.xueyuan,
        gangwei: param.gangwei,
        zaiku: param.zaiku,
        qingonggang: param.qingonggang,
        status,
        department_id: param.department_id,
    };
    let res = service::qnxg::user::add_user(&info, &param.password, &param.role_id).await?;
    let new_user = service::qnxg::user::get_user(res)
        .await?
        .ok_or(anyhow!("新增用户失败"))?;
    Ok(new_user.into())
}

#[handler]
async fn put_user(req: &mut salvo::Request) -> RouterResult {
    let user = utils::auth::parse_token(req).await?;
    let permission = service::qnxg::user::get_user_permission(user.id).await?;
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "body"), rename_all = "camelCase"))]
    struct PutUserReq {
        #[salvo(extract(source(from = "param")))]
        id: u32,
        username: Option<String>,
        // 普通用户不能更改
        name: String,
        // 普通用户不能更改
        stu_id: String,
        email: Option<String>,
        // 普通用户不能更改
        xueyuan: u32,
        // 普通用户不能更改
        gangwei: Option<String>,
        // 普通用户不能更改
        zaiku: bool,
        // 普通用户不能更改
        qingonggang: bool,
        // 普通用户不能更改。非管理员只能保持和自己部门一致
        department_id: u32,
        // 普通用户不能更改
        status: u32,
        // 为 None 说明不更改，普通用户不能更改
        password: Option<String>,
        // 为 None 说明不更改，普通用户不能更改
        role_id: Option<Vec<u32>>,
    }
    let param: PutUserReq = req.extract().await?;
    let status = UserStatus::from(param.status);
    // 没权限的话只能改自己的
    if !permission.has(&format!("{}:edit", USER_PERMISSION_PREFIX)) && user.id != param.id {
        return Err(AppError::PermissionDenied);
    }
    let Some(res_user) = service::qnxg::user::get_user(param.id).await? else {
        return Err(anyhow!("用户不存在").into());
    };
    // 非管理员只能改自己部门的用户
    if !permission.is_admin() && user.info.department_id != res_user.info.department_id {
        return Err(AppError::PermissionDenied);
    }
    // 一些字段普通用户不能改
    if !permission.has(&format!("{}:edit", USER_PERMISSION_PREFIX))
        && (param.name != res_user.info.name
            || param.stu_id != res_user.info.stu_id
            || param.gangwei != res_user.info.gangwei
            || param.zaiku != res_user.info.zaiku
            || param.qingonggang != res_user.info.qingonggang
            || param.department_id != res_user.info.department_id
            || status != res_user.info.status
            || param.password.is_some()
            || param.role_id.is_some()
            || param.xueyuan != res_user.info.xueyuan)
    {
        return Err(AppError::PermissionDenied);
    }
    if !permission.is_admin() {
        // 非管理员部门不能变更部门信息
        if param.department_id != user.info.department_id {
            return Err(AppError::PermissionDenied);
        }
        // 非管理员不能把用户的角色改成自己的子集之外
        if let Some(role_id) = &param.role_id {
            let user_roles = service::qnxg::role::get_user_roles(user.id)
                .await?
                .into_iter()
                .map(|r| r.id)
                .collect::<Vec<_>>();
            if !role_id.iter().all(|id| user_roles.contains(id)) {
                return Err(AppError::PermissionDenied);
            }
        }
    }
    // 部门必须存在
    if param.department_id != res_user.info.department_id
        && !service::qnxg::department::get_department_list()
            .await?
            .iter()
            .any(|v| v.id == param.department_id)
    {
        return Err(anyhow!("部门不存在").into());
    }
    // 学号是唯一的
    if param.stu_id != res_user.info.stu_id
        && service::qnxg::user::get_user_by_stu_id(&param.stu_id)
            .await?
            .is_some()
    {
        return Err(anyhow!("学号已存在").into());
    }
    let info = UserBasicInfo {
        username: param.username,
        name: param.name,
        stu_id: param.stu_id,
        email: param.email,
        xueyuan: param.xueyuan,
        gangwei: param.gangwei,
        zaiku: param.zaiku,
        qingonggang: param.qingonggang,
        status,
        department_id: param.department_id,
    };
    service::qnxg::user::update_user(param.id, &info).await?;
    if let Some(password) = param.password {
        service::qnxg::user::update_user_password(param.id, &password).await?;
    }
    if let Some(role_id) = param.role_id {
        service::qnxg::role::update_user_roles(param.id, &role_id).await?;
    }
    let new_user = service::qnxg::user::get_user(param.id)
        .await?
        .ok_or(anyhow!("更新用户失败"))?;
    Ok(new_user.into())
}

#[handler]
async fn delete_user(req: &mut salvo::Request) -> RouterResult {
    let user = utils::auth::parse_token(req).await?;
    let permission = service::qnxg::user::get_user_permission(user.id).await?;
    if !permission.has(&format!("{}:delete", USER_PERMISSION_PREFIX)) {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "param")))]
    struct DeleteUserReq {
        id: u32,
    }
    let DeleteUserReq { id } = req.extract().await?;
    let Some(res_user) = service::qnxg::user::get_user(id).await? else {
        return Err(anyhow!("用户不存在").into());
    };
    // 非管理员只能删自己部门的用户
    if !permission.is_admin() && user.info.department_id != res_user.info.department_id {
        return Err(AppError::PermissionDenied);
    }
    // 不能删自己
    if user.id == id {
        return Err(AppError::PermissionDenied);
    }
    service::qnxg::user::delete_user(id).await?;
    Ok(().into())
}

#[handler]
async fn put_pwd(req: &mut salvo::Request) -> RouterResult {
    let user_id = utils::auth::parse_token(req).await?.id;
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "body"), rename_all = "camelCase"))]
    struct PutPwdReq {
        old_password: String,
        new_password: String,
    }
    let PutPwdReq {
        old_password,
        new_password,
    } = req.extract().await?;
    if service::qnxg::user::get_user(user_id).await?.is_none() {
        return Err(AppError::Unauthorized);
    };
    let Some(pwd) = service::qnxg::user::get_user_password(user_id).await? else {
        return Err(AppError::Unauthorized);
    };
    let old_password = utils::md5_hash(&old_password);
    if pwd != old_password {
        return Err(anyhow!("旧密码错误").into());
    }
    service::qnxg::user::update_user_password(user_id, &new_password).await?;
    Ok(().into())
}

#[handler]
async fn get_whoami(req: &mut salvo::Request) -> RouterResult {
    #[derive(serde::Serialize, Debug)]
    struct GetWhoamiResp {
        user: User,
        permissions: Vec<PermissionItem>,
    }
    let user = utils::auth::parse_token(req).await?;
    let permissions = service::qnxg::user::get_user_permission(user.id).await?;
    Ok(GetWhoamiResp {
        user,
        permissions: permissions.into_inner(),
    }
    .into())
}
