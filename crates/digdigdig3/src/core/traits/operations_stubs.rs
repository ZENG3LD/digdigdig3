//! Stub impls of operations-module traits for connectors that don't natively
//! support them. Each empty body inherits the default NotImplemented
//! method bodies from the trait definitions.

// Every impl in this file is gated to one non-Binance exchange feature or
// another (see the per-impl `#[cfg(feature = "exchange-...")]` attributes
// below) — with a narrow build (e.g. `--features exchange-binance` alone,
// Binance's own real impls live in its own connector.rs, never here) every
// impl below compiles out, which would make this whole trait import
// otherwise unused.
#[cfg(any(
    feature = "exchange-coinbase", feature = "exchange-gemini", feature = "exchange-mexc",
    feature = "exchange-htx", feature = "exchange-crypto-com", feature = "exchange-upbit",
    feature = "exchange-deribit", feature = "exchange-hyperliquid", feature = "exchange-kraken",
    feature = "exchange-bitstamp", feature = "exchange-bingx", feature = "exchange-bitget",
    feature = "exchange-lighter", feature = "exchange-dydx", feature = "exchange-polygon",
    feature = "exchange-finnhub", feature = "exchange-tiingo", feature = "exchange-twelvedata",
    feature = "exchange-alpaca", feature = "exchange-angel-one", feature = "exchange-zerodha",
    feature = "exchange-upstox", feature = "exchange-dhan", feature = "exchange-fyers",
    feature = "exchange-jquants", feature = "exchange-krx", feature = "exchange-moex",
    feature = "exchange-tinkoff", feature = "exchange-oanda", feature = "exchange-dukascopy",
    feature = "exchange-alphavantage", feature = "exchange-polymarket", feature = "exchange-yahoo",
    feature = "exchange-cryptocompare",
))]
use super::{
    CancelAll, AmendOrder, BatchOrders,
    AccountTransfers, CustodialFunds, SubAccounts,
    FundingHistory, AccountLedger,
};

// ── CEX — only connectors that appear in at least one stub section ────────────

#[cfg(feature = "exchange-coinbase")]
use crate::l3::open::crypto::cex::coinbase::CoinbaseConnector;
#[cfg(feature = "exchange-gemini")]
use crate::l3::open::crypto::cex::gemini::GeminiConnector;
#[cfg(feature = "exchange-mexc")]
use crate::l3::open::crypto::cex::mexc::MexcConnector;
#[cfg(feature = "exchange-htx")]
use crate::l3::open::crypto::cex::htx::HtxConnector;
#[cfg(feature = "exchange-crypto-com")]
use crate::l3::open::crypto::cex::crypto_com::CryptoComConnector;
#[cfg(feature = "exchange-upbit")]
use crate::l3::open::crypto::cex::upbit::UpbitConnector;
#[cfg(feature = "exchange-deribit")]
use crate::l3::open::crypto::cex::deribit::DeribitConnector;
#[cfg(feature = "exchange-hyperliquid")]
use crate::l3::open::crypto::dex::hyperliquid::HyperliquidConnector;
#[cfg(feature = "exchange-kraken")]
use crate::l3::open::crypto::cex::kraken::KrakenConnector;
#[cfg(feature = "exchange-bitstamp")]
use crate::l3::open::crypto::cex::bitstamp::BitstampConnector;
#[cfg(feature = "exchange-bingx")]
use crate::l3::open::crypto::cex::bingx::BingxConnector;
#[cfg(feature = "exchange-bitget")]
use crate::l3::open::crypto::cex::bitget::BitgetConnector;

// ── DEX ─────────────────────────────────────────────────────────────────────

#[cfg(feature = "exchange-lighter")]
use crate::l3::open::crypto::dex::lighter::LighterConnector;
#[cfg(feature = "exchange-dydx")]
use crate::l3::open::crypto::dex::dydx::DydxConnector;

// ── Stocks US ────────────────────────────────────────────────────────────────

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

// ── Stocks India ─────────────────────────────────────────────────────────────

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

// ── Stocks Other ─────────────────────────────────────────────────────────────

