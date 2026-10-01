//! The gRPC `Gateway` service implementation.

use contracts::v1 as pb;
use contracts::v1::gateway_server::{Gateway, GatewayServer};
use tonic::{Request, Response, Status};

/// Serves the client-facing gRPC API over an in-memory cluster description.
///
/// Phase 0 reports a fixed single-node, single-partition topology; the other
/// RPCs return `unimplemented` until their capabilities land.
#[derive(Debug, Clone)]
pub struct GatewayService {
    brokers: Vec<pb::BrokerInfo>,
    partitions_count: i32,
    replication_factor: i32,
}

impl Default for GatewayService {
    fn default() -> Self {
        Self {
            brokers: vec![pb::BrokerInfo {
                node_id: 0,
                host: "127.0.0.1".to_owned(),
                port: 26500,
            }],
            partitions_count: 1,
            replication_factor: 1,
        }
    }
}

impl GatewayService {
    /// Wraps a default in-memory service as a gRPC server ready to be served.
    #[must_use]
    pub fn server() -> GatewayServer<Self> {
        GatewayServer::new(Self::default())
    }

    fn topology_response(&self) -> pb::TopologyResponse {
        let cluster_size = i32::try_from(self.brokers.len()).unwrap_or(i32::MAX);
        pb::TopologyResponse {
            brokers: self.brokers.clone(),
            cluster_size,
            partitions_count: self.partitions_count,
            replication_factor: self.replication_factor,
        }
    }
}

// The methods are `async` because the generated `Gateway` trait requires it;
// the Phase 0 bodies do no awaiting yet.
#[allow(
    clippy::unused_async,
    reason = "signatures fixed by the generated trait"
)]
#[tonic::async_trait]
impl Gateway for GatewayService {
    async fn topology(
        &self,
        _request: Request<pb::TopologyRequest>,
    ) -> Result<Response<pb::TopologyResponse>, Status> {
        Ok(Response::new(self.topology_response()))
    }

    async fn deploy_process(
        &self,
        _request: Request<pb::DeployProcessRequest>,
    ) -> Result<Response<pb::DeployProcessResponse>, Status> {
        Err(Status::unimplemented(
            "deploy_process is not implemented yet",
        ))
    }

    async fn create_process_instance(
        &self,
        _request: Request<pb::CreateProcessInstanceRequest>,
    ) -> Result<Response<pb::CreateProcessInstanceResponse>, Status> {
        Err(Status::unimplemented(
            "create_process_instance is not implemented yet",
        ))
    }

    type ActivateJobsStream = tokio_stream::Empty<Result<pb::ActivateJobsResponse, Status>>;

    async fn activate_jobs(
        &self,
        _request: Request<pb::ActivateJobsRequest>,
    ) -> Result<Response<Self::ActivateJobsStream>, Status> {
        Err(Status::unimplemented(
            "activate_jobs is not implemented yet",
        ))
    }

    async fn complete_job(
        &self,
        _request: Request<pb::CompleteJobRequest>,
    ) -> Result<Response<pb::CompleteJobResponse>, Status> {
        Err(Status::unimplemented("complete_job is not implemented yet"))
    }

    async fn publish_message(
        &self,
        _request: Request<pb::PublishMessageRequest>,
    ) -> Result<Response<pb::PublishMessageResponse>, Status> {
        Err(Status::unimplemented(
            "publish_message is not implemented yet",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::GatewayService;
    use contracts::v1::TopologyRequest;
    use contracts::v1::gateway_server::Gateway;
    use tonic::Request;

    #[tokio::test]
    async fn topology_reports_the_in_memory_cluster() {
        let service = GatewayService::default();
        let response = service
            .topology(Request::new(TopologyRequest {}))
            .await
            .expect("topology should succeed")
            .into_inner();

        assert_eq!(response.cluster_size, 1);
        assert_eq!(response.partitions_count, 1);
        assert_eq!(response.replication_factor, 1);
        assert_eq!(response.brokers.len(), 1);
    }
}
