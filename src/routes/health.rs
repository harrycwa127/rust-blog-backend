use axum::Json;
use serde::Serialize;
use tracing::{info, instrument};
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct HealthCheck {
    status: String,
    timestamp: String,
    version: String,
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "健康檢查", body = HealthCheck)
    )
)]
#[instrument]
pub async fn health_check() -> Json<HealthCheck> {
    info!("執行健康檢查");

    Json(HealthCheck {
        status: "healthy".into(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}