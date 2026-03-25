use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{database, state::AppState};

#[derive(Serialize, Deserialize, ToSchema)]
pub struct HealthCheck {
    #[schema(example = "healthy")]
    pub status: String,
    #[schema(example = "2024-01-15T10:30:00Z")]
    pub timestamp: String,
    #[schema(example = "0.1.0")]
    pub version: String,
    // 🆕 資料庫狀態
    #[schema(example = "connected")]
    pub database: String,
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "服務健康狀態", body = HealthCheck),
        (status = 503, description = "服務不可用")
    )
)]
pub async fn health_check(State(app_state): State<AppState>) -> Json<HealthCheck> {
    // 🆕 檢查資料庫連線
    let database_status = match database::health_check(&app_state.db).await {
        Ok(_) => "connected".to_string(),
        Err(_) => "disconnected".to_string(),
    };

    Json(HealthCheck {
        status: if database_status == "connected" {
            "healthy".to_string()
        } else {
            "unhealthy".to_string()
        },
        timestamp: chrono::Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        database: database_status,
    })
}