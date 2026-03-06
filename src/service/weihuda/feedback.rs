pub use crate::infra::mysql::feedback::{
    Feedback, FeedbackMsgType, FeedbackStatus, delete_feedback, delete_feedback_msg, get_feedback,
    get_feedback_list, get_feedback_msg_list, update_feedback,
};

use crate::service;
use crate::service::qnxg::user::User;
use crate::{infra, result::AppResult};

pub async fn add_feedback_msg(
    typ: FeedbackMsgType,
    msg: Option<&str>,
    feedback: &Feedback,
    user: &User,
) -> AppResult<u32> {
    let feedback_msg_id =
        infra::mysql::feedback::add_feedback_msg(typ, msg, &user.info.stu_id, feedback.id).await?;
    // 发送通知
    match typ {
        FeedbackMsgType::Comment => {
            if let Some((stu_id, msg)) = feedback
                .stu_id
                .as_ref()
                .and_then(|stu_id| msg.map(|msg| (stu_id, msg)))
            {
                service::weihuda::notice::add_notice(
                    stu_id,
                    &format!("您的问题反馈有了新的进展：{}", msg),
                    true,
                    None,
                )
                .await?;
            }
        }
    }
    Ok(feedback_msg_id)
}
