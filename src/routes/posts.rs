use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use tracing::{debug, info};
use validator::Validate;

use crate::{
    cache::PostCache, dtos::{CreatePostRequest, DeletePostResponse, PostDetailResponse, PostListQuery, PostListResponse, PostResponse, UpdatePostRequest}, error::AppError, services::PostService, state::AppState
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
    // 產生快取鍵
    let cache_key = PostCache::generate_list_cache_key(
        query.page,
        query.page_size,
        query.tag.as_deref(),
    );

    // 先嘗試從快取取得
    if let Some(cached_posts) = app_state.post_cache.get_post_list(&cache_key).await {
        debug!("✅ 文章列表快取命中：{}", cache_key);
        return Ok(Json(cached_posts));
    }

    // 快取未命中，從資料庫查詢
    debug!("❌ 文章列表快取未命中，查詢資料庫：{}", cache_key);
    let posts = PostService::get_published_posts(&app_state.db, query).await?;

    // 將結果存入快取
    app_state.post_cache.cache_post_list(cache_key.clone(), posts.clone()).await;
    debug!("💾 文章列表已快取：{}", cache_key);

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
    Path(slug): Path<String>,
) -> Result<Json<PostDetailResponse>, AppError> {
    // 先嘗試從快取取得
    if let Some(cached_post) = app_state.post_cache.get_post_detail(&slug).await {
        debug!("✅ 文章詳情快取命中：{}", slug);
        
        // 非同步更新瀏覽次數（不阻塞回應）
        let db_clone = app_state.db.clone();
        let post_id = cached_post.id;
        tokio::spawn(async move {
            let _ = PostService::increment_view_count(&db_clone, post_id).await;
        });
        
        return Ok(Json(cached_post));
    }

    // 快取未命中，從資料庫查詢
    debug!("❌ 文章詳情快取未命中，查詢資料庫：{}", slug);
    
    let post = PostService::get_post_by_slug_or_id(&app_state.db, &slug).await?;
    
    // 非同步更新瀏覽次數
    let db_clone = app_state.db.clone();
    let post_id = post.id;
    tokio::spawn(async move {
        let _ = PostService::increment_view_count(&db_clone, post_id).await;
    });
    
    // 將結果存入快取
    app_state.post_cache.cache_post_detail(slug.clone(), post.clone()).await;
    debug!("💾 文章詳情已快取：{}", slug);

    Ok(Json(post))
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
) -> Result<(StatusCode, Json<crate::dtos::PostResponse>), AppError> {
    req.validate().map_err(|e| AppError::ValidationError(e.to_string()))?;
    
    let post = PostService::create_post(&app_state.db, req).await?;
    
    // 新增文章後清理相關快取
    app_state.post_cache.invalidate_all().await;
    app_state.tag_cache.invalidate_all().await;
    
    info!("📝 文章建立成功，已清理相關快取");
    
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
    
    // 先獲取舊文章資訊以便清理快取
    let old_post = PostService::get_post_for_admin(&app_state.db, id).await?;
    
    let updated_post = PostService::update_post(&app_state.db, id, req).await?;
    
    // 清理相關快取
    app_state.post_cache.invalidate_post(&old_post.slug).await;
    if old_post.slug != updated_post.slug {
        app_state.post_cache.invalidate_post(&updated_post.slug).await;
    }
    app_state.tag_cache.invalidate_all().await;
    
    info!("✏️ 文章更新成功，已清理相關快取");
    
    Ok(Json(updated_post))
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
) -> Result<Json<crate::dtos::DeletePostResponse>, AppError> {
    // 先獲取文章資訊以便清理快取
    let post = PostService::get_post_for_admin(&app_state.db, id).await?;
    
    let result = PostService::delete_post(&app_state.db, id).await?;
    
    // 清理相關快取
    app_state.post_cache.invalidate_post(&post.slug).await;
    app_state.tag_cache.invalidate_all().await;
    
    info!("🗑️ 文章刪除成功，已清理相關快取");
    
    Ok(Json(result))
}