#[cfg(feature = "exchange-jquants")]
use crate::l1::paid::jquants::JQuantsConnector;
#[cfg(feature = "exchange-krx")]
use crate::l1::free::krx::KrxConnector;
#[cfg(feature = "exchange-moex")]
use crate::l2::free::moex::MoexConnector;
#[cfg(feature = "exchange-tinkoff")]
use crate::l3::gated::stocks::russia::tinkoff::TinkoffConnector;

// ── Forex ────────────────────────────────────────────────────────────────────

#[cfg(feature = "exchange-oanda")]
use crate::l3::gated::forex::oanda::OandaConnector;
#[cfg(feature = "exchange-dukascopy")]
use crate::l3::gated::forex::dukascopy::DukascopyConnector;
#[cfg(feature = "exchange-alphavantage")]
use crate::l1::paid::alphavantage::AlphaVantageConnector;

// ── Prediction ───────────────────────────────────────────────────────────────

#[cfg(feature = "exchange-polymarket")]
use crate::l3::open::prediction::polymarket::PolymarketConnector;

// ── Brokers ──────────────────────────────────────────────────────────────────

// ── Data Feeds ───────────────────────────────────────────────────────────────

#[cfg(feature = "exchange-yahoo")]
use crate::l1::free::yahoo::YahooFinanceConnector;
#[cfg(feature = "exchange-cryptocompare")]
use crate::l2::paid::cryptocompare::CryptoCompareConnector;

// ═══════════════════════════════════════════════════════════════════════════════
// CancelAll stubs
// Real impls: Alpaca, Binance, BingX, Bitfinex, Bitget, Bitstamp, Bybit,
//             Coinbase, CryptoCom, Deribit, GateIO, Gemini, HTX, HyperLiquid,
//             Kraken, KuCoin, MEXC, OKX, Upbit, Upstox
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(feature = "exchange-lighter")]
impl CancelAll for LighterConnector {}
#[cfg(feature = "exchange-dydx")]
impl CancelAll for DydxConnector {}
#[cfg(feature = "exchange-polygon")]
impl CancelAll for PolygonConnector {}
#[cfg(feature = "exchange-finnhub")]
impl CancelAll for FinnhubConnector {}
#[cfg(feature = "exchange-tiingo")]
impl CancelAll for TiingoConnector {}
#[cfg(feature = "exchange-twelvedata")]
impl CancelAll for TwelvedataConnector {}
#[cfg(feature = "exchange-angel-one")]
impl CancelAll for AngelOneConnector {}
#[cfg(feature = "exchange-zerodha")]
impl CancelAll for ZerodhaConnector {}
#[cfg(feature = "exchange-dhan")]
impl CancelAll for DhanConnector {}
#[cfg(feature = "exchange-fyers")]
impl CancelAll for FyersConnector {}
#[cfg(feature = "exchange-jquants")]
impl CancelAll for JQuantsConnector {}
#[cfg(feature = "exchange-krx")]
impl CancelAll for KrxConnector {}
#[cfg(feature = "exchange-moex")]
impl CancelAll for MoexConnector {}
#[cfg(feature = "exchange-tinkoff")]
impl CancelAll for TinkoffConnector {}
#[cfg(feature = "exchange-oanda")]
impl CancelAll for OandaConnector {}
#[cfg(feature = "exchange-dukascopy")]
impl CancelAll for DukascopyConnector {}
#[cfg(feature = "exchange-alphavantage")]
impl CancelAll for AlphaVantageConnector {}
#[cfg(feature = "exchange-polymarket")]
impl CancelAll for PolymarketConnector {}
#[cfg(feature = "exchange-yahoo")]
impl CancelAll for YahooFinanceConnector {}
#[cfg(feature = "exchange-cryptocompare")]
impl CancelAll for CryptoCompareConnector {}

