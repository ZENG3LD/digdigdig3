use std::path::PathBuf;
use std::sync::Arc;

use digdigdig3_core::connector_manager::ExchangeHub;
use digdigdig3_core::core::types::{StreamEvent, SubscriptionRequest, Symbol};
use digdigdig3_core::core::utils::SymbolNormalizer;
use futures_util::StreamExt;
use tokio::sync::{mpsc, oneshot};

use crate::persistence::TradeWriter;
use crate::subscription::{Entry, Event, Stream};
use crate::{
    PersistenceConfig, Result, StationBuilder, StationError, SubscriptionHandle, SubscriptionSet,
};

/// Phase 1 `Station`. Owns an `ExchangeHub`, a configured storage root, and
/// optional persistence. `subscribe()` connects each entry, opens a per-entry
/// `TradeWriter` if persistence is on, and spawns a forwarder task that
/// translates `StreamEvent::Trade` into station-level `Event::Trade` and
/// pushes it to the handle's mpsc.
pub struct Station {
    hub: Arc<ExchangeHub>,
    storage_root: PathBuf,
    persistence: PersistenceConfig,
}

impl std::fmt::Debug for Station {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Station")
            .field("storage_root", &self.storage_root)
            .field("persistence", &self.persistence)
            .finish()
    }
}

impl Station {
    pub fn builder() -> StationBuilder {
        StationBuilder::new()
    }

    pub fn storage_root(&self) -> &std::path::Path {
        &self.storage_root
    }

    pub(crate) async fn from_builder(b: StationBuilder) -> Result<Self> {
        // rustls 0.23 panics at TLS init unless exactly one CryptoProvider
        // is registered process-wide. core/http/client.rs installs ring when
        // a HttpClient is constructed; Station may not pull any REST, so we
        // install here too. Idempotent — silently ignored if already set.
        let _ = digdigdig3_core::core::install_default_crypto_provider();

        if b.persistence.enabled {
            std::fs::create_dir_all(&b.storage_root).map_err(StationError::Io)?;
        }

        Ok(Self {
            hub: Arc::new(ExchangeHub::new()),
            storage_root: b.storage_root,
            persistence: b.persistence,
        })
    }

    /// Connect each entry's WS, optionally open a `TradeWriter` for it, and
    /// start forwarding `StreamEvent::Trade` into a single `SubscriptionHandle`.
    /// Other stream kinds in the entry are accepted but no-op in Phase 1.
    pub async fn subscribe(&self, set: SubscriptionSet) -> Result<SubscriptionHandle> {
        if set.is_empty() {
            return Err(StationError::Subscribe("empty SubscriptionSet".into()));
        }

        let (tx, rx) = mpsc::unbounded_channel::<Event>();
        let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();

        for entry in set.entries {
            self.start_entry(entry, tx.clone()).await?;
        }

        drop(tx);

        tokio::spawn(async move {
            let _ = shutdown_rx.try_recv();
        });

        Ok(SubscriptionHandle {
            rx,
            _shutdown: shutdown_tx,
        })
    }

    async fn start_entry(&self, entry: Entry, tx: mpsc::UnboundedSender<Event>) -> Result<()> {
        self.hub
            .connect_websocket(entry.exchange, entry.account_type, false)
            .await
            .map_err(|e| StationError::Core(format!("connect_websocket: {e}")))?;

        let ws = self
            .hub
            .ws(entry.exchange, entry.account_type)
            .ok_or_else(|| StationError::Core("ws handle missing post-connect".into()))?;

        let canonical = parse_symbol(&entry.symbol);
        let raw_for_exchange = SymbolNormalizer::to_exchange(
            entry.exchange,
            &canonical,
            entry.account_type,
        )
        .map_err(|e| StationError::Subscribe(format!("symbol normalize: {e}")))?;
        let sym = Symbol::with_raw(&canonical.base, &canonical.quote, raw_for_exchange.clone());

        let want_trade = entry.streams.iter().any(|s| matches!(s, Stream::Trade));
        if !want_trade {
            return Ok(());
        }

        ws.subscribe(SubscriptionRequest::trade_for(sym.clone(), entry.account_type))
            .await
            .map_err(|e| StationError::Subscribe(format!("ws.subscribe trade: {e}")))?;

        // Open trade writer if persistence is on for this build.
        let mut writer: Option<TradeWriter> = None;
        if self.persistence.enabled && self.persistence.trades {
            match TradeWriter::new(
                &self.storage_root,
                &format!("{:?}", entry.exchange).to_lowercase(),
                entry.account_type.as_key_str(),
                &raw_for_exchange,
            ) {
                Ok(w) => writer = Some(w),
                Err(e) => {
                    tracing::warn!(?e, exchange=?entry.exchange, "trade writer open failed; continuing without persistence");
                }
            }
        }

        let exchange = entry.exchange;
        let symbol_label = entry.symbol.clone();
        let mut stream = ws.event_stream();

        tokio::spawn(async move {
            let mut writer = writer; // moved into task
            while let Some(item) = stream.next().await {
                let ev = match item {
                    Ok(ev) => ev,
                    Err(e) => {
                        tracing::warn!(?e, "ws event_stream yielded err");
                        continue;
                    }
                };
                if let StreamEvent::Trade(t) = ev {
                    if let Some(w) = writer.as_mut() {
                        if let Err(e) = w.append(t.timestamp, t.price, t.quantity, t.side, &t.id) {
                            tracing::warn!(?e, "trade writer append failed");
                        }
                    }
                    let out = Event::Trade {
                        exchange,
                        symbol: symbol_label.clone(),
                        price: t.price,
                        quantity: t.quantity,
                        side: format!("{:?}", t.side),
                        timestamp: t.timestamp,
                    };
                    if tx.send(out).is_err() {
                        break;
                    }
                }
            }
            // writer Drops here, flushing.
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
