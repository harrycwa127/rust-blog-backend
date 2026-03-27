use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use validator::Validate;

use crate::{
    dtos::{CreatePostRequest, PostListQuery, PostListResponse, PostResponse, UpdatePostRequest, PostDetailResponse, DeletePostResponse},
    error::AppError,
    services::PostService,
    state::AppState,
};

pub fn create_post_routes() -> Router<AppState> {
    Router::new()
        .route("/posts", get(get_posts).post(create_post))
        .route("/posts/{identifier}", get(get_post_by_slug)) // ← 與文件一致
        .route(
            "/admin/posts/{id}",
            get(get_post_for_admin).put(update_post).delete(delete_post),
        )
}

/// 取得已發布文章列表
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

/// 根據 slug 或 id 取得文章詳情（公開查看）
#[utoipa::path(
    get,
    path = "/posts/{identifier}", // ← 跟上面 route 完全相同
    tag = "posts",
    params(
        ("identifier" = String, Path, description = "文章 slug 或 ID")
    ),
    responses(
        (status = 200, description = "文章詳情", body = PostDetailResponse),
        (status = 404, description = "文章不存在")
    )
)]
pub async fn get_post_by_slug(
    State(app_state): State<AppState>,
    Path(identifier): Path<String>,
) -> Result<Json<PostDetailResponse>, AppError> {
    if let Ok(post) = PostService::get_post_by_slug_or_id(&app_state.db, &identifier).await {
        let db = app_state.db.clone();
        let post_id = post.id;
        tokio::spawn(async move {
            let _ = PostService::increment_view_count(&db, post_id).await;
        });
        Ok(Json(post))
    } else {
        Err(AppError::NotFound("文章不存在".to_string()))
    }
}

#[utoipa::path(
    post,
    path = "/posts",
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
    req.validate().map_err(|e| AppError::ValidationError(e.to_string()))?;
    let post = PostService::create_post(&app_state.db, req).await?;
    Ok((StatusCode::CREATED, Json(post)))
}

#[utoipa::path(
    get,
    path = "/admin/posts/{id}",
    tag = "admin",
    params(("id" = i32, Path, description = "文章 ID")),
    responses(
        (status = 200, description = "文章詳情", body = PostDetailResponse),
        (status = 404, description = "文章不存在")
    )
)]
pub async fn get_post_for_admin(
    State(app_state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<PostDetailResponse>, AppError> {
    let post = PostService::get_post_for_admin(&app_state.db, id).await?;
    Ok(Json(post))
}

#[utoipa::path(
    put,
    path = "/admin/posts/{id}",
    tag = "admin",
    params(("id" = i32, Path, description = "文章 ID")),
    request_body = UpdatePostRequest,
    responses(
        (status = 200, description = "文章更新成功", body = PostDetailResponse),
        (status = 400, description = "請求資料錯誤"),
        (status = 404, description = "文章不存在"),
        (status = 409, description = "Slug 衝突")
    )
)]
pub async fn update_post(
    State(app_state): State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<UpdatePostRequest>,
) -> Result<Json<PostDetailResponse>, AppError> {
    req.validate().map_err(|e| AppError::ValidationError(e.to_string()))?;
    let post = PostService::update_post(&app_state.db, id, req).await?;
    Ok(Json(post))
}

#[utoipa::path(
    delete,
    path = "/admin/posts/{id}",
    tag = "admin",
    params(("id" = i32, Path, description = "文章 ID")),
    responses(
        (status = 200, description = "文章刪除成功", body = DeletePostResponse),
        (status = 404, description = "文章不存在")
    )
)]
pub async fn delete_post(
    State(app_state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<DeletePostResponse>, AppError> {
    let result = PostService::delete_post(&app_state.db, id).await?;
    Ok(Json(result))
}
