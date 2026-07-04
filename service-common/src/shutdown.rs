// service-common/src/shutdown.rs
/// Waits for a shutdown signal (Ctrl+C or SIGTERM on Unix) and returns
/// when one has been received. Used as a future of
/// "graceful shutdown" for `axum::serve`.
pub async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install ctrl+c handler")
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => tracing::warn!("received Ctrl-C, shutting down"),
        _ = terminate => tracing::warn!("received SIGTERM, shutting down"),
    }
}
