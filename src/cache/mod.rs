use moka::future::Cache;
use std::time::Duration;
use serde::{Serialize, Deserialize};

pub mod post_cache;
pub mod tag_cache;

pub use post_cache::PostCache;
pub use tag_cache::TagCache;

/// 快取配置
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub post_list_ttl: Duration,
    pub post_detail_ttl: Duration,
    pub tag_list_ttl: Duration,
    pub max_capacity: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            // 文章列表快取 5 分鐘（可能有新文章）
            post_list_ttl: Duration::from_secs(5 * 60),
            
            // 文章詳情快取 30 分鐘（內容穩定）
            post_detail_ttl: Duration::from_secs(30 * 60),
            
            // 標籤列表快取 15 分鐘（標籤相對穩定）
            tag_list_ttl: Duration::from_secs(15 * 60),
            
            // 最大快取項目數（控制記憶體使用）
            max_capacity: 1000,
        }
    }
}

/// 快取統計資訊
#[derive(Debug, Serialize)]
pub struct CacheStats {
    pub entry_count: u64,
    pub hit_count: u64,
    pub miss_count: u64,
    pub hit_rate: f64,
}