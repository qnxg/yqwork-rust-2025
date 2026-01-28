mod auth;
mod department;
mod permission;
mod role;
mod statistics;
mod user;
mod work_hour;

pub fn routers() -> salvo::Router {
    salvo::Router::new()
        .push(auth::routers())
        .push(department::routers())
        .push(permission::routers())
        .push(role::routers())
        .push(user::routers())
        .push(work_hour::routers())
        .push(statistics::routers())
}
