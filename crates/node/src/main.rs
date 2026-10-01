//! Workflow engine node: hosts the capabilities and serves the API.

// Use mimalloc in production for lower latency and fragmentation (M-MIMALLOC-APPS).
// Behind a feature so the default scaffold build needs no C toolchain.
#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    init_telemetry();
    tracing::info!(version = env!("CARGO_PKG_VERSION"), "node starting");
    // TODO(phase-0): load config, then start persistence, coordination,
    // the workflow capabilities and the API.
    Ok(())
}

/// Installs the process-wide structured logging subscriber.
fn init_telemetry() {
    tracing_subscriber::fmt().with_target(false).init();
}
