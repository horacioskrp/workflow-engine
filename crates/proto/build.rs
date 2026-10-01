//! Build-script placeholder for gRPC/protobuf code generation.

fn main() {
    // Kept as a no-op so the workspace compiles without `protoc` installed.
    // TODO(phase-0): compile `../../proto/gateway.proto` with tonic-build.
    println!("cargo:rerun-if-changed=../../proto/gateway.proto");
}
