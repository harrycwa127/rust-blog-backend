pub mod health;
pub mod blog;
pub mod posts;
pub mod tags; // 新增

use axum::Router;
use crate::state::AppState;

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .merge(health::create_health_routes())
        .merge(blog::create_blog_routes())
        .merge(posts::create_post_routes())
        .merge(tags::create_tag_routes()) // 新增
}