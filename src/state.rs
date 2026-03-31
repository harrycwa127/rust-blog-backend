use sea_orm::DatabaseConnection;
use crate::cache::CacheConfig;
use crate::cache::post::PostCache;
use crate::cache::tag::TagCache;
use crate::config::Config;
use crate::auth::JwtService;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub config: Config,
    pub post_cache: PostCache,
    pub tag_cache: TagCache,
    pub jwt_service: JwtService,
}

impl AppState {
    pub fn new(db: DatabaseConnection, config: Config, jwt_service: JwtService) -> Self {

        let cache_config = CacheConfig::default();

        Self {
            db,
            config ,
            post_cache: PostCache::new(&cache_config),
            tag_cache: TagCache::new(&cache_config),
            jwt_service,
        }
    }
}