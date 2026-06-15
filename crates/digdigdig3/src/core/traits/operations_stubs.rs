//! Stub impls of operations-module traits for connectors that don't natively
//! support them. Each empty body inherits the default NotImplemented
//! method bodies from the trait definitions.

use super::{
    CancelAll, AmendOrder, BatchOrders,
    AccountTransfers, CustodialFunds, SubAccounts,
    FundingHistory, AccountLedger,
};

// ── CEX — only connectors that appear in at least one stub section ────────────

use crate::l3::open::crypto::cex::coinbase::CoinbaseConnector;
use crate::l3::open::crypto::cex::gemini::GeminiConnector;
use crate::l3::open::crypto::cex::mexc::MexcConnector;
use crate::l3::open::crypto::cex::htx::HtxConnector;
use crate::l3::open::crypto::cex::crypto_com::CryptoComConnector;
use crate::l3::open::crypto::cex::upbit::UpbitConnector;
use crate::l3::open::crypto::cex::deribit::DeribitConnector;
#[cfg(feature = "onchain-evm")]
use crate::l3::open::crypto::dex::hyperliquid::HyperliquidConnector;
use crate::l3::open::crypto::cex::kraken::KrakenConnector;
use crate::l3::open::crypto::cex::bitstamp::BitstampConnector;
use crate::l3::open::crypto::cex::bingx::BingxConnector;
use crate::l3::open::crypto::cex::bitget::BitgetConnector;

// ── DEX ─────────────────────────────────────────────────────────────────────

use crate::l3::open::crypto::dex::lighter::LighterConnector;
use crate::l3::open::crypto::dex::dydx::DydxConnector;

// ── Stocks US ────────────────────────────────────────────────────────────────

use crate::l2::paid::polygon::PolygonConnector;
use crate::l1::free::finnhub::FinnhubConnector;
use crate::l1::paid::tiingo::TiingoConnector;
use crate::l1::paid::twelvedata::TwelvedataConnector;
use crate::l3::gated::stocks::us::alpaca::AlpacaConnector;

// ── Stocks India ─────────────────────────────────────────────────────────────

use crate::l3::gated::stocks::india::angel_one::AngelOneConnector;
use crate::l3::gated::stocks::india::zerodha::ZerodhaConnector;
use crate::l3::gated::stocks::india::upstox::UpstoxConnector;
use crate::l3::gated::stocks::india::dhan::DhanConnector;
use crate::l3::gated::stocks::india::fyers::FyersConnector;

// ── Stocks Other ─────────────────────────────────────────────────────────────

use crate::l1::paid::jquants::JQuantsConnector;
use crate::l1::free::krx::KrxConnector;
use crate::l2::free::moex::MoexConnector;
use crate::l3::gated::stocks::russia::tinkoff::TinkoffConnector;

// ── Forex ────────────────────────────────────────────────────────────────────

use crate::l3::gated::forex::oanda::OandaConnector;
use crate::l3::gated::forex::dukascopy::DukascopyConnector;
use crate::l1::paid::alphavantage::AlphaVantageConnector;

// ── Prediction ───────────────────────────────────────────────────────────────

use crate::l3::open::prediction::polymarket::PolymarketConnector;

// ── Brokers ──────────────────────────────────────────────────────────────────

// ── Data Feeds ───────────────────────────────────────────────────────────────

use crate::l1::free::yahoo::YahooFinanceConnector;
use crate::l2::paid::cryptocompare::CryptoCompareConnector;

// ═══════════════════════════════════════════════════════════════════════════════
// CancelAll stubs
// Real impls: Alpaca, Binance, BingX, Bitfinex, Bitget, Bitstamp, Bybit,
//             Coinbase, CryptoCom, Deribit, GateIO, Gemini, HTX, HyperLiquid,
//             Kraken, KuCoin, MEXC, OKX, Upbit, Upstox
// ═══════════════════════════════════════════════════════════════════════════════

