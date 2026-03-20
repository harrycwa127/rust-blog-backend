mod config;
mod error;

mod app;
mod docs;
mod middleware;
mod routes;
mod startup;

use config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = Config::from_env()?;
    startup::run(config).await
}