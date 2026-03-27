use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use validator::Validate;

use crate::{
    dtos::{
        CreateTagRequest, UpdateTagRequest, TagResponse, TagWithPostsResponse,
        DeleteTagResponse, TagListQuery, TagSuggestionResponse,
    },
    error::AppError,
    services::TagService,
    state::AppState,
};

pub fn create_tag_routes() -> Router<AppState> {
    Router::new()
        .route("/tags", get(get_tags).post(create_tag))
        .route("/tags/suggestions", get(get_tag_suggestions))
        .route("/tags/:name/posts", get(get_tag_with_posts))
        .route(
            "/admin/tags/:id",
            get(get_tag_by_id).put(update_tag).delete(delete_tag),
        )
}

/// 取得標籤列表
#[utoipa::path(
    get,
    path = "/tags",
    tag = "tags",
    params(TagListQuery),
    responses(
        (status = 200, description = "標籤列表", body = Vec<TagResponse>),
        (status = 400, description = "請求參數錯誤")
    )
)]
pub async fn get_tags(
    State(app_state): State<AppState>,
    Query(query): Query<TagListQuery>,
) -> Result<Json<Vec<TagResponse>>, AppError> {
    let tags = TagService::get_tags(&app_state.db, query).await?;
    Ok(Json(tags))
}

/// 根據 ID 取得單一標籤（管理員用）
#[utoipa::path(
    get,
    path = "/admin/tags/{id}",
    tag = "admin",
    params(("id" = i32, Path, description = "標籤 ID")),
    responses(
        (status = 200, description = "標籤詳情", body = TagResponse),
        (status = 404, description = "標籤不存在")
    )
)]
pub async fn get_tag_by_id(
    State(app_state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<TagResponse>, AppError> {
    let tag = TagService::get_tag_by_id(&app_state.db, id).await?;
    Ok(Json(tag))
}

/// 根據名稱取得標籤及其文章
#[utoipa::path(
    get,
    path = "/tags/{name}/posts",
    tag = "tags",
    params(
        ("name" = String, Path, description = "標籤名稱"),
        ("page" = Option<u64>, Query, description = "頁碼"),
        ("page_size" = Option<u64>, Query, description = "每頁筆數")
    ),
    responses(
        (status = 200, description = "標籤及其文章", body = TagWithPostsResponse),
        (status = 404, description = "標籤不存在")
    )
)]
pub async fn get_tag_with_posts(
    State(app_state): State<AppState>,
    Path(name): Path<String>,
    Query(params): Query<TagListQuery>,
) -> Result<Json<TagWithPostsResponse>, AppError> {
    let tag_with_posts = TagService::get_tag_with_posts(
        &app_state.db, 
        &name, 
        params.page, 
        params.page_size
    ).await?;
    Ok(Json(tag_with_posts))
}

/// 建立新標籤
#[utoipa::path(
    post,
    path = "/tags",
    tag = "admin",
    request_body = CreateTagRequest,
    responses(
        (status = 201, description = "標籤建立成功", body = TagResponse),
        (status = 400, description = "請求參數錯誤"),
        (status = 409, description = "標籤已存在")
    )
)]
pub async fn create_tag(
    State(app_state): State<AppState>,
    Json(req): Json<CreateTagRequest>,
) -> Result<(StatusCode, Json<TagResponse>), AppError> {
    req.validate().map_err(|e| AppError::ValidationError(e.to_string()))?;
    
    let tag = TagService::create_tag(&app_state.db, req).await?;
    Ok((StatusCode::CREATED, Json(tag)))
}

/// 更新標籤
#[utoipa::path(
    put,
    path = "/admin/tags/{id}",
    tag = "admin",
    params(("id" = i32, Path, description = "標籤 ID")),
    request_body = UpdateTagRequest,
    responses(
        (status = 200, description = "標籤更新成功", body = TagResponse),
        (status = 400, description = "請求參數錯誤"),
        (status = 404, description = "標籤不存在"),
        (status = 409, description = "標籤名稱已存在")
    )
)]
pub async fn update_tag(
    State(app_state): State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateTagRequest>,
) -> Result<Json<TagResponse>, AppError> {
    req.validate().map_err(|e| AppError::ValidationError(e.to_string()))?;
    
    let tag = TagService::update_tag(&app_state.db, id, req).await?;
    Ok(Json(tag))
}

/// 刪除標籤
#[utoipa::path(
    delete,
    path = "/admin/tags/{id}",
    tag = "admin",
    params(("id" = i32, Path, description = "標籤 ID")),
    responses(
        (status = 200, description = "標籤刪除成功", body = DeleteTagResponse),
        (status = 404, description = "標籤不存在")
    )
)]
pub async fn delete_tag(
    State(app_state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<DeleteTagResponse>, AppError> {
    let result = TagService::delete_tag(&app_state.db, id).await?;
    Ok(Json(result))
}

/// 取得標籤建議
#[utoipa::path(
    get,
    path = "/tags/suggestions",
    tag = "tags",
    params(
        ("query" = Option<String>, Query, description = "搜尋關鍵字"),
        ("limit" = Option<u64>, Query, description = "建議數量限制")
    ),
    responses(
        (status = 200, description = "標籤建議", body = TagSuggestionResponse),
        (status = 400, description = "請求參數錯誤")
    )
)]
pub async fn get_tag_suggestions(
    State(app_state): State<AppState>,
    Query(params): Query<serde_json::Value>,
) -> Result<Json<TagSuggestionResponse>, AppError> {
    let query = params.get("query").and_then(|v| v.as_str()).map(|s| s.to_string());
    let limit = params.get("limit").and_then(|v| v.as_u64());
    
    let suggestions = TagService::get_tag_suggestions(&app_state.db, query, limit).await?;
    Ok(Json(suggestions))
}