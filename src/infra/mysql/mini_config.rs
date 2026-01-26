use super::get_db_pool;
use crate::result::AppResult;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct MiniConfig {
    pub key: String,
    pub value: String,
}

pub async fn get_mini_config() -> AppResult<Vec<MiniConfig>> {
    let res = sqlx::query_as!(
        MiniConfig,
        r#"
        SELECT `key`, `value` FROM weihuda.mini_configs
        "#
    )
    .fetch_all(get_db_pool().await)
    .await?;
    Ok(res)
}

pub async fn update_mini_config(key: &str, value: &str) -> AppResult<()> {
    let now = chrono::Utc::now().naive_utc();
    sqlx::query!(
        r#"
        UPDATE weihuda.mini_configs SET `value` = ?, updatedAt = ? WHERE `key` = ? 
        "#,
        value,
        now,
        key,
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(())
}
