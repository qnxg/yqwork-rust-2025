pub use crate::infra::mysql::user::{
    User, UserBasicInfo, UserStatus, delete_user, get_user, get_user_by_stu_id, get_user_list,
    get_user_password, update_user, update_user_password,
};
use crate::service;
pub use crate::service::qnxg::permission::Permission;
use crate::service::qnxg::role::get_role_permission;
pub use crate::service::qnxg::role::get_user_roles;
use crate::{infra, result::AppResult};

pub async fn get_user_permission(user_id: u32) -> AppResult<Permission> {
    let role_id = get_user_roles(user_id)
        .await?
        .into_iter()
        .map(|r| r.id)
        .collect::<Vec<_>>();
    let permission = get_role_permission(&role_id).await?;
    Ok(permission)
}

pub async fn add_user(info: &UserBasicInfo, password: &str, role_id: &[u32]) -> AppResult<u32> {
    let user_id = infra::mysql::user::add_user(info, password).await?;
    service::qnxg::role::update_user_roles(user_id, role_id).await?;
    Ok(user_id)
}
