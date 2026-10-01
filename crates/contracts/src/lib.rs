//! Generated gRPC and protobuf types for the engine API.
//!
//! Compiled from `proto/gateway.proto` (package `workflow.v1`) by `build.rs`
//! using tonic-build, and included here. Downstream crates use `contracts::v1`:
//! the message types, the `gateway_server::Gateway` service trait plus
//! `GatewayServer`, and the `gateway_client::GatewayClient`.

/// Types generated from the `workflow.v1` gateway contract.
#[allow(clippy::all, clippy::pedantic, missing_docs, reason = "generated code")]
pub mod v1 {
    tonic::include_proto!("workflow.v1");
}