impl CancelAll for LighterConnector {}
impl CancelAll for DydxConnector {}
impl CancelAll for PolygonConnector {}
impl CancelAll for FinnhubConnector {}
impl CancelAll for TiingoConnector {}
impl CancelAll for TwelvedataConnector {}
impl CancelAll for AngelOneConnector {}
impl CancelAll for ZerodhaConnector {}
impl CancelAll for DhanConnector {}
impl CancelAll for FyersConnector {}
impl CancelAll for JQuantsConnector {}
impl CancelAll for KrxConnector {}
impl CancelAll for MoexConnector {}
impl CancelAll for TinkoffConnector {}
impl CancelAll for OandaConnector {}
impl CancelAll for DukascopyConnector {}
impl CancelAll for AlphaVantageConnector {}
impl CancelAll for PolymarketConnector {}
impl CancelAll for YahooFinanceConnector {}
impl CancelAll for CryptoCompareConnector {}

// ═══════════════════════════════════════════════════════════════════════════════
// AmendOrder stubs
// Real impls: Alpaca, AngelOne, Binance, BingX, Bitfinex, Bitget, Bitstamp,
//             Bybit, CryptoCom, Deribit, Dhan, Fyers, GateIO, HyperLiquid,
//             Kraken, KuCoin, Oanda, OKX, Tinkoff, Upbit, Upstox, Zerodha
// ═══════════════════════════════════════════════════════════════════════════════

impl AmendOrder for CoinbaseConnector {}
impl AmendOrder for GeminiConnector {}
impl AmendOrder for MexcConnector {}
impl AmendOrder for HtxConnector {}
impl AmendOrder for LighterConnector {}
impl AmendOrder for DydxConnector {}
impl AmendOrder for PolygonConnector {}
impl AmendOrder for FinnhubConnector {}
impl AmendOrder for TiingoConnector {}
impl AmendOrder for TwelvedataConnector {}
impl AmendOrder for JQuantsConnector {}
impl AmendOrder for KrxConnector {}
impl AmendOrder for MoexConnector {}
impl AmendOrder for DukascopyConnector {}
impl AmendOrder for AlphaVantageConnector {}
impl AmendOrder for PolymarketConnector {}
impl AmendOrder for YahooFinanceConnector {}
impl AmendOrder for CryptoCompareConnector {}

// ═══════════════════════════════════════════════════════════════════════════════
// BatchOrders stubs
// Real impls: Binance, BingX, Bitfinex, Bitget, Bybit, CryptoCom, Fyers,
//             GateIO, HTX, HyperLiquid, Kraken, KuCoin, MEXC, OKX, Upstox
// ═══════════════════════════════════════════════════════════════════════════════

impl BatchOrders for CoinbaseConnector {}
impl BatchOrders for BitstampConnector {}
impl BatchOrders for GeminiConnector {}
impl BatchOrders for UpbitConnector {}
impl BatchOrders for DeribitConnector {}
impl BatchOrders for LighterConnector {}
impl BatchOrders for DydxConnector {}
impl BatchOrders for PolygonConnector {}
impl BatchOrders for FinnhubConnector {}
impl BatchOrders for TiingoConnector {}
impl BatchOrders for TwelvedataConnector {}
impl BatchOrders for AlpacaConnector {}
impl BatchOrders for AngelOneConnector {}
impl BatchOrders for ZerodhaConnector {}
impl BatchOrders for DhanConnector {}
impl BatchOrders for JQuantsConnector {}
impl BatchOrders for KrxConnector {}
impl BatchOrders for MoexConnector {}
impl BatchOrders for TinkoffConnector {}
impl BatchOrders for OandaConnector {}
impl BatchOrders for DukascopyConnector {}
impl BatchOrders for AlphaVantageConnector {}
impl BatchOrders for PolymarketConnector {}
impl BatchOrders for YahooFinanceConnector {}
impl BatchOrders for CryptoCompareConnector {}

// ═══════════════════════════════════════════════════════════════════════════════
// AccountTransfers stubs
// Real impls: Binance, BingX, Bitfinex, Bitget, Bybit, GateIO, HTX,
//             HyperLiquid, KuCoin, MEXC, OKX
// ═══════════════════════════════════════════════════════════════════════════════