// ═══════════════════════════════════════════════════════════════════════════════
// AmendOrder stubs
// Real impls: Alpaca, AngelOne, Binance, BingX, Bitfinex, Bitget, Bitstamp,
//             Bybit, CryptoCom, Deribit, Dhan, Fyers, GateIO, HyperLiquid,
//             Kraken, KuCoin, Oanda, OKX, Tinkoff, Upbit, Upstox, Zerodha
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(feature = "exchange-coinbase")]
impl AmendOrder for CoinbaseConnector {}
#[cfg(feature = "exchange-gemini")]
impl AmendOrder for GeminiConnector {}
#[cfg(feature = "exchange-mexc")]
impl AmendOrder for MexcConnector {}
#[cfg(feature = "exchange-htx")]
impl AmendOrder for HtxConnector {}
#[cfg(feature = "exchange-lighter")]
impl AmendOrder for LighterConnector {}
#[cfg(feature = "exchange-dydx")]
impl AmendOrder for DydxConnector {}
#[cfg(feature = "exchange-polygon")]
impl AmendOrder for PolygonConnector {}
#[cfg(feature = "exchange-finnhub")]
impl AmendOrder for FinnhubConnector {}
#[cfg(feature = "exchange-tiingo")]
impl AmendOrder for TiingoConnector {}
#[cfg(feature = "exchange-twelvedata")]
impl AmendOrder for TwelvedataConnector {}
#[cfg(feature = "exchange-jquants")]
impl AmendOrder for JQuantsConnector {}
#[cfg(feature = "exchange-krx")]
impl AmendOrder for KrxConnector {}
#[cfg(feature = "exchange-moex")]
impl AmendOrder for MoexConnector {}
#[cfg(feature = "exchange-dukascopy")]
impl AmendOrder for DukascopyConnector {}
#[cfg(feature = "exchange-alphavantage")]
impl AmendOrder for AlphaVantageConnector {}
#[cfg(feature = "exchange-polymarket")]
impl AmendOrder for PolymarketConnector {}
#[cfg(feature = "exchange-yahoo")]
impl AmendOrder for YahooFinanceConnector {}
#[cfg(feature = "exchange-cryptocompare")]
impl AmendOrder for CryptoCompareConnector {}

// ═══════════════════════════════════════════════════════════════════════════════
// BatchOrders stubs
// Real impls: Binance, BingX, Bitfinex, Bitget, Bybit, CryptoCom, Fyers,
//             GateIO, HTX, HyperLiquid, Kraken, KuCoin, MEXC, OKX, Upstox
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(feature = "exchange-coinbase")]
impl BatchOrders for CoinbaseConnector {}
#[cfg(feature = "exchange-bitstamp")]
impl BatchOrders for BitstampConnector {}
#[cfg(feature = "exchange-gemini")]
impl BatchOrders for GeminiConnector {}
#[cfg(feature = "exchange-upbit")]
impl BatchOrders for UpbitConnector {}
#[cfg(feature = "exchange-deribit")]
impl BatchOrders for DeribitConnector {}
#[cfg(feature = "exchange-lighter")]
impl BatchOrders for LighterConnector {}
#[cfg(feature = "exchange-dydx")]
impl BatchOrders for DydxConnector {}
#[cfg(feature = "exchange-polygon")]
impl BatchOrders for PolygonConnector {}
#[cfg(feature = "exchange-finnhub")]
impl BatchOrders for FinnhubConnector {}
#[cfg(feature = "exchange-tiingo")]
impl BatchOrders for TiingoConnector {}
#[cfg(feature = "exchange-twelvedata")]
impl BatchOrders for TwelvedataConnector {}
#[cfg(feature = "exchange-alpaca")]
impl BatchOrders for AlpacaConnector {}
#[cfg(feature = "exchange-angel-one")]
impl BatchOrders for AngelOneConnector {}
#[cfg(feature = "exchange-zerodha")]
impl BatchOrders for ZerodhaConnector {}
#[cfg(feature = "exchange-dhan")]
impl BatchOrders for DhanConnector {}
#[cfg(feature = "exchange-jquants")]
impl BatchOrders for JQuantsConnector {}
#[cfg(feature = "exchange-krx")]
impl BatchOrders for KrxConnector {}
#[cfg(feature = "exchange-moex")]
impl BatchOrders for MoexConnector {}
#[cfg(feature = "exchange-tinkoff")]
impl BatchOrders for TinkoffConnector {}
#[cfg(feature = "exchange-oanda")]
impl BatchOrders for OandaConnector {}
#[cfg(feature = "exchange-dukascopy")]
impl BatchOrders for DukascopyConnector {}
#[cfg(feature = "exchange-alphavantage")]
impl BatchOrders for AlphaVantageConnector {}
#[cfg(feature = "exchange-polymarket")]
impl BatchOrders for PolymarketConnector {}
#[cfg(feature = "exchange-yahoo")]
impl BatchOrders for YahooFinanceConnector {}
#[cfg(feature = "exchange-cryptocompare")]
impl BatchOrders for CryptoCompareConnector {}

