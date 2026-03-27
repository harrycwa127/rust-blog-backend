use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateCommentRequest {
    #[validate(length(min = 1, max = 100, message = "姓名長度必須在 1-100 字元之間"))]
    #[schema(example = "讀者小明")]
    pub author_name: String,

    #[validate(email(message = "請提供有效的電子郵件地址"))]
    #[validate(length(max = 255, message = "電子郵件長度不能超過 255 字元"))]
    #[schema(example = "reader@example.com")]
    pub author_email: String,

    #[validate(url(message = "請提供有效的網址格式"))]
    #[validate(length(max = 255, message = "網站 URL 長度不能超過 255 字元"))]
    #[schema(example = "https://example.com")]
    pub author_website: Option<String>,

    #[validate(length(min = 1, max = 2000, message = "留言內容長度必須在 1-2000 字元之間"))]
    #[schema(example = "很棒的文章！學到很多東西。")]
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CommentResponse {
    #[schema(example = 1)]
    pub id: i32,

    #[schema(example = 1)]
    pub post_id: i32,

    #[schema(example = "讀者小明")]
    pub author_name: String,

    #[schema(example = "https://example.com")]
    pub author_website: Option<String>,

    #[schema(example = "很棒的文章！學到很多東西。")]
    pub content: String,

    #[schema(example = "approved")]
    pub status: String,

    #[schema(value_type = String, example = "2024-01-15T10:30:00Z")]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct CommentListQuery {
    #[schema(example = 1)]
    pub page: Option<u64>,

    #[schema(example = 20)]
    pub page_size: Option<u64>,

    #[schema(example = "approved")]
    pub status: Option<String>, // pending, approved, rejected, all (admin only)

    #[schema(example = "desc")]
    pub sort_order: Option<String>, // asc, desc
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateCommentStatusRequest {
    #[schema(example = "approved")]
    pub status: String, // approved, rejected, pending

    #[schema(example = "留言內容符合社群規範")]
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CommentModerationResponse {
    #[schema(example = 1)]
    pub id: i32,

    #[schema(example = 1)]
    pub post_id: i32,

    #[schema(example = "讀者小明")]
    pub author_name: String,

    #[schema(example = "reader@example.com")]
    pub author_email: String,

    #[schema(example = "https://example.com")]
    pub author_website: Option<String>,

    #[schema(example = "很棒的文章！學到很多東西。")]
    pub content: String,

    #[schema(example = "pending")]
    pub status: String,

    #[schema(example = "192.168.1.1")]
    pub ip_address: Option<String>,

    #[schema(value_type = String, example = "2024-01-15T10:30:00Z")]
    pub created_at: chrono::DateTime<chrono::Utc>,

    #[schema(example = "Rust學習心得分享")]
    pub post_title: String,
}