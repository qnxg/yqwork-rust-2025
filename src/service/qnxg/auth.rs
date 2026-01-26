use crate::{infra, result::AppResult, utils};

pub use crate::infra::weihuda::auth::{
    get_auth_qrcode, get_auth_qrcode_info, get_auth_qrcode_status,
};

pub async fn login(user_id: u32) -> AppResult<String> {
    infra::mysql::user::update_user_last_login(user_id).await?;
    let token = utils::auth::generate_token(user_id)?;
    Ok(token)
}
