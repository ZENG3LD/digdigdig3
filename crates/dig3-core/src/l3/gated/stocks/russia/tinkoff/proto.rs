//! # Tinkoff Invest API — Protobuf Message Types
//!
//! Hand-written prost structs mirroring the canonical proto definitions from:
//!   https://github.com/RussianInvestments/investAPI/tree/main/src/docs/contracts
//!
//! ## Services covered
//! - `MarketDataService`   — GetCandles, GetOrderBook, GetLastPrices, GetTradingStatus
//! - `OrdersService`       — PostOrder, CancelOrder, GetOrders, GetOrderState
//! - `OperationsService`   — GetPortfolio, GetPositions, GetOperations
//! - `InstrumentsService`  — GetInstrumentBy, FindInstrument
//!
//! ## Field numbering source
//! Proto field numbers are taken from the official proto contracts at the URL
//! above.  Only the fields necessary for the implemented connector methods are
//! included — unused optional fields are omitted for brevity.
//!
//! ## Feature gate
//! Everything in this module is compiled only when `features = ["grpc"]`.

#[cfg(feature = "grpc")]
pub mod tinkoff {
    // ─────────────────────────────────────────────────────────────────────────
    // Shared scalar types
    // ─────────────────────────────────────────────────────────────────────────

    /// Tinkoff `Quotation` — fixed-point number with 9 decimal places.
    ///
    /// `value = units + nano / 1_000_000_000`
    ///
    /// Proto path: `tinkoff.public.invest.api.contract.v1.Quotation`
    /// - field 1: units (int64)
    /// - field 2: nano  (int32)
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Quotation {
        /// Integer part of the value.
        #[prost(int64, tag = "1")]
        pub units: i64,

        /// Fractional part in nano-units (1/1_000_000_000).
        /// Range: -999_999_999 to 999_999_999.
        #[prost(int32, tag = "2")]
        pub nano: i32,
    }

    /// Tinkoff `MoneyValue` — amount in a specific currency.
    ///
    /// Proto path: `tinkoff.public.invest.api.contract.v1.MoneyValue`
    /// - field 1: currency (string) — ISO 4217, e.g. "rub"
    /// - field 2: units    (int64)
    /// - field 3: nano     (int32)
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct MoneyValue {
        /// ISO 4217 currency code in lower-case (e.g. `"rub"`, `"usd"`).
        #[prost(string, tag = "1")]
        pub currency: ::prost::alloc::string::String,

        /// Integer part.
        #[prost(int64, tag = "2")]
        pub units: i64,

        /// Nano part.
        #[prost(int32, tag = "3")]
        pub nano: i32,
    }

    /// Tinkoff `Timestamp` — mirrors `google.protobuf.Timestamp`.
    ///
    /// Proto path: `google.protobuf.Timestamp`
    /// - field 1: seconds (int64) — UTC epoch seconds
    /// - field 2: nanos   (int32) — nanoseconds
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Timestamp {
        /// Seconds of UTC time since Unix epoch.
        #[prost(int64, tag = "1")]
        pub seconds: i64,

        /// Non-negative fractions of a second.
        #[prost(int32, tag = "2")]
        pub nanos: i32,
    }

    // ─────────────────────────────────────────────────────────────────────────
    // CandleInterval enum
    // ─────────────────────────────────────────────────────────────────────────

    /// Historical candle interval.
    ///
    /// Proto path: `tinkoff.public.invest.api.contract.v1.CandleInterval`
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum CandleInterval {
        /// Unspecified (do not use).
        Unspecified = 0,
        /// 1 minute.
        Min1 = 1,
        /// 5 minutes.
        Min5 = 2,
        /// 15 minutes.
        Min15 = 3,
        /// 1 hour.
        Hour = 4,
        /// 1 day.
        Day = 5,
        /// 2 minutes.
        Min2 = 6,
        /// 3 minutes.
        Min3 = 7,
        /// 10 minutes.
        Min10 = 8,
        /// 30 minutes.
        Min30 = 9,
        /// 2 hours.
        Hour2 = 10,
        /// 4 hours.
        Hour4 = 11,
        /// 1 week.
        Week = 12,
        /// 1 month.
        Month = 13,
        /// 5 seconds.
        Sec5 = 14,
        /// 10 seconds.
        Sec10 = 15,
        /// 30 seconds.
        Sec30 = 16,
    }

    // ─────────────────────────────────────────────────────────────────────────
    // OrderDirection / OrderType enums
    // ─────────────────────────────────────────────────────────────────────────

    /// Order direction.
    ///
    /// Proto path: `tinkoff.public.invest.api.contract.v1.OrderDirection`
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum OrderDirection {
        /// Unspecified (do not use).
        Unspecified = 0,
        /// Buy.
        Buy = 1,
        /// Sell.
        Sell = 2,
    }

    /// Order type.
    ///
    /// Proto path: `tinkoff.public.invest.api.contract.v1.OrderType`
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum OrderType {
        /// Unspecified (do not use).
        Unspecified = 0,
        /// Limit order.
        Limit = 1,
        /// Market order.
        Market = 2,
        /// Best-price order.
        Bestprice = 3,
    }

    // ─────────────────────────────────────────────────────────────────────────
    // OrderExecutionReportStatus enum
    // ─────────────────────────────────────────────────────────────────────────

    /// Order execution status.
    ///
    /// Proto path: `tinkoff.public.invest.api.contract.v1.OrderExecutionReportStatus`
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum OrderExecutionReportStatus {
        /// Unspecified.
        ExecutionReportStatusUnspecified = 0,
        /// Filled.
        ExecutionReportStatusFill = 1,
        /// Rejected.
        ExecutionReportStatusRejected = 2,
        /// Cancelled.
        ExecutionReportStatusCancelled = 3,
        /// New / active.
        ExecutionReportStatusNew = 4,
        /// Partially filled.
        ExecutionReportStatusPartiallyfill = 5,
    }

    // ─────────────────────────────────────────────────────────────────────────
    // TradingStatus enum
    // ─────────────────────────────────────────────────────────────────────────

