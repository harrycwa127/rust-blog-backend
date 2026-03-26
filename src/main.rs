mod app;
mod config;
mod database;
mod docs;
mod dtos;
mod entities;
mod error;
mod middleware;
mod routes;
mod services;
mod startup;
mod state;

use anyhow::Result;
use config::Config;
use database::establish_connection;
use state::AppState;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // 載入環境變數
    dotenvy::dotenv().ok();

    // 初始化日誌
    tracing_subscriber::fmt::init();

    // 載入設定
    let config = Config::from_env()?;
    info!("設定載入完成");

    // 🆕 建立資料庫連線
    let db = establish_connection(&config).await?;
    info!("資料庫連線建立完成");

    // 🆕 執行遷移（開發期間自動執行）
    #[cfg(debug_assertions)]
    {
        use sea_orm_migration::MigratorTrait;
        migration::Migrator::up(&db, None).await?;
        info!("資料庫遷移完成");
    }

    // 🆕 建立應用程式狀態
    let app_state = AppState::new(db, config.clone());

    // 啟動服務
    startup::run(app_state).await?;

    Ok(())
}