pub mod announcement;
pub mod department;
pub mod feedback;
pub mod jifen;
pub mod left_message;
pub mod mail;
pub mod mini_config;
pub mod permission;
pub mod recruitment;
pub mod role;
pub mod user;
pub mod work_hour;
pub mod zhihu;

use std::time::Duration;

use crate::config::CFG;

static DB_POOL: tokio::sync::OnceCell<sqlx::MySqlPool> = tokio::sync::OnceCell::const_new();
pub async fn get_db_pool() -> &'static sqlx::MySqlPool {
    DB_POOL
        .get_or_init(|| async {
            sqlx::mysql::MySqlPoolOptions::new()
                .max_connections(CFG.database.max_connections)
                .acquire_timeout(Duration::from_secs(3))
                .connect(&CFG.database.database_url)
                .await
                .expect("连接数据库失败")
        })
        .await
}
