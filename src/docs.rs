use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use utoipa::{Modify, OpenApi};


#[derive(OpenApi)]
#[openapi(
    paths(
        // 健康檢查
        crate::routes::health::health_check,
        
        // 部落格資訊
        crate::routes::blog::blog_info,
        
        // 文章相關
        crate::routes::posts::get_posts,
        crate::routes::posts::get_post_by_slug,
        crate::routes::posts::create_post,
        crate::routes::posts::get_post_for_admin,
        crate::routes::posts::update_post,
        crate::routes::posts::delete_post,
        
        // 🆕 標籤相關
        crate::routes::tags::get_tags,
        crate::routes::tags::get_tag_by_id,
        crate::routes::tags::get_tag_with_posts,
        crate::routes::tags::create_tag,
        crate::routes::tags::update_tag,
        crate::routes::tags::delete_tag,
        crate::routes::tags::get_tag_suggestions,

        // Comment
        crate::routes::comments::get_comments,
        crate::routes::comments::create_comment,
        crate::routes::comments::get_admin_comments,
        crate::routes::comments::update_comment_status,
        crate::routes::comments::delete_comment,
    ),
    components(
        schemas(
            // 健康檢查
            crate::routes::health::HealthCheck,
            
            // 部落格資訊
            crate::routes::blog::BlogInfo,
            
            // 文章相關
            crate::dtos::CreatePostRequest,
            crate::dtos::UpdatePostRequest,
            crate::dtos::PostResponse,
            crate::dtos::PostDetailResponse,
            crate::dtos::PostListResponse,
            crate::dtos::DeletePostResponse,
            crate::dtos::PostListQuery,
            
            // 🆕 標籤相關
            crate::dtos::CreateTagRequest,
            crate::dtos::UpdateTagRequest,
            crate::dtos::TagResponse,
            crate::dtos::TagWithPostsResponse,
            crate::dtos::DeleteTagResponse,
            crate::dtos::TagListQuery,
            crate::dtos::TagSuggestionResponse,

            // Comment
            crate::dtos::CreateCommentRequest,
            crate::dtos::CommentResponse,
            crate::dtos::CommentListQuery,
            crate::dtos::UpdateCommentStatusRequest,
            crate::dtos::CommentModerationResponse,
        )
    ),
    tags(
        (name = "health", description = "系統健康狀態 API"),
        (name = "blog", description = "部落格基本資訊 API"),
        (name = "posts", description = "文章相關 API"),
        (name = "tags", description = "標籤相關 API"), // 🆕
        (name = "admin", description = "管理員 API"),
        (name = "comments", description = "留言相關 API"),
    )
)]
pub struct ApiDoc;