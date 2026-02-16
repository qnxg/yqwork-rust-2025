use crate::config;
use crate::result::AppResult;
use anyhow::anyhow;
use std::fmt::Debug;

pub mod auth;

static CLIENT: tokio::sync::OnceCell<reqwest::Client> = tokio::sync::OnceCell::const_new();

async fn get_client() -> &'static reqwest::Client {
    CLIENT
        .get_or_init(|| async {
            reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap()
        })
        .await
}

#[derive(serde::Deserialize, Debug)]
struct WeihudaResponse<T> {
    pub code: u32,
    #[expect(unused)]
    pub msg: String,
    pub data: T,
}

async fn get_weihuda_api<T>(url: &str) -> AppResult<T>
where
    T: serde::de::DeserializeOwned + Debug,
{
    let client = get_client().await;
    let url = format!("{}{}", config::CFG.weihuda.api_url, url);
    let resp: WeihudaResponse<T> = client
        .get(url)
        .send()
        .await
        .map_err(|err| anyhow!("请求微生活后端错误: {:?}", err))?
        .json()
        .await
        .map_err(|err| anyhow!("请求微生活后端错误: {:?}", err))?;
    if resp.code != 200 {
        return Err(anyhow!("请求微生活后端错误: {:?}", resp).into());
    }
    Ok(resp.data)
}