impl AccountTransfers for KrakenConnector {}
impl AccountTransfers for CoinbaseConnector {}
impl AccountTransfers for BitstampConnector {}
impl AccountTransfers for GeminiConnector {}
impl AccountTransfers for CryptoComConnector {}
impl AccountTransfers for UpbitConnector {}
impl AccountTransfers for DeribitConnector {}
impl AccountTransfers for LighterConnector {}
impl AccountTransfers for DydxConnector {}
impl AccountTransfers for PolygonConnector {}
impl AccountTransfers for FinnhubConnector {}
impl AccountTransfers for TiingoConnector {}
impl AccountTransfers for TwelvedataConnector {}
impl AccountTransfers for AlpacaConnector {}
impl AccountTransfers for AngelOneConnector {}
impl AccountTransfers for ZerodhaConnector {}
impl AccountTransfers for UpstoxConnector {}
impl AccountTransfers for DhanConnector {}
impl AccountTransfers for FyersConnector {}
impl AccountTransfers for JQuantsConnector {}
impl AccountTransfers for KrxConnector {}
impl AccountTransfers for MoexConnector {}
impl AccountTransfers for TinkoffConnector {}
impl AccountTransfers for OandaConnector {}
impl AccountTransfers for DukascopyConnector {}
impl AccountTransfers for AlphaVantageConnector {}
impl AccountTransfers for PolymarketConnector {}
impl AccountTransfers for YahooFinanceConnector {}
impl AccountTransfers for CryptoCompareConnector {}

// ═══════════════════════════════════════════════════════════════════════════════
// CustodialFunds stubs
// Real impls: Binance, BingX, Bitfinex, Bitget, Bitstamp, Bybit, Coinbase,
//             CryptoCom, Deribit, GateIO, Gemini, HTX, Kraken, KuCoin, MEXC,
//             OKX, Upbit
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(feature = "onchain-evm")]
impl CustodialFunds for HyperliquidConnector {}
impl CustodialFunds for LighterConnector {}
impl CustodialFunds for DydxConnector {}
impl CustodialFunds for PolygonConnector {}
impl CustodialFunds for FinnhubConnector {}
impl CustodialFunds for TiingoConnector {}
impl CustodialFunds for TwelvedataConnector {}
impl CustodialFunds for AlpacaConnector {}
impl CustodialFunds for AngelOneConnector {}
impl CustodialFunds for ZerodhaConnector {}
impl CustodialFunds for UpstoxConnector {}
impl CustodialFunds for DhanConnector {}
impl CustodialFunds for FyersConnector {}
impl CustodialFunds for JQuantsConnector {}
impl CustodialFunds for KrxConnector {}
impl CustodialFunds for MoexConnector {}
impl CustodialFunds for TinkoffConnector {}
impl CustodialFunds for OandaConnector {}
impl CustodialFunds for DukascopyConnector {}
impl CustodialFunds for AlphaVantageConnector {}
impl CustodialFunds for PolymarketConnector {}
impl CustodialFunds for YahooFinanceConnector {}
impl CustodialFunds for CryptoCompareConnector {}

// ═══════════════════════════════════════════════════════════════════════════════
// SubAccounts stubs
// Real impls: Binance, BingX, Bitfinex, Bitget, Bybit, CryptoCom, GateIO,
//             HTX, Kraken, KuCoin, MEXC, OKX
// ═══════════════════════════════════════════════════════════════════════════════

impl SubAccounts for CoinbaseConnector {}
impl SubAccounts for BitstampConnector {}
impl SubAccounts for GeminiConnector {}
impl SubAccounts for UpbitConnector {}
impl SubAccounts for DeribitConnector {}
#[cfg(feature = "onchain-evm")]
impl SubAccounts for HyperliquidConnector {}
impl SubAccounts for LighterConnector {}
impl SubAccounts for DydxConnector {}
impl SubAccounts for PolygonConnector {}
impl SubAccounts for FinnhubConnector {}
impl SubAccounts for TiingoConnector {}
impl SubAccounts for TwelvedataConnector {}
impl SubAccounts for AlpacaConnector {}
impl SubAccounts for AngelOneConnector {}
impl SubAccounts for ZerodhaConnector {}
impl SubAccounts for UpstoxConnector {}
impl SubAccounts for DhanConnector {}
impl SubAccounts for FyersConnector {}
impl SubAccounts for JQuantsConnector {}
impl SubAccounts for KrxConnector {}
impl SubAccounts for MoexConnector {}
impl SubAccounts for TinkoffConnector {}
impl SubAccounts for OandaConnector {}
impl SubAccounts for DukascopyConnector {}
impl SubAccounts for AlphaVantageConnector {}
impl SubAccounts for PolymarketConnector {}
impl SubAccounts for YahooFinanceConnector {}
impl SubAccounts for CryptoCompareConnector {}

