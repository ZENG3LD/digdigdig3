//! Wasm runtime impl: wasm-bindgen-futures + gloo-timers + web_sys WebSocket actor.
//!
//! The key design decision (from docs/research/wasm-wave2/websys-actor-design.md §3):
//!
//! ## Actor pattern
//!
//! `web_sys::WebSocket` and its `Closure` callbacks are `!Send`. Rather than
//! propagating `!Send` into `WsConn` callers, we encapsulate all JS-side state
//! inside a `spawn_local`'d actor task. The `WasmConn` public handle exposes
//! only two `futures_channel::mpsc` channel endpoints — both `!Send` on the
//! single-threaded wasm target, which matches the `?Send` bound on wasm's
//! `WsConn` impl.
//!
//! ## Closure lifetime
//!
//! `Closure::forget()` leaks memory. The correct approach is to keep every
//! `Closure` object alive for the duration of the connection. We move all four
//! closures (onopen, onmessage, onerror, onclose) into the actor task's
//! `_keep` binding, which is dropped when `actor_loop` returns (i.e. when the
//! connection closes or the caller drops both channel ends). The JS GC then
//! sees zero references to the WebSocket object and collects it.
//!
//! ## Channel topology
//!
//! ```text
//! JS callbacks → ev_tx → [actor_loop] → in_tx → WasmConn.in_rx  (inbound)
//!                         [actor_loop] ← out_rx ← WasmConn.out_tx (outbound)
//! ```
//!
//! Three mpsc channels, all unbounded:
//! - `ev_*`: carries `WsMsg` variants from the four JS callbacks into the actor.
//! - `out_*`: carries outbound `WsFrame`s from the caller into the actor, which
//!   forwards them to `ws.send_with_str` / `ws.send_with_u8_array`.
//! - `in_*`: carries inbound `Result<WsFrame, WsRtError>` from the actor to
//!   the caller's `WsConn::next_frame`.

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use async_trait::async_trait;
use futures_channel::mpsc::{self, UnboundedReceiver, UnboundedSender};
use futures_util::StreamExt;
use gloo_timers::future::sleep as gloo_sleep;
use js_sys::Uint8Array;
use wasm_bindgen::{closure::Closure, JsCast};
use wasm_bindgen_futures::spawn_local;
use web_sys::{BinaryType, CloseEvent, ErrorEvent, MessageEvent, WebSocket};

use super::{Spawn, Timer, WsConn, WsConnector, WsFrame, WsRtError};

// ─── WasmRuntime ──────────────────────────────────────────────────────────────

/// Zero-size token representing the wasm single-threaded runtime.
pub struct WasmRuntime;

impl Spawn for WasmRuntime {
    fn spawn(&self, fut: Pin<Box<dyn Future<Output = ()> + 'static>>) {
        spawn_local(fut);
    }
}

impl Timer for WasmRuntime {
    fn sleep(&self, dur: Duration) -> Pin<Box<dyn Future<Output = ()> + 'static>> {
        Box::pin(gloo_sleep(dur))
    }
}

// ─── Internal message type (JS callbacks → actor) ─────────────────────────────

enum WsMsg {
    Open,
    Text(String),
    Binary(Vec<u8>),
    Error(String),
    Close { code: u16, reason: String },
}

// ─── WsConnector for WasmRuntime ─────────────────────────────────────────────

#[async_trait(?Send)]
impl WsConnector for WasmRuntime {
    async fn connect(&self, url: &str, timeout: Duration) -> Result<Box<dyn WsConn>, WsRtError> {
        connect_wasm(url, timeout).await.map(|c| Box::new(c) as Box<dyn WsConn>)
    }
}

// ─── Connection constructor ───────────────────────────────────────────────────

