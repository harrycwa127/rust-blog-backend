use sea_orm::DatabaseConnection;
use crate::{
    config::Config, 
    cache::{PostCache, TagCache, CacheConfig}
};

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub config: Config,
    // 新增快取服務
    pub post_cache: PostCache,
    pub tag_cache: TagCache,
}

impl AppState {
    pub fn new(db: DatabaseConnection, config: Config) -> Self {
        let cache_config = CacheConfig::default();
        
        Self {
            db,
            config,
            post_cache: PostCache::new(&cache_config),
            tag_cache: TagCache::new(&cache_config),
        }
    }
}