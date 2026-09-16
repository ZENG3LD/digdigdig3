//! `HasCapabilities` impls for non-crypto connectors (stocks, forex, data feeds, brokers).
//!
//! These are real reflections of what each connector does — not all-false defaults.
//! Connectors that are pure data providers have market-data flags true, trading flags false.

// This whole file only has something to compile when at least one of the
// non-CEX/DEX exchange features below is enabled — with only crypto
// exchanges selected (e.g. `--features exchange-binance`), every impl in
// this file is compiled out and these two imports would otherwise be
// genuinely unused.
#[cfg(any(
    feature = "exchange-polygon", feature = "exchange-finnhub", feature = "exchange-tiingo",
    feature = "exchange-twelvedata", feature = "exchange-alpaca", feature = "exchange-angel-one",
    feature = "exchange-zerodha", feature = "exchange-upstox", feature = "exchange-dhan",
    feature = "exchange-fyers", feature = "exchange-jquants", feature = "exchange-krx",
    feature = "exchange-moex", feature = "exchange-tinkoff", feature = "exchange-oanda",
    feature = "exchange-dukascopy", feature = "exchange-alphavantage", feature = "exchange-polymarket",
    feature = "exchange-ib", feature = "exchange-yahoo", feature = "exchange-cryptocompare",
    feature = "exchange-futu",
))]
use crate::core::traits::HasCapabilities;
#[cfg(any(
    feature = "exchange-polygon", feature = "exchange-finnhub", feature = "exchange-tiingo",
    feature = "exchange-twelvedata", feature = "exchange-alpaca", feature = "exchange-angel-one",
    feature = "exchange-zerodha", feature = "exchange-upstox", feature = "exchange-dhan",
    feature = "exchange-fyers", feature = "exchange-jquants", feature = "exchange-krx",
    feature = "exchange-moex", feature = "exchange-tinkoff", feature = "exchange-oanda",
    feature = "exchange-dukascopy", feature = "exchange-alphavantage", feature = "exchange-polymarket",
    feature = "exchange-ib", feature = "exchange-yahoo", feature = "exchange-cryptocompare",
    feature = "exchange-futu",
))]
use crate::core::types::ConnectorCapabilities;

// ── Stocks US ─────────────────────────────────────────────────────────────────

#[cfg(feature = "exchange-polygon")]
use crate::l2::paid::polygon::PolygonConnector;
#[cfg(feature = "exchange-finnhub")]
use crate::l1::free::finnhub::FinnhubConnector;
#[cfg(feature = "exchange-tiingo")]
use crate::l1::paid::tiingo::TiingoConnector;
#[cfg(feature = "exchange-twelvedata")]
use crate::l1::paid::twelvedata::TwelvedataConnector;
#[cfg(feature = "exchange-alpaca")]
use crate::l3::gated::stocks::us::alpaca::AlpacaConnector;

// ── Stocks India ──────────────────────────────────────────────────────────────

#[cfg(feature = "exchange-angel-one")]
use crate::l3::gated::stocks::india::angel_one::AngelOneConnector;
#[cfg(feature = "exchange-zerodha")]
use crate::l3::gated::stocks::india::zerodha::ZerodhaConnector;
#[cfg(feature = "exchange-upstox")]
use crate::l3::gated::stocks::india::upstox::UpstoxConnector;
#[cfg(feature = "exchange-dhan")]
use crate::l3::gated::stocks::india::dhan::DhanConnector;
#[cfg(feature = "exchange-fyers")]
use crate::l3::gated::stocks::india::fyers::FyersConnector;

// ── Stocks Japan ──────────────────────────────────────────────────────────────

#[cfg(feature = "exchange-jquants")]
use crate::l1::paid::jquants::JQuantsConnector;

// ── Stocks Korea ──────────────────────────────────────────────────────────────

#[cfg(feature = "exchange-krx")]
use crate::l1::free::krx::KrxConnector;

