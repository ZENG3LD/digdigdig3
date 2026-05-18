//! LiveEvents gRPC service — server-streaming subscription to live WS events.

use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt as _;
use tonic::{Request, Response, Status};

use crate::server::bus::BusEvent;
use crate::server::proto::{
    live_events_server::LiveEvents, EventMessage, SubscribeRequest,
};
use crate::server::state::ServerState;

pub struct LiveEventsService {
    pub state: ServerState,
}

#[tonic::async_trait]
impl LiveEvents for LiveEventsService {
    type SubscribeStream =
        std::pin::Pin<Box<dyn futures_util::Stream<Item = Result<EventMessage, Status>> + Send>>;

    async fn subscribe(
        &self,
        request: Request<SubscribeRequest>,
    ) -> Result<Response<Self::SubscribeStream>, Status> {
        let req = request.into_inner();
        let exchange = req.exchange.to_lowercase();
        let account = req.account.to_lowercase();
        let symbol = req.symbol.to_lowercase();
        let stream_kind = req.stream_kind.to_lowercase();

        let rx = self.state.bus.subscribe();
        let bus = self.state.bus.clone();

        let stream = BroadcastStream::new(rx)
            .take_while(|r| r.is_ok())
            .filter_map(move |r| match r {
                Ok(ev) => {
                    let matches = matches_filter(&ev, &exchange, &account, &symbol, &stream_kind);
                    if matches {
                        Some(Ok(EventMessage {
                            timestamp_ms: ev.timestamp_ms,
                            event_type: ev.event_type,
                            payload_json: ev.payload_json,
                        }))
                    } else {
                        None
                    }
                }
                Err(_) => None,
            });

        // Wrap to decrement counter on drop
        let stream = DropGuard { inner: stream, bus };

        Ok(Response::new(Box::pin(stream)))
    }
}

fn matches_filter(ev: &BusEvent, exchange: &str, account: &str, symbol: &str, kind: &str) -> bool {
    (exchange.is_empty() || ev.exchange.to_lowercase() == exchange)
        && (account.is_empty() || ev.account.to_lowercase() == account)
        && (symbol.is_empty() || ev.symbol.to_lowercase() == symbol)
        && (kind.is_empty() || ev.stream_kind.to_lowercase() == kind)
}

/// Thin wrapper that calls `bus.unsubscribe()` when the stream is dropped.
struct DropGuard<S> {
    inner: S,
    bus: crate::server::bus::EventBus,
}

impl<S> Drop for DropGuard<S> {
    fn drop(&mut self) {
        self.bus.unsubscribe();
    }
}

impl<S: futures_util::Stream + Unpin> futures_util::Stream for DropGuard<S> {
    type Item = S::Item;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        std::pin::Pin::new(&mut self.inner).poll_next(cx)
    }
}
