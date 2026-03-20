use utoipa::OpenApi;

use crate::routes::{blog::BlogInfo, health::HealthCheck};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::blog::blog_info,
        crate::routes::health::health_check,
    ),
    components(
        schemas(BlogInfo, HealthCheck)
    ),
    tags(
        (name = "blog", description = "部落格資訊相關 API"),
        (name = "health", description = "服務健康檢查")
    ),
    info(
        title = "個人部落格 API",
        version = "0.1.0",
        description = "個人部落格後端 API 服務"
    )
)]
pub struct ApiDoc;