/// Build a `WasmConn` for `url`, waiting up to `timeout` for the open event.
async fn connect_wasm(url: &str, timeout: Duration) -> Result<WasmConn, WsRtError> {
    let ws = WebSocket::new(url).map_err(|e| WsRtError::Connect(format!("{e:?}")))?;
    ws.set_binary_type(BinaryType::Arraybuffer);

    // Channel A: JS callbacks → actor loop
    let (ev_tx, ev_rx): (UnboundedSender<WsMsg>, UnboundedReceiver<WsMsg>) = mpsc::unbounded();

    // Channel B: caller → actor (outbound frames)
    let (out_tx, out_rx): (UnboundedSender<WsFrame>, UnboundedReceiver<WsFrame>) =
        mpsc::unbounded();

    // Channel C: actor → caller (inbound frames)
    let (in_tx, in_rx): (
        UnboundedSender<Result<WsFrame, WsRtError>>,
        UnboundedReceiver<Result<WsFrame, WsRtError>>,
    ) = mpsc::unbounded();

    // One-shot channel used to signal open/error before we return.
    let (ready_tx, ready_rx) = futures_channel::oneshot::channel::<Result<(), WsRtError>>();
    // Wrapped so we can move a clone into each closure (only onopen / onerror use it).
    use std::cell::RefCell;
    use std::rc::Rc;
    let ready_tx = Rc::new(RefCell::new(Some(ready_tx)));

    // ── Install 4 closures ────────────────────────────────────────────────────

    // onopen: signals the one-shot that the connection is live.
    let ev_tx_open = ev_tx.clone();
    let ready_open = ready_tx.clone();
    let on_open = Closure::<dyn FnMut(_)>::new(move |_ev: web_sys::Event| {
        let _ = ev_tx_open.unbounded_send(WsMsg::Open);
        if let Some(tx) = ready_open.borrow_mut().take() {
            let _ = tx.send(Ok(()));
        }
    });
    ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));

    // onmessage: parse text or ArrayBuffer binary frames.
    let ev_tx_msg = ev_tx.clone();
    let on_message = Closure::<dyn FnMut(_)>::new(move |ev: MessageEvent| {
        let data = ev.data();
        if let Some(text) = data.as_string() {
            let _ = ev_tx_msg.unbounded_send(WsMsg::Text(text));
        } else if let Ok(buf) = data.dyn_into::<js_sys::ArrayBuffer>() {
            let arr = Uint8Array::new(&buf);
            let mut bytes = vec![0u8; arr.length() as usize];
            arr.copy_to(&mut bytes);
            let _ = ev_tx_msg.unbounded_send(WsMsg::Binary(bytes));
        }
    });
    ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));

    // onerror: forward error + fire ready oneshot if not yet open.
    let ev_tx_err = ev_tx.clone();
    let ready_err = ready_tx.clone();
    let on_error = Closure::<dyn FnMut(_)>::new(move |ev: ErrorEvent| {
        let msg = ev.message();
        let _ = ev_tx_err.unbounded_send(WsMsg::Error(msg.clone()));
        if let Some(tx) = ready_err.borrow_mut().take() {
            let _ = tx.send(Err(WsRtError::Connect(msg)));
        }
    });
    ws.set_onerror(Some(on_error.as_ref().unchecked_ref()));

    // onclose: forward close event.
    let ev_tx_close = ev_tx;
    let on_close = Closure::<dyn FnMut(_)>::new(move |ev: CloseEvent| {
        let _ = ev_tx_close.unbounded_send(WsMsg::Close {
            code: ev.code(),
            reason: ev.reason(),
        });
    });
    ws.set_onclose(Some(on_close.as_ref().unchecked_ref()));

    // ── Wait for open or timeout ──────────────────────────────────────────────

    let timeout_fut = gloo_sleep(timeout);
    futures_util::pin_mut!(timeout_fut);
    futures_util::pin_mut!(ready_rx);

    match futures_util::future::select(ready_rx, timeout_fut).await {
        futures_util::future::Either::Left((Ok(Ok(())), _)) => { /* connected */ }
        futures_util::future::Either::Left((Ok(Err(e)), _)) => return Err(e),
        futures_util::future::Either::Left((Err(_), _)) => {
            return Err(WsRtError::Connect("ready channel closed".into()))
        }
        futures_util::future::Either::Right(_) => return Err(WsRtError::Timeout),
    }

    // ── Spawn the actor task ──────────────────────────────────────────────────
    // The actor owns:
    //   - a clone of the WebSocket (cheap JsValue ref-count)
    //   - all 4 Closure objects (kept alive via `_keep` until the loop exits)
    //   - the internal event channel receiver
    //   - the outbound channel receiver
    //   - the inbound channel sender

    let ws_actor = ws.clone();
    spawn_local(async move {
        // SAFETY of lifetime: `_keep` binds the four Closure objects for the
        // entire duration of `actor_loop`. When the loop exits (channel closed
        // or onclose received), the closures drop, releasing the JS event
        // handlers. The WebSocket refcount reaches zero once `ws_actor` also
        // drops at end of this async block.
        let _keep = (on_open, on_message, on_error, on_close);
        actor_loop(ws_actor, ev_rx, out_rx, in_tx).await;
    });

    Ok(WasmConn { _ws: ws, in_rx, out_tx })
}

