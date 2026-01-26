mod announcement;
mod config;
mod feedback;
mod jifen;
mod left_message;
mod zhihu;

pub fn routers() -> salvo::Router {
    salvo::Router::new()
        .push(announcement::routers())
        .push(config::routers())
        .push(feedback::routers())
        .push(jifen::routers())
        // .push(left_message::routers())
        .push(zhihu::routers())
}
