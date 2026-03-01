pub use crate::infra::mysql::user::{
    User, UserBasicInfo, UserStatus, delete_user, get_user, get_user_by_stu_id, get_user_list,
    get_user_password, update_user,
};
pub use crate::service::qnxg::permission::Permission;
use crate::service::qnxg::role::get_role_permission;
pub use crate::service::qnxg::role::get_user_roles;
use crate::{infra, result::AppResult};
use crate::{service, utils};

pub async fn get_user_permission(user_id: u32) -> AppResult<Permission> {
    let role_id = get_user_roles(user_id)
        .await?
        .into_iter()
        .map(|r| r.id)
        .collect::<Vec<_>>();
    let permission = get_role_permission(&role_id).await?;
    Ok(permission)
}

/// password 参数为明文
pub async fn add_user(info: &UserBasicInfo, password: &str, role_id: &[u32]) -> AppResult<u32> {
    let password = utils::md5_hash(password);
    let user_id = infra::mysql::user::add_user(info, &password).await?;
    service::qnxg::role::update_user_roles(user_id, role_id).await?;
    Ok(user_id)
}

/// 更新用户密码
/// password 参数为明文
pub async fn update_user_password(user_id: u32, password: &str) -> AppResult<()> {
    let password = utils::md5_hash(password);
    infra::mysql::user::update_user_password(user_id, &password).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    const USER_ID: u32 = 0;

    #[tokio::test]
    async fn test_get_user_permission() {
        let roles = get_user_roles(USER_ID).await.unwrap();
        println!("{:#?}", roles);
        let perm = get_user_permission(USER_ID).await.unwrap();
        println!("{:#?}", perm.into_inner());
    }
}
