pub mod auth;

/// 获得当前时间（UTC+8）
pub fn now_time() -> chrono::NaiveDateTime {
    let utc_now = chrono::Utc::now();
    utc_now.naive_utc() + chrono::Duration::hours(8)
}

pub fn md5_hash(input: &str) -> String {
    format!("{:x}", md5::compute(input.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_md5_hash() {
        let input = "test";
        let hash = md5_hash(input);
        assert_eq!(hash, "098f6bcd4621d373cade4e832627b4f6");
    }
}
