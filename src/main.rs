mod app;
mod cache; // 新增快取模組
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
mod utils;

use anyhow::Result;
use config::Config;
use database::establish_connection;
use state::AppState;
use tracing::info;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // 載入環境變數
    dotenvy::dotenv().ok();

    // 初始化日誌
    tracing_subscriber::fmt::init();

    // 載入設定
    let config = Config::from_env()?;
    info!("設定載入完成");

    // 建立資料庫連線
    let db = establish_connection(&config).await?;
    info!("資料庫連線建立完成");

    // 🆕 執行遷移（開發期間自動執行）
    // #[cfg(debug_assertions)]
    // {
    //     use sea_orm_migration::MigratorTrait;
    //     migration::Migrator::up(&db, None).await?;
    //     info!("資料庫遷移完成");
    // }
    
    // 建立應用程式狀態
    let app_state = AppState::new(db, config.clone());

    // 🆕 啟動快取維護任務
    start_cache_maintenance_task(app_state.clone());

    // 啟動服務
    startup::run(app_state).await?;

    Ok(())
}

/// 啟動背景快取維護任務
fn start_cache_maintenance_task(app_state: AppState) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60)); // 每分鐘
        
        loop {
            interval.tick().await;
            
            // moka 會自動處理過期項目，這裡主要是觸發統計更新
            app_state.post_cache.run_pending_tasks().await;
            app_state.tag_cache.run_pending_tasks().await;
            
            // 每 10 次輸出一次統計（每 10 分鐘）
            static mut COUNTER: u32 = 0;
            unsafe {
                COUNTER += 1;
                if COUNTER % 10 == 0 {
                    let post_stats = app_state.post_cache.stats();
                    let tag_stats = app_state.tag_cache.stats();
                    
                    info!(
                        "快取統計 - 文章快取：{} 項目，命中率 {:.2}%", 
                        post_stats["list_cache"]["entry_count"].as_u64().unwrap_or(0) + 
                        post_stats["detail_cache"]["entry_count"].as_u64().unwrap_or(0),
                        post_stats["list_cache"]["hit_rate"].as_f64().unwrap_or(0.0) * 100.0
                    );
                    
                    info!(
                        "快取統計 - 標籤快取：{} 項目，命中率 {:.2}%",
                        tag_stats.entry_count,
                        tag_stats.hit_rate * 100.0
                    );
                }
            }
        }
    });
}