// ═══════════════════════════════════════════════════════════════════════════════
// FundingHistory stubs
// Real impls: Binance, Bitfinex, Bybit, Deribit, DyDx, GateIO, HyperLiquid,
//             Kraken, KuCoin, OKX
// ═══════════════════════════════════════════════════════════════════════════════

impl FundingHistory for CoinbaseConnector {}
impl FundingHistory for BitstampConnector {}
impl FundingHistory for GeminiConnector {}
impl FundingHistory for MexcConnector {}
impl FundingHistory for HtxConnector {}
impl FundingHistory for BitgetConnector {}
impl FundingHistory for BingxConnector {}
impl FundingHistory for CryptoComConnector {}
impl FundingHistory for UpbitConnector {}
impl FundingHistory for LighterConnector {}
impl FundingHistory for PolygonConnector {}
impl FundingHistory for FinnhubConnector {}
impl FundingHistory for TiingoConnector {}
impl FundingHistory for TwelvedataConnector {}
impl FundingHistory for AlpacaConnector {}
impl FundingHistory for AngelOneConnector {}
impl FundingHistory for ZerodhaConnector {}
impl FundingHistory for UpstoxConnector {}
impl FundingHistory for DhanConnector {}
impl FundingHistory for FyersConnector {}
impl FundingHistory for JQuantsConnector {}
impl FundingHistory for KrxConnector {}
impl FundingHistory for MoexConnector {}
impl FundingHistory for TinkoffConnector {}
impl FundingHistory for OandaConnector {}
impl FundingHistory for DukascopyConnector {}
impl FundingHistory for AlphaVantageConnector {}
impl FundingHistory for PolymarketConnector {}
impl FundingHistory for YahooFinanceConnector {}
impl FundingHistory for CryptoCompareConnector {}

// ═══════════════════════════════════════════════════════════════════════════════
// AccountLedger stubs
// Real impls: Alpaca, Binance, Bitfinex, Bitget, Bitstamp, Bybit, CryptoCom,
//             Deribit, GateIO, Kraken, KuCoin, OKX
// ═══════════════════════════════════════════════════════════════════════════════

impl AccountLedger for CoinbaseConnector {}
impl AccountLedger for GeminiConnector {}
impl AccountLedger for MexcConnector {}
impl AccountLedger for HtxConnector {}
impl AccountLedger for BingxConnector {}
impl AccountLedger for UpbitConnector {}
#[cfg(feature = "onchain-evm")]
impl AccountLedger for HyperliquidConnector {}
impl AccountLedger for LighterConnector {}
impl AccountLedger for DydxConnector {}
impl AccountLedger for PolygonConnector {}
impl AccountLedger for FinnhubConnector {}
impl AccountLedger for TiingoConnector {}
impl AccountLedger for TwelvedataConnector {}
impl AccountLedger for AngelOneConnector {}
impl AccountLedger for ZerodhaConnector {}
impl AccountLedger for UpstoxConnector {}
impl AccountLedger for DhanConnector {}
impl AccountLedger for FyersConnector {}
impl AccountLedger for JQuantsConnector {}
impl AccountLedger for KrxConnector {}
impl AccountLedger for MoexConnector {}
impl AccountLedger for TinkoffConnector {}
impl AccountLedger for OandaConnector {}
impl AccountLedger for DukascopyConnector {}
impl AccountLedger for AlphaVantageConnector {}
impl AccountLedger for PolymarketConnector {}
impl AccountLedger for YahooFinanceConnector {}
impl AccountLedger for CryptoCompareConnector {}
