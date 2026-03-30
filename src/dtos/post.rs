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

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdatePostRequest {
    #[validate(length(min = 1, max = 255, message = "標題長度必須在 1-255 字元之間"))]
    #[schema(example = "更新後的文章標題")]
    pub title: Option<String>,

    #[validate(length(min = 1, message = "內容不能為空"))]
    #[schema(example = "更新後的文章內容...")]
    pub content: Option<String>,

    #[validate(length(max = 500, message = "摘要不能超過 500 字元"))]
    #[schema(example = "更新後的摘要")]
    pub excerpt: Option<String>,

    #[schema(example = "updated-article-slug")]
    pub slug: Option<String>,

    #[schema(example = true)]
    pub is_published: Option<bool>,

    #[schema(example = json!(["rust", "更新", "程式設計"]))]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PostDetailResponse {
    #[schema(example = 1)]
    pub id: i32,

    #[schema(example = "我的第一篇 Rust 文章")]
    pub title: String,

    #[schema(example = "# 開始學習 Rust\n\n今天開始我的 Rust 學習之旅...")]
    pub content: String,

    #[schema(example = "這篇文章分享我學習 Rust 的心得")]
    pub excerpt: Option<String>,

    #[schema(example = "my-first-rust-article")]
    pub slug: String,

    #[schema(example = true)]
    pub is_published: bool,

    #[schema(example = 128)]
    pub view_count: i32,

    #[schema(value_type = String, example = "2024-01-15T10:30:00Z")]
    pub created_at: chrono::DateTime<chrono::Utc>,

    #[schema(value_type = String, example = "2024-01-15T15:45:00Z")]
    pub updated_at: chrono::DateTime<chrono::Utc>,

    #[schema(value_type = Option<String>, example = "2024-01-15T12:00:00Z")]
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,

    #[schema(example = json!(["rust", "程式設計", "學習心得"]))]
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DeletePostResponse {
    #[schema(example = true)]
    pub success: bool,

    #[schema(example = "文章已成功刪除")]
    pub message: String,

    #[schema(example = 1)]
    pub deleted_id: i32,
}

// 新增進階搜尋查詢結構
#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct PostSearchQuery {
    #[validate(length(min = 1, max = 100, message = "搜尋關鍵字長度必須在 1-100 字元之間"))]
    #[schema(example = "rust 教學")]
    pub q: Option<String>, // 關鍵字搜尋

    #[schema(example = "rust,程式語言")]
    pub tags: Option<String>, // 標籤篩選，逗號分隔

    #[schema(example = "published")]
    pub status: Option<String>, // published, draft, all (admin only)

    #[schema(example = "2024-01-01")]
    pub from_date: Option<String>, // 起始日期 (YYYY-MM-DD)

    #[schema(example = "2024-12-31")]
    pub to_date: Option<String>, // 結束日期 (YYYY-MM-DD)

    #[schema(example = 1)]
    pub page: Option<u64>,

    #[schema(example = 10)]
    pub page_size: Option<u64>,

    #[schema(example = "created_at")]
    pub sort_by: Option<String>, // created_at, updated_at, title, view_count

    #[schema(example = "desc")]
    pub sort_order: Option<String>, // asc, desc

    #[schema(example = false)]
    pub include_content: Option<bool>, // 是否包含完整內容（預設只有摘要）
}

// 搜尋結果回應
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PostSearchResponse {
    #[schema(example = 25)]
    pub total_count: i64,

    #[schema(example = 3)]
    pub total_pages: u64,

    #[schema(example = 1)]
    pub current_page: u64,

    #[schema(example = 10)]
    pub page_size: u64,

    pub posts: Vec<PostSummaryResponse>,

    #[schema(example = "搜尋到 25 篇與 'rust' 相關的文章")]
    pub search_summary: String,

    pub filters_applied: SearchFiltersApplied,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SearchFiltersApplied {
    #[schema(example = "rust 教學")]
    pub keyword: Option<String>,

    pub tags: Vec<String>,

    #[schema(example = "published")]
    pub status: Option<String>,

    #[schema(example = "2024-01-01")]
    pub date_range_start: Option<String>,

    #[schema(example = "2024-12-31")]
    pub date_range_end: Option<String>,
}

// 優化的文章摘要回應（用於列表）
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PostSummaryResponse {
    #[schema(example = 1)]
    pub id: i32,

    #[schema(example = "學習 Rust 的第一天")]
    pub title: String,

    #[schema(example = "今天開始踏上 Rust 學習之旅...")]
    pub excerpt: Option<String>,

    #[schema(example = "學習-rust-的第一天")]
    pub slug: String,

    #[schema(example = true)]
    pub is_published: bool,

    #[schema(example = 42)]
    pub view_count: i32,

    #[schema(value_type = String, example = "2024-01-15T10:30:00Z")]
    pub created_at: chrono::DateTime<chrono::Utc>,

    #[schema(value_type = String, example = "2024-01-16T14:20:00Z")]
    pub updated_at: chrono::DateTime<chrono::Utc>,

    #[schema(value_type = String, example = "2024-01-15T10:30:00Z")]
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,

    pub tags: Vec<TagSummaryResponse>,

    #[schema(example = 5)]
    pub comment_count: i32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TagSummaryResponse {
    #[schema(example = 1)]
    pub id: i32,

    #[schema(example = "rust")]
    pub name: String,

    #[schema(example = "#E74C3C")]
    pub color: String,
}

// 熱門搜尋統計回應
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PopularSearchResponse {
    pub popular_tags: Vec<String>,
    pub recent_posts: Vec<PostSummaryResponse>,
    pub search_suggestions: Vec<String>,
    pub total_posts: i64,
    pub total_published: i64,
}