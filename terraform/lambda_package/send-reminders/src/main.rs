use lambda_runtime::{run, service_fn, tracing, Error};

mod handler;
mod config;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _ = &*config::APP_CONFIG;

    tracing::init_default_subscriber();

    run(service_fn(handler::function_handler)).await
}
