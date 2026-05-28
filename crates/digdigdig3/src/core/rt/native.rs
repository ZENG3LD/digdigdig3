//! Native runtime impl: tokio + tokio_tungstenite.
//!
//! `TokioRuntime` implements [`Spawn`], [`Timer`], and [`WsConnector`].
//! All three traits require `Send + Sync + 'static` on native (multi-threaded
//! tokio scheduler). `ToksteniteConn` wraps the raw
//! `tokio_tungstenite::WebSocketStream` and translates between
//! `tungstenite::Message` and our target-independent [`WsFrame`].

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::time::timeout as tokio_timeout;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use super::{Spawn, Timer, WsConn, WsConnector, WsFrame, WsRtError};

// ─── TokioRuntime ─────────────────────────────────────────────────────────────

/// Zero-size token representing the tokio runtime.
pub struct TokioRuntime;

impl Spawn for TokioRuntime {
    fn spawn(&self, fut: Pin<Box<dyn Future<Output = ()> + Send + 'static>>) {
        tokio::spawn(fut);
    }
}

impl Timer for TokioRuntime {
    fn sleep(&self, dur: Duration) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
        Box::pin(tokio::time::sleep(dur))
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl WsConnector for TokioRuntime {
    async fn connect(&self, url: &str, timeout: Duration) -> Result<Box<dyn WsConn>, WsRtError> {
        let connect_fut = connect_async(url);
        let (ws, _resp) = tokio_timeout(timeout, connect_fut)
            .await
            .map_err(|_| WsRtError::Timeout)?
            .map_err(|e| WsRtError::Connect(e.to_string()))?;
        Ok(Box::new(TungsteniteConn { inner: ws }))
    }
}

// ─── TungsteniteConn ──────────────────────────────────────────────────────────

type WsStream = tokio_tungstenite::WebSocketStream<
    tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
>;

/// Wraps a raw `tokio_tungstenite` stream and adapts it to [`WsConn`].
pub struct TungsteniteConn {
    inner: WsStream,
}

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl WsConn for TungsteniteConn {
    async fn send(&mut self, frame: WsFrame) -> Result<(), WsRtError> {
        let msg = ws_frame_to_message(frame);
        self.inner
            .send(msg)
            .await
            .map_err(|e| WsRtError::Send(e.to_string()))
    }

    async fn next_frame(&mut self) -> Option<Result<WsFrame, WsRtError>> {
        let raw = self.inner.next().await?;
        Some(message_to_ws_frame(raw))
    }

    async fn close(&mut self) -> Result<(), WsRtError> {
        self.inner
            .close(None)
            .await
            .map_err(|e| WsRtError::Send(e.to_string()))
    }
}

// ─── Frame translation helpers ────────────────────────────────────────────────

fn ws_frame_to_message(frame: WsFrame) -> Message {
    match frame {
        WsFrame::Text(s) => Message::Text(s.into()),
        WsFrame::Binary(b) => Message::Binary(b.into()),
        WsFrame::Ping(p) => Message::Ping(p.into()),
        WsFrame::Pong(p) => Message::Pong(p.into()),
        WsFrame::Close => Message::Close(None),
    }
}

fn message_to_ws_frame(
    raw: Result<Message, tokio_tungstenite::tungstenite::Error>,
) -> Result<WsFrame, WsRtError> {
    match raw {
        Ok(Message::Text(s)) => Ok(WsFrame::Text(s.to_string())),
        Ok(Message::Binary(b)) => Ok(WsFrame::Binary(b.to_vec())),
        Ok(Message::Ping(p)) => Ok(WsFrame::Ping(p.to_vec())),
        Ok(Message::Pong(p)) => Ok(WsFrame::Pong(p.to_vec())),
        Ok(Message::Close(_)) => Ok(WsFrame::Close),
        // tungstenite can expose raw frames in some configurations; treat as recv error
        Ok(Message::Frame(_)) => Err(WsRtError::Recv("unexpected raw frame".into())),
        Err(e) => Err(WsRtError::Recv(e.to_string())),
    }
}
