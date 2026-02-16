pub mod auth;

/// 获得当前时间（UTC+8）
pub fn now_time() -> chrono::NaiveDateTime {
    let utc_now = chrono::Utc::now();
    utc_now.naive_utc() + chrono::Duration::hours(8)
}
