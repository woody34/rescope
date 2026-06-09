use rescope::{config::EmulatorConfig, seed, server, state::EmulatorState};
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("rescope=info".parse()?),
        )
        .init();

    let config = EmulatorConfig::from_env();

    // ── Startup banner ───────────────────────────────────────────────
    info!("─────────────────────────────────────────────");
    info!("  Rescope — Descope Emulator");
    info!("─────────────────────────────────────────────");
    info!(port = config.port, project_id = %config.project_id, "Config");
    let masked_key = if config.management_key.len() > 6 {
        format!("{}…", &config.management_key[..6])
    } else {
        config.management_key.clone()
    };
    info!(management_key = %masked_key, "Config");
    if let Some(ref seed_path) = config.seed_file {
        info!(seed_file = %seed_path, "Config");
    }
    info!("⚠️  Do not use in production");

    let state = EmulatorState::new(&config).await?;

    if let Some(seed_path) = &config.seed_file {
        if !std::path::Path::new(seed_path).exists() {
            tracing::warn!(path = %seed_path, "Seed file not found — starting with empty state");
        } else {
            info!(path = %seed_path, "Loading seed file");
            seed::load(seed_path, &state).await?;
            let user_count = state.users.read().await.count();
            let tenant_count = state.tenants.read().await.count();
            info!(users = user_count, tenants = tenant_count, "Seed loaded");
        }
    }

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("✅ Ready → http://localhost:{}", config.port);

    // Run the server until a shutdown signal is received.
    // We then call std::process::exit(0) to trigger atexit handlers,
    // which lets the LLVM coverage runtime flush .profraw files when the
    // server is used as an instrumented binary for integration coverage.
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        let mut sigterm =
            signal(SignalKind::terminate()).expect("failed to set up SIGTERM handler");
        tokio::select! {
            result = axum::serve(listener, server::build_router(state)) => {
                result?;
            }
            _ = tokio::signal::ctrl_c() => {
                info!("Received SIGINT, shutting down");
            }
            _ = sigterm.recv() => {
                info!("Received SIGTERM, shutting down");
            }
        }
    }
    #[cfg(not(unix))]
    {
        tokio::select! {
            result = axum::serve(listener, server::build_router(state)) => {
                result?;
            }
            _ = tokio::signal::ctrl_c() => {
                info!("Received shutdown signal");
            }
        }
    }

    // Explicit exit so atexit handlers (including LLVM profiling flush) run.
    std::process::exit(0);
}
