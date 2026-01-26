use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::CFG;
use crate::result::{AppError, AppResult};
use anyhow::anyhow;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct Payload {
    pub id: u32,
    pub exp: usize,
}

pub fn parse_token(req: &mut salvo::Request) -> AppResult<u32> {
    let token = req
        .headers()
        .get("Authorization")
        .ok_or(AppError::Unauthorized)?
        .to_str()
        .map_err(|_| AppError::Unauthorized)?;
    let res = jsonwebtoken::decode::<Payload>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(CFG.jwt.secret.as_bytes()),
        &jsonwebtoken::Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized)?;
    Ok(res.claims.id)
}

pub fn generate_token(id: u32) -> AppResult<String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as usize;
    let payload = Payload {
        id,
        exp: now + 60 * 60 * 24, // 1 天过期
    };
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &payload,
        &jsonwebtoken::EncodingKey::from_secret(CFG.jwt.secret.as_bytes()),
    )
    .map_err(|e| AppError::from(anyhow!("生成 token 失败: {}", e)))?;
    Ok(token)
}
