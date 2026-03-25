use std::env;
use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub protocol: String,
    pub blog_name: String,
    pub blog_description: String,
    pub blog_author: String,
    pub cors_origins: Vec<String>,
    // 🆕 資料庫設定
    pub database_url: String,
    pub database_max_connections: u32,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into());
        let port = env::var("PORT")
            .unwrap_or_else(|_| "3000".into())
            .parse::<u16>()
            .context("PORT 必須是整數（u16）")?;
        
        let protocol = env::var("PROTOCOL").unwrap_or_else(|_| "http".into());
        let blog_name = env::var("BLOG_NAME").unwrap_or_else(|_| "個人部落格".into());
        let blog_description = env::var("BLOG_DESCRIPTION")
            .unwrap_or_else(|_| "分享想法與心得".into());
        let blog_author = env::var("BLOG_AUTHOR").unwrap_or_else(|_| "部落格作者".into());
        
        let cors_origins = env::var("CORS_ORIGIN")
            .unwrap_or_else(|_| "*".into())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        // 🆕 資料庫設定
        let database_url = env::var("DATABASE_URL")
            .context("請設定 DATABASE_URL 環境變數")?;
        let database_max_connections = env::var("DATABASE_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "10".into())
            .parse::<u32>()
            .context("DATABASE_MAX_CONNECTIONS 必須是整數")?;

        Ok(Self {
            host,
            port,
            protocol,
            blog_name,
            blog_description,
            blog_author,
            cors_origins,
            database_url,
            database_max_connections,
        })
    }

    pub fn server_url(&self) -> String {
        format!("{}://{}:{}", self.protocol, self.host, self.port)
    }

    // 為了安全，在日誌中隱藏敏感資訊
    pub fn sanitized_for_log(&self) -> Self {
        let mut config = self.clone();
        config.database_url = "postgres://***:***@***/***".to_string();
        config
    }
}