    /// Instrument trading status.
    ///
    /// Proto path: `tinkoff.public.invest.api.contract.v1.SecurityTradingStatus`
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum SecurityTradingStatus {
        Unspecified = 0,
        NormalTrading = 1,
        NotAvailableForTrading = 2,
        ClosingAuction = 3,
        ClosingPeriod = 4,
        BreakInTrading = 5,
        DealerNormalTrading = 6,
        DealerBreakInTrading = 7,
        DealerNotAvailableForTrading = 8,
        OpeningPeriod = 17,
    }

    // ─────────────────────────────────────────────────────────────────────────
    // InstrumentIdType enum
    // ─────────────────────────────────────────────────────────────────────────

    /// Identifier type for instrument lookup.
    ///
    /// Proto path: `tinkoff.public.invest.api.contract.v1.InstrumentIdType`
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum InstrumentIdType {
        /// Unspecified.
        Unspecified = 0,
        /// FIGI.
        Figi = 1,
        /// Ticker + class_code.
        Ticker = 2,
        /// UID.
        Uid = 3,
        /// Position UID.
        PositionUid = 4,
    }

    // ─────────────────────────────────────────────────────────────────────────
    // HistoricCandle
    // ─────────────────────────────────────────────────────────────────────────

    /// A single OHLCV candle returned by GetCandles.
    ///
    /// Proto path: `tinkoff.public.invest.api.contract.v1.HistoricCandle`
    /// - field 1: open       (Quotation)
    /// - field 2: high       (Quotation)
    /// - field 3: low        (Quotation)
    /// - field 4: close      (Quotation)
    /// - field 5: volume     (int64)
    /// - field 6: time       (Timestamp)
    /// - field 7: is_complete (bool)
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct HistoricCandle {
        /// Opening price.
        #[prost(message, optional, tag = "1")]
        pub open: ::core::option::Option<Quotation>,

        /// High price.
        #[prost(message, optional, tag = "2")]
        pub high: ::core::option::Option<Quotation>,

        /// Low price.
        #[prost(message, optional, tag = "3")]
        pub low: ::core::option::Option<Quotation>,

        /// Closing price.
        #[prost(message, optional, tag = "4")]
        pub close: ::core::option::Option<Quotation>,

        /// Trading volume in lots.
        #[prost(int64, tag = "5")]
        pub volume: i64,

        /// Candle open time (UTC).
        #[prost(message, optional, tag = "6")]
        pub time: ::core::option::Option<Timestamp>,

        /// `true` if the candle is complete (closed).
        #[prost(bool, tag = "7")]
        pub is_complete: bool,
    }

    // ─────────────────────────────────────────────────────────────────────────
    // MarketDataService — GetCandles
    // ─────────────────────────────────────────────────────────────────────────

    /// Request for GetCandles.
    ///
    /// gRPC path: `/tinkoff.public.invest.api.contract.v1.MarketDataService/GetCandles`
    ///
    /// Proto path: `tinkoff.public.invest.api.contract.v1.GetCandlesRequest`
    /// - field 1: figi          (string, deprecated — prefer instrument_id)
    /// - field 2: from          (Timestamp)
    /// - field 3: to            (Timestamp)
    /// - field 4: interval      (CandleInterval)
    /// - field 10: instrument_id (string)
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct GetCandlesRequest {
        /// FIGI of the instrument (deprecated; prefer `instrument_id`).
        #[prost(string, tag = "1")]
        pub figi: ::prost::alloc::string::String,

        /// Start of the requested range (UTC).
        #[prost(message, optional, tag = "2")]
        pub from: ::core::option::Option<Timestamp>,

        /// End of the requested range (UTC).
        #[prost(message, optional, tag = "3")]
        pub to: ::core::option::Option<Timestamp>,

        /// Candle interval.
        #[prost(enumeration = "CandleInterval", tag = "4")]
        pub interval: i32,

        /// Instrument UID (preferred over `figi` for non-share instruments).
        #[prost(string, tag = "10")]
        pub instrument_id: ::prost::alloc::string::String,
    }

    /// Response from GetCandles.
    ///
    /// Proto path: `tinkoff.public.invest.api.contract.v1.GetCandlesResponse`
    /// - field 1: candles (repeated HistoricCandle)
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct GetCandlesResponse {
        /// Candles, ordered by time ascending.
        #[prost(message, repeated, tag = "1")]
        pub candles: ::prost::alloc::vec::Vec<HistoricCandle>,
    }

    // ─────────────────────────────────────────────────────────────────────────
    // MarketDataService — GetOrderBook
    // ─────────────────────────────────────────────────────────────────────────

    /// A single bid or ask level in the order book.
    ///
    /// Proto path: `tinkoff.public.invest.api.contract.v1.Order` (order book level)
    /// - field 1: price    (Quotation)
    /// - field 2: quantity (int64)
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct OrderBookLevel {
        /// Price at this level.
        #[prost(message, optional, tag = "1")]
        pub price: ::core::option::Option<Quotation>,

        /// Quantity available at this level (in lots).
        #[prost(int64, tag = "2")]
        pub quantity: i64,
    }

    /// Request for GetOrderBook.
    ///
    /// gRPC path: `/tinkoff.public.invest.api.contract.v1.MarketDataService/GetOrderBook`
    ///
    /// Proto path: `tinkoff.public.invest.api.contract.v1.GetOrderBookRequest`
    /// - field 1: figi          (string)
    /// - field 2: depth         (int32)
    /// - field 3: instrument_id (string)
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct GetOrderBookRequest {
        /// FIGI of the instrument.
        #[prost(string, tag = "1")]
        pub figi: ::prost::alloc::string::String,

        /// Depth: 1, 10, 20, 30, 40, or 50.
        #[prost(int32, tag = "2")]
        pub depth: i32,

        /// Instrument UID (alternative to `figi`).
        #[prost(string, tag = "3")]
        pub instrument_id: ::prost::alloc::string::String,
    }

