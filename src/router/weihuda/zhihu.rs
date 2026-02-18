use crate::service::weihuda::zhihu::{ZhihuBasicInfo, ZhihuStatus, ZhihuType};
use crate::{
    result::{AppError, AppResult, RouterResult},
    service, utils,
};
use anyhow::anyhow;
use salvo::handler;
use salvo::macros::Extractible;
use salvo::http::{header, StatusCode};
use serde_json::json;

const ZHIHU_PERMISSION_PREFIX: &str = "hdwsh:zhihu";

pub fn routers() -> salvo::Router {
    salvo::Router::with_path("zhihu")
        .get(get_zhihu_list)
        .post(post_zhihu)
        .push(salvo::Router::with_path("url-resolve").get(get_url_resolve))
        .push(salvo::Router::with_path("wx-img-proxy").get(get_wx_img_proxy))
        .push(
            salvo::Router::with_path("{id}")
                .get(get_zhihu)
                .put(put_zhihu)
                .delete(delete_zhihu),
        )
}

#[handler]
async fn get_zhihu_list(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req).await?.id)
        .await?
        .has(&format!("{}:query", ZHIHU_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "query"), rename_all = "camelCase"))]
    struct GetZhihuListReq {
        page: Option<u32>,
        page_size: Option<u32>,
        title: Option<String>,
        tags: Option<String>,
        status: Option<u32>,
        stu_id: Option<String>,
    }
    let GetZhihuListReq {
        page,
        page_size,
        title,
        tags,
        status,
        stu_id,
    } = req.extract().await?;
    let status = status.map(ZhihuStatus::from);
    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(10);
    let (count, rows) = service::weihuda::zhihu::get_zhihu_list(
        page,
        page_size,
        title.as_deref(),
        tags.as_deref(),
        status,
        stu_id.as_deref(),
    )
    .await?;
    Ok(json!({
        "count": count,
        "rows": rows,
    })
    .into())
}

#[handler]
async fn get_zhihu(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req).await?.id)
        .await?
        .has(&format!("{}:query", ZHIHU_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "param")))]
    struct GetZhihuReq {
        id: u32,
    }
    let GetZhihuReq { id } = req.extract().await?;
    let zhihu = service::weihuda::zhihu::get_zhihu(id).await?;
    Ok(zhihu.into())
}

#[handler]
async fn post_zhihu(req: &mut salvo::Request) -> RouterResult {
    let user = utils::auth::parse_token(req).await?;
    if !service::qnxg::user::get_user_permission(user.id)
        .await?
        .has(&format!("{}:add", ZHIHU_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "body"), rename_all = "camelCase"))]
    struct PostZhihuReq {
        title: String,
        typ: String,
        content: String,
        tags: String,
        cover: Option<String>,
        status: u32,
        top: bool,
    }
    let param: PostZhihuReq = req.extract().await?;
    let status = ZhihuStatus::from(param.status);
    let typ = ZhihuType::from(param.typ.as_str());
    let info = ZhihuBasicInfo {
        title: param.title,
        typ,
        content: param.content,
        tags: param.tags,
        cover: param.cover,
        status,
        stu_id: user.info.stu_id,
        top: param.top,
        created_at: utils::now_time(),
    };
    let res = service::weihuda::zhihu::add_zhihu(&info).await?;
    let new_zhihu = service::weihuda::zhihu::get_zhihu(res)
        .await?
        .ok_or(anyhow!("新增知湖文章失败"))?;
    Ok(new_zhihu.into())
}

#[handler]
async fn put_zhihu(req: &mut salvo::Request) -> RouterResult {
    let user_id = utils::auth::parse_token(req).await?.id;
    if !service::qnxg::user::get_user_permission(user_id)
        .await?
        .has(&format!("{}:edit", ZHIHU_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "body"), rename_all = "camelCase"))]
    struct PutZhihuReq {
        #[salvo(extract(source(from = "param")))]
        id: u32,
        title: String,
        content: String,
        tags: String,
        cover: Option<String>,
        status: u32,
        top: bool,
    }
    let param: PutZhihuReq = req.extract().await?;
    let status = ZhihuStatus::from(param.status);
    let Some(zhihu) = service::weihuda::zhihu::get_zhihu(param.id).await? else {
        return Err(anyhow!("知湖文章不存在").into());
    };
    let info = ZhihuBasicInfo {
        title: param.title,
        typ: zhihu.info.typ,
        content: param.content,
        tags: param.tags,
        cover: param.cover,
        status,
        top: param.top,
        created_at: utils::now_time(),
        stu_id: zhihu.info.stu_id,
    };
    service::weihuda::zhihu::update_zhihu(param.id, &info).await?;
    let new_zhihu = service::weihuda::zhihu::get_zhihu(param.id)
        .await?
        .ok_or(anyhow!("更新知湖文章失败"))?;
    Ok(new_zhihu.into())
}

#[handler]
async fn delete_zhihu(req: &mut salvo::Request) -> RouterResult {
    let user_id = utils::auth::parse_token(req).await?.id;
    if !service::qnxg::user::get_user_permission(user_id)
        .await?
        .has(&format!("{}:delete", ZHIHU_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "param")))]
    struct DeleteZhihuReq {
        id: u32,
    }
    let DeleteZhihuReq { id } = req.extract().await?;
    if service::weihuda::zhihu::get_zhihu(id).await?.is_none() {
        return Err(anyhow!("知湖文章不存在").into());
    }
    service::weihuda::zhihu::delete_zhihu(id).await?;
    Ok(().into())
}

#[handler]
async fn get_url_resolve(req: &mut salvo::Request) -> RouterResult {
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "query")))]
    struct GetUrlResolveReq {
        url: String,
    }
    let GetUrlResolveReq { url } = req.extract().await?;
    let res = service::weihuda::zhihu::wx_url_resolve(&url).await?;
    Ok(res.into())
}

#[handler]
async fn get_wx_img_proxy(req: &mut salvo::Request, res: &mut salvo::Response) -> AppResult<()> {
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "query")))]
    struct GetWxImgProxyReq {
        url: String,
    }
    let GetWxImgProxyReq { url } = req.extract().await?;
    let proxy = service::weihuda::zhihu::wx_url_proxy(&url).await?;
    let content_type = proxy
        .content_type
        .as_deref()
        .unwrap_or("image/jpeg");
    let _ = res.add_header(header::CONTENT_TYPE, content_type, true);
    res.status_code(StatusCode::OK);
    res.body(proxy.bytes);
    Ok(())
}
