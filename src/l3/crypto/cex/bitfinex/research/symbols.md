# Bitfinex Symbol Format and Conventions

## Symbol Prefixes

Bitfinex uses single-character prefixes to distinguish between asset types:

| Prefix | Type | Example | Description |
|--------|------|---------|-------------|
| `t` | Trading Pair | tBTCUSD | Spot and margin trading pairs |
| `f` | Funding Currency | fUSD | Margin funding currencies |

## Trading Pair Format

### Standard Format
```
t[BASE][QUOTE]
```

**Examples**:
- `tBTCUSD` - Bitcoin vs US Dollar
- `tETHUSD` - Ethereum vs US Dollar
- `tLTCBTC` - Litecoin vs Bitcoin
- `tETHBTC` - Ethereum vs Bitcoin

### Case Sensitivity
- **All symbols must be UPPERCASE** when used in API requests
- `tBTCUSD` ✓ Correct
- `tbtcusd` ✗ Incorrect (will return error)
- `TBTCUSD` ✗ Incorrect (missing prefix)

## Funding Currency Format

### Standard Format
```
f[CURRENCY]
```

**Examples**:
- `fUSD` - US Dollar funding
- `fBTC` - Bitcoin funding
- `fETH` - Ethereum funding
- `fEUR` - Euro funding

## Special Formats

### Derivative Symbols (Perpetual Futures)
```
t[BASE]F0:[QUOTE]F0
```

**Examples**:
- `tBTCF0:USTF0` - Bitcoin perpetual futures
- `tETHF0:USTF0` - Ethereum perpetual futures
- `tAMPF0:USTF0` - Ampleforth perpetual futures
- `tBTCDOMF0:USTF0` - Bitcoin dominance futures

### Testnet Symbols
```
tTEST[BASE]F0:TEST[QUOTE]F0
```

**Example**:
- `tTESTBTCF0:TESTUSDTF0` - Paper trading BTC futures

### Colon-Separated Format
Some pairs use colons for specific purposes:
```
tDAPP:UST
```

## Symbol Components

### Base Currency
The asset being traded (first currency in the pair).

**Examples**:
- In `tBTCUSD`: BTC is the base
- In `tETHBTC`: ETH is the base

### Quote Currency
The currency used to price the base (second currency in the pair).

**Examples**:
- In `tBTCUSD`: USD is the quote
- In `tETHBTC`: BTC is the quote

## Common Quote Currencies

| Symbol | Currency | Type |
|--------|----------|------|
| USD | US Dollar | Fiat |
| EUR | Euro | Fiat |
| GBP | British Pound | Fiat |
| JPY | Japanese Yen | Fiat |
| BTC | Bitcoin | Crypto |
| ETH | Ethereum | Crypto |
| UST | Tether USD | Stablecoin |
| USDT | Tether USD (alternative) | Stablecoin |
| USDC | USD Coin | Stablecoin |

## Retrieving Available Symbols

### Trading Pairs List
```
GET /v2/conf/pub:list:pair:exchange
```

**Response**:
```json
[
  [
    "BTCUSD",
    "ETHUSD",
    "ETHBTC",
    "LTCUSD",
    "LTCBTC",
    ...
  ]
]
```

Note: Response returns pairs **without** the `t` prefix. You must add `t` when using in API calls.

### Currency List
```
GET /v2/conf/pub:list:currency
```

**Response**:
```json
[
  [
    "BTC",
    "ETH",
    "USD",
    "EUR",
    "USDT",
    ...
  ]
]
```

### Currency Details with Labels
```
GET /v2/conf/pub:map:currency:label
```

**Response**:
```json
[
  {
    "BTC": "BITCOIN",
    "ETH": "ETHEREUM",
    "USD": "UNITED STATES DOLLAR",
    "EUR": "EURO",
    "USDT": "TETHER USD",
    ...
  }
]
```

### Symbol Information
```
GET /v2/conf/pub:info:pair
```

Returns detailed information about trading pairs including minimum order sizes and price precision.

## Symbol Validation

### Valid Symbol Examples
```
tBTCUSD     ✓
tETHUSD     ✓
tLTCBTC     ✓
fUSD        ✓
fBTC        ✓
tBTCF0:USTF0 ✓
```

### Invalid Symbol Examples
```
BTCUSD      ✗ Missing 't' prefix
btcusd      ✗ Not uppercase
tbtcusd     ✗ Not uppercase
t-BTCUSD    ✗ Invalid format
BTC-USD     ✗ Invalid format
BTC/USD     ✗ Invalid format
```

