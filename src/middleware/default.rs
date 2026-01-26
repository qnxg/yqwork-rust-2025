use anyhow::anyhow;
use salvo::{Depot, FlowCtrl, Request, Response, handler, writing::Json};
use serde_json::json;

use crate::result::AppError;

/// 中间件，处理任何无返回体的结果
///
/// 主要用途：
/// 1.  在请求不存在的接口时返回错误信息
/// 2.  在接口（错误地）没有返回体的时候返回错误信息
#[handler]
pub async fn default_middleware(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) {
    ctrl.call_next(req, depot, res).await;
    let body_size = res.body.size().unwrap_or(0);
    if body_size > 0 {
        return;
    }

    match res.status_code {
        None => res.render(AppError::Anyhow(anyhow!("服务器未返回有效信息"))),
        Some(status_code) => {
            res.stuff(
                status_code,
                Json(json!({
                    "code": status_code.as_u16(),
                    "data": null,
                    "msg": status_code.canonical_reason().unwrap_or("未知错误"),
                })),
            );
        }
    }
}
