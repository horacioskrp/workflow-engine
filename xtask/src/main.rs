//! Workspace automation tasks (code generation, release helpers).
//!
//! Run via the cargo alias: `cargo xtask <task>`.

use std::process::Command;

use anyhow::{Context, Result, bail};

fn main() -> Result<()> {
    match std::env::args().nth(1).as_deref() {
        Some("codegen") => codegen(),
        Some(other) => bail!("unknown task: {other}"),
        None => {
            eprintln!("usage: cargo xtask <codegen>");
            Ok(())
        }
    }
}

/// Regenerates the gRPC/protobuf bindings by rebuilding `contracts`, whose
/// `build.rs` compiles `proto/gateway.proto` with tonic-build.
fn codegen() -> Result<()> {
    let status = Command::new(env!("CARGO"))
        .args(["build", "--package", "contracts"])
        .status()
        .context("failed to spawn cargo")?;
    if !status.success() {
        bail!("contracts codegen build failed ({status})");
    }
    println!("codegen: contracts rebuilt — proto bindings regenerated");
    Ok(())
}
