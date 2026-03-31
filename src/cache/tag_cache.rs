use moka::future::Cache;
use super::{CacheConfig, CacheStats};
use crate::dtos::TagResponse;

/// 標籤快取服務
#[derive(Debug, Clone)]
pub struct TagCache {
    cache: Cache<String, Vec<TagResponse>>,
}

impl TagCache {
    pub fn new(config: &CacheConfig) -> Self {
        Self {
            cache: Cache::builder()
                .max_capacity(100) // 標籤快取不需要太多空間
                .time_to_live(config.tag_list_ttl)
                .build(),
        }
    }

    /// 快取標籤列表
    pub async fn cache_tags(&self, tags: Vec<TagResponse>) {
        self.cache.insert("all_tags".to_string(), tags).await;
    }

    /// 取得快取的標籤列表
    pub async fn get_tags(&self) -> Option<Vec<TagResponse>> {
        self.cache.get("all_tags").await
    }

    /// 清理標籤快取（當標籤被更新時）
    pub async fn invalidate_all(&self) {
        self.cache.invalidate_all();
    }

    /// 取得快取統計（moka.futures 沒有公開命中計數 API，暫時只提供項目數）
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            entry_count: self.cache.entry_count(),
            hit_count: 0,
            miss_count: 0,
            hit_rate: 0.0,
        }
    }

    /// 執行快取維護
    pub async fn run_pending_tasks(&self) {
        self.cache.run_pending_tasks().await;
    }
}