use tokio::signal;
use tracing::debug;

/// Wait for a shutdown signal (Ctrl+C or SIGTERM).
///
/// ## Panics
///
/// Panics if the signal handler cannot be installed.
pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {
            debug!("Ctrl+C received ~ shutting down... :(");
        },
        () = terminate => {
            debug!("SIGTERM received ~ shutting down... :(");
        },
    }
}
