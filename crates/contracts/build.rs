//! Build script: compiles the gateway gRPC contract with tonic-build.
//!
//! Requires `protoc` on the build host (provided by the Docker build image).

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=../../proto/gateway.proto");
    tonic_build::compile_protos("../../proto/gateway.proto")?;
    Ok(())
}
