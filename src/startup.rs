use axum::Router;
use tokio::net::TcpListener;
use tracing::info;

use crate::{app::build_app, state::AppState};

pub async fn run(app_state: AppState) -> anyhow::Result<()> {
    let addr = format!("{}:{}", app_state.config.host, app_state.config.port);
    let server_url = app_state.config.server_url();

    info!("設定載入完成: {:?}", app_state.config.sanitized_for_log());

    let listener = TcpListener::bind(&addr).await?;
    let app: Router = build_app(app_state);

    info!("🚀 個人部落格服務啟動於 {}", server_url);
    info!("📚 API 文件：{}/docs", server_url);
    info!("🔍 健康檢查：{}/health", server_url);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("🛑 個人部落格服務正在關閉...");
}