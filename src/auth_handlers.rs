use crate::{
    auth::{Claims, LoginRequest, LoginResponse},
    error::AppError,
    state::AppState,
};
use axum::{extract::{State, Extension}, Json};
use std::env;

/// 管理員登入取得 JWT
#[utoipa::path(
    post,
    path = "/api/admin/login",
    tag = "admin",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "登入成功，回傳 JWT", body = LoginResponse),
        (status = 401, description = "用戶名或密碼錯誤")
    )
)]
pub async fn admin_login(
    State(app_state): State<AppState>,
    Json(login_request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    // 簡單的管理員驗證
    let admin_username = env::var("ADMIN_USERNAME")
        .unwrap_or_else(|_| "admin".to_string());
    let admin_password = env::var("ADMIN_PASSWORD")
        .unwrap_or_else(|_| "admin123".to_string());

    // 驗證用戶名和密碼
    if login_request.username != admin_username || login_request.password != admin_password {
        return Err(AppError::Unauthorized("用戶名或密碼錯誤".to_string()));
    }

    // 生成 JWT token
    let token = app_state
        .jwt_service
        .generate_token(&admin_username)
        .map_err(|_| AppError::InternalServerError("Token 生成失敗".to_string()))?;

    Ok(Json(LoginResponse {
        token,
        expires_in: 24 * 3600, // 24 小時
    }))
}

/// 取得當前管理員資訊
#[utoipa::path(
    get,
    path = "/api/admin/info",
    tag = "admin",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "當前管理員資訊", body = serde_json::Value),
        (status = 401, description = "未授權")
    )
)]
pub async fn admin_info(Extension(claims): Extension<Claims>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "user_id": claims.sub,
        "role": claims.role,
        "expires_at": claims.exp
    }))
}