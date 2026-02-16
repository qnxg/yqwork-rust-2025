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
        SELECT `key`, `value` FROM weihuda_new.mini_configs
        "#
    )
    .fetch_all(get_db_pool().await)
    .await?;
    Ok(res)
}

pub async fn update_mini_config(key: &str, value: &str) -> AppResult<()> {
    sqlx::query!(
        r#"
        UPDATE weihuda_new.mini_configs SET `value` = ? WHERE `key` = ? 
        "#,
        value,
        key,
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(())
}
