use axum::Router;
use tokio::net::TcpListener;
use tracing::info;

use crate::{app::build_app, config::Config};

pub async fn run(config: Config) -> anyhow::Result<()> {
    let addr = format!("{}:{}", config.host, config.port);
    let server_url = config.server_url();

    info!("設定載入完成: {:?}", config.sanitized_for_log());

    let listener = TcpListener::bind(&addr).await?;
    let app: Router = build_app(config);

    info!("🚀 個人部落格服務啟動於 {}", server_url);
    info!("📚 API 文件：{}/docs", server_url);
    info!("🔍 健康檢查：{}/health", server_url);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        let mut sigint = signal(SignalKind::interrupt()).expect("sigint handler");
        let mut sigterm = signal(SignalKind::terminate()).expect("sigterm handler");
        tokio::select! { _ = sigint.recv() => {}, _ = sigterm.recv() => {}, }
    }
    #[cfg(not(unix))]
    { let _ = tokio::signal::ctrl_c().await; }

    println!("🛑 收到關閉信號，正在優雅關閉服務…");
}