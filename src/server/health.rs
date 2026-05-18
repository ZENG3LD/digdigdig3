//! Health gRPC service — uptime, connected exchanges, active subscriptions.

use tonic::{Request, Response, Status};

use crate::server::proto::{
    health_server::Health, HealthRequest, HealthResponse,
};
use crate::server::state::ServerState;

pub struct HealthService {
    pub state: ServerState,
}

#[tonic::async_trait]
impl Health for HealthService {
    async fn status(
        &self,
        _request: Request<HealthRequest>,
    ) -> Result<Response<HealthResponse>, Status> {
        let connected = self.state.hub.len_rest() as i32;
        let active_subs = self.state.bus.active_subscriptions() as i32;
        let uptime = self.state.uptime_secs();

        // Collect capability JSON per connected exchange
        let caps: Vec<String> = self
            .state
            .hub
            .ids()
            .into_iter()
            .filter_map(|id| {
                self.state.hub.capabilities(id).and_then(|c| {
                    serde_json::to_string(&format!("{:?}: {:?}", id, c)).ok()
                })
            })
            .collect();

        Ok(Response::new(HealthResponse {
            uptime_secs: uptime,
            connected_exchanges: connected,
            active_subscriptions: active_subs,
            capabilities_json: caps,
        }))
    }
}
