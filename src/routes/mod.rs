use axum::{routing::get, Router};

use crate::state::AppState;

pub mod blog;
pub mod health;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(blog::blog_info))
        .route("/health", get(health::health_check))
}