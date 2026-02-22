use anyhow::anyhow;

pub use crate::infra::mysql::zhihu::{
    ZhihuBasicInfo, ZhihuStatus, ZhihuType, add_zhihu, delete_zhihu, get_zhihu, get_zhihu_list,
    update_zhihu,
};
use crate::result::AppResult;

#[derive(Debug, serde::Serialize)]
pub struct WxUrlResolve {
    pub title: String,
    pub cover: String,
}

pub async fn wx_url_resolve(url: &str) -> AppResult<WxUrlResolve> {
    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .header(reqwest::header::USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/129.0.0.0 Safari/537.36")
        .send()
        .await?
        .text()
        .await?;
    let title_regex = regex::Regex::new(r#"var msg_title = '(.+?)'"#).unwrap();
    let cover_regex = regex::Regex::new(r#"var cdn_url_1_1 = "(.+?)""#).unwrap();
    let title = title_regex
        .captures(&res)
        .and_then(|cat| cat.get(1))
        .map(|mat| mat.as_str().to_string())
        .ok_or(anyhow!("无法获取文章标题"))?;
    let cover = cover_regex
        .captures(&res)
        .and_then(|cat| cat.get(1))
        .map(|mat| mat.as_str().to_string())
        .ok_or(anyhow!("无法获取文章封面"))?;
    Ok(WxUrlResolve { title, cover })
}

#[derive(Debug)]
pub struct WxUrlProxy {
    pub content_type: Option<String>,
    pub bytes: Vec<u8>,
}

pub async fn wx_url_proxy(url: &str) -> AppResult<WxUrlProxy> {
    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .header(reqwest::header::USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/129.0.0.0 Safari/537.36")
        .send()
        .await?;
    let content_type = res
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let bytes = res.bytes().await?.to_vec();
    Ok(WxUrlProxy {
        content_type,
        bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_wx_url_resolve() {
        let res = wx_url_resolve("https://mp.weixin.qq.com/s/DrqGYQUCTNqkLdYFLmX-pg")
            .await
            .unwrap();
        println!("{:?}", res);
    }
}
