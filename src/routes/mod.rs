pub mod health;
pub mod blog;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use crate::{error::AppError, services::PostService, state::AppState};
use crate::dtos::{CreatePostRequest, PostListQuery, PostListResponse, PostResponse};

/// 建立完整路由
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(blog::blog_info))
        .route("/health", get(health::health_check))
        .nest("/api", create_post_routes())
}

/// 建立文章相關路由（會被 nest 到 /api 底下）
pub fn create_post_routes() -> Router<AppState> {
    Router::new()
        .route("/posts", get(get_posts))          // 實際對外是 /api/posts
        .route("/admin/posts", post(create_post)) // 實際對外是 /api/admin/posts
}


#[utoipa::path(
    get,
    path = "/posts",
    tag = "posts",
    params(PostListQuery),
    responses(
        (status = 200, description = "文章列表", body = Vec<PostListResponse>),
        (status = 400, description = "請求參數錯誤")
    )
)]
pub async fn get_posts(
    State(app_state): State<AppState>,
    Query(query): Query<PostListQuery>,
) -> Result<Json<Vec<PostListResponse>>, AppError> {
    let posts = PostService::get_published_posts(&app_state.db, query).await?;
    Ok(Json(posts))
}

/// 建立文章 (管理員 API)
#[utoipa::path(
    post,
    path = "/admin/posts",
    tag = "admin",
    request_body = CreatePostRequest,
    responses(
        (status = 201, description = "文章建立成功", body = PostResponse),
        (status = 400, description = "請求資料錯誤"),
        (status = 409, description = "Slug 已存在")
    )
)]
pub async fn create_post(
    State(app_state): State<AppState>,
    Json(req): Json<CreatePostRequest>,
) -> Result<(StatusCode, Json<PostResponse>), AppError> {
    let post = PostService::create_post(&app_state.db, req).await?;
    Ok((StatusCode::CREATED, Json(post)))
}