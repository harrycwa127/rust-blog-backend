use axum::{
    extract::{Path, Query, State, ConnectInfo},
    http::{StatusCode, HeaderMap},
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use std::net::SocketAddr;

use crate::{
    dtos::{
        CreateCommentRequest, CommentResponse, CommentListQuery, 
        UpdateCommentStatusRequest, CommentModerationResponse
    },
    error::AppError,
    services::CommentService,
    state::AppState,
};

pub fn create_comment_routes() -> Router<AppState> {
    Router::new()
        .route("/posts/{post_id}/comments", get(get_comments).post(create_comment))
        .route("/admin/comments", get(get_admin_comments))
        .route("/admin/comments/{id}", put(update_comment_status).delete(delete_comment))
}

/// 取得文章的留言列表
#[utoipa::path(
    get,
    path = "/posts/{post_id}/comments",
    tag = "comments",
    params(
        ("post_id" = i32, Path, description = "文章 ID"),
        CommentListQuery
    ),
    responses(
        (status = 200, description = "留言列表", body = Vec<CommentResponse>),
        (status = 404, description = "文章不存在")
    )
)]
pub async fn get_comments(
    State(app_state): State<AppState>,
    Path(post_id): Path<i32>,
    Query(query): Query<CommentListQuery>,
) -> Result<Json<Vec<CommentResponse>>, AppError> {
    let comments = CommentService::get_comments_for_post(&app_state.db, post_id, query).await?;
    Ok(Json(comments))
}

/// 為文章建立留言
#[utoipa::path(
    post,
    path = "/posts/{post_id}/comments",
    tag = "comments",
    params(("post_id" = i32, Path, description = "文章 ID")),
    request_body = CreateCommentRequest,
    responses(
        (status = 201, description = "留言建立成功", body = CommentResponse),
        (status = 400, description = "請求資料錯誤"),
        (status = 404, description = "文章不存在"),
        (status = 422, description = "驗證失敗")
    )
)]
pub async fn create_comment(
    State(app_state): State<AppState>,
    Path(post_id): Path<i32>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(req): Json<CreateCommentRequest>,
) -> Result<(StatusCode, Json<CommentResponse>), AppError> {
    // 取得 IP 地址
    let ip_address = Some(addr.ip().to_string());
    
    // 取得 User-Agent
    let user_agent = headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    let comment = CommentService::create_comment(&app_state.db, post_id, req, ip_address, user_agent).await?;
    Ok((StatusCode::CREATED, Json(comment)))
}

/// 取得留言列表（管理員用）
#[utoipa::path(
    get,
    path = "/admin/comments",
    tag = "admin",
    params(CommentListQuery),
    responses(
        (status = 200, description = "留言列表", body = Vec<CommentModerationResponse>),
        (status = 401, description = "需要管理員權限")
    )
)]
pub async fn get_admin_comments(
    State(app_state): State<AppState>,
    Query(query): Query<CommentListQuery>,
) -> Result<Json<Vec<CommentModerationResponse>>, AppError> {
    let comments = CommentService::get_comments_for_admin(&app_state.db, query).await?;
    Ok(Json(comments))
}

/// 更新留言狀態（管理員審核）
#[utoipa::path(
    put,
    path = "/admin/comments/{id}",
    tag = "admin",
    params(("id" = i32, Path, description = "留言 ID")),
    request_body = UpdateCommentStatusRequest,
    responses(
        (status = 200, description = "留言狀態更新成功", body = CommentResponse),
        (status = 404, description = "留言不存在"),
        (status = 401, description = "需要管理員權限")
    )
)]
pub async fn update_comment_status(
    State(app_state): State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateCommentStatusRequest>,
) -> Result<Json<CommentResponse>, AppError> {
    let comment = CommentService::update_comment_status(&app_state.db, id, req).await?;
    Ok(Json(comment))
}

/// 刪除留言（管理員用）
#[utoipa::path(
    delete,
    path = "/admin/comments/{id}",
    tag = "admin",
    params(("id" = i32, Path, description = "留言 ID")),
    responses(
        (status = 204, description = "留言刪除成功"),
        (status = 404, description = "留言不存在"),
        (status = 401, description = "需要管理員權限")
    )
)]
pub async fn delete_comment(
    State(app_state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    CommentService::delete_comment(&app_state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}