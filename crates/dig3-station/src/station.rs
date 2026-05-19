use std::sync::Arc;

use digdigdig3_core::connector_manager::ExchangeHub;
use digdigdig3_core::core::types::{StreamEvent, SubscriptionRequest, Symbol};
use digdigdig3_core::core::utils::SymbolNormalizer;
use futures_util::StreamExt;
use tokio::sync::{mpsc, oneshot};

use crate::subscription::{Entry, Event, Stream};
use crate::{Result, StationBuilder, StationError, SubscriptionHandle, SubscriptionSet};

/// Phase 1 `Station`. Owns an `ExchangeHub`; `subscribe()` connects each entry
/// and spawns a forwarder task that translates `StreamEvent::Trade` into the
/// station-level `Event::Trade` and pushes it to the handle's mpsc.
pub struct Station {
    hub: Arc<ExchangeHub>,
}

impl std::fmt::Debug for Station {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Station").finish()
    }
}

impl Station {
    pub fn builder() -> StationBuilder {
        StationBuilder::new()
    }

    pub(crate) async fn from_builder() -> Result<Self> {
        // rustls 0.23 panics at TLS init unless exactly one CryptoProvider
        // is registered process-wide. core/http/client.rs installs ring when
        // a HttpClient is constructed; Station may not pull any REST, so we
        // install here too. Idempotent — silently ignored if already set.
        let _ = digdigdig3_core::core::install_default_crypto_provider();
        Ok(Self {
            hub: Arc::new(ExchangeHub::new()),
        })
    }

    /// Connect each entry's WS and start forwarding its trade stream into a
    /// single `SubscriptionHandle`. Other streams in the entry are accepted but
    /// silently ignored in Phase 1.
    pub async fn subscribe(&self, set: SubscriptionSet) -> Result<SubscriptionHandle> {
        if set.is_empty() {
            return Err(StationError::Subscribe(
                "empty SubscriptionSet".into(),
            ));
        }

        let (tx, rx) = mpsc::unbounded_channel::<Event>();
        let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();

        for entry in set.entries {
            self.start_entry(entry, tx.clone()).await?;
        }

        // Drop our `tx`. Subscriber tasks each hold a clone — when they exit
        // the channel closes naturally. We keep the shutdown oneshot as the
        // RAII anchor on the handle side.
        drop(tx);

        // Reaper: when the handle drops, fire the oneshot, then the spawned
        // forwarder tasks will exit on their next `select!` poll.
        tokio::spawn(async move {
            let _ = shutdown_rx.try_recv();
        });

        Ok(SubscriptionHandle {
            rx,
            _shutdown: shutdown_tx,
        })
    }

    async fn start_entry(&self, entry: Entry, tx: mpsc::UnboundedSender<Event>) -> Result<()> {
        // Connect WS for (exchange, account_type).
        self.hub
            .connect_websocket(entry.exchange, entry.account_type, false)
            .await
            .map_err(|e| StationError::Core(format!("connect_websocket: {e}")))?;

        let ws = self
            .hub
            .ws(entry.exchange, entry.account_type)
            .ok_or_else(|| StationError::Core("ws handle missing post-connect".into()))?;

        // Parse symbol "BTC-USDT" / "BTCUSDT" → canonical Symbol{base,quote},
        // then normalize to the exchange-native raw form (per AccountType).
        let canonical = parse_symbol(&entry.symbol);
        let raw_for_exchange = SymbolNormalizer::to_exchange(
            entry.exchange,
            &canonical,
            entry.account_type,
        )
        .map_err(|e| StationError::Subscribe(format!("symbol normalize: {e}")))?;
        let sym = Symbol::with_raw(&canonical.base, &canonical.quote, raw_for_exchange);

        // Phase 1: forward only Trade. Other Stream variants are no-op.
        let mut want_trade = false;
        for s in &entry.streams {
            if matches!(s, Stream::Trade) {
                want_trade = true;
            }
        }
        if !want_trade {
            return Ok(()); // nothing to do in Phase 1 for this entry
        }

        ws.subscribe(SubscriptionRequest::trade_for(sym.clone(), entry.account_type))
            .await
            .map_err(|e| StationError::Subscribe(format!("ws.subscribe trade: {e}")))?;

        let exchange = entry.exchange;
        let symbol_label = entry.symbol.clone();
        let mut stream = ws.event_stream();

        tokio::spawn(async move {
            while let Some(item) = stream.next().await {
                let ev = match item {
                    Ok(ev) => ev,
                    Err(e) => {
                        tracing::warn!(?e, "ws event_stream yielded err");
                        continue;
                    }
                };
                if let StreamEvent::Trade(t) = ev {
                    let out = Event::Trade {
                        exchange,
                        symbol: symbol_label.clone(),
                        price: t.price,
                        quantity: t.quantity,
                        side: format!("{:?}", t.side),
                        timestamp: t.timestamp,
                    };
                    if tx.send(out).is_err() {
                        break; // consumer dropped
                    }
                }
            }
        });

        Ok(())
    }
}

/// Best-effort canonical `Symbol` parse: accepts `"BTC-USDT"`, `"BTC/USDT"`,
/// `"BTC_USDT"` or `"BTCUSDT"` (with the common USDT/USDC/USD/BTC/ETH quote
/// stripped). Returns a `Symbol` WITHOUT a `raw` override so the caller can
/// pick the exchange-native form via `SymbolNormalizer::to_exchange`.
fn parse_symbol(s: &str) -> Symbol {
    if let Some((b, q)) = s.split_once(['-', '/', '_']) {
        return Symbol::new(b, q);
    }
    let upper = s.to_uppercase();
    for q in ["USDT", "USDC", "USD", "BTC", "ETH", "BUSD", "EUR", "JPY"] {
        if let Some(base) = upper.strip_suffix(q) {
            if !base.is_empty() {
                return Symbol::new(base, q);
            }
        }
    }
    Symbol::new(&upper, "")
}