// ─── Actor loop ───────────────────────────────────────────────────────────────

/// Drives the WebSocket: fans inbound JS events to the caller channel,
/// fans outbound caller frames to the WebSocket.
///
/// Exits when either channel is closed (caller dropped the handle) or
/// an onclose event arrives from the server.
async fn actor_loop(
    ws: WebSocket,
    mut ev_rx: UnboundedReceiver<WsMsg>,
    mut out_rx: UnboundedReceiver<WsFrame>,
    in_tx: UnboundedSender<Result<WsFrame, WsRtError>>,
) {
    loop {
        tokio::select! {
            // Inbound from JS
            ev = ev_rx.next() => match ev {
                Some(WsMsg::Text(s)) => {
                    if in_tx.unbounded_send(Ok(WsFrame::Text(s))).is_err() {
                        break; // caller dropped in_rx
                    }
                }
                Some(WsMsg::Binary(b)) => {
                    if in_tx.unbounded_send(Ok(WsFrame::Binary(b))).is_err() {
                        break;
                    }
                }
                Some(WsMsg::Error(msg)) => {
                    let _ = in_tx.unbounded_send(Err(WsRtError::Recv(msg)));
                }
                Some(WsMsg::Close { code, reason }) => {
                    let detail = format!("code={code} reason={reason}");
                    let _ = in_tx.unbounded_send(Err(WsRtError::Recv(detail)));
                    break;
                }
                Some(WsMsg::Open) => { /* already handled in constructor */ }
                None => break, // ev channel closed
            },
            // Outbound from caller
            frame = out_rx.next() => match frame {
                Some(WsFrame::Text(s)) => {
                    let _ = ws.send_with_str(&s);
                }
                Some(WsFrame::Binary(b)) => {
                    let _ = ws.send_with_u8_array(&b);
                }
                Some(WsFrame::Close) => {
                    let _ = ws.close();
                    break;
                }
                // Ping/Pong: browsers handle PING automatically; PONG not
                // directly sendable via web_sys. Silently ignore.
                Some(WsFrame::Ping(_)) | Some(WsFrame::Pong(_)) => {}
                None => break, // caller dropped out_tx
            },
        }
    }
}

// ─── WasmConn ────────────────────────────────────────────────────────────────

/// Public handle for a browser WebSocket connection.
///
/// Holds only channel endpoints — `web_sys::WebSocket` and its `Closure`
/// objects live inside the `spawn_local`'d actor task, not here. This keeps
/// the `!Send` JS state fully encapsulated.
pub struct WasmConn {
    /// Kept alive so the actor task's `ws_actor.clone()` shares the same
    /// underlying JS object. Dropping this alone does not close the socket —
    /// that only happens when the actor loop exits.
    _ws: WebSocket,
    in_rx: UnboundedReceiver<Result<WsFrame, WsRtError>>,
    out_tx: UnboundedSender<WsFrame>,
}

#[async_trait(?Send)]
impl WsConn for WasmConn {
    async fn send(&mut self, frame: WsFrame) -> Result<(), WsRtError> {
        self.out_tx
            .unbounded_send(frame)
            .map_err(|e| WsRtError::Send(e.to_string()))
    }

    async fn next_frame(&mut self) -> Option<Result<WsFrame, WsRtError>> {
        self.in_rx.next().await
    }

    async fn close(&mut self) -> Result<(), WsRtError> {
        self.send(WsFrame::Close).await
    }
}

