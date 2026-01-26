pub use crate::infra::mysql::feedback::{
    FeedbackStatus, delete_feedback, get_feedback, get_feedback_list,
};

use crate::{infra, result::AppResult, service::qnxg::user::User};

pub async fn update_feedback(
    id: u32,
    status: FeedbackStatus,
    comment: &str,
    updated_by: &User,
) -> AppResult<()> {
    infra::mysql::feedback::update_feedback(id, status, comment, &updated_by.info.name).await?;
    Ok(())
}
