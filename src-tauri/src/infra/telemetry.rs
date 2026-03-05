use tracing_subscriber::{fmt, EnvFilter};

/// Initializes structured logging and telemetry for the application.
///
/// Sets up the tracing subscriber with an environment filter, defaulting to INFO level.
pub fn init_telemetry() {
    fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(tracing::Level::INFO.into())
                .from_env_lossy(),
        )
        .init();

    tracing::info!("Telemetry initialized.");
}
