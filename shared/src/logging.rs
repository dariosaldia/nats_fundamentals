use tracing_subscriber::EnvFilter;

pub fn init() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive("info".parse().unwrap_or_default()),
        )
        .try_init();
}
