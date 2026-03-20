use axum::{extract::Request, middleware::Next, response::Response};
use tracing::info;

pub async fn request_logging(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = std::time::Instant::now();

    let response = next.run(request).await;

    let elapsed = start.elapsed();
    let status = response.status();

    info!(
        method = %method,
        uri = %uri,
        status = %status,
        elapsed = ?elapsed,
        "請求處理完成"
    );

    response
}