use anyhow::anyhow;

use super::get_weihuda_api;
use crate::result::AppResult;

pub async fn get_auth_qrcode() -> AppResult<String> {
    let code: String = get_weihuda_api("/auth-qrcode").await?;
    Ok(code)
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub enum AuthQrCodeStatus {
    #[serde(rename = "unused")]
    Unused,
    #[serde(rename = "using")]
    Using,
    #[serde(rename = "used")]
    Used,
}
pub async fn get_auth_qrcode_status(code: &str) -> AppResult<AuthQrCodeStatus> {
    let status =
        get_weihuda_api::<Option<AuthQrCodeStatus>>(&format!("/auth-qrcode/status/{}", code))
            .await?
            .ok_or(anyhow!("获取二维码状态为空"))?;
    Ok(status)
}

pub async fn get_auth_qrcode_info(code: &str) -> AppResult<String> {
    let status: serde_json::Value = get_weihuda_api(&format!("/auth-qrcode/info/{}", code)).await?;
    let stu_id = status
        .get("info")
        .and_then(|info| info.get("stu_id"))
        .and_then(|stu_id| stu_id.as_str())
        .ok_or(anyhow!("获取二维码信息失败"))?;
    Ok(stu_id.to_string())
}
