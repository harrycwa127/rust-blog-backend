use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreatePostRequest {
    #[validate(length(min = 1, max = 255, message = "標題長度必須在 1-255 字元之間"))]
    #[schema(example = "我的第一篇 Rust 文章")]
    pub title: String,

    #[validate(length(min = 1, message = "內容不能為空"))]
    #[schema(example = "今天開始學習 Rust，發現它真的很棒...")]
    pub content: String,

    #[validate(length(max = 500, message = "摘要不能超過 500 字元"))]
    #[schema(example = "這篇文章分享我學習 Rust 的心得")]
    pub excerpt: Option<String>,

    #[schema(example = "my-first-rust-article")]
    pub slug: Option<String>,

    #[schema(example = false)]
    pub is_published: Option<bool>,

    #[schema(example = json!(["rust", "程式設計", "學習心得"]))]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PostResponse {
    #[schema(example = 1)]
    pub id: i32,

    #[schema(example = "我的第一篇 Rust 文章")]
    pub title: String,

    #[schema(example = "今天開始學習 Rust...")]
    pub content: String,

    #[schema(example = "這篇文章分享我學習 Rust 的心得")]
    pub excerpt: Option<String>,

    #[schema(example = "my-first-rust-article")]
    pub slug: String,

    #[schema(example = true)]
    pub is_published: bool,

    #[schema(example = 42)]
    pub view_count: i32,

    #[schema(value_type = String, example = "2024-01-15T10:30:00Z")]
    pub created_at: chrono::DateTime<chrono::Utc>,

    #[schema(value_type = String, example = "2024-01-15T10:30:00Z")]
    pub updated_at: chrono::DateTime<chrono::Utc>,

    #[schema(value_type = Option<String>, example = "2024-01-15T10:30:00Z")]
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,

    #[schema(example = json!(["rust", "程式設計", "學習心得"]))]
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PostListResponse {
    #[schema(example = 1)]
    pub id: i32,

    #[schema(example = "我的第一篇 Rust 文章")]
    pub title: String,

    #[schema(example = "這篇文章分享我學習 Rust 的心得")]
    pub excerpt: Option<String>,

    #[schema(example = "my-first-rust-article")]
    pub slug: String,

    #[schema(example = 42)]
    pub view_count: i32,

    #[schema(value_type = String, example = "2024-01-15T10:30:00Z")]
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,

    #[schema(example = json!(["rust", "程式設計", "學習心得"]))]
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct PostListQuery {
    #[schema(example = 1)]
    pub page: Option<u64>,

    #[schema(example = 10)]
    pub page_size: Option<u64>,

    #[schema(example = "rust")]
    pub tag: Option<String>,
}