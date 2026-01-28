use crate::result::{AppError, RouterResult};
use crate::service::weihuda::jifen::GoodsRecordStatus;
use crate::{service, utils};
use anyhow::anyhow;
use salvo::handler;
use salvo::macros::Extractible;
use serde_json::json;

const GOODS_RECORD_PERMISSION_PREFIX: &str = "hdwsh:goodsRecord";
const JIFEN_GOODS_PERMISSION_PREFIX: &str = "hdwsh:jifenGoods";
const JIFEN_RECORD_PERMISSION_PREFIX: &str = "hdwsh:jifenRecord";
const JIFEN_RULE_PERMISSION_PREFIX: &str = "hdwsh:jifenRule";

pub fn routers() -> salvo::Router {
    salvo::Router::new()
        .push(
            salvo::Router::with_path("goods-record")
                .get(get_goods_record_list)
                .push(
                    salvo::Router::with_path("{id}")
                        .get(get_goods_record)
                        .delete(delete_goods_record),
                ),
        )
        .push(
            salvo::Router::with_path("jifen-goods")
                .get(get_goods_list)
                .post(post_goods)
                .push(
                    salvo::Router::with_path("{id}")
                        .put(put_goods)
                        .delete(delete_goods),
                ),
        )
        .push(
            salvo::Router::with_path("jifen-record")
                .get(get_record_list)
                .post(post_record)
                .push(
                    salvo::Router::with_path("{id}")
                        .get(get_record)
                        .delete(delete_record),
                ),
        )
        .push(
            salvo::Router::with_path("jifen-rule")
                .get(get_rule_list)
                .post(post_rule)
                .push(
                    salvo::Router::with_path("{id}")
                        .put(put_rule)
                        .delete(delete_rule),
                ),
        )
}

#[handler]
async fn get_goods_record_list(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req).await?.id)
        .await?
        .has(&format!("{}:query", GOODS_RECORD_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "query"), rename_all = "camelCase"))]
    struct GetGoodsRecordListReq {
        page: Option<u32>,
        page_size: Option<u32>,
        stu_id: Option<String>,
        goods_id: Option<u32>,
        status: Option<u32>,
    }
    let GetGoodsRecordListReq {
        page,
        page_size,
        stu_id,
        goods_id,
        status,
    } = req.extract().await?;
    let status = status.map(GoodsRecordStatus::from);
    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(10);
    let (count, rows) = service::weihuda::jifen::get_goods_record_list(
        page,
        page_size,
        stu_id.as_deref(),
        goods_id,
        status,
    )
    .await?;
    Ok(json!({
        "count": count,
        "rows": rows,
    })
    .into())
}

#[handler]
async fn get_goods_record(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req).await?.id)
        .await?
        .has(&format!("{}:query", GOODS_RECORD_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "param")))]
    struct GetGoodsRecordReq {
        id: u32,
    }
    let GetGoodsRecordReq { id } = req.extract().await?;
    let record = service::weihuda::jifen::get_goods_record(id).await?;
    Ok(record.into())
}

#[handler]
async fn delete_goods_record(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req).await?.id)
        .await?
        .has(&format!("{}:delete", GOODS_RECORD_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "param")))]
    struct DeleteGoodsRecordReq {
        id: u32,
    }
    let DeleteGoodsRecordReq { id } = req.extract().await?;
    let record = service::weihuda::jifen::get_goods_record(id).await?;
    if record.is_none() {
        return Err(anyhow!("兑换记录不存在").into());
    }
    service::weihuda::jifen::delete_goods_record(id).await?;
    Ok(().into())
}

#[handler]
async fn get_goods_list(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req).await?.id)
        .await?
        .has(&format!("{}:query", JIFEN_GOODS_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    let goods = service::weihuda::jifen::get_goods_list().await?;
    Ok(goods.into())
}

#[handler]
async fn post_goods(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req).await?.id)
        .await?
        .has(&format!("{}:add", JIFEN_GOODS_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "body"), rename_all = "camelCase"))]
    struct PostGoodsReq {
        count: u32,
        cover: String,
        description: Option<String>,
        enabled: bool,
        name: String,
        price: u32,
    }
    let PostGoodsReq {
        count,
        cover,
        description,
        enabled,
        name,
        price,
    } = req.extract().await?;
    let res = service::weihuda::jifen::add_goods(
        name.as_str(),
        cover.as_str(),
        count,
        price,
        description.as_deref(),
        enabled,
    )
    .await?;
    let new_goods = service::weihuda::jifen::get_goods_list()
        .await?
        .into_iter()
        .find(|g| g.id == res)
        .ok_or(anyhow!("新增积分商品失败"))?;
    Ok(new_goods.into())
}

#[handler]
async fn put_goods(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req).await?.id)
        .await?
        .has(&format!("{}:edit", JIFEN_GOODS_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }

    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "body"), rename_all = "camelCase"))]
    struct PutGoodsReq {
        #[salvo(extract(source(from = "param")))]
        id: u32,
        count: u32,
        cover: String,
        description: Option<String>,
        enabled: u32,
        name: String,
        price: u32,
    }
    let PutGoodsReq {
        id,
        count,
        cover,
        description,
        enabled,
        name,
        price,
    } = req.extract().await?;

    let goods = service::weihuda::jifen::get_goods_list().await?;
    if !goods.iter().any(|v| v.id == id) {
        return Err(anyhow!("积分商品不存在").into());
    }

    service::weihuda::jifen::update_goods(
        id,
        &name,
        &cover,
        count,
        price,
        description.as_deref(),
        enabled != 0,
    )
    .await?;
    let new_goods = service::weihuda::jifen::get_goods_list()
        .await?
        .into_iter()
        .find(|g| g.id == id)
        .ok_or(anyhow!("更新积分商品失败"))?;

    Ok(new_goods.into())
}

