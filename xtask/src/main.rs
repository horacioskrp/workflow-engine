//! Workspace automation tasks (code generation, release helpers).
//!
//! Run via the cargo alias: `cargo xtask <task>`.

use anyhow::{Result, bail};

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

/// Regenerates gRPC/protobuf bindings from `proto/`.
#[expect(
    clippy::unnecessary_wraps,
    reason = "returns errors once codegen is implemented"
)]
fn codegen() -> Result<()> {
    // TODO(phase-0): invoke tonic-build / prost-build here.
    println!("codegen: not yet implemented");
    Ok(())
}
