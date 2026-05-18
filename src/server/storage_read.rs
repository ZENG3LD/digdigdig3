//! StorageRead gRPC service — server-streaming read from EventLog.

use tokio_stream::iter as stream_iter;
use tonic::{Request, Response, Status};

use crate::core::storage::StreamKey;
use crate::server::proto::{
    storage_read_server::StorageRead, EventMessage, ReadRangeRequest,
};
use crate::server::state::ServerState;

pub struct StorageReadService {
    pub state: ServerState,
}

#[tonic::async_trait]
impl StorageRead for StorageReadService {
    type ReadRangeStream =
        std::pin::Pin<Box<dyn futures_util::Stream<Item = Result<EventMessage, Status>> + Send>>;

    async fn read_range(
        &self,
        request: Request<ReadRangeRequest>,
    ) -> Result<Response<Self::ReadRangeStream>, Status> {
        let req = request.into_inner();
        let key = StreamKey {
            exchange: req.exchange,
            account: req.account,
            symbol: req.symbol,
            stream_kind: req.stream_kind,
        };

        let records = self
            .state
            .storage
            .read_range(&key, req.from_ms, req.to_ms)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let messages: Vec<Result<EventMessage, Status>> = records
            .into_iter()
            .map(|(ts_ms, payload)| {
                Ok(EventMessage {
                    timestamp_ms: ts_ms,
                    event_type: String::new(),
                    payload_json: payload,
                })
            })
            .collect();

        Ok(Response::new(Box::pin(stream_iter(messages))))
    }
}