    /// Response from GetOrderBook.
    ///
    /// Proto path: `tinkoff.public.invest.api.contract.v1.GetOrderBookResponse`
    /// - field 1: figi               (string)
    /// - field 2: depth              (int32)
    /// - field 3: bids               (repeated OrderBookLevel)
    /// - field 4: asks               (repeated OrderBookLevel)
    /// - field 5: last_price         (Quotation)
    /// - field 6: close_price        (Quotation)
    /// - field 7: limit_up           (Quotation)
    /// - field 8: limit_down         (Quotation)
    /// - field 9: last_price_ts      (Timestamp)
    /// - field 10: close_price_ts    (Timestamp)
    /// - field 11: orderbook_ts      (Timestamp)
    /// - field 12: instrument_uid    (string)
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct GetOrderBookResponse {
        /// FIGI of the instrument.
        #[prost(string, tag = "1")]
        pub figi: ::prost::alloc::string::String,

        /// Depth of the order book snapshot.
        #[prost(int32, tag = "2")]
        pub depth: i32,

        /// Bid levels (buy orders), best price first.
        #[prost(message, repeated, tag = "3")]
        pub bids: ::prost::alloc::vec::Vec<OrderBookLevel>,

        /// Ask levels (sell orders), best price first.
        #[prost(message, repeated, tag = "4")]
        pub asks: ::prost::alloc::vec::Vec<OrderBookLevel>,

        /// Last trade price.
        #[prost(message, optional, tag = "5")]
        pub last_price: ::core::option::Option<Quotation>,

        /// Session closing price.
        #[prost(message, optional, tag = "6")]
        pub close_price: ::core::option::Option<Quotation>,

        /// Instrument UID.
        #[prost(string, tag = "12")]
        pub instrument_uid: ::prost::alloc::string::String,
    }

    // ─────────────────────────────────────────────────────────────────────────
    // MarketDataService — GetLastPrices
    // ─────────────────────────────────────────────────────────────────────────

    /// Request for GetLastPrices.
    ///
    /// gRPC path: `/tinkoff.public.invest.api.contract.v1.MarketDataService/GetLastPrices`
    ///
    /// Proto path: `tinkoff.public.invest.api.contract.v1.GetLastPricesRequest`
    /// - field 1: figi           (repeated string)
    /// - field 2: instrument_id  (repeated string)
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct GetLastPricesRequest {
        /// List of FIGIs to query (deprecated; prefer `instrument_id`).
        #[prost(string, repeated, tag = "1")]
        pub figi: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,

        /// List of instrument UIDs to query.
        #[prost(string, repeated, tag = "2")]
        pub instrument_id: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    }

    /// A single last-price entry.
    ///
    /// Proto path: `tinkoff.public.invest.api.contract.v1.LastPrice`
    /// - field 1: figi           (string)
    /// - field 2: price          (Quotation)
    /// - field 3: time           (Timestamp)
    /// - field 4: instrument_uid (string)
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct LastPrice {
        /// FIGI of the instrument.
        #[prost(string, tag = "1")]
        pub figi: ::prost::alloc::string::String,

        /// Last trade price.
        #[prost(message, optional, tag = "2")]
        pub price: ::core::option::Option<Quotation>,

        /// Timestamp of the last trade.
        #[prost(message, optional, tag = "3")]
        pub time: ::core::option::Option<Timestamp>,

        /// Instrument UID.
        #[prost(string, tag = "4")]
        pub instrument_uid: ::prost::alloc::string::String,
    }

    /// Response from GetLastPrices.
    ///
    /// Proto path: `tinkoff.public.invest.api.contract.v1.GetLastPricesResponse`
    /// - field 1: last_prices (repeated LastPrice)
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct GetLastPricesResponse {
        /// Last prices for each requested instrument.
        #[prost(message, repeated, tag = "1")]
        pub last_prices: ::prost::alloc::vec::Vec<LastPrice>,
    }

    // ─────────────────────────────────────────────────────────────────────────
    // MarketDataService — GetTradingStatus
    // ─────────────────────────────────────────────────────────────────────────

    /// Request for GetTradingStatus.
    ///
    /// gRPC path: `/tinkoff.public.invest.api.contract.v1.MarketDataService/GetTradingStatus`
    ///
    /// Proto path: `tinkoff.public.invest.api.contract.v1.GetTradingStatusRequest`
    /// - field 1: figi          (string)
    /// - field 2: instrument_id (string)
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct GetTradingStatusRequest {
        /// FIGI of the instrument.
        #[prost(string, tag = "1")]
        pub figi: ::prost::alloc::string::String,

        /// Instrument UID.
        #[prost(string, tag = "2")]
        pub instrument_id: ::prost::alloc::string::String,
    }

    /// Response from GetTradingStatus.
    ///
    /// Proto path: `tinkoff.public.invest.api.contract.v1.GetTradingStatusResponse`
    /// - field 1: figi                     (string)
    /// - field 2: trading_status           (SecurityTradingStatus)
    /// - field 3: limit_order_available    (bool)
    /// - field 4: market_order_available   (bool)
    /// - field 5: api_trade_available_flag (bool)
    /// - field 6: instrument_uid           (string)
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct GetTradingStatusResponse {
        /// FIGI of the instrument.
        #[prost(string, tag = "1")]
        pub figi: ::prost::alloc::string::String,

        /// Current trading status.
        #[prost(enumeration = "SecurityTradingStatus", tag = "2")]
        pub trading_status: i32,

        /// Whether limit orders are currently accepted.
        #[prost(bool, tag = "3")]
        pub limit_order_available_flag: bool,

        /// Whether market orders are currently accepted.
        #[prost(bool, tag = "4")]
        pub market_order_available_flag: bool,

        /// Whether API trading is enabled for this instrument.
        #[prost(bool, tag = "5")]
        pub api_trade_available_flag: bool,

        /// Instrument UID.
        #[prost(string, tag = "6")]
        pub instrument_uid: ::prost::alloc::string::String,
    }

    // ─────────────────────────────────────────────────────────────────────────
    // OrdersService — PostOrder
    // ─────────────────────────────────────────────────────────────────────────

    /// Request for PostOrder.
    ///
    /// gRPC path: `/tinkoff.public.invest.api.contract.v1.OrdersService/PostOrder`
    ///
    /// Proto path: `tinkoff.public.invest.api.contract.v1.PostOrderRequest`
    /// - field 1:  figi          (string)
    /// - field 2:  quantity      (int64) — number of lots
    /// - field 3:  price         (Quotation) — limit price (omit for market)
    /// - field 4:  direction     (OrderDirection)
    /// - field 5:  account_id    (string)
    /// - field 6:  order_type    (OrderType)
    /// - field 7:  order_id      (string) — idempotency key
    /// - field 11: instrument_id (string)
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct PostOrderRequest {
        /// FIGI of the instrument (deprecated; prefer `instrument_id`).
        #[prost(string, tag = "1")]
        pub figi: ::prost::alloc::string::String,

        /// Number of lots to buy/sell.
        #[prost(int64, tag = "2")]
        pub quantity: i64,

        /// Limit price (omit or zero for market orders).
        #[prost(message, optional, tag = "3")]
        pub price: ::core::option::Option<Quotation>,

        /// Buy or sell.
        #[prost(enumeration = "OrderDirection", tag = "4")]
        pub direction: i32,

        /// Account ID (from GetAccounts).
        #[prost(string, tag = "5")]
        pub account_id: ::prost::alloc::string::String,

        /// Limit or market order.
        #[prost(enumeration = "OrderType", tag = "6")]
        pub order_type: i32,

        /// Client-supplied idempotency key (UUID recommended).
        #[prost(string, tag = "7")]
        pub order_id: ::prost::alloc::string::String,

        /// Instrument UID (preferred over `figi`).
        #[prost(string, tag = "11")]
        pub instrument_id: ::prost::alloc::string::String,
    }

    /// Response from PostOrder.
    ///
    /// Proto path: `tinkoff.public.invest.api.contract.v1.PostOrderResponse`
    /// - field 1:  order_id              (string) — server-assigned order ID
    /// - field 2:  execution_report_status (OrderExecutionReportStatus)
    /// - field 3:  lots_requested        (int64)
    /// - field 4:  lots_executed         (int64)
    /// - field 5:  initial_order_price   (MoneyValue)
    /// - field 6:  executed_order_price  (MoneyValue)
    /// - field 7:  total_order_amount    (MoneyValue)
    /// - field 11: figi                  (string)
    /// - field 12: direction             (OrderDirection)
    /// - field 13: initial_security_price (MoneyValue)
    /// - field 14: order_type            (OrderType)
    /// - field 15: message               (string)
    /// - field 16: initial_order_price_pt (Quotation)
    /// - field 17: instrument_uid        (string)
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct PostOrderResponse {
        /// Server-assigned order ID.
        #[prost(string, tag = "1")]
        pub order_id: ::prost::alloc::string::String,

        /// Execution status.
        #[prost(enumeration = "OrderExecutionReportStatus", tag = "2")]
        pub execution_report_status: i32,

        /// Number of lots requested.
        #[prost(int64, tag = "3")]
        pub lots_requested: i64,

        /// Number of lots executed.
        #[prost(int64, tag = "4")]
        pub lots_executed: i64,

        /// Total executed value.
        #[prost(message, optional, tag = "7")]
        pub total_order_amount: ::core::option::Option<MoneyValue>,

        /// FIGI of the placed instrument.
        #[prost(string, tag = "11")]
        pub figi: ::prost::alloc::string::String,

        /// Direction (buy/sell).
        #[prost(enumeration = "OrderDirection", tag = "12")]
        pub direction: i32,

        /// Order type placed.
        #[prost(enumeration = "OrderType", tag = "14")]
        pub order_type: i32,

        /// Human-readable status message (present on error/reject).
        #[prost(string, tag = "15")]
        pub message: ::prost::alloc::string::String,

        /// Instrument UID.
        #[prost(string, tag = "17")]
        pub instrument_uid: ::prost::alloc::string::String,
    }

    // ─────────────────────────────────────────────────────────────────────────
    // OrdersService — CancelOrder
    // ─────────────────────────────────────────────────────────────────────────

    /// Request for CancelOrder.
    ///
    /// gRPC path: `/tinkoff.public.invest.api.contract.v1.OrdersService/CancelOrder`
    ///
    /// Proto path: `tinkoff.public.invest.api.contract.v1.CancelOrderRequest`
    /// - field 1: account_id (string)
    /// - field 2: order_id   (string)
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct CancelOrderRequest {
        /// Account ID.
        #[prost(string, tag = "1")]
        pub account_id: ::prost::alloc::string::String,

        /// Order ID to cancel.
        #[prost(string, tag = "2")]
        pub order_id: ::prost::alloc::string::String,
    }

    /// Response from CancelOrder.
    ///
    /// Proto path: `tinkoff.public.invest.api.contract.v1.CancelOrderResponse`
    /// - field 1: time (Timestamp) — time at which the order was cancelled
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct CancelOrderResponse {
        /// Time at which the order was cancelled.
        #[prost(message, optional, tag = "1")]
        pub time: ::core::option::Option<Timestamp>,
    }

    // ─────────────────────────────────────────────────────────────────────────
    // OrdersService — GetOrders
    // ─────────────────────────────────────────────────────────────────────────

    /// Request for GetOrders.
    ///
    /// gRPC path: `/tinkoff.public.invest.api.contract.v1.OrdersService/GetOrders`
    ///
    /// Proto path: `tinkoff.public.invest.api.contract.v1.GetOrdersRequest`
    /// - field 1: account_id (string)
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct GetOrdersRequest {
        /// Account ID.
        #[prost(string, tag = "1")]
        pub account_id: ::prost::alloc::string::String,
    }