// ── Stocks Russia ─────────────────────────────────────────────────────────────

#[cfg(feature = "exchange-moex")]
use crate::l2::free::moex::MoexConnector;
#[cfg(feature = "exchange-tinkoff")]
use crate::l3::gated::stocks::russia::tinkoff::TinkoffConnector;

// ── Forex ─────────────────────────────────────────────────────────────────────

#[cfg(feature = "exchange-oanda")]
use crate::l3::gated::forex::oanda::OandaConnector;
#[cfg(feature = "exchange-dukascopy")]
use crate::l3::gated::forex::dukascopy::DukascopyConnector;
#[cfg(feature = "exchange-alphavantage")]
use crate::l1::paid::alphavantage::AlphaVantageConnector;

// ── Prediction ────────────────────────────────────────────────────────────────

#[cfg(feature = "exchange-polymarket")]
use crate::l3::open::prediction::polymarket::PolymarketConnector;

// ── Brokers ───────────────────────────────────────────────────────────────────

#[cfg(all(feature = "exchange-ib", not(target_arch = "wasm32")))]
use crate::l3::gated::multi::ib::IBConnector;

// ── Data Feeds ────────────────────────────────────────────────────────────────

#[cfg(feature = "exchange-yahoo")]
use crate::l1::free::yahoo::YahooFinanceConnector;
#[cfg(feature = "exchange-cryptocompare")]
use crate::l2::paid::cryptocompare::CryptoCompareConnector;

// ── Stocks China ──────────────────────────────────────────────────────────────

#[cfg(feature = "exchange-futu")]
use crate::l3::gated::stocks::china::futu::FutuConnector;

// ═══════════════════════════════════════════════════════════════════════════════
// Stocks US
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(feature = "exchange-polygon")]
impl HasCapabilities for PolygonConnector {
    fn capabilities(&self) -> ConnectorCapabilities {
        ConnectorCapabilities {
            // MarketData — data provider, no recent trades, no live exchange info
            has_ticker: true,
            has_orderbook: false,
            has_klines: true,
            has_recent_trades: false,
            has_exchange_info: true,
            // Trading — read-only data provider
            has_market_order: false,
            has_limit_order: false,
            has_open_orders: false,
            has_order_history: false,
            has_user_trades: false,
            // Account
            has_balance: false,
            has_account_info: false,
            has_fees: false,
            has_transfers: false,
            has_deposit_withdraw: false,
            has_sub_accounts: false,
            has_funding_payments: false,
            has_ledger: false,
            ..Default::default()
        }
    }
}

#[cfg(feature = "exchange-finnhub")]
impl HasCapabilities for FinnhubConnector {
    fn capabilities(&self) -> ConnectorCapabilities {
        ConnectorCapabilities {
            has_ticker: true,
            has_orderbook: false,
            has_klines: true,
            has_recent_trades: false,
            has_exchange_info: true,
            has_websocket: true,
            has_ws_ticker: true,
            has_ws_trades: true,
            ..Default::default()
        }
    }
}

#[cfg(feature = "exchange-tiingo")]
impl HasCapabilities for TiingoConnector {
    fn capabilities(&self) -> ConnectorCapabilities {
        ConnectorCapabilities {
            has_ticker: true,
            has_orderbook: false,
            has_klines: true,
            has_recent_trades: false,
            has_exchange_info: false,
            ..Default::default()
        }
    }
}

#[cfg(feature = "exchange-twelvedata")]
impl HasCapabilities for TwelvedataConnector {
    fn capabilities(&self) -> ConnectorCapabilities {
        ConnectorCapabilities {
            has_ticker: true,
            has_orderbook: false,
            has_klines: true,
            has_recent_trades: false,
            has_exchange_info: true,
            has_websocket: true,
            has_ws_ticker: true,
            has_ws_klines: true,
            ..Default::default()
        }
    }
}

#[cfg(feature = "exchange-alpaca")]
impl HasCapabilities for AlpacaConnector {
    fn capabilities(&self) -> ConnectorCapabilities {
        ConnectorCapabilities {
            // MarketData
            has_ticker: true,
            has_orderbook: true,
            has_klines: true,
            has_recent_trades: true,
            has_exchange_info: true,
            // Trading — Alpaca is a full broker
            has_market_order: true,
            has_limit_order: true,
            has_open_orders: true,
            has_order_history: true,
            has_user_trades: true,
            // Operations
            has_cancel_all: true,
            has_amend_order: true,
            has_batch_place: false,
            has_batch_cancel: false,
            // Account
            has_balance: true,
            has_account_info: true,
            has_fees: true,
            has_transfers: false,
            has_deposit_withdraw: false,
            has_sub_accounts: false,
            has_funding_payments: false,
            has_ledger: true,
            // WebSocket
            has_websocket: true,
            has_ws_ticker: true,
            has_ws_trades: true,
            has_ws_orderbook: true,
            has_ws_klines: true,
            ..Default::default()
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Stocks India
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(feature = "exchange-angel-one")]
impl HasCapabilities for AngelOneConnector {
    fn capabilities(&self) -> ConnectorCapabilities {
        ConnectorCapabilities {
            has_ticker: true,
            has_orderbook: true,
            has_klines: true,
            has_recent_trades: false,
            has_exchange_info: true,
            has_market_order: true,
            has_limit_order: true,
            has_open_orders: true,
            has_order_history: true,
            has_user_trades: true,
            has_cancel_all: false,
            has_amend_order: true,
            has_balance: true,
            has_account_info: true,
            has_fees: true,
            ..Default::default()
        }
    }
}

#[cfg(feature = "exchange-zerodha")]
impl HasCapabilities for ZerodhaConnector {
    fn capabilities(&self) -> ConnectorCapabilities {
        ConnectorCapabilities {
            has_ticker: true,
            has_orderbook: true,
            has_klines: true,
            has_recent_trades: false,
            has_exchange_info: true,
            has_market_order: true,
            has_limit_order: true,
            has_open_orders: true,
            has_order_history: true,
            has_user_trades: true,
            has_amend_order: true,
            has_balance: true,
            has_account_info: true,
            has_fees: true,
            has_websocket: true,
            has_ws_ticker: true,
            has_ws_orderbook: true,
            ..Default::default()
        }
    }
}

#[cfg(feature = "exchange-upstox")]
impl HasCapabilities for UpstoxConnector {
    fn capabilities(&self) -> ConnectorCapabilities {
        ConnectorCapabilities {
            has_ticker: true,
            has_orderbook: true,
            has_klines: true,
            has_recent_trades: false,
            has_exchange_info: true,
            has_market_order: true,
            has_limit_order: true,
            has_open_orders: true,
            has_order_history: true,
            has_user_trades: true,
            has_cancel_all: true,
            has_amend_order: true,
            has_batch_place: true,
            has_batch_cancel: false,
            max_batch_place_size: 25,
            has_balance: true,
            has_account_info: true,
            has_fees: true,
            has_websocket: true,
            has_ws_ticker: true,
            has_ws_orderbook: true,
            has_ws_trades: true,
            ..Default::default()
        }
    }
}

#[cfg(feature = "exchange-dhan")]
impl HasCapabilities for DhanConnector {
    fn capabilities(&self) -> ConnectorCapabilities {
        ConnectorCapabilities {
            has_ticker: true,
            has_orderbook: true,
            has_klines: true,
            has_recent_trades: false,
            has_exchange_info: true,
            has_market_order: true,
            has_limit_order: true,
            has_open_orders: true,
            has_order_history: true,
            has_user_trades: true,
            has_amend_order: true,
            has_balance: true,
            has_account_info: true,
            has_fees: true,
            has_websocket: true,
            has_ws_ticker: true,
            has_ws_orderbook: true,
            ..Default::default()
        }
    }
}

#[cfg(feature = "exchange-fyers")]
impl HasCapabilities for FyersConnector {
    fn capabilities(&self) -> ConnectorCapabilities {
        ConnectorCapabilities {
            has_ticker: true,
            has_orderbook: true,
            has_klines: true,
            has_recent_trades: false,
            has_exchange_info: true,
            has_market_order: true,
            has_limit_order: true,
            has_open_orders: true,
            has_order_history: true,
            has_user_trades: true,
            has_amend_order: true,
            has_batch_place: true,
            has_batch_cancel: false,
            max_batch_place_size: 10,
            has_balance: true,
            has_account_info: true,
            has_fees: true,
            has_websocket: true,
            has_ws_ticker: true,
            has_ws_orderbook: true,
            ..Default::default()
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Stocks Japan / Korea / Russia
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(feature = "exchange-jquants")]
impl HasCapabilities for JQuantsConnector {
    fn capabilities(&self) -> ConnectorCapabilities {
        // minimal profile — verify
        ConnectorCapabilities {
            has_ticker: true,
            has_klines: true,
            has_exchange_info: true,
            ..Default::default()
        }
    }
}

#[cfg(feature = "exchange-krx")]
impl HasCapabilities for KrxConnector {
    fn capabilities(&self) -> ConnectorCapabilities {
        // minimal profile — verify
        ConnectorCapabilities {
            has_ticker: true,
            has_klines: true,
            has_exchange_info: true,
            ..Default::default()
        }
    }
}

#[cfg(feature = "exchange-moex")]
impl HasCapabilities for MoexConnector {
    fn capabilities(&self) -> ConnectorCapabilities {
        // minimal profile — verify
        ConnectorCapabilities {
            has_ticker: true,
            has_orderbook: true,
            has_klines: true,
            has_exchange_info: true,
            validation: self.validation_status(),
            ..Default::default()
        }
    }

    fn validation_status(&self) -> Option<&'static crate::core::types::ValidationStamp> {
        crate::core::utils::validation_snapshot::validation_for(crate::core::types::ExchangeId::Moex)
    }
}

#[cfg(feature = "exchange-tinkoff")]
impl HasCapabilities for TinkoffConnector {
    fn capabilities(&self) -> ConnectorCapabilities {
        ConnectorCapabilities {
            has_ticker: true,
            has_orderbook: true,
            has_klines: true,
            has_recent_trades: false,
            has_exchange_info: true,
            has_market_order: true,
            has_limit_order: true,
            has_open_orders: true,
            has_order_history: true,
            has_user_trades: true,
            has_amend_order: true,
            has_balance: true,
            has_account_info: true,
            has_fees: true,
            has_websocket: true,
            has_ws_ticker: true,
            has_ws_orderbook: true,
            has_ws_trades: true,
            ..Default::default()
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Forex
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(feature = "exchange-oanda")]
impl HasCapabilities for OandaConnector {
    fn capabilities(&self) -> ConnectorCapabilities {
        ConnectorCapabilities {
            has_ticker: true,
            has_orderbook: true,
            has_klines: true,
            has_recent_trades: false,
            has_exchange_info: true,
            has_market_order: true,
            has_limit_order: true,
            has_open_orders: true,
            has_order_history: true,
            has_user_trades: true,
            has_amend_order: true,
            has_balance: true,
            has_account_info: true,
            has_fees: true,
            has_positions: true,
            ..Default::default()
        }
    }
}

#[cfg(feature = "exchange-dukascopy")]
impl HasCapabilities for DukascopyConnector {
    fn capabilities(&self) -> ConnectorCapabilities {
        // minimal profile — verify
        ConnectorCapabilities {
            has_ticker: true,
            has_orderbook: true,
            has_klines: true,
            ..Default::default()
        }
    }
}

#[cfg(feature = "exchange-alphavantage")]
impl HasCapabilities for AlphaVantageConnector {
    fn capabilities(&self) -> ConnectorCapabilities {
        ConnectorCapabilities {
            has_ticker: true,
            has_klines: true,
            has_exchange_info: false,
            ..Default::default()
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Prediction
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(feature = "exchange-polymarket")]
impl HasCapabilities for PolymarketConnector {
    fn capabilities(&self) -> ConnectorCapabilities {
        ConnectorCapabilities {
            // Prediction market — price data and order placement, no klines
            has_ticker: true,
            has_orderbook: true,
            has_klines: false,
            has_recent_trades: true,
            has_exchange_info: true,
            has_market_order: true,
            has_limit_order: true,
            has_open_orders: true,
            has_order_history: true,
            has_user_trades: true,
            has_balance: true,
            has_account_info: true,
            has_websocket: true,
            has_ws_orderbook: true,
            has_ws_trades: true,
            has_ws_ticker: true,
            ..Default::default()
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Brokers
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(all(feature = "exchange-ib", not(target_arch = "wasm32")))]
impl HasCapabilities for IBConnector {
    fn capabilities(&self) -> ConnectorCapabilities {
        // minimal profile — verify (IB connector is incomplete)
        ConnectorCapabilities {
            has_ticker: true,
            has_orderbook: true,
            has_klines: true,
            has_exchange_info: true,
            has_market_order: true,
            has_limit_order: true,
            has_open_orders: true,
            has_order_history: true,
            has_user_trades: true,
            has_balance: true,
            has_account_info: true,
            has_fees: true,
            has_positions: true,
            has_websocket: true,
            has_ws_ticker: true,
            has_ws_orderbook: true,
            ..Default::default()
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Data Feeds
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(feature = "exchange-yahoo")]
impl HasCapabilities for YahooFinanceConnector {
    fn capabilities(&self) -> ConnectorCapabilities {
        ConnectorCapabilities {
            has_ticker: true,
            has_orderbook: false,
            has_klines: true,
            has_recent_trades: false,
            has_exchange_info: true,
            validation: self.validation_status(),
            ..Default::default()
        }
    }

    fn validation_status(&self) -> Option<&'static crate::core::types::ValidationStamp> {
        crate::core::utils::validation_snapshot::validation_for(crate::core::types::ExchangeId::YahooFinance)
    }
}

#[cfg(feature = "exchange-cryptocompare")]
impl HasCapabilities for CryptoCompareConnector {
    fn capabilities(&self) -> ConnectorCapabilities {
        ConnectorCapabilities {
            has_ticker: true,
            has_orderbook: true,
            has_klines: true,
            has_recent_trades: true,
            has_exchange_info: true,
            has_websocket: true,
            has_ws_ticker: true,
            has_ws_trades: true,
            has_ws_klines: true,
            validation: self.validation_status(),
            ..Default::default()
        }
    }

    fn validation_status(&self) -> Option<&'static crate::core::types::ValidationStamp> {
        crate::core::utils::validation_snapshot::validation_for(crate::core::types::ExchangeId::CryptoCompare)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Stocks China
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(feature = "exchange-futu")]
impl HasCapabilities for FutuConnector {
    fn capabilities(&self) -> ConnectorCapabilities {
        ConnectorCapabilities {
            has_ticker: true,
            has_orderbook: true,
            has_klines: true,
            has_recent_trades: false,
            has_exchange_info: true,
            has_market_order: true,
            has_limit_order: true,
            has_open_orders: true,
            has_order_history: true,
            has_user_trades: true,
            has_amend_order: true,
            has_balance: true,
            has_account_info: true,
            has_fees: true,
            has_positions: true,
            has_websocket: true,
            has_ws_ticker: true,
            has_ws_orderbook: true,
            ..Default::default()
        }
    }
}
