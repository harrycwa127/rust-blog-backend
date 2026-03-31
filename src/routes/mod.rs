use axum::{routing::get, Router};
use axum::routing::{delete, post, put};
use crate::auth_handlers::{admin_info, admin_login};
use crate::auth_middleware::auth_middleware;
use crate::state::AppState;


pub mod blog;
pub mod health;
pub mod posts;
pub mod tags;
pub mod comments;

pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/", get(blog::blog_info))
        .route("/health", get(health::health_check))
        .merge(posts::create_post_routes())
        .merge(tags::create_tag_routes())
        .merge(comments::create_comment_routes())
        // 登入端點為公開
        .route("/api/admin/login", post(admin_login))
}

pub fn admin_router() -> Router<AppState> {
    Router::new()
        .route("/info", get(admin_info))
        // posts 管理端點
        .route("/posts", post(posts::create_post))
        .route("/posts/{id}", get(posts::get_post_for_admin))
        .route("/posts/{id}", put(posts::update_post))
        .route("/posts/{id}", delete(posts::delete_post))
        .route("/posts/search", get(posts::admin_search_posts))
        // tags 管理端點
        .route("/tags", post(tags::create_tag))
        .route("/tags/{id}", get(tags::get_tag_by_id))
        .route("/tags/{id}", put(tags::update_tag))
        .route("/tags/{id}", delete(tags::delete_tag))
        // comments 管理端點
        .route("/comments", get(comments::get_admin_comments))
        .route("/comments/{id}", put(comments::update_comment_status))
        .route("/comments/{id}", delete(comments::delete_comment))
}
