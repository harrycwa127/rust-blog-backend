use sea_orm::{Database, DatabaseConnection, DbErr};
use tracing::info;

use crate::config::Config;

pub async fn establish_connection(config: &Config) -> Result<DatabaseConnection, DbErr> {
    info!("正在連接到資料庫...");

    let mut opt = sea_orm::ConnectOptions::new(&config.database_url);
    opt.max_connections(config.database_max_connections)
        .min_connections(1)
        .connect_timeout(std::time::Duration::from_secs(8))
        .acquire_timeout(std::time::Duration::from_secs(8))
        .idle_timeout(std::time::Duration::from_secs(8))
        .max_lifetime(std::time::Duration::from_secs(8))
        .sqlx_logging(true)  // 在開發期間顯示 SQL 查詢
        .sqlx_logging_level(log::LevelFilter::Info);

    let db = Database::connect(opt).await?;
    
    info!("資料庫連線成功！");
    Ok(db)
}

// 健康檢查：測試資料庫連線
pub async fn health_check(db: &DatabaseConnection) -> Result<(), DbErr> {
    use sea_orm::Statement;
    
    db.execute(Statement::from_string(
        sea_orm::DatabaseBackend::Postgres,
        "SELECT 1".to_string(),
    ))
    .await?;
    
    Ok(())
}
