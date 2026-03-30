use super::CacheConfig;
use crate::dtos::{PostDetailResponse, PostListResponse};
use moka::future::Cache;

/// 文章快取服務
#[derive(Clone)]
pub struct PostCache {
    // 文章列表快取
    list_cache: Cache<String, Vec<PostListResponse>>,

    // 單篇文章快取
    detail_cache: Cache<String, PostDetailResponse>,
}

impl std::fmt::Debug for PostCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PostCache")
            .field("list_cache", &format!("Cache with {} entries", self.list_cache.entry_count()))
            .field("detail_cache", &format!("Cache with {} entries", self.detail_cache.entry_count()))
            .finish()
    }
}

impl PostCache {
    pub fn new(config: &CacheConfig) -> Self {
        Self {
            list_cache: Cache::builder()
                .max_capacity(config.max_capacity / 2) // 分配一半容量給列表
                .time_to_live(config.post_list_ttl)
                .build(),

            detail_cache: Cache::builder()
                .max_capacity(config.max_capacity / 2) // 分配一半容量給詳情
                .time_to_live(config.post_detail_ttl)
                .build(),
        }
    }

    /// 快取文章列表
    pub async fn cache_post_list(&self, key: String, posts: Vec<PostListResponse>) {
        self.list_cache.insert(key, posts).await;
    }

    /// 取得快取的文章列表
    pub async fn get_post_list(&self, key: &str) -> Option<Vec<PostListResponse>> {
        self.list_cache.get(key).await
    }

    /// 快取文章詳情
    pub async fn cache_post_detail(&self, slug: String, post: PostDetailResponse) {
        // 只快取已發布的文章
        if post.is_published {
            self.detail_cache.insert(slug, post).await;
        }
    }

    /// 取得快取的文章詳情
    pub async fn get_post_detail(&self, slug: &str) -> Option<PostDetailResponse> {
        self.detail_cache.get(slug).await
    }

    /// 清理文章相關快取（當文章被更新時）
    pub async fn invalidate_post(&self, slug: &str) {
        // 移除特定文章的快取
        self.detail_cache.invalidate(slug).await;

        // 清空所有列表快取（因為文章可能影響列表）
        self.list_cache.invalidate_all();
    }

    /// 清理所有文章快取
    pub async fn invalidate_all(&self) {
        self.list_cache.invalidate_all();
        self.detail_cache.invalidate_all();
    }

    /// 產生文章列表的快取鍵
    pub fn generate_list_cache_key(
        page: Option<u64>,
        page_size: Option<u64>,
        tag: Option<&str>,
    ) -> String {
        format!(
            "posts:page:{}:size:{}:tag:{}",
            page.unwrap_or(1),
            page_size.unwrap_or(10),
            tag.unwrap_or("all")
        )
    }

    /// 取得快取統計
    pub fn stats(&self) -> serde_json::Value {
        serde_json::json!({
            "list_cache": {
                "entry_count": self.list_cache.entry_count(),
            },
            "detail_cache": {
                "entry_count": self.detail_cache.entry_count(),
            }
        })
    }

    pub async fn run_pending_tasks(&self) {
        self.list_cache.run_pending_tasks().await;
        self.detail_cache.run_pending_tasks().await;
    }
}