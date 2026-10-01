//! End-to-end test: a real gRPC client calls `Topology` against a running server.
//!
//! Binds the gateway to an ephemeral local port, connects over the network with
//! the generated client, and checks the reported topology — the Phase 0 goal.

use std::net::SocketAddr;
use std::time::Duration;

use api::GatewayService;
use contracts::v1::TopologyRequest;
use contracts::v1::gateway_client::GatewayClient;
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::{Channel, Endpoint, Server};

#[tokio::test]
async fn client_receives_topology_over_grpc() {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind ephemeral port");
    let addr = listener.local_addr().expect("local addr");

    tokio::spawn(async move {
        Server::builder()
            .add_service(GatewayService::server())
            .serve_with_incoming(TcpListenerStream::new(listener))
            .await
            .expect("server runs");
    });

    let mut client = connect(addr).await;
    let response = client
        .topology(TopologyRequest {})
        .await
        .expect("topology rpc succeeds")
        .into_inner();

    assert_eq!(response.cluster_size, 1);
    assert_eq!(response.partitions_count, 1);
    assert_eq!(response.replication_factor, 1);
    assert_eq!(response.brokers.len(), 1);
}

/// Connects to the gateway, retrying while the spawned server starts up.
async fn connect(addr: SocketAddr) -> GatewayClient<Channel> {
    let endpoint = Endpoint::from_shared(format!("http://{addr}")).expect("valid endpoint");
    for _ in 0..50 {
        if let Ok(channel) = endpoint.connect().await {
            return GatewayClient::new(channel);
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("gateway not reachable at {addr}");
}
