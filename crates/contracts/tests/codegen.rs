//! Smoke test: the generated gateway types are reachable and usable.
//!
//! Proves that `build.rs` (tonic-build) compiled `proto/gateway.proto` and that
//! the generated items are exposed through `contracts::v1`.

#[test]
fn generated_messages_are_available() {
    let resp = contracts::v1::TopologyResponse::default();
    assert_eq!(resp.cluster_size, 0);
    assert_eq!(resp.partitions_count, 0);
}
