use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateTagRequest {
    #[validate(length(min = 1, max = 50, message = "標籤名稱長度必須在 1-50 字元之間"))]
    #[validate(regex(
        path = "crate::utils::validation::TAG_NAME_REGEX",
        message = "標籤名稱只能包含中英文、數字、連字符和底線"
    ))]
    #[schema(example = "rust")]
    pub name: String,

    #[validate(length(max = 255, message = "描述不能超過 255 字元"))]
    #[schema(example = "Rust 程式語言相關文章")]
    pub description: Option<String>,

    #[validate(regex(
        path = "crate::utils::validation::COLOR_REGEX",
        message = "顏色必須是有效的十六進位色碼格式 (#RRGGBB)"
    ))]
    #[schema(example = "#8B5CF6")]
    pub color: Option<String>,
}


#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateTagRequest {
    #[validate(length(min = 1, max = 50, message = "標籤名稱長度必須在 1-50 字元之間"))]
    #[validate(regex(
        path = "crate::utils::validation::TAG_NAME_REGEX",
        message = "標籤名稱只能包含中英文、數字、連字符和底線"
    ))]
    #[schema(example = "rust-updated")]
    pub name: Option<String>,

    #[validate(length(max = 255, message = "描述不能超過 255 字元"))]
    #[schema(example = "更新後的 Rust 程式語言描述")]
    pub description: Option<String>,

    #[validate(regex(
        path = "crate::utils::validation::COLOR_REGEX",
        message = "顏色必須是有效的十六進位色碼格式 (#RRGGBB)"
    ))]
    #[schema(example = "#10B981")]
    pub color: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct TagResponse {
    #[schema(example = 1)]
    pub id: i32,

    #[schema(example = "rust")]
    pub name: String,

    #[schema(example = "Rust 程式語言相關文章")]
    pub description: Option<String>,

    #[schema(example = "#8B5CF6")]
    pub color: String,

    #[schema(example = 15)]
    pub post_count: i32,

    #[schema(value_type = String, example = "2024-01-15T10:30:00Z")]
    pub created_at: chrono::DateTime<chrono::Utc>,

    #[schema(value_type = String, example = "2024-01-15T15:45:00Z")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TagWithPostsResponse {
    #[schema(example = 1)]
    pub id: i32,

    #[schema(example = "rust")]
    pub name: String,

    #[schema(example = "Rust 程式語言相關文章")]
    pub description: Option<String>,

    #[schema(example = "#8B5CF6")]
    pub color: String,

    #[schema(example = 15)]
    pub post_count: i32,

    #[schema(value_type = String, example = "2024-01-15T10:30:00Z")]
    pub created_at: chrono::DateTime<chrono::Utc>,

    pub posts: Vec<crate::dtos::PostListResponse>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DeleteTagResponse {
    #[schema(example = true)]
    pub success: bool,

    #[schema(example = "標籤已成功刪除")]
    pub message: String,

    #[schema(example = 1)]
    pub deleted_id: i32,

    #[schema(example = 15)]
    pub affected_posts: i32,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct TagListQuery {
    #[schema(example = 1)]
    pub page: Option<u64>,

    #[schema(example = 20)]
    pub page_size: Option<u64>,

    #[schema(example = "rust")]
    pub search: Option<String>,

    #[schema(example = "post_count")]
    pub sort_by: Option<String>, // name, post_count, created_at

    #[schema(example = "desc")]
    pub sort_order: Option<String>, // asc, desc

    #[schema(example = true)]
    pub include_empty: Option<bool>, // 是否包含沒有文章的標籤
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TagSuggestionResponse {
    #[schema(example = json!(["rust", "程式設計", "學習心得"]))]
    pub suggestions: Vec<String>,

    #[schema(example = 10)]
    pub total_tags: i32,

    #[schema(example = json!(["rust", "javascript", "python"]))]
    pub popular_tags: Vec<String>,
}