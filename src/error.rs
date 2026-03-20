use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("內部伺服器錯誤")]
    InternalServerError,
    #[error("找不到資源: {0}")]
    NotFound(String),
    #[error("請求無效: {0}")]
    BadRequest(String),
    #[error("未授權")]
    Unauthorized,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::InternalServerError => {
                tracing::error!("內部伺服器錯誤");
                (StatusCode::INTERNAL_SERVER_ERROR, "內部伺服器錯誤")
            }
            AppError::NotFound(msg) => {
                tracing::warn!("資源未找到: {}", msg);
                (StatusCode::NOT_FOUND, "找不到資源")
            }
            AppError::BadRequest(msg) => {
                tracing::warn!("請求無效: {}", msg);
                (StatusCode::BAD_REQUEST, "請求無效")
            }
            AppError::Unauthorized => {
                tracing::warn!("未授權的存取嘗試");
                (StatusCode::UNAUTHORIZED, "未授權")
            }
        };

        let body = Json(json!({
            "error": error_message,
            "status": status.as_u16(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }));

        (status, body).into_response()
    }
}