// ═══════════════════════════════════════════════════════════════════════════════
// AccountTransfers stubs
// Real impls: Binance, BingX, Bitfinex, Bitget, Bybit, GateIO, HTX,
//             HyperLiquid, KuCoin, MEXC, OKX
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(feature = "exchange-kraken")]
impl AccountTransfers for KrakenConnector {}
#[cfg(feature = "exchange-coinbase")]
impl AccountTransfers for CoinbaseConnector {}
#[cfg(feature = "exchange-bitstamp")]
impl AccountTransfers for BitstampConnector {}
#[cfg(feature = "exchange-gemini")]
impl AccountTransfers for GeminiConnector {}
#[cfg(feature = "exchange-crypto-com")]
impl AccountTransfers for CryptoComConnector {}
#[cfg(feature = "exchange-upbit")]
impl AccountTransfers for UpbitConnector {}
#[cfg(feature = "exchange-deribit")]
impl AccountTransfers for DeribitConnector {}
#[cfg(feature = "exchange-lighter")]
impl AccountTransfers for LighterConnector {}
#[cfg(feature = "exchange-dydx")]
impl AccountTransfers for DydxConnector {}
#[cfg(feature = "exchange-polygon")]
impl AccountTransfers for PolygonConnector {}
#[cfg(feature = "exchange-finnhub")]
impl AccountTransfers for FinnhubConnector {}
#[cfg(feature = "exchange-tiingo")]
impl AccountTransfers for TiingoConnector {}
#[cfg(feature = "exchange-twelvedata")]
impl AccountTransfers for TwelvedataConnector {}
#[cfg(feature = "exchange-alpaca")]
impl AccountTransfers for AlpacaConnector {}
#[cfg(feature = "exchange-angel-one")]
impl AccountTransfers for AngelOneConnector {}
#[cfg(feature = "exchange-zerodha")]
impl AccountTransfers for ZerodhaConnector {}
#[cfg(feature = "exchange-upstox")]
impl AccountTransfers for UpstoxConnector {}
#[cfg(feature = "exchange-dhan")]
impl AccountTransfers for DhanConnector {}
#[cfg(feature = "exchange-fyers")]
impl AccountTransfers for FyersConnector {}
#[cfg(feature = "exchange-jquants")]
impl AccountTransfers for JQuantsConnector {}
#[cfg(feature = "exchange-krx")]
impl AccountTransfers for KrxConnector {}
#[cfg(feature = "exchange-moex")]
impl AccountTransfers for MoexConnector {}
#[cfg(feature = "exchange-tinkoff")]
impl AccountTransfers for TinkoffConnector {}
#[cfg(feature = "exchange-oanda")]
impl AccountTransfers for OandaConnector {}
#[cfg(feature = "exchange-dukascopy")]
impl AccountTransfers for DukascopyConnector {}
#[cfg(feature = "exchange-alphavantage")]
impl AccountTransfers for AlphaVantageConnector {}
#[cfg(feature = "exchange-polymarket")]
impl AccountTransfers for PolymarketConnector {}
#[cfg(feature = "exchange-yahoo")]
impl AccountTransfers for YahooFinanceConnector {}
#[cfg(feature = "exchange-cryptocompare")]
impl AccountTransfers for CryptoCompareConnector {}

