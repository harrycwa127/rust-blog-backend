use axum::{routing::get, Json, Router};
use serde::Serialize;
use tokio::net::TcpListener;
use std::env;

// ---- 新增：utoipa 匯入 ----
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

#[derive(Serialize, ToSchema)]
struct BlogInfo {
    name: String,
    description: String,
    author: String,
    status: String,
}

#[utoipa::path(
    get,
    path = "/",
    tag = "blog",
    responses(
        (status = 200, description = "取得部落格資訊", body = BlogInfo)
    )
)]
async fn blog_info() -> Json<BlogInfo> {
    Json(BlogInfo {
        name: "我的個人技術部落格".to_string(),
        description: "分享程式設計學習心得與生活感悟".to_string(),
        author: "你的名字".to_string(),
        status: "running".to_string(),
    })
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "健康檢查 OK", body = String)
    )
)]
async fn health_check() -> &'static str {
    "個人部落格運行正常！"
}

// ---- 新增：彙整 API 文件 ----
#[derive(OpenApi)]
#[openapi(
    paths(
        blog_info,
        health_check,
    ),
    components(
        schemas(BlogInfo)
    ),
    tags(
        (name = "blog", description = "部落格資訊相關 API"),
        (name = "health", description = "服務健康檢查")
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    // 載入 .env（沒有就略過）
    let _ = dotenvy::dotenv();

    // 從環境變數讀 HOST/PORT，沒有就用預設
    let protocol = env::var("PROTOCOL").unwrap_or_else(|_| "http".into());
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port = env::var("PORT").unwrap_or_else(|_| "3000".into());
    let addr = format!("{host}:{port}");

    // 你的原本路由
    let app = Router::new()
        .route("/", get(blog_info))
        .route("/health", get(health_check))
        .merge(
            SwaggerUi::new("/docs")
                .url("/api-docs/openapi.json", ApiDoc::openapi()),
        );

    let listener = TcpListener::bind(&addr).await.unwrap();
    println!("🚀 個人部落格服務啟動於 {protocol}://{addr} ；Swagger UI 在 {protocol}://{addr}/docs");
    axum::serve(listener, app).await.unwrap();
}