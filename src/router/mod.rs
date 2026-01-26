mod qnxg;
mod weihuda;

pub fn routers() -> salvo::Router {
    salvo::Router::new()
        .push(qnxg::routers())
        .push(weihuda::routers())
}