## Symbol Usage in Endpoints

### REST API

**Market Data** (can use lowercase in some cases):
```
GET /v2/ticker/tBTCUSD
GET /v2/book/tETHUSD/P0
GET /v2/trades/tBTCUSD/hist
```

**Trading** (must be uppercase):
```json
{
  "symbol": "tBTCUSD",
  "amount": "0.5",
  "price": "10000",
  "type": "EXCHANGE LIMIT"
}
```

### WebSocket

**Subscription**:
```json
{
  "event": "subscribe",
  "channel": "ticker",
  "symbol": "tBTCUSD"
}
```

**Response** includes both symbol and pair:
```json
{
  "event": "subscribed",
  "channel": "ticker",
  "chanId": 1,
  "symbol": "tBTCUSD",
  "pair": "BTCUSD"
}
```

## Converting Between Formats

### API Response Format to Request Format
```
Response: "BTCUSD"  →  Request: "tBTCUSD"
Response: "USD"     →  Request: "fUSD"
```

### Display Format Conversion
```rust
// Remove prefix for display
fn format_for_display(symbol: &str) -> &str {
    symbol.trim_start_matches('t').trim_start_matches('f')
}

// tBTCUSD → BTCUSD
// fUSD → USD
```

```rust
// Add prefix for API request
fn format_for_api(pair: &str, is_funding: bool) -> String {
    if is_funding {
        format!("f{}", pair)
    } else {
        format!("t{}", pair)
    }
}

// BTCUSD, false → tBTCUSD
// USD, true → fUSD
```

## Currency Codes

### Major Cryptocurrencies
- BTC - Bitcoin
- ETH - Ethereum
- LTC - Litecoin
- XRP - Ripple
- BCH - Bitcoin Cash
- EOS - EOS
- XLM - Stellar
- TRX - Tron
- ADA - Cardano
- DOT - Polkadot

### Stablecoins
- UST - Tether (on Bitfinex)
- USDT - Tether USD (standard)
- USDC - USD Coin
- DAI - Dai
- USDT0 - Tether for derivatives (requires conversion)

### Fiat Currencies
- USD - US Dollar
- EUR - Euro
- GBP - British Pound
- JPY - Japanese Yen
- CNH - Chinese Yuan (offshore)

### Special Currencies
- IOTA - Uses Mi (MegaIOTA) as unit
- Various DeFi tokens with pool associations
- Wrapped tokens (e.g., WBTC)

## Trading vs Funding Pairs

### Trading Pairs (t prefix)
- Used for spot and margin trading
- Have order books
- Support LIMIT, MARKET, STOP orders
- Examples: tBTCUSD, tETHBTC

### Funding Currencies (f prefix)
- Used for margin lending/borrowing
- Have funding books with rates and periods
- Support funding offers
- Examples: fUSD, fBTC, fETH

## Symbol Length Considerations

Most symbols are 7-9 characters:
- `tBTCUSD` - 7 characters
- `tETHBTC` - 7 characters
- `tBTCF0:USTF0` - 13 characters (derivatives)

Always use dynamic string handling rather than fixed-length buffers.

## Error Handling

### Invalid Symbol Error
```json
["error", 10020, "symbol: invalid"]
```

**Common Causes**:
- Missing `t` or `f` prefix
- Lowercase characters
- Pair doesn't exist
- Incorrect format

### Validation Function (Rust)
```rust
fn is_valid_symbol(symbol: &str) -> bool {
    if symbol.len() < 4 {
        return false;
    }

    // Check prefix
    let first_char = symbol.chars().next().unwrap();
    if first_char != 't' && first_char != 'f' {
        return false;
    }

    // Check uppercase
    symbol.chars().all(|c| c.is_uppercase() || c == ':' || c.is_numeric())
}
```

## Best Practices

1. **Always validate symbols** before making API calls
2. **Store symbols with prefix** (tBTCUSD, not BTCUSD)
3. **Use uppercase** for all symbol operations
4. **Fetch symbol list** on startup to validate user input
5. **Handle both formats** (with/without prefix) in user input
6. **Cache symbol lists** to reduce API calls
7. **Update symbol cache** periodically (new pairs added regularly)

## WebSocket Symbol Responses

In WebSocket messages, you'll receive both formats:

```json
{
  "symbol": "tBTCUSD",  // With prefix
  "pair": "BTCUSD"      // Without prefix
}
```

Store both if needed for different purposes:
- Use `symbol` for API calls
- Use `pair` for display or internal logic
