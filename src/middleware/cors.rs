use salvo::cors::{Any, Cors, CorsHandler};
use salvo::http::Method;

/// 跨域中间件
#[inline]
pub fn cors_middleware() -> CorsHandler {
    Cors::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::PUT])
        .allow_headers(Any)
        .into_handler()
}
