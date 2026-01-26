use std::time::Duration;

use salvo::{
    Depot, FlowCtrl, Request, Response, handler,
    http::headers::{Connection, HeaderMapExt},
};

use crate::result::AppError;

const TIMEOUT_SECS: u64 = 6;

#[handler]
pub async fn timeout_middleware(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) {
    tokio::select! {
        _ = ctrl.call_next(req, depot, res) => {},
        _ = tokio::time::sleep(Duration::from_secs(TIMEOUT_SECS)) => {
            res.headers_mut().typed_insert(Connection::close());
            res.render(AppError::TimeoutError);
            ctrl.skip_rest();
        }
    }
}