// ═══════════════════════════════════════════════════════════════════════════════
// CustodialFunds stubs
// Real impls: Binance, BingX, Bitfinex, Bitget, Bitstamp, Bybit, Coinbase,
//             CryptoCom, Deribit, GateIO, Gemini, HTX, Kraken, KuCoin, MEXC,
//             OKX, Upbit
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(feature = "exchange-hyperliquid")]
impl CustodialFunds for HyperliquidConnector {}
#[cfg(feature = "exchange-lighter")]
impl CustodialFunds for LighterConnector {}
#[cfg(feature = "exchange-dydx")]
impl CustodialFunds for DydxConnector {}
#[cfg(feature = "exchange-polygon")]
impl CustodialFunds for PolygonConnector {}
#[cfg(feature = "exchange-finnhub")]
impl CustodialFunds for FinnhubConnector {}
#[cfg(feature = "exchange-tiingo")]
impl CustodialFunds for TiingoConnector {}
#[cfg(feature = "exchange-twelvedata")]
impl CustodialFunds for TwelvedataConnector {}
#[cfg(feature = "exchange-alpaca")]
impl CustodialFunds for AlpacaConnector {}
#[cfg(feature = "exchange-angel-one")]
impl CustodialFunds for AngelOneConnector {}
#[cfg(feature = "exchange-zerodha")]
impl CustodialFunds for ZerodhaConnector {}
#[cfg(feature = "exchange-upstox")]
impl CustodialFunds for UpstoxConnector {}
#[cfg(feature = "exchange-dhan")]
impl CustodialFunds for DhanConnector {}
#[cfg(feature = "exchange-fyers")]
impl CustodialFunds for FyersConnector {}
#[cfg(feature = "exchange-jquants")]
impl CustodialFunds for JQuantsConnector {}
#[cfg(feature = "exchange-krx")]
impl CustodialFunds for KrxConnector {}
#[cfg(feature = "exchange-moex")]
impl CustodialFunds for MoexConnector {}
#[cfg(feature = "exchange-tinkoff")]
impl CustodialFunds for TinkoffConnector {}
#[cfg(feature = "exchange-oanda")]
impl CustodialFunds for OandaConnector {}
#[cfg(feature = "exchange-dukascopy")]
impl CustodialFunds for DukascopyConnector {}
#[cfg(feature = "exchange-alphavantage")]
impl CustodialFunds for AlphaVantageConnector {}
#[cfg(feature = "exchange-polymarket")]
impl CustodialFunds for PolymarketConnector {}
#[cfg(feature = "exchange-yahoo")]
impl CustodialFunds for YahooFinanceConnector {}
#[cfg(feature = "exchange-cryptocompare")]
impl CustodialFunds for CryptoCompareConnector {}

