pub mod health;
pub mod blog;
pub mod posts;

use axum::{
    routing::{get, post},
    Router,
};
use crate::state::AppState;
use posts::create_post_routes;

/// 建立完整路由
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(blog::blog_info))
        .route("/health", get(health::health_check))
        .nest("/api", create_post_routes())
}