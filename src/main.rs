use crate::config::CFG;
use salvo::prelude::*;
mod config;
mod infra;
mod middleware;
mod result;
mod router;
mod service;
mod utils;

#[tokio::main]
async fn main() {
    let _guard = clia_tracing_config::build()
        .filter_level(&CFG.log.filter_level)
        .with_ansi(CFG.log.with_ansi)
        .to_stdout(CFG.log.to_stdout)
        .directory(&CFG.log.directory)
        .file_name(&CFG.log.file_name)
        .rolling(&CFG.log.rolling)
        .with_source_location(false) // 在调试时候可以打开，确认日志所处的代码位置
        .with_thread_ids(false) // 无需打开，线程模型有tokio调度
        .with_thread_names(false) // 无需打开，线程模型有tokio调度
        .with_target(false) // 无需打开，打开后日志很累赘
        .init();
    tracing::info!("📓 Log level: {}", &CFG.log.filter_level);
    tracing::info!("🚀 Yqwork is starting");
    tracing::info!("🔄 Listening on port: {}", &CFG.server.address);
    let listener = TcpListener::new(&CFG.server.address).bind().await;
    let routers = router::routers();
    let service = Service::new(routers)
        .hoop(middleware::default_middleware)
        .hoop(Logger::new())
        .hoop(middleware::cors_middleware())
        .hoop(middleware::timeout_middleware);
    Server::new(listener).serve(service).await;
}