// ═══════════════════════════════════════════════════════════════════════════════
// SubAccounts stubs
// Real impls: Binance, BingX, Bitfinex, Bitget, Bybit, CryptoCom, GateIO,
//             HTX, Kraken, KuCoin, MEXC, OKX
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(feature = "exchange-coinbase")]
impl SubAccounts for CoinbaseConnector {}
#[cfg(feature = "exchange-bitstamp")]
impl SubAccounts for BitstampConnector {}
#[cfg(feature = "exchange-gemini")]
impl SubAccounts for GeminiConnector {}
#[cfg(feature = "exchange-upbit")]
impl SubAccounts for UpbitConnector {}
#[cfg(feature = "exchange-deribit")]
impl SubAccounts for DeribitConnector {}
#[cfg(feature = "exchange-hyperliquid")]
impl SubAccounts for HyperliquidConnector {}
#[cfg(feature = "exchange-lighter")]
impl SubAccounts for LighterConnector {}
#[cfg(feature = "exchange-dydx")]
impl SubAccounts for DydxConnector {}
#[cfg(feature = "exchange-polygon")]
impl SubAccounts for PolygonConnector {}
#[cfg(feature = "exchange-finnhub")]
impl SubAccounts for FinnhubConnector {}
#[cfg(feature = "exchange-tiingo")]
impl SubAccounts for TiingoConnector {}
#[cfg(feature = "exchange-twelvedata")]
impl SubAccounts for TwelvedataConnector {}
#[cfg(feature = "exchange-alpaca")]
impl SubAccounts for AlpacaConnector {}
#[cfg(feature = "exchange-angel-one")]
impl SubAccounts for AngelOneConnector {}
#[cfg(feature = "exchange-zerodha")]
impl SubAccounts for ZerodhaConnector {}
#[cfg(feature = "exchange-upstox")]
impl SubAccounts for UpstoxConnector {}
#[cfg(feature = "exchange-dhan")]
impl SubAccounts for DhanConnector {}
#[cfg(feature = "exchange-fyers")]
impl SubAccounts for FyersConnector {}
#[cfg(feature = "exchange-jquants")]
impl SubAccounts for JQuantsConnector {}
#[cfg(feature = "exchange-krx")]
impl SubAccounts for KrxConnector {}
#[cfg(feature = "exchange-moex")]
impl SubAccounts for MoexConnector {}
#[cfg(feature = "exchange-tinkoff")]
impl SubAccounts for TinkoffConnector {}
#[cfg(feature = "exchange-oanda")]
impl SubAccounts for OandaConnector {}
#[cfg(feature = "exchange-dukascopy")]
impl SubAccounts for DukascopyConnector {}
#[cfg(feature = "exchange-alphavantage")]
impl SubAccounts for AlphaVantageConnector {}
#[cfg(feature = "exchange-polymarket")]
impl SubAccounts for PolymarketConnector {}
#[cfg(feature = "exchange-yahoo")]
impl SubAccounts for YahooFinanceConnector {}
#[cfg(feature = "exchange-cryptocompare")]
impl SubAccounts for CryptoCompareConnector {}

// ═══════════════════════════════════════════════════════════════════════════════
// FundingHistory stubs
// Real impls: Binance, Bitfinex, Bybit, Deribit, DyDx, GateIO, HyperLiquid,
//             Kraken, KuCoin, OKX
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(feature = "exchange-coinbase")]
impl FundingHistory for CoinbaseConnector {}
#[cfg(feature = "exchange-bitstamp")]
impl FundingHistory for BitstampConnector {}
#[cfg(feature = "exchange-gemini")]
impl FundingHistory for GeminiConnector {}
#[cfg(feature = "exchange-mexc")]
impl FundingHistory for MexcConnector {}
#[cfg(feature = "exchange-htx")]
impl FundingHistory for HtxConnector {}
#[cfg(feature = "exchange-bitget")]
impl FundingHistory for BitgetConnector {}
#[cfg(feature = "exchange-bingx")]
impl FundingHistory for BingxConnector {}
#[cfg(feature = "exchange-crypto-com")]
impl FundingHistory for CryptoComConnector {}
#[cfg(feature = "exchange-upbit")]
impl FundingHistory for UpbitConnector {}
#[cfg(feature = "exchange-lighter")]
impl FundingHistory for LighterConnector {}
#[cfg(feature = "exchange-polygon")]
impl FundingHistory for PolygonConnector {}
#[cfg(feature = "exchange-finnhub")]
impl FundingHistory for FinnhubConnector {}
#[cfg(feature = "exchange-tiingo")]
impl FundingHistory for TiingoConnector {}
#[cfg(feature = "exchange-twelvedata")]
impl FundingHistory for TwelvedataConnector {}
#[cfg(feature = "exchange-alpaca")]
impl FundingHistory for AlpacaConnector {}
#[cfg(feature = "exchange-angel-one")]
impl FundingHistory for AngelOneConnector {}
#[cfg(feature = "exchange-zerodha")]
impl FundingHistory for ZerodhaConnector {}
#[cfg(feature = "exchange-upstox")]
impl FundingHistory for UpstoxConnector {}
#[cfg(feature = "exchange-dhan")]
impl FundingHistory for DhanConnector {}
#[cfg(feature = "exchange-fyers")]
impl FundingHistory for FyersConnector {}
#[cfg(feature = "exchange-jquants")]
impl FundingHistory for JQuantsConnector {}
#[cfg(feature = "exchange-krx")]
impl FundingHistory for KrxConnector {}
#[cfg(feature = "exchange-moex")]
impl FundingHistory for MoexConnector {}
#[cfg(feature = "exchange-tinkoff")]
impl FundingHistory for TinkoffConnector {}
#[cfg(feature = "exchange-oanda")]
impl FundingHistory for OandaConnector {}
#[cfg(feature = "exchange-dukascopy")]
impl FundingHistory for DukascopyConnector {}
#[cfg(feature = "exchange-alphavantage")]
impl FundingHistory for AlphaVantageConnector {}
#[cfg(feature = "exchange-polymarket")]
impl FundingHistory for PolymarketConnector {}
#[cfg(feature = "exchange-yahoo")]
impl FundingHistory for YahooFinanceConnector {}
#[cfg(feature = "exchange-cryptocompare")]
impl FundingHistory for CryptoCompareConnector {}

