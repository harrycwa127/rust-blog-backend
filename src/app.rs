use axum::{middleware, Router};
use http::HeaderValue;
use tower_http::cors::{Any, AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{docs::ApiDoc, middleware::request_logging, routes, state::AppState};

pub fn build_app(app_state: AppState) -> Router {
    let cors = if app_state.config.cors_origins.iter().any(|o| o == "*") {
        // 開發期萬用
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        // 嚴格白名單
        let origins = app_state
            .config
            .cors_origins
            .iter()
            .filter_map(|o| HeaderValue::from_str(o).ok());
        CorsLayer::new()
            .allow_origin(AllowOrigin::list(origins))
            .allow_methods(Any)
            .allow_headers(Any)
    };

    let api = routes::create_routes().with_state(app_state);

    Router::new()
        .merge(api)
        .merge(
            SwaggerUi::new("/docs")
                .url("/api-docs/openapi.json", ApiDoc::openapi()),
        )
        .layer(middleware::from_fn(request_logging))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}