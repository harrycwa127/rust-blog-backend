use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::state::AppState;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct BlogInfo {
    #[schema(example = "我的個人技術部落格")]
    pub name: String,
    #[schema(example = "分享程式設計學習心得與生活感悟")]
    pub description: String,
    #[schema(example = "你的名字")]
    pub author: String,
    #[schema(example = "0.1.0")]
    pub version: String,
    #[schema(example = "2024-01-15T10:30:00Z")]
    pub timestamp: String,
}

#[utoipa::path(
    get,
    path = "/",
    tag = "blog",
    responses(
        (status = 200, description = "部落格基本資訊", body = BlogInfo)
    )
)]
pub async fn blog_info(State(app_state): State<AppState>) -> Json<BlogInfo> {
    Json(BlogInfo {
        name: app_state.config.blog_name.clone(),
        description: app_state.config.blog_description.clone(),
        author: app_state.config.blog_author.clone(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}