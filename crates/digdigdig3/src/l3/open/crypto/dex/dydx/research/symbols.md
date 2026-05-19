# dYdX v4 Symbol Format and Market Structure

## Symbol Format

### Perpetual Markets
dYdX v4 uses a simple hyphenated format for all perpetual markets:

```
{BASE_ASSET}-{QUOTE_ASSET}
```

**Examples**:
- `BTC-USD`
- `ETH-USD`
- `SOL-USD`
- `DOGE-USD`
- `AVAX-USD`
- `MATIC-USD`

### Quote Asset
All perpetual markets on dYdX v4 use **USDC** as the quote asset:
- Margin/collateral is denominated in USDC
- All prices are in USDC terms
- P&L is calculated in USDC

**Note**: While the symbol shows "USD", the actual settlement asset is USDC (Circle's stablecoin).

## Market Identifiers

### Ticker vs. clobPairId

dYdX v4 uses two identifiers for markets:

1. **Ticker** (string): Human-readable market symbol
   - Example: `"BTC-USD"`
   - Used in REST API queries
   - Used in WebSocket subscriptions
   - User-facing identifier

2. **clobPairId** (integer as string): Internal market identifier
   - Example: `"0"` for BTC-USD
   - Used in gRPC order placement
   - Used in protobuf messages
   - Protocol-level identifier

**Mapping Example**:
```json
{
  "ticker": "BTC-USD",
  "clobPairId": "0"
}
```

**Important**: When placing orders via gRPC, you must use the `clobPairId`, not the ticker.

### Getting Market Mappings

**Endpoint**: `GET https://indexer.dydx.trade/v4/perpetualMarkets`

Response includes both identifiers:
```json
{
  "markets": {
    "BTC-USD": {
      "clobPairId": "0",
      "ticker": "BTC-USD"
    },
    "ETH-USD": {
      "clobPairId": "1",
      "ticker": "ETH-USD"
    }
  }
}
```

## Market Types

### Cross-Margin Markets (Default)
- **Most markets** are cross-margined by default
- Positions share collateral from the same subaccount
- More capital-efficient
- Higher risk (one bad position can affect entire account)

**Characteristics**:
- Use parent subaccounts (0-127)
- Multiple positions per subaccount
- Shared margin pool
- Standard liquidation logic

### Isolated Markets
- **Introduced in Release 5.0** of v4 software
- Separate collateral pools per position
- Independent insurance fund per market
- Manual collateral allocation

**Characteristics**:
- Use child subaccounts (128-128,000)
- One position per child subaccount
- Isolated margin (doesn't affect other positions)
- Each market has its own risk properties

**Benefits**:
- Enable more exotic/risky assets
- Limit risk exposure
- Protect main account from volatile positions

**Market Universe Expansion**:
- Cross-margin markets: ~100 markets
- After isolated markets: **800+ potential markets**

### Market Status

Markets can have different statuses:

1. **ACTIVE**
   - Normal trading
   - All order types allowed

2. **PAUSED**
   - Trading temporarily disabled
   - No new orders

3. **CANCEL_ONLY**
   - Only order cancellations allowed
   - No new orders or modifications

4. **POST_ONLY**
   - Only post-only orders allowed
   - No market orders or immediate fills

## Symbol Normalization

### Case Sensitivity
- Tickers are **case-sensitive**
- Always use uppercase (e.g., `BTC-USD`, not `btc-usd`)

### Validation Pattern
```regex
^[A-Z0-9]+-USD$
```

Examples:
- Valid: `BTC-USD`, `ETH-USD`, `1INCH-USD`, `DOGE-USD`
- Invalid: `btc-usd`, `BTCUSD`, `BTC/USD`, `BTC_USD`

## Available Markets

### Top Markets (as of 2026)
- `BTC-USD` - Bitcoin
- `ETH-USD` - Ethereum
- `SOL-USD` - Solana
- `DOGE-USD` - Dogecoin
- `AVAX-USD` - Avalanche
- `MATIC-USD` - Polygon
- `LINK-USD` - Chainlink
- `UNI-USD` - Uniswap
- `AAVE-USD` - Aave
- `XRP-USD` - Ripple

### Total Market Count
- **100+ markets** currently available
- **182+ markets** supported across all deployments
- **800+ potential markets** with isolated margin feature

### Querying All Markets

**Endpoint**: `GET /v4/perpetualMarkets`

Returns all active markets with details:
```json
{
  "markets": {
    "BTC-USD": {
      "ticker": "BTC-USD",
      "clobPairId": "0",
      "status": "ACTIVE",
      "baseAsset": "BTC",
      "quoteAsset": "USDC",
      "marketType": "PERPETUAL"
    }
  }
}
```

## Market Parameters

### Size and Price Precision

Each market has specific precision parameters:

**stepSize**: Minimum order size increment
- Example: `"0.0001"` for BTC-USD
- Orders must be multiples of stepSize

**tickSize**: Minimum price increment
- Example: `"1"` for BTC-USD (1 USD)
- Prices must be multiples of tickSize

**atomicResolution**: Quantum resolution for size
- Example: `-10` for BTC-USD
- 1 quantum = 10^(-10) BTC

**quantumConversionExponent**: Price conversion exponent
- Example: `-9` for BTC-USD
- Used to convert between subticks and human-readable prices

**subticksPerTick**: Subticks per tick
- Example: `100000` for BTC-USD
- Defines relationship between protocol ticks and display ticks

### Example: BTC-USD Market Parameters
```json
{
  "ticker": "BTC-USD",
  "clobPairId": "0",
  "baseAsset": "BTC",
  "quoteAsset": "USDC",
  "stepSize": "0.0001",
  "tickSize": "1",
  "atomicResolution": -10,
  "quantumConversionExponent": -9,
  "subticksPerTick": 100000,
  "stepBaseQuantums": 1000000
}
```

**Interpretation**:
- Minimum order: 0.0001 BTC
- Price increment: 1 USD
- 1 quantum = 0.0000000001 BTC (10^-10)
- Prices represented as subticks internally

## Proposing New Markets

dYdX v4 allows governance to propose new perpetual markets.

### Proposal Format
```json
{
  "title": "Add BTC-USD perpetual market",
  "ticker": "BTC-USD",
  "params": {
    "atomicResolution": -10,
    "stepBaseQuantums": 1000000,
    "subticksPerTick": 100000,
    "quantumConversionExponent": -9
  }
}
```

### Market Selection
- Markets are added through governance votes
- Community can propose new assets
- Risk parameters set by governance

## Symbol Conversion for API Usage

### For REST API (Indexer)
Use ticker directly:
```rust
let ticker = "BTC-USD";
let url = format!("/v4/perpetualMarkets/{}", ticker);
```

### For gRPC API (Node)
Convert ticker to clobPairId:
```rust
// First, fetch market info
let markets = indexer_client.get_perpetual_markets().await?;

// Find clobPairId
let market = markets.get("BTC-USD").ok_or("Market not found")?;
let clob_pair_id = market.clob_pair_id.parse::<u32>()?;

// Use in order placement
let order = MsgPlaceOrder {
    clob_pair_id,
    // ... other fields
};
```

### Caching Market Mappings
Since market mappings rarely change, cache them:
```rust
use std::collections::HashMap;

struct MarketCache {
    ticker_to_clob_id: HashMap<String, u32>,
    clob_id_to_ticker: HashMap<u32, String>,
}

impl MarketCache {
    async fn refresh(&mut self) -> Result<(), ExchangeError> {
        let markets = fetch_markets().await?;

        self.ticker_to_clob_id.clear();
        self.clob_id_to_ticker.clear();

        for (ticker, market) in markets {
            let clob_id = market.clob_pair_id.parse::<u32>()?;
            self.ticker_to_clob_id.insert(ticker.clone(), clob_id);
            self.clob_id_to_ticker.insert(clob_id, ticker);
        }

        Ok(())
    }

    fn get_clob_id(&self, ticker: &str) -> Option<u32> {
        self.ticker_to_clob_id.get(ticker).copied()
    }

    fn get_ticker(&self, clob_id: u32) -> Option<&str> {
        self.clob_id_to_ticker.get(&clob_id).map(|s| s.as_str())
    }
}
```

## WebSocket Symbol Format

WebSocket subscriptions use ticker format:
```json
{
  "type": "subscribe",
  "channel": "v4_orderbook",
  "id": "BTC-USD"
}
```

## Market Data Fields

### From /v4/perpetualMarkets Response

```json
{
  "ticker": "BTC-USD",
  "clobPairId": "0",
  "status": "ACTIVE",
  "baseAsset": "BTC",
  "quoteAsset": "USDC",
  "stepSize": "0.0001",
  "tickSize": "1",
  "indexPrice": "50000.5",
  "oraclePrice": "50000.0",
  "priceChange24H": "1250.75",
  "volume24H": "125000000.50",
  "trades24H": 12543,
  "nextFundingRate": "0.00001",
  "initialMarginFraction": "0.05",
  "maintenanceMarginFraction": "0.03",
  "openInterest": "10000.5",
  "atomicResolution": -10,
  "quantumConversionExponent": -9,
  "subticksPerTick": 100000,
  "stepBaseQuantums": 1000000,
  "marketType": "PERPETUAL"
}
```

**Key Fields**:
- `ticker`: Symbol for API queries
- `clobPairId`: ID for order placement
- `status`: Market trading status
- `baseAsset`/`quoteAsset`: Asset pair
- `stepSize`/`tickSize`: Precision constraints
- `initialMarginFraction`: Initial margin requirement (e.g., "0.05" = 5% = 20x leverage)
- `maintenanceMarginFraction`: Maintenance margin (e.g., "0.03" = 3%)
- `marketType`: "PERPETUAL", "CROSS", or "ISOLATED"

## Leverage and Margin

### Max Leverage
Calculated from initial margin fraction:
```
max_leverage = 1 / initial_margin_fraction
```

Examples:
- Initial margin 5% → 20x leverage
- Initial margin 10% → 10x leverage
- Initial margin 20% → 5x leverage

### Market-Specific Leverage
Different markets have different margin requirements:
- **Major assets** (BTC, ETH): Up to 20-25x leverage
- **Mid-cap assets**: 10-20x leverage
- **Small-cap/volatile assets**: 5-10x leverage
- **Isolated markets**: Variable, governance-determined

## Example: Complete Market Info Query

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct Market {
    ticker: String,
    clob_pair_id: String,
    status: String,
    base_asset: String,
    quote_asset: String,
    step_size: String,
    tick_size: String,
    atomic_resolution: i32,
    quantum_conversion_exponent: i32,
    subticks_per_tick: u64,
    market_type: String,
}

async fn get_market_info(ticker: &str) -> Result<Market, ExchangeError> {
    let url = format!("https://indexer.dydx.trade/v4/perpetualMarkets");
    let response: HashMap<String, Market> = reqwest::get(&url)
        .await?
        .json()
        .await?;

    response.get(ticker)
        .cloned()
        .ok_or(ExchangeError::MarketNotFound)
}

// Usage
let btc_market = get_market_info("BTC-USD").await?;
println!("BTC-USD clobPairId: {}", btc_market.clob_pair_id);
println!("Step size: {}", btc_market.step_size);
```

## Symbol Parsing Utilities

```rust
/// Parse dYdX ticker into base and quote assets
fn parse_ticker(ticker: &str) -> Result<(String, String), ExchangeError> {
    let parts: Vec<&str> = ticker.split('-').collect();
    if parts.len() != 2 {
        return Err(ExchangeError::InvalidSymbol(ticker.to_string()));
    }

    Ok((parts[0].to_string(), parts[1].to_string()))
}

/// Validate dYdX ticker format
fn is_valid_ticker(ticker: &str) -> bool {
    let re = regex::Regex::new(r"^[A-Z0-9]+-USD$").unwrap();
    re.is_match(ticker)
}

/// Format ticker from base asset
fn format_ticker(base_asset: &str) -> String {
    format!("{}-USD", base_asset.to_uppercase())
}

// Examples
assert_eq!(parse_ticker("BTC-USD")?, ("BTC", "USD"));
assert!(is_valid_ticker("ETH-USD"));
assert!(!is_valid_ticker("btc-usd"));
assert_eq!(format_ticker("btc"), "BTC-USD");
```

## Important Notes

### Symbol Standardization
- Always use uppercase tickers
- Always use hyphen separator (not slash or underscore)
- All perpetuals end with "-USD"
- No spot markets (perpetuals only)

### Market Discovery
- Fetch markets list on startup
- Cache ticker ↔ clobPairId mappings
- Refresh periodically (markets rarely change, but can be added)

### Error Handling
- Invalid ticker → 404 Not Found
- Market not active → Check status field
- Order size/price not aligned with stepSize/tickSize → Validation error

### Subaccount Assignment
- **Cross-margin positions**: Use parent subaccounts (0-127)
- **Isolated positions**: Use child subaccounts (128-128,000)
- Frontend typically uses subaccount 0 for cross-margin
- Child subaccounts automatically created for isolated positions
