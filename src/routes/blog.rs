use axum::{extract::State, Json};
use serde::Serialize;
use tracing::{info, instrument};
use utoipa::ToSchema;

use crate::{config::Config, error::AppError};

#[derive(Serialize, ToSchema)]
pub struct BlogInfo {
    name: String,
    description: String,
    author: String,
    status: String,
    timestamp: String,
}

#[utoipa::path(
    get,
    path = "/",
    tag = "blog",
    responses(
        (status = 200, description = "取得部落格資訊", body = BlogInfo)
    )
)]
#[instrument]
pub async fn blog_info(
    State(config): State<Config>,
) -> Result<Json<BlogInfo>, AppError> {
    info!("回傳部落格資訊");

    Ok(Json(BlogInfo {
        name: config.blog_name.clone(),
        description: config.blog_description.clone(),
        author: config.blog_author.clone(),
        status: "running".into(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    }))
}