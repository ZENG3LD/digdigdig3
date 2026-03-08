# GMX Symbols and Market Structure

GMX uses a unique market structure based on **isolated liquidity pools** with specific token pairs for long and short collateral.

## Market Naming Convention

### Format

GMX markets follow this naming pattern:

```
{INDEX_TOKEN}/{QUOTE_CURRENCY} [{LONG_TOKEN}-{SHORT_TOKEN}]
```

**Examples:**
- `ETH/USD [ETH-USDC]`
- `BTC/USD [BTC-USDT]`
- `ARB/USD [ARB-USDC]`
- `SOL/USD [SOL-USDC]`

### Components

1. **Index Token** - The asset being traded (e.g., ETH, BTC)
2. **Quote Currency** - Always USD for perpetual markets
3. **Long Token** - Collateral token for long positions
4. **Short Token** - Collateral token for short positions

### Multiple Pools Per Market

A single index token can have multiple markets with different collateral pairs:

- `ETH/USD [ETH-USDC]` - ETH collateral for longs, USDC for shorts
- `ETH/USD [ETH-USDT]` - ETH collateral for longs, USDT for shorts
- `ETH/USD [WETH-DAI]` - WETH collateral for longs, DAI for shorts

Traders choose which pool based on:
- Preferred collateral token
- Pool liquidity depth
- Funding rates
- Fee APY

## Token Types

### 1. Index Tokens (Tradeable Assets)

Index tokens are the assets you trade. These determine the price exposure.

**Arbitrum Index Tokens:**
- `ETH` - Ethereum
- `BTC` - Bitcoin (Wrapped)
- `WBTC` - Wrapped Bitcoin
- `LINK` - Chainlink
- `ARB` - Arbitrum
- `UNI` - Uniswap
- `DOGE` - Dogecoin (Wrapped)
- `SOL` - Solana (Wrapped)
- `XRP` - XRP (Wrapped)
- `LTC` - Litecoin (Wrapped)

**Avalanche Index Tokens:**
- `AVAX` - Avalanche (Native)
- `ETH` - Ethereum (Wrapped)
- `BTC` - Bitcoin (Wrapped)
- `WBTC.e` - Wrapped Bitcoin (Bridged)

### 2. Collateral Tokens

Collateral tokens are used to back positions. These are deposited when opening trades.

**Long Collateral Tokens:**
- Typically the index token itself (e.g., ETH for ETH/USD longs)
- Provides native exposure to the underlying asset
- Example: Opening long ETH/USD with ETH collateral

**Short Collateral Tokens:**
- Typically stablecoins (USDC, USDT, DAI)
- Provides stable value backing for shorts
- Example: Opening short ETH/USD with USDC collateral

**Common Collateral Tokens (Arbitrum):**
- `USDC` - USD Coin (Native)
- `USDT` - Tether USD
- `DAI` - Dai Stablecoin
- `ETH` - Ethereum (Wrapped)
- `WETH` - Wrapped Ethereum
- `WBTC` - Wrapped Bitcoin

**Common Collateral Tokens (Avalanche):**
- `USDC` - USD Coin
- `USDC.e` - USD Coin (Bridged)
- `USDT` - Tether USD
- `USDT.e` - Tether (Bridged)
- `WAVAX` - Wrapped AVAX
- `WETH.e` - Wrapped Ethereum (Bridged)
- `WBTC.e` - Wrapped Bitcoin (Bridged)

### 3. Market Tokens (GM Tokens)

Each market has a corresponding **GM token** representing liquidity provider shares.

**Format:** `GM: {MARKET_NAME}`

**Examples:**
- `GM: ETH/USD [ETH-USDC]` - Address: `0x70d95587d40A2caf56bd97485aB3Eec10Bee6336`
- `GM: BTC/USD [BTC-USDC]` - Address: `0x47c031236e19d024b42f8AE6780E44A573170703`

**Purpose:**
- Liquidity providers deposit tokens and receive GM tokens
- GM tokens accrue trading fees and funding payments
- Redeemable for underlying pool tokens

### 4. GLV Tokens (GMX Liquidity Vaults)

GLV tokens are **meta-vaults** that aggregate multiple GM pools.

**Format:** `GLV-{INDEX}`

**Examples:**
- `GLV-ETH` - Aggregates multiple ETH/USD pools
- `GLV-BTC` - Aggregates multiple BTC/USD pools

**Purpose:**
- Diversifies liquidity across multiple pools
- Auto-rebalances based on performance
- Simplified LP experience

## Symbol Formatting

### Token Address to Symbol

GMX APIs return token addresses. Map them to symbols using the tokens endpoint.

**Example Mapping (Arbitrum):**
```json
{
  "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1": "WETH",
  "0xaf88d065e77c8cC2239327C5EDb3A432268e5831": "USDC",
  "0x2f2a2543B76A4166549F7aaB2e75Bef0aefC5B0f": "WBTC",
  "0xFd086bC7CD5C481DCC9C85ebE478A1C0b69FCbb9": "USDT",
  "0xDA10009cBd5D07dd0CeCc66161FC93D7c9000da1": "DAI"
}
```

### Symbol Normalization

GMX uses various token representations:

**Native vs Wrapped:**
- `ETH` on Ethereum = `WETH` on Arbitrum
- `AVAX` on Avalanche = `WAVAX` when wrapped
- API may return either format

**Bridged Tokens (Avalanche):**
- `USDC` - Native Circle USDC
- `USDC.e` - Bridged USDC from Ethereum
- Both accepted but different addresses

**Normalization Function:**
```rust
fn normalize_gmx_symbol(symbol: &str, chain: &str) -> String {
    match (symbol, chain) {
        ("WETH", "arbitrum") => "ETH".to_string(),
        ("WAVAX", "avalanche") => "AVAX".to_string(),
        ("WBTC", _) => "BTC".to_string(),
        ("USDC.e", _) => "USDC".to_string(),
        ("USDT.e", _) => "USDT".to_string(),
        (s, _) => s.to_string(),
    }
}
```

## Market Structure

### Market Identification

Each market is uniquely identified by:

1. **Market Token Address** - The GM token contract address
2. **Index Token** - The traded asset
3. **Long Token** - Collateral for longs
4. **Short Token** - Collateral for shorts

**Example:**
```json
{
  "marketToken": "0x70d95587d40A2caf56bd97485aB3Eec10Bee6336",
  "indexToken": "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1",
  "longToken": "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1",
  "shortToken": "0xaf88d065e77c8cC2239327C5EDb3A432268e5831",
  "marketSymbol": "ETH/USD [ETH-USDC]"
}
```

### Spot vs Perpetual Markets

**Perpetual Markets:**
- Have an index token different from long/short tokens
- Support leveraged positions
- Example: `ETH/USD [ETH-USDC]` - Trade ETH with leverage

**Spot Markets (Swap-Only):**
- Index token same as one of the collateral tokens
- No leverage, only swaps
- Market name shows as "SWAP-ONLY"
- Example: `[ETH-USDC]` - Spot swap between ETH and USDC

**Identifying Perpetual vs Spot:**
```rust
fn is_perpetual_market(market: &Market) -> bool {
    market.index_token != market.long_token
        && market.index_token != market.short_token
}
```

### Single-Token vs Dual-Token Markets

**Single-Token Markets:**
- Long and short collateral are the same token
- Example: `ETH/USD [USDC-USDC]` - Both sides use USDC
- Market divisor: 2
- Simpler pool accounting

**Dual-Token Markets:**
- Long and short collateral are different tokens
- Example: `ETH/USD [ETH-USDC]` - Longs use ETH, shorts use USDC
- Market divisor: 1
- More complex but better capital efficiency

## Position Collateral Selection

When opening a position, you choose collateral based on direction:

### Long Positions

Use the **long token** as collateral:

**Example: Long ETH/USD [ETH-USDC]**
- Collateral: ETH
- You deposit ETH to open long
- Gains paid in ETH
- More ETH exposure (amplified upside/downside)

**Alternative: Long ETH/USD [USDC-USDC]**
- Collateral: USDC
- You deposit USDC to open long
- Gains paid in USDC
- Pure ETH price exposure without additional ETH risk

### Short Positions

Use the **short token** as collateral:

**Example: Short ETH/USD [ETH-USDC]**
- Collateral: USDC
- You deposit USDC to open short
- Gains paid in USDC
- Stable collateral for short positions

**Alternative: Short ETH/USD [ETH-ETH]**
- Collateral: ETH (unusual but possible)
- You deposit ETH to short ETH
- Hedging strategy (short delta, long ETH)

## Symbol Lookup

### Get All Markets

**Endpoint:** `GET /{chain}-api.gmxinfra.io/markets`

**Response:**
```json
[
  {
    "marketToken": "0x70d95587d40A2caf56bd97485aB3Eec10Bee6336",
    "indexToken": "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1",
    "longToken": "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1",
    "shortToken": "0xaf88d065e77c8cC2239327C5EDb3A432268e5831",
    "indexTokenSymbol": "ETH",
    "longTokenSymbol": "ETH",
    "shortTokenSymbol": "USDC",
    "marketSymbol": "ETH/USD"
  }
]
```

### Get All Tokens

**Endpoint:** `GET /{chain}-api.gmxinfra.io/tokens`

**Response:**
```json
[
  {
    "symbol": "ETH",
    "name": "Ethereum",
    "address": "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1",
    "decimals": 18,
    "isNative": false,
    "isShortable": true,
    "isStable": false
  }
]
```

## Token Addresses by Chain

### Arbitrum (Chain ID: 42161)

**Major Tokens:**
```
WETH:  0x82aF49447D8a07e3bd95BD0d56f35241523fBab1
USDC:  0xaf88d065e77c8cC2239327C5EDb3A432268e5831
USDT:  0xFd086bC7CD5C481DCC9C85ebE478A1C0b69FCbb9
DAI:   0xDA10009cBd5D07dd0CeCc66161FC93D7c9000da1
WBTC:  0x2f2a2543B76A4166549F7aaB2e75Bef0aefC5B0f
ARB:   0x912CE59144191C1204E64559FE8253a0e49E6548
LINK:  0xf97f4df75117a78c1A5a0DBb814Af92458539FB4
UNI:   0xFa7F8980b0f1E64A2062791cc3b0871572f1F7f0
```

**Major Markets:**
```
ETH/USD [ETH-USDC]:  0x70d95587d40A2caf56bd97485aB3Eec10Bee6336
BTC/USD [BTC-USDC]:  0x47c031236e19d024b42f8AE6780E44A573170703
ARB/USD [ARB-USDC]:  0xC25cEf6061Cf5dE5eb761b50E4743c1F5D7E5407
LINK/USD [LINK-USDC]: 0x7f1fa204bb700853D36994DA19F830b6Ad18455C
```

### Avalanche (Chain ID: 43114)

**Major Tokens:**
```
WAVAX: 0xB31f66AA3C1e785363F0875A1B74E27b85FD66c7
USDC:  0xB97EF9Ef8734C71904D8002F8b6Bc66Dd9c48a6E
USDC.e: 0xA7D7079b0FEaD91F3e65f86E8915Cb59c1a4C664
USDT:  0x9702230A8Ea53601f5cD2dc00fDBc13d4dF4A8c7
USDT.e: 0xc7198437980c041c805A1EDcbA50c1Ce5db95118
WETH.e: 0x49D5c2BdFfac6CE2BFdB6640F4F80f226bc10bAB
WBTC.e: 0x50b7545627a5162F82A992c33b87aDc75187B218
```