    /// A single active order in the GetOrders response.
    ///
    /// Proto path: `tinkoff.public.invest.api.contract.v1.OrderState`
    /// - field 1:  order_id                  (string)
    /// - field 2:  execution_report_status   (OrderExecutionReportStatus)
    /// - field 3:  lots_requested            (int64)
    /// - field 4:  lots_executed             (int64)
    /// - field 5:  initial_order_price       (MoneyValue)
    /// - field 6:  executed_order_price      (MoneyValue)
    /// - field 7:  total_order_amount        (MoneyValue)
    /// - field 8:  average_position_price    (MoneyValue)
    /// - field 9:  stages                   (repeated OrderStage)
    /// - field 10: service_commission        (MoneyValue)
    /// - field 11: currency                  (string)
    /// - field 12: order_type               (OrderType)
    /// - field 13: order_date               (Timestamp)
    /// - field 14: instrument_uid           (string)
    /// - field 15: order_request_id         (string)
    /// - field 16: figi                     (string)
    /// - field 17: direction                (OrderDirection)
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct OrderState {
        /// Server-assigned order ID.
        #[prost(string, tag = "1")]
        pub order_id: ::prost::alloc::string::String,

        /// Current execution status.
        #[prost(enumeration = "OrderExecutionReportStatus", tag = "2")]
        pub execution_report_status: i32,

        /// Number of lots requested.
        #[prost(int64, tag = "3")]
        pub lots_requested: i64,

        /// Number of lots executed.
        #[prost(int64, tag = "4")]
        pub lots_executed: i64,

        /// Order type.
        #[prost(enumeration = "OrderType", tag = "12")]
        pub order_type: i32,

        /// Order placement time.
        #[prost(message, optional, tag = "13")]
        pub order_date: ::core::option::Option<Timestamp>,

        /// Instrument UID.
        #[prost(string, tag = "14")]
        pub instrument_uid: ::prost::alloc::string::String,

        /// FIGI.
        #[prost(string, tag = "16")]
        pub figi: ::prost::alloc::string::String,

        /// Direction (buy/sell).
        #[prost(enumeration = "OrderDirection", tag = "17")]
        pub direction: i32,
    }

    /// Response from GetOrders.
    ///
    /// Proto path: `tinkoff.public.invest.api.contract.v1.GetOrdersResponse`
    /// - field 1: orders (repeated OrderState)
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct GetOrdersResponse {
        /// Active orders.
        #[prost(message, repeated, tag = "1")]
        pub orders: ::prost::alloc::vec::Vec<OrderState>,
    }

    // ─────────────────────────────────────────────────────────────────────────
    // OrdersService — GetOrderState
    // ─────────────────────────────────────────────────────────────────────────

    /// Request for GetOrderState.
    ///
    /// gRPC path: `/tinkoff.public.invest.api.contract.v1.OrdersService/GetOrderState`
    ///
    /// Proto path: `tinkoff.public.invest.api.contract.v1.GetOrderStateRequest`
    /// - field 1: account_id (string)
    /// - field 2: order_id   (string)
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct GetOrderStateRequest {
        /// Account ID.
        #[prost(string, tag = "1")]
        pub account_id: ::prost::alloc::string::String,

        /// Order ID.
        #[prost(string, tag = "2")]
        pub order_id: ::prost::alloc::string::String,
    }

    // ─────────────────────────────────────────────────────────────────────────
    // OperationsService — GetPortfolio
    // ─────────────────────────────────────────────────────────────────────────

    /// Request for GetPortfolio.
    ///
    /// gRPC path: `/tinkoff.public.invest.api.contract.v1.OperationsService/GetPortfolio`
    ///
    /// Proto path: `tinkoff.public.invest.api.contract.v1.PortfolioRequest`
    /// - field 1: account_id (string)
    /// - field 2: currency   (PortfolioRequest_CurrencyRequest)
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct PortfolioRequest {
        /// Account ID.
        #[prost(string, tag = "1")]
        pub account_id: ::prost::alloc::string::String,
    }

    /// A single position in the portfolio.
    ///
    /// Proto path: `tinkoff.public.invest.api.contract.v1.PortfolioPosition`
    /// - field 1:  figi                     (string)
    /// - field 2:  instrument_type          (string)
    /// - field 3:  quantity                 (Quotation)
    /// - field 4:  average_position_price   (MoneyValue)
    /// - field 5:  expected_yield           (Quotation)
    /// - field 6:  current_nkd              (MoneyValue) — accrued coupon
    /// - field 7:  average_position_price_pt (Quotation) — in points
    /// - field 8:  current_price            (MoneyValue)
    /// - field 9:  average_position_price_fifo (MoneyValue)
    /// - field 10: quantity_lots            (Quotation)
    /// - field 11: blocked                  (bool)
    /// - field 12: blocked_lots             (Quotation)
    /// - field 13: position_uid             (string)
    /// - field 14: instrument_uid           (string)
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct PortfolioPosition {
        /// FIGI.
        #[prost(string, tag = "1")]
        pub figi: ::prost::alloc::string::String,

        /// Instrument type: "share", "bond", "etf", "currency", "futures", "option".
        #[prost(string, tag = "2")]
        pub instrument_type: ::prost::alloc::string::String,

        /// Quantity in units (may be fractional for currencies).
        #[prost(message, optional, tag = "3")]
        pub quantity: ::core::option::Option<Quotation>,

        /// Average position price (weighted average cost).
        #[prost(message, optional, tag = "4")]
        pub average_position_price: ::core::option::Option<MoneyValue>,

        /// Unrealised P&L in percent (expected_yield / average_position_price * 100).
        #[prost(message, optional, tag = "5")]
        pub expected_yield: ::core::option::Option<Quotation>,

        /// Current market price.
        #[prost(message, optional, tag = "8")]
        pub current_price: ::core::option::Option<MoneyValue>,

        /// Quantity in lots.
        #[prost(message, optional, tag = "10")]
        pub quantity_lots: ::core::option::Option<Quotation>,

        /// Instrument UID.
        #[prost(string, tag = "14")]
        pub instrument_uid: ::prost::alloc::string::String,
    }