// ═══════════════════════════════════════════════════════════════════════════════
// AccountLedger stubs
// Real impls: Alpaca, Binance, Bitfinex, Bitget, Bitstamp, Bybit, CryptoCom,
//             Deribit, GateIO, Kraken, KuCoin, OKX
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(feature = "exchange-coinbase")]
impl AccountLedger for CoinbaseConnector {}
#[cfg(feature = "exchange-gemini")]
impl AccountLedger for GeminiConnector {}
#[cfg(feature = "exchange-mexc")]
impl AccountLedger for MexcConnector {}
#[cfg(feature = "exchange-htx")]
impl AccountLedger for HtxConnector {}
#[cfg(feature = "exchange-bingx")]
impl AccountLedger for BingxConnector {}
#[cfg(feature = "exchange-upbit")]
impl AccountLedger for UpbitConnector {}
#[cfg(feature = "exchange-hyperliquid")]
impl AccountLedger for HyperliquidConnector {}
#[cfg(feature = "exchange-lighter")]
impl AccountLedger for LighterConnector {}
#[cfg(feature = "exchange-dydx")]
impl AccountLedger for DydxConnector {}
#[cfg(feature = "exchange-polygon")]
impl AccountLedger for PolygonConnector {}
#[cfg(feature = "exchange-finnhub")]
impl AccountLedger for FinnhubConnector {}
#[cfg(feature = "exchange-tiingo")]
impl AccountLedger for TiingoConnector {}
#[cfg(feature = "exchange-twelvedata")]
impl AccountLedger for TwelvedataConnector {}
#[cfg(feature = "exchange-angel-one")]
impl AccountLedger for AngelOneConnector {}
#[cfg(feature = "exchange-zerodha")]
impl AccountLedger for ZerodhaConnector {}
#[cfg(feature = "exchange-upstox")]
impl AccountLedger for UpstoxConnector {}
#[cfg(feature = "exchange-dhan")]
impl AccountLedger for DhanConnector {}
#[cfg(feature = "exchange-fyers")]
impl AccountLedger for FyersConnector {}
#[cfg(feature = "exchange-jquants")]
impl AccountLedger for JQuantsConnector {}
#[cfg(feature = "exchange-krx")]
impl AccountLedger for KrxConnector {}
#[cfg(feature = "exchange-moex")]
impl AccountLedger for MoexConnector {}
#[cfg(feature = "exchange-tinkoff")]
impl AccountLedger for TinkoffConnector {}
#[cfg(feature = "exchange-oanda")]
impl AccountLedger for OandaConnector {}
#[cfg(feature = "exchange-dukascopy")]
impl AccountLedger for DukascopyConnector {}
#[cfg(feature = "exchange-alphavantage")]
impl AccountLedger for AlphaVantageConnector {}
#[cfg(feature = "exchange-polymarket")]
impl AccountLedger for PolymarketConnector {}
#[cfg(feature = "exchange-yahoo")]
impl AccountLedger for YahooFinanceConnector {}
#[cfg(feature = "exchange-cryptocompare")]
impl AccountLedger for CryptoCompareConnector {}
