use utoipa::{Modify, OpenApi, openapi::security::{ApiKey, ApiKeyValue, SecurityScheme}};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::health::health_check,
        crate::routes::blog::blog_info,
        crate::routes::posts::get_posts,
        crate::routes::posts::get_post_by_slug,
        crate::routes::posts::create_post,
        crate::routes::posts::get_post_for_admin,
        crate::routes::posts::update_post,
        crate::routes::posts::delete_post,
    ),
    components(
        schemas(
            crate::routes::health::HealthCheck,
            crate::routes::blog::BlogInfo,
            crate::dtos::CreatePostRequest,
            crate::dtos::UpdatePostRequest,
            crate::dtos::PostResponse,
            crate::dtos::PostDetailResponse,
            crate::dtos::PostListResponse,
            crate::dtos::DeletePostResponse,
            crate::dtos::PostListQuery,
        )
    ),
    tags(
        (name = "health", description = "系統健康檢查"),
        (name = "blog", description = "部落格基本資訊"),
        (name = "posts", description = "文章相關 API"),
        (name = "admin", description = "管理員 API"),
    ),
    info(
        title = "個人部落格 API",
        version = "0.1.0",
        description = "個人部落格後端 API 文件",
        contact(
            name = "API 支援",
            email = "support@example.com"
        )
    ),
    servers(
        (url = "http://localhost:3000", description = "本地開發環境"),
        (url = "https://api.myblog.com", description = "正式環境")
    )
)]
pub struct ApiDoc;

impl Modify for ApiDoc {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("Authorization"))),
            );
        }
    }
}