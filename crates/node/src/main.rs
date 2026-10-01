//! Workflow engine node: hosts the capabilities and serves the API.

// Use mimalloc in production for lower latency and fragmentation (M-MIMALLOC-APPS).
// Behind a feature so the default scaffold build needs no C toolchain.
#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use anyhow::Result;
use api::GatewayService;
use tonic::transport::Server;

/// Address the gRPC gateway listens on (fixed for Phase 0).
const LISTEN_ADDR: &str = "0.0.0.0:26500";

#[tokio::main]
async fn main() -> Result<()> {
    init_telemetry();
    let addr = LISTEN_ADDR.parse()?;
    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        %addr,
        "node starting; serving gateway API"
    );
    // TODO(phase-0+): load config, then start persistence, coordination and the
    // workflow capabilities behind the gateway.
    Server::builder()
        .add_service(GatewayService::server())
        .serve(addr)
        .await?;
    Ok(())
}

/// Installs the process-wide structured logging subscriber.
fn init_telemetry() {
    tracing_subscriber::fmt().with_target(false).init();
}
