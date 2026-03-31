use std::{
    collections::HashMap,
    net::IpAddr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use axum::{
    extract::{ConnectInfo, Request},
    http::StatusCode,
    middleware::Next,
    response::Response,
};

#[derive(Debug)]
struct RateLimitEntry {
    count: u32,
    window_start: Instant,
}

type RateLimitStore = Arc<Mutex<HashMap<IpAddr, RateLimitEntry>>>;

#[derive(Clone)]
pub struct RateLimiter {
    store: RateLimitStore,
    max_requests: u32,
}

impl RateLimiter {
    pub fn new(max_requests: u32) -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
        }
    }

    fn check_rate_limit(&self, ip: IpAddr) -> bool {
        let mut store = self.store.lock().unwrap();
        let now = Instant::now();
        let window_duration = Duration::from_secs(60); // 1 分鐘窗口

        match store.get_mut(&ip) {
            Some(entry) => {
                // 如果窗口已過期，重置計數
                if now.duration_since(entry.window_start) >= window_duration {
                    entry.count = 1;
                    entry.window_start = now;
                    true
                } else if entry.count >= self.max_requests {
                    false // 超過限制
                } else {
                    entry.count += 1;
                    true
                }
            }
            None => {
                // 新 IP
                store.insert(
                    ip,
                    RateLimitEntry {
                        count: 1,
                        window_start: now,
                    },
                );
                true
            }
        }
    }
}

pub async fn rate_limit_middleware(
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    static RATE_LIMITER: std::sync::OnceLock<RateLimiter> = std::sync::OnceLock::new();
    let limiter = RATE_LIMITER.get_or_init(|| RateLimiter::new(60)); // 每分鐘 60 次

    if !limiter.check_rate_limit(addr.ip()) {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(request).await)
}