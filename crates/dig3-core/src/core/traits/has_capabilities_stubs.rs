//! `HasCapabilities` impls for non-crypto connectors (stocks, forex, data feeds, brokers).
//!
//! These are real reflections of what each connector does — not all-false defaults.
//! Connectors that are pure data providers have market-data flags true, trading flags false.

use crate::core::traits::HasCapabilities;
use crate::core::types::ConnectorCapabilities;

// ── Stocks US ─────────────────────────────────────────────────────────────────

use crate::l2::paid::polygon::PolygonConnector;
use crate::l1::free::finnhub::FinnhubConnector;
use crate::l1::paid::tiingo::TiingoConnector;
use crate::l1::paid::twelvedata::TwelvedataConnector;
use crate::l3::gated::stocks::us::alpaca::AlpacaConnector;

// ── Stocks India ──────────────────────────────────────────────────────────────

use crate::l3::gated::stocks::india::angel_one::AngelOneConnector;
use crate::l3::gated::stocks::india::zerodha::ZerodhaConnector;
use crate::l3::gated::stocks::india::upstox::UpstoxConnector;
use crate::l3::gated::stocks::india::dhan::DhanConnector;
use crate::l3::gated::stocks::india::fyers::FyersConnector;

// ── Stocks Japan ──────────────────────────────────────────────────────────────

use crate::l1::paid::jquants::JQuantsConnector;

// ── Stocks Korea ──────────────────────────────────────────────────────────────

use crate::l1::free::krx::KrxConnector;

// ── Stocks Russia ─────────────────────────────────────────────────────────────

use crate::l2::free::moex::MoexConnector;
use crate::l3::gated::stocks::russia::tinkoff::TinkoffConnector;

// ── Forex ─────────────────────────────────────────────────────────────────────

use crate::l3::gated::forex::oanda::OandaConnector;
use crate::l3::gated::forex::dukascopy::DukascopyConnector;
use crate::l1::paid::alphavantage::AlphaVantageConnector;

// ── Prediction ────────────────────────────────────────────────────────────────

use crate::l3::open::prediction::polymarket::PolymarketConnector;

// ── Brokers ───────────────────────────────────────────────────────────────────

use crate::l3::gated::multi::ib::IBConnector;

// ── Data Feeds ────────────────────────────────────────────────────────────────

use crate::l1::free::yahoo::YahooFinanceConnector;
use crate::l2::paid::cryptocompare::CryptoCompareConnector;

// ── Stocks China ──────────────────────────────────────────────────────────────

use crate::l3::gated::stocks::china::futu::FutuConnector;

// ═══════════════════════════════════════════════════════════════════════════════
// Stocks US
// ═══════════════════════════════════════════════════════════════════════════════

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
