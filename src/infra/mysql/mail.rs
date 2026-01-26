// TODO 暂时没搞邮件

// #[derive(sqlx::FromRow, Debug)]
// pub struct Mail {
//     pub id: u32,
//     #[sqlx(json)]
//     pub to: Vec<String>,
//     pub subject: String,
//     // TODO 为什么有两种？数据库中只用了 text
//     pub text: Option<String>,
//     pub html: Option<String>,
//     // TODO 何意味，数据库里全是 NULL
//     pub result: Option<String>,
//     // TODO 这是事实意义上的 result
//     #[sqlx(json)]
//     pub success: MailResult,
// }

// #[derive(Debug, serde::Serialize, serde::Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct MailResult {
//     pub accepted: Vec<String>,
//     pub rejected: Vec<String>,
//     pub message_id: String,
// }
