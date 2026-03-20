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
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into());
        let port = env::var("PORT")
            .unwrap_or_else(|_| "3000".into())
            .parse::<u16>()
            .context("PORT 必須是整數（u16）")?;
        let protocol = env::var("PROTOCOL").unwrap_or_else(|_| "http".into());
        let blog_name = env::var("BLOG_NAME").unwrap_or_else(|_| "我的個人技術部落格".into());
        let blog_description = env::var("BLOG_DESCRIPTION")
            .unwrap_or_else(|_| "分享程式設計學習心得與生活感悟".into());
        let blog_author = env::var("BLOG_AUTHOR").unwrap_or_else(|_| "Blog Author".into());
        let cors_origins = env::var("CORS_ORIGIN")
            .unwrap_or_else(|_| "*".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        Ok(Self {
            host,
            port,
            protocol,
            blog_name,
            blog_description,
            blog_author,
            cors_origins,
        })
    }

    pub fn server_url(&self) -> String {
        format!("{}://{}:{}", self.protocol, self.host, self.port)
    }

    pub fn sanitized_for_log(&self) -> Self {
        self.clone()
    }
}