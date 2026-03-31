use crate::{rate_limit::rate_limit_middleware, routes, state::AppState};
use axum::{middleware, Router};
use std::env;
use tower_http::cors::{Any, CorsLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use crate::docs::ApiDoc;
use crate::auth_middleware::auth_middleware;

pub fn build_app(app_state: AppState) -> Router {
    let cors = if env::var("ENVIRONMENT").unwrap_or_default() == "production" {
        let allowed_origins = env::var("CORS_ORIGINS")
            .unwrap_or_else(|_| "https://yourdomain.com".to_string())
            .split(',')
            .filter_map(|origin| origin.parse().ok())
            .collect::<Vec<_>>();

        CorsLayer::new()
            .allow_origin(allowed_origins)
            .allow_methods([
                http::Method::GET,
                http::Method::POST,
                http::Method::PUT,
                http::Method::DELETE,
            ])
            .allow_headers([
                http::header::AUTHORIZATION,
                http::header::CONTENT_TYPE,
            ])
    } else {
        // 開發環境：寬鬆設定
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    };

    let public = routes::public_router();
    let admin = routes::admin_router();
    let admin_protected = admin.layer(middleware::from_fn_with_state(app_state.clone(), auth_middleware));

    // Swagger UI 與 OpenAPI 提供路由
    let swagger_router = Router::new().merge(
        SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()),
    );

    Router::new()
        .merge(swagger_router)
        .merge(public)
        .nest("/api/admin", admin_protected)
        .layer(middleware::from_fn(rate_limit_middleware))
        .layer(cors)
        .with_state(app_state)
}