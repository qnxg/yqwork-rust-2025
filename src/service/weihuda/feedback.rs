pub use crate::infra::mysql::feedback::{
    Feedback, FeedbackMsgType, FeedbackStatus, delete_feedback, delete_feedback_msg, get_feedback,
    get_feedback_list, get_feedback_msg_list, update_feedback,
};

use crate::service::qnxg::user::User;
use crate::{infra, result::AppResult};

pub async fn add_feedback_msg(
    typ: FeedbackMsgType,
    msg: Option<&str>,
    feedback: &Feedback,
    user: &User,
) -> AppResult<u32> {
    infra::mysql::feedback::add_feedback_msg(typ, msg, &user.info.stu_id, feedback.id).await?;
    Ok(feedback.id)
}
