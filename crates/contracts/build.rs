//! Build-script placeholder for gRPC/protobuf code generation.

fn main() {
    // No-op so the workspace compiles without `protoc`.
    // TODO(phase-0): compile `../../proto/gateway.proto` with tonic-build.
    println!("cargo:rerun-if-changed=../../proto/gateway.proto");
}