    /// Response from GetPortfolio.
    ///
    /// Proto path: `tinkoff.public.invest.api.contract.v1.PortfolioResponse`
    /// - field 1:  total_amount_shares      (MoneyValue)
    /// - field 2:  total_amount_bonds       (MoneyValue)
    /// - field 3:  total_amount_etf         (MoneyValue)
    /// - field 4:  total_amount_currencies  (MoneyValue)
    /// - field 5:  total_amount_futures     (MoneyValue)
    /// - field 6:  expected_yield           (Quotation)
    /// - field 7:  positions                (repeated PortfolioPosition)
    /// - field 8:  account_id              (string)
    /// - field 9:  total_amount_options     (MoneyValue)
    /// - field 10: total_amount_sp          (MoneyValue)
    /// - field 11: total_amount_portfolio   (MoneyValue)
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct PortfolioResponse {
        /// Total value of stock positions.
        #[prost(message, optional, tag = "1")]
        pub total_amount_shares: ::core::option::Option<MoneyValue>,

        /// Total value of bond positions.
        #[prost(message, optional, tag = "2")]
        pub total_amount_bonds: ::core::option::Option<MoneyValue>,

        /// Total value of ETF positions.
        #[prost(message, optional, tag = "3")]
        pub total_amount_etf: ::core::option::Option<MoneyValue>,

        /// Total value of currency positions.
        #[prost(message, optional, tag = "4")]
        pub total_amount_currencies: ::core::option::Option<MoneyValue>,

        /// Total value of futures positions.
        #[prost(message, optional, tag = "5")]
        pub total_amount_futures: ::core::option::Option<MoneyValue>,

        /// Overall portfolio expected yield (percent).
        #[prost(message, optional, tag = "6")]
        pub expected_yield: ::core::option::Option<Quotation>,

        /// Individual position entries.
        #[prost(message, repeated, tag = "7")]
        pub positions: ::prost::alloc::vec::Vec<PortfolioPosition>,

        /// Account ID this portfolio belongs to.
        #[prost(string, tag = "8")]
        pub account_id: ::prost::alloc::string::String,

        /// Total portfolio value.
        #[prost(message, optional, tag = "11")]
        pub total_amount_portfolio: ::core::option::Option<MoneyValue>,
    }

    // ─────────────────────────────────────────────────────────────────────────
    // OperationsService — GetPositions
    // ─────────────────────────────────────────────────────────────────────────

    /// Request for GetPositions.
    ///
    /// gRPC path: `/tinkoff.public.invest.api.contract.v1.OperationsService/GetPositions`
    ///
    /// Proto path: `tinkoff.public.invest.api.contract.v1.PositionsRequest`
    /// - field 1: account_id (string)
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct PositionsRequest {
        /// Account ID.
        #[prost(string, tag = "1")]
        pub account_id: ::prost::alloc::string::String,
    }

    /// A securities (stock/bond/ETF) position entry.
    ///
    /// Proto path: `tinkoff.public.invest.api.contract.v1.PositionsSecurities`
    /// - field 1: figi           (string)
    /// - field 2: blocked        (int64) — lots blocked in orders
    /// - field 3: balance        (int64) — total lots held
    /// - field 4: position_uid   (string)
    /// - field 5: instrument_uid (string)
    /// - field 6: exchange_blocked (bool)
    /// - field 7: instrument_type  (string)
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct PositionsSecurities {
        /// FIGI.
        #[prost(string, tag = "1")]
        pub figi: ::prost::alloc::string::String,

        /// Lots blocked in open orders.
        #[prost(int64, tag = "2")]
        pub blocked: i64,

        /// Total lots held (including blocked).
        #[prost(int64, tag = "3")]
        pub balance: i64,

        /// Instrument UID.
        #[prost(string, tag = "5")]
        pub instrument_uid: ::prost::alloc::string::String,

        /// Instrument type.
        #[prost(string, tag = "7")]
        pub instrument_type: ::prost::alloc::string::String,
    }

    /// A money position (cash balance).
    ///
    /// Proto path: `tinkoff.public.invest.api.contract.v1.PositionsMoney`
    /// - field 1: available_value (MoneyValue) — cash available
    /// - field 2: blocked_value   (MoneyValue) — cash blocked in orders
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct PositionsMoney {
        /// Cash available for trading.
        #[prost(message, optional, tag = "1")]
        pub available_value: ::core::option::Option<MoneyValue>,

        /// Cash blocked in open orders.
        #[prost(message, optional, tag = "2")]
        pub blocked_value: ::core::option::Option<MoneyValue>,
    }

    /// Response from GetPositions.
    ///
    /// Proto path: `tinkoff.public.invest.api.contract.v1.PositionsResponse`
    /// - field 1: money      (repeated PositionsMoney)
    /// - field 2: blocked    (repeated MoneyValue)
    /// - field 3: securities (repeated PositionsSecurities)
    /// - field 4: limits_loading_in_progress (bool)
    /// - field 5: futures    (repeated PositionsFutures)
    /// - field 6: options    (repeated PositionsOptions)
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct PositionsResponse {
        /// Cash balances (one entry per currency).
        #[prost(message, repeated, tag = "1")]
        pub money: ::prost::alloc::vec::Vec<PositionsMoney>,

        /// Cash amounts blocked in orders.
        #[prost(message, repeated, tag = "2")]
        pub blocked: ::prost::alloc::vec::Vec<MoneyValue>,

        /// Securities (stocks, bonds, ETFs) positions.
        #[prost(message, repeated, tag = "3")]
        pub securities: ::prost::alloc::vec::Vec<PositionsSecurities>,

        /// Indicates limits are still being loaded.
        #[prost(bool, tag = "4")]
        pub limits_loading_in_progress: bool,
    }

    // ─────────────────────────────────────────────────────────────────────────
    // OperationsService — GetOperations
    // ─────────────────────────────────────────────────────────────────────────

    /// Request for GetOperations.
    ///
    /// gRPC path: `/tinkoff.public.invest.api.contract.v1.OperationsService/GetOperations`
    ///
    /// Proto path: `tinkoff.public.invest.api.contract.v1.OperationsRequest`
    /// - field 1: account_id (string)
    /// - field 2: from       (Timestamp)
    /// - field 3: to         (Timestamp)
    /// - field 4: state      (OperationState)
    /// - field 5: figi       (string)
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct OperationsRequest {
        /// Account ID.
        #[prost(string, tag = "1")]
        pub account_id: ::prost::alloc::string::String,

        /// Start of query range (UTC).
        #[prost(message, optional, tag = "2")]
        pub from: ::core::option::Option<Timestamp>,

        /// End of query range (UTC).
        #[prost(message, optional, tag = "3")]
        pub to: ::core::option::Option<Timestamp>,

        /// State filter (0=unspecified, 1=executed, 2=cancelled, 3=all).
        #[prost(int32, tag = "4")]
        pub state: i32,

        /// FIGI filter (empty = all instruments).
        #[prost(string, tag = "5")]
        pub figi: ::prost::alloc::string::String,
    }

    /// A single operation (trade, commission, dividend, etc.).
    ///
    /// Proto path: `tinkoff.public.invest.api.contract.v1.Operation`
    /// - field 1:  id             (string)
    /// - field 2:  parent_operation_id (string)
    /// - field 3:  currency       (string)
    /// - field 4:  payment        (MoneyValue)
    /// - field 5:  price          (MoneyValue)
    /// - field 6:  state          (OperationState)
    /// - field 7:  quantity       (int64)
    /// - field 8:  quantity_rest  (int64)
    /// - field 9:  figi           (string)
    /// - field 10: instrument_type (string)
    /// - field 11: date           (Timestamp)
    /// - field 12: type           (string)
    /// - field 13: operation_type (OperationType)
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Operation {
        /// Server-assigned operation ID.
        #[prost(string, tag = "1")]
        pub id: ::prost::alloc::string::String,

        /// Currency of the payment.
        #[prost(string, tag = "3")]
        pub currency: ::prost::alloc::string::String,

        /// Payment amount (positive = credit, negative = debit).
        #[prost(message, optional, tag = "4")]
        pub payment: ::core::option::Option<MoneyValue>,

        /// Trade price per unit.
        #[prost(message, optional, tag = "5")]
        pub price: ::core::option::Option<MoneyValue>,

        /// State: 1=executed, 2=cancelled, 3=all.
        #[prost(int32, tag = "6")]
        pub state: i32,

        /// Quantity of lots.
        #[prost(int64, tag = "7")]
        pub quantity: i64,

        /// FIGI of the traded instrument.
        #[prost(string, tag = "9")]
        pub figi: ::prost::alloc::string::String,

        /// Instrument type.
        #[prost(string, tag = "10")]
        pub instrument_type: ::prost::alloc::string::String,

        /// Operation timestamp (UTC).
        #[prost(message, optional, tag = "11")]
        pub date: ::core::option::Option<Timestamp>,

        /// Human-readable operation type (e.g. "Покупка ценных бумаг").
        #[prost(string, tag = "12")]
        pub r#type: ::prost::alloc::string::String,
    }

    /// Response from GetOperations.
    ///
    /// Proto path: `tinkoff.public.invest.api.contract.v1.OperationsResponse`
    /// - field 1: operations (repeated Operation)
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct OperationsResponse {
        /// Operations list.
        #[prost(message, repeated, tag = "1")]
        pub operations: ::prost::alloc::vec::Vec<Operation>,
    }

    // ─────────────────────────────────────────────────────────────────────────
    // InstrumentsService — GetInstrumentBy
    // ─────────────────────────────────────────────────────────────────────────

    /// Request for GetInstrumentBy.
    ///
    /// gRPC path: `/tinkoff.public.invest.api.contract.v1.InstrumentsService/GetInstrumentBy`
    ///
    /// Proto path: `tinkoff.public.invest.api.contract.v1.InstrumentRequest`
    /// - field 1: id_type    (InstrumentIdType)
    /// - field 2: class_code (string) — required when id_type=TICKER
    /// - field 3: id         (string) — FIGI, ticker, or UID
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct InstrumentRequest {
        /// How to interpret the `id` field.
        #[prost(enumeration = "InstrumentIdType", tag = "1")]
        pub id_type: i32,

        /// Exchange class code (e.g. `"TQBR"` for MOEX main board shares).
        /// Required when `id_type = TICKER`.
        #[prost(string, tag = "2")]
        pub class_code: ::prost::alloc::string::String,

        /// The instrument identifier value.
        #[prost(string, tag = "3")]
        pub id: ::prost::alloc::string::String,
    }

    /// A generic instrument descriptor returned by GetInstrumentBy.
    ///
    /// Proto path: `tinkoff.public.invest.api.contract.v1.Instrument`
    /// - field 1:  figi           (string)
    /// - field 2:  ticker         (string)
    /// - field 3:  class_code     (string)
    /// - field 4:  isin           (string)
    /// - field 5:  lot            (int32) — lot size
    /// - field 6:  currency       (string)
    /// - field 7:  klong          (Quotation)
    /// - field 8:  kshort         (Quotation)
    /// - field 9:  dlong          (Quotation)
    /// - field 10: dshort         (Quotation)
    /// - field 11: dlong_min      (Quotation)
    /// - field 12: dshort_min     (Quotation)
    /// - field 13: short_enabled  (bool)
    /// - field 14: name           (string)
    /// - field 15: exchange       (string)
    /// - field 16: country_of_risk (string)
    /// - field 17: country_of_risk_name (string)
    /// - field 18: instrument_type (string)
    /// - field 19: trading_status (SecurityTradingStatus)
    /// - field 20: otc_flag       (bool)
    /// - field 21: buy_available  (bool)
    /// - field 22: sell_available (bool)
    /// - field 23: min_price_increment (Quotation)
    /// - field 24: api_trade_available (bool)
    /// - field 25: uid            (string)
    /// - field 26: real_exchange  (RealExchange)
    /// - field 27: position_uid   (string)
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Instrument {
        /// FIGI.
        #[prost(string, tag = "1")]
        pub figi: ::prost::alloc::string::String,

        /// Exchange ticker symbol.
        #[prost(string, tag = "2")]
        pub ticker: ::prost::alloc::string::String,

        /// Exchange class code (e.g. `"TQBR"`).
        #[prost(string, tag = "3")]
        pub class_code: ::prost::alloc::string::String,

        /// ISIN.
        #[prost(string, tag = "4")]
        pub isin: ::prost::alloc::string::String,

        /// Lot size (number of shares per lot).
        #[prost(int32, tag = "5")]
        pub lot: i32,

        /// Settlement currency (ISO 4217 lower-case).
        #[prost(string, tag = "6")]
        pub currency: ::prost::alloc::string::String,

        /// Instrument name.
        #[prost(string, tag = "14")]
        pub name: ::prost::alloc::string::String,

        /// Exchange name.
        #[prost(string, tag = "15")]
        pub exchange: ::prost::alloc::string::String,

        /// Instrument type.
        #[prost(string, tag = "18")]
        pub instrument_type: ::prost::alloc::string::String,

        /// Current trading status.
        #[prost(enumeration = "SecurityTradingStatus", tag = "19")]
        pub trading_status: i32,

        /// Whether API trading is available for this instrument.
        #[prost(bool, tag = "24")]
        pub api_trade_available_flag: bool,

        /// Instrument UID.
        #[prost(string, tag = "25")]
        pub uid: ::prost::alloc::string::String,
    }

    /// Response from GetInstrumentBy.
    ///
    /// Proto path: `tinkoff.public.invest.api.contract.v1.InstrumentResponse`
    /// - field 1: instrument (Instrument)
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct InstrumentResponse {
        /// The instrument data.
        #[prost(message, optional, tag = "1")]
        pub instrument: ::core::option::Option<Instrument>,
    }

    // ─────────────────────────────────────────────────────────────────────────
    // InstrumentsService — FindInstrument
    // ─────────────────────────────────────────────────────────────────────────

    /// Request for FindInstrument.
    ///
    /// gRPC path: `/tinkoff.public.invest.api.contract.v1.InstrumentsService/FindInstrument`
    ///
    /// Proto path: `tinkoff.public.invest.api.contract.v1.FindInstrumentRequest`
    /// - field 1: query             (string) — ticker, FIGI, ISIN, or name prefix
    /// - field 2: instrument_kind   (InstrumentType)
    /// - field 3: api_trade_available (bool)
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct FindInstrumentRequest {
        /// Search query: ticker, FIGI, ISIN, or company name prefix.
        #[prost(string, tag = "1")]
        pub query: ::prost::alloc::string::String,
    }

    /// A brief instrument entry in FindInstrument results.
    ///
    /// Proto path: `tinkoff.public.invest.api.contract.v1.InstrumentShort`
    /// - field 1:  isin                (string)
    /// - field 2:  figi                (string)
    /// - field 3:  ticker              (string)
    /// - field 4:  class_code          (string)
    /// - field 5:  instrument_type     (string)
    /// - field 6:  name                (string)
    /// - field 7:  uid                 (string)
    /// - field 8:  position_uid        (string)
    /// - field 9:  api_trade_available (bool)
    /// - field 10: for_iis_flag        (bool)
    /// - field 11: first_1min_candle_date (Timestamp)
    /// - field 12: first_1day_candle_date (Timestamp)
    /// - field 13: lot                 (int32)
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct InstrumentShort {
        /// ISIN.
        #[prost(string, tag = "1")]
        pub isin: ::prost::alloc::string::String,

        /// FIGI.
        #[prost(string, tag = "2")]
        pub figi: ::prost::alloc::string::String,

        /// Exchange ticker.
        #[prost(string, tag = "3")]
        pub ticker: ::prost::alloc::string::String,

        /// Exchange class code.
        #[prost(string, tag = "4")]
        pub class_code: ::prost::alloc::string::String,

        /// Instrument type.
        #[prost(string, tag = "5")]
        pub instrument_type: ::prost::alloc::string::String,

        /// Instrument name.
        #[prost(string, tag = "6")]
        pub name: ::prost::alloc::string::String,

        /// Instrument UID.
        #[prost(string, tag = "7")]
        pub uid: ::prost::alloc::string::String,

        /// Whether API trading is available.
        #[prost(bool, tag = "9")]
        pub api_trade_available_flag: bool,
    }

    /// Response from FindInstrument.
    ///
    /// Proto path: `tinkoff.public.invest.api.contract.v1.FindInstrumentResponse`
    /// - field 1: instruments (repeated InstrumentShort)
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct FindInstrumentResponse {
        /// Matching instruments.
        #[prost(message, repeated, tag = "1")]
        pub instruments: ::prost::alloc::vec::Vec<InstrumentShort>,
    }

    // ─────────────────────────────────────────────────────────────────────────
    // gRPC service path constants
    // ─────────────────────────────────────────────────────────────────────────

    /// gRPC endpoint: `https://invest-public-api.tinkoff.ru:443`
    pub const GRPC_ENDPOINT: &str = "https://invest-public-api.tinkoff.ru:443";

    /// Service/method paths for `tonic::codegen::http::uri::PathAndQuery`.
    pub mod paths {
        pub const GET_CANDLES: &str =
            "/tinkoff.public.invest.api.contract.v1.MarketDataService/GetCandles";
        pub const GET_ORDER_BOOK: &str =
            "/tinkoff.public.invest.api.contract.v1.MarketDataService/GetOrderBook";
        pub const GET_LAST_PRICES: &str =
            "/tinkoff.public.invest.api.contract.v1.MarketDataService/GetLastPrices";
        pub const GET_TRADING_STATUS: &str =
            "/tinkoff.public.invest.api.contract.v1.MarketDataService/GetTradingStatus";
        pub const POST_ORDER: &str =
            "/tinkoff.public.invest.api.contract.v1.OrdersService/PostOrder";
        pub const CANCEL_ORDER: &str =
            "/tinkoff.public.invest.api.contract.v1.OrdersService/CancelOrder";
        pub const GET_ORDERS: &str =
            "/tinkoff.public.invest.api.contract.v1.OrdersService/GetOrders";
        pub const GET_ORDER_STATE: &str =
            "/tinkoff.public.invest.api.contract.v1.OrdersService/GetOrderState";
        pub const GET_PORTFOLIO: &str =
            "/tinkoff.public.invest.api.contract.v1.OperationsService/GetPortfolio";
        pub const GET_POSITIONS: &str =
            "/tinkoff.public.invest.api.contract.v1.OperationsService/GetPositions";
        pub const GET_OPERATIONS: &str =
            "/tinkoff.public.invest.api.contract.v1.OperationsService/GetOperations";
        pub const GET_INSTRUMENT_BY: &str =
            "/tinkoff.public.invest.api.contract.v1.InstrumentsService/GetInstrumentBy";
        pub const FIND_INSTRUMENT: &str =
            "/tinkoff.public.invest.api.contract.v1.InstrumentsService/FindInstrument";
    }
}