#[handler]
async fn delete_goods(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req).await?.id)
        .await?
        .has(&format!("{}:delete", JIFEN_GOODS_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }

    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "param")))]
    struct DeleteGoodsReq {
        id: u32,
    }
    let DeleteGoodsReq { id } = req.extract().await?;

    let goods = service::weihuda::jifen::get_goods_list().await?;
    if !goods.iter().any(|v| v.id == id) {
        return Err(anyhow!("积分商品不存在").into());
    }

    service::weihuda::jifen::delete_goods(id).await?;
    Ok(().into())
}

#[handler]
async fn get_record_list(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req).await?.id)
        .await?
        .has(&format!("{}:query", JIFEN_RECORD_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "query"), rename_all = "camelCase"))]
    struct GetRecordListReq {
        page: Option<u32>,
        page_size: Option<u32>,
        key: Option<String>,
        param: Option<String>,
        stu_id: Option<String>,
    }
    let GetRecordListReq {
        page,
        page_size,
        key,
        param,
        stu_id,
    } = req.extract().await?;
    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(10);
    let (count, rows) = service::weihuda::jifen::get_record_list(
        page,
        page_size,
        key.as_deref(),
        param.as_deref(),
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
async fn get_record(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req).await?.id)
        .await?
        .has(&format!("{}:query", JIFEN_RECORD_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "param")))]
    struct GetRecordReq {
        id: u32,
    }
    let GetRecordReq { id } = req.extract().await?;
    let record = service::weihuda::jifen::get_record(id).await?;
    Ok(record.into())
}

#[handler]
async fn post_record(req: &mut salvo::Request) -> RouterResult {
    let user = utils::auth::parse_token(req).await?;
    if !service::qnxg::user::get_user_permission(user.id)
        .await?
        .has(&format!("{}:add", JIFEN_RECORD_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "body"), rename_all = "camelCase"))]
    struct PostRecordReq {
        stu_id: String,
        desc: String,
        jifen: i32,
    }
    let PostRecordReq {
        stu_id,
        desc,
        jifen,
    } = req.extract().await?;
    let res = service::weihuda::jifen::add_record(&user, &stu_id, jifen, &desc).await?;
    let record = service::weihuda::jifen::get_record(res)
        .await?
        .ok_or(anyhow!("新增积分记录失败"))?;
    Ok(record.into())
}

#[handler]
async fn delete_record(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req).await?.id)
        .await?
        .has(&format!("{}:delete", JIFEN_RECORD_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "param")))]
    struct DeleteRecordReq {
        id: u32,
    }
    let DeleteRecordReq { id } = req.extract().await?;
    let record = service::weihuda::jifen::get_record(id).await?;
    if record.is_none() {
        return Err(anyhow!("积分记录不存在").into());
    }
    service::weihuda::jifen::delete_record(id).await?;
    Ok(().into())
}

#[handler]
async fn get_rule_list(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req).await?.id)
        .await?
        .has(&format!("{}:query", JIFEN_RULE_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    let rules = service::weihuda::jifen::get_rule_list().await?;
    Ok(json!({
        "count": rules.len(),
        "rows": rules,
    })
    .into())
}

#[handler]
async fn post_rule(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req).await?.id)
        .await?
        .has(&format!("{}:add", JIFEN_RULE_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "body"), rename_all = "camelCase"))]
    struct PostRuleReq {
        cycle: u32,
        enabled: bool,
        jifen: i32,
        key: String,
        max_count: u32,
        name: String,
    }
    let PostRuleReq {
        cycle,
        enabled,
        jifen,
        key,
        max_count,
        name,
    } = req.extract().await?;
    let res =
        service::weihuda::jifen::add_rule(&name, &key, jifen, cycle, max_count, enabled).await?;
    let new_rule = service::weihuda::jifen::get_rule_list()
        .await?
        .into_iter()
        .find(|r| r.id == res)
        .ok_or(anyhow!("新增积分规则失败"))?;
    Ok(new_rule.into())
}

#[handler]
async fn put_rule(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req).await?.id)
        .await?
        .has(&format!("{}:edit", JIFEN_RULE_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "body"), rename_all = "camelCase"))]
    struct PutRuleReq {
        #[salvo(extract(source(from = "param")))]
        id: u32,
        cycle: u32,
        enable: u32,
        jifen: i32,
        key: String,
        max_count: u32,
        name: String,
    }
    let PutRuleReq {
        id,
        cycle,
        enable,
        jifen,
        key,
        max_count,
        name,
    } = req.extract().await?;
    service::weihuda::jifen::update_rule(id, &name, &key, jifen, cycle, max_count, enable != 0)
        .await?;
    let new_rule = service::weihuda::jifen::get_rule_list()
        .await?
        .into_iter()
        .find(|r| r.id == id)
        .ok_or(anyhow!("更新积分规则失败"))?;
    Ok(new_rule.into())
}

#[handler]
async fn delete_rule(req: &mut salvo::Request) -> RouterResult {
    if !service::qnxg::user::get_user_permission(utils::auth::parse_token(req).await?.id)
        .await?
        .has(&format!("{}:delete", JIFEN_RULE_PERMISSION_PREFIX))
    {
        return Err(AppError::PermissionDenied);
    }
    #[derive(serde::Deserialize, Extractible, Debug)]
    #[salvo(extract(default_source(from = "param")))]
    struct DeleteRuleReq {
        id: u32,
    }
    let DeleteRuleReq { id } = req.extract().await?;
    service::weihuda::jifen::delete_rule(id).await?;
    Ok(().into())
}