**Major Markets:**
```
AVAX/USD [AVAX-USDC]:  (Check official docs for current address)
ETH/USD [WETH-USDC]:   (Check official docs for current address)
BTC/USD [WBTC-USDC]:   (Check official docs for current address)
```

## Symbol Parsing Examples

### Parse Market Symbol

```rust
struct ParsedMarketSymbol {
    index: String,      // "ETH"
    quote: String,      // "USD"
    long: String,       // "ETH"
    short: String,      // "USDC"
}

fn parse_market_symbol(symbol: &str) -> Result<ParsedMarketSymbol> {
    // Format: "ETH/USD [ETH-USDC]"
    let parts: Vec<&str> = symbol.split(" [").collect();

    let pair = parts[0]; // "ETH/USD"
    let collateral = parts[1].trim_end_matches(']'); // "ETH-USDC"

    let pair_parts: Vec<&str> = pair.split('/').collect();
    let index = pair_parts[0].to_string(); // "ETH"
    let quote = pair_parts[1].to_string(); // "USD"

    let collateral_parts: Vec<&str> = collateral.split('-').collect();
    let long = collateral_parts[0].to_string(); // "ETH"
    let short = collateral_parts[1].to_string(); // "USDC"

    Ok(ParsedMarketSymbol { index, quote, long, short })
}
```

### Build Market Symbol

```rust
fn build_market_symbol(
    index: &str,
    quote: &str,
    long: &str,
    short: &str,
) -> String {
    format!("{}/{} [{}-{}]", index, quote, long, short)
}

// Example:
let symbol = build_market_symbol("ETH", "USD", "ETH", "USDC");
// Result: "ETH/USD [ETH-USDC]"
```

### Unified Symbol for Connectors

For consistency across exchanges in the V5 connector system, use a unified symbol format:

**Recommended Format:** `{INDEX}-{QUOTE}-{COLLATERAL}`

**Examples:**
- `ETH-USD-ETH` - Long ETH/USD with ETH collateral
- `ETH-USD-USDC` - Long ETH/USD with USDC collateral
- `BTC-USD-BTC` - Long BTC/USD with BTC collateral

**Conversion:**
```rust
fn to_unified_symbol(gmx_symbol: &str, is_long: bool) -> String {
    let parsed = parse_market_symbol(gmx_symbol)?;
    let collateral = if is_long {
        &parsed.long
    } else {
        &parsed.short
    };
    format!("{}-{}-{}", parsed.index, parsed.quote, collateral)
}

// Example:
let unified = to_unified_symbol("ETH/USD [ETH-USDC]", true);
// Result: "ETH-USD-ETH"
```

## Implementation Checklist

### Symbol Handling in V5 Connector

- [ ] Fetch and cache markets list from REST API
- [ ] Fetch and cache tokens list from REST API
- [ ] Build address-to-symbol mapping
- [ ] Build symbol-to-address mapping
- [ ] Implement market symbol parser
- [ ] Implement symbol normalization (WETH → ETH)
- [ ] Support unified symbol format conversion
- [ ] Handle bridged token variants (.e suffix)
- [ ] Detect perpetual vs spot markets
- [ ] Detect single-token vs dual-token markets
- [ ] Validate collateral token for position direction

### Symbol Validation

- [ ] Verify market exists before trading
- [ ] Verify token is shortable (for short positions)
- [ ] Verify collateral token matches position direction
- [ ] Check if market is disabled (isDisabled flag)
- [ ] Validate token decimals for amount calculations

## Sources

- [GMX Trading V2 Documentation](https://docs.gmx.io/docs/trading/v2/)
- [GMX Markets SDK Utils](https://docs.gmx.io/docs/sdk/exports/utils/markets/)
- [GMX REST API](https://docs.gmx.io/docs/api/rest/)
- [GMX Synthetics Contracts](https://github.com/gmx-io/gmx-synthetics)
- [Arbitrum Token Addresses](https://arbiscan.io/)
- [Avalanche Token Addresses](https://snowtrace.io/)
