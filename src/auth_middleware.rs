use axum::{
    extract::{Request, State as AxumState},
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};
use crate::{error::AppError, state::AppState};

pub async fn auth_middleware(
    AxumState(app_state): AxumState<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {

    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("缺少認證資訊".to_string()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Unauthorized("無效的認證格式".to_string()))?;

    let claims = app_state
        .jwt_service
        .verify_token(token)
        .map_err(|_| AppError::Unauthorized("無效的認證 token".to_string()))?;

    // 將用戶資訊加入請求
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}