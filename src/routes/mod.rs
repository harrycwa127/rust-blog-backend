use axum::{routing::get, Router};
use crate::config::Config;

pub mod blog;
pub mod health;

pub fn router() -> Router<Config> {
    Router::new()
        .route("/", get(blog::blog_info))
        .route("/health", get(health::health_check))
}