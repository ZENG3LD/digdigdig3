# Angel One SmartAPI Connector

Production connector for Angel One (formerly Angel Broking) Indian stock broker.

## Status

✅ **READY FOR USE** - Authentication working, REST endpoints implemented, WebSocket pending

## Quick Start

### 1. Get API Credentials (5 minutes)

```
https://smartapi.angelone.in/publisher-login
→ Create SmartAPI app (requires active Angel One trading account)
→ Dashboard → My Apps → Create New App
→ Copy API Key
→ Enable TOTP in your Angel One account settings → copy TOTP secret
```

Angel One requires an active trading account with KYC. API access is free for all clients.

### 2. Set Environment Variables

**Linux/macOS/Git Bash:**
```bash
export ANGEL_ONE_API_KEY="your_api_key"
export ANGEL_ONE_CLIENT_CODE="your_client_code"
export ANGEL_ONE_PIN="your_pin"
export ANGEL_ONE_TOTP_SECRET="your_totp_secret_base32"
```

**Windows PowerShell:**
```powershell
$env:ANGEL_ONE_API_KEY="your_api_key"
$env:ANGEL_ONE_CLIENT_CODE="your_client_code"
$env:ANGEL_ONE_PIN="your_pin"
$env:ANGEL_ONE_TOTP_SECRET="your_totp_secret_base32"
```

### 3. Test

```bash
# Rust integration tests
cd zengeld-terminal/crates/connectors/crates/v5
cargo test --test angel_one_integration -- --nocapture
```

## Usage

```rust
use digdigdig3::stocks::india::angel_one::AngelOneConnector;
use digdigdig3::core::{Symbol, AccountType};
use digdigdig3::core::traits::{MarketData, Trading, Account};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize connector (performs login + TOTP authentication)
    let connector = AngelOneConnector::new(
        api_key,
        client_code,
        pin,
        totp_secret,
        false, // testnet = false (no testnet available)
    ).await?;

    // Get current price (NSE equity)
    let symbol = Symbol::new("RELIANCE", "INR");
    let price = connector.get_price(symbol.clone(), AccountType::Spot).await?;
    println!("RELIANCE: Rs.{}", price);

    // Get historical candles (up to 8000 per request, free)
    let candles = connector.get_klines(symbol.clone(), "1h", Some(100), AccountType::Spot).await?;
    println!("Got {} candles", candles.len());

    // Get account balance (margins)
    let balances = connector.get_balance(None, AccountType::Spot).await?;
    for b in balances {
        println!("{}: Rs.{}", b.asset, b.total);
    }

    // Place market order
    let order = connector.market_order(
        symbol,
        OrderSide::Buy,
        1.0, // quantity in shares
        AccountType::Spot,
    ).await?;
    println!("Order placed: {}", order.id);

    Ok(())
}
```

## Features

### Market Data
- ✅ Real-time LTP (Last Traded Price)
- ✅ OHLC quotes
- ✅ Full market depth (20 levels — unique to Angel One)
- ✅ Historical OHLCV (free, all segments, up to 2000 days)
- ✅ Ticker snapshots
- ✅ 120+ indices (NSE, BSE, MCX)

### Trading
- ✅ Market orders
- ✅ Limit orders
- ✅ SL / SL-M orders
- ✅ GTT orders (Good Till Triggered, valid 1 year, OCO support)
- ✅ AMO (After Market Orders)
- ✅ Modify and cancel orders

### Account
- ✅ Balance / margin queries
- ✅ Holdings (delivery equity)
- ✅ Positions (intraday + F&O)
- ✅ P&L summary
- ✅ Trade history
- ✅ Margin calculator (pre-trade validation)

### WebSocket
- ❌ WebSocket V2 not yet implemented (TODO in `mod.rs`)
- Planned modes: LTP, Quote, Snap Quote, Depth 20

### NOT Supported
- ❌ Testnet / sandbox (Angel One has no sandbox environment)
- ❌ Crypto trading (Indian broker only)
- ❌ International exchanges

## Authentication

**Type:** 3-factor login (Client Code + PIN + TOTP)

**Flow:**
1. POST `/rest/auth/angelbroking/user/v1/loginByPassword` with client code, PIN, and TOTP code
2. Server returns three tokens:
   - `jwtToken` — used as Bearer token for all REST requests
   - `refreshToken` — renew session without re-login
   - `feedToken` — WebSocket authentication

**Headers after login:**
```http
Authorization: Bearer <jwt_token>
X-UserType: USER
X-SourceID: WEB
X-ClientLocalIP: <local_ip>
X-ClientPublicIP: <public_ip>
X-MACAddress: <mac>
X-PrivateKey: <api_key>
```

**Session lifetime:** Tokens expire at midnight (market close). Refresh token can renew the session. No HMAC signature required — token-based only.

**TOTP setup:** Enable in Angel One account → Security Settings → set up authenticator app → save the base32 secret (used in `ANGEL_ONE_TOTP_SECRET`).

## Exchanges & Segments

| Exchange | Segment | Instruments |
|----------|---------|-------------|
| NSE | Equity | ~2,000 stocks |
| BSE | Equity | ~5,000 stocks |
| NFO | F&O | NSE futures + options |
| BFO | F&O | BSE futures + options |
| MCX | Commodity | Gold, Silver, Crude, etc. |
| NCDEX | Commodity | Agricultural commodities |
| CDS | Currency | USD/INR, EUR/INR, GBP/INR, JPY/INR |

## Rate Limits

| Operation | Limit |
|-----------|-------|
| Place / Modify / Cancel order | 20/sec |
| GTT Create / Modify / Cancel | 20/sec (shared) |
| Individual order status | 10/sec |
| Margin calculator | 10/sec |
| WebSocket subscriptions | 1000 tokens max |

## Files

```
angel_one/
├── README.md          # This file
├── mod.rs             # Module exports
├── auth.rs            # 3-factor login, token management, TOTP generation
├── endpoints.rs       # API URLs and endpoint enum
├── connector.rs       # Trait implementations (MarketData, Trading, Account, Positions)
├── parser.rs          # JSON response parsing
└── research/
    ├── api_overview.md
    ├── authentication.md
    ├── endpoints_full.md
    ├── websocket_full.md
    ├── tiers_and_limits.md
    ├── data_types.md
    ├── coverage.md
    └── response_formats.md
```

## Troubleshooting

### Login fails / 401 Unauthorized

**Cause:** Incorrect credentials or expired TOTP code

**Solution:**
1. Verify `ANGEL_ONE_CLIENT_CODE` is your Angel One account ID (e.g. `A123456`)
2. Verify `ANGEL_ONE_PIN` is your account PIN/password
3. Verify `ANGEL_ONE_TOTP_SECRET` is the base32 secret (not a 6-digit code)
4. Ensure your system clock is accurate (TOTP is time-sensitive, within 30 sec)
5. Confirm SmartAPI app is active in the developer portal

### Session expired mid-day

**Cause:** Tokens expire at midnight IST

**Solution:** Re-initialize connector. Use refresh token endpoint to extend session without full re-login.

### No testnet available

Angel One has no sandbox or paper trading environment via the SmartAPI. All API calls hit production. Use small quantities for testing.

## Market Hours

- **Equity (NSE/BSE):** 9:15 AM - 3:30 PM IST, Mon-Fri
- **F&O (NFO/BFO):** 9:15 AM - 3:30 PM IST, Mon-Fri
- **Commodity (MCX):** 9:00 AM - 11:30 PM IST (international session)
- **Currency (CDS):** 9:00 AM - 5:00 PM IST, Mon-Fri

## Documentation

- **SmartAPI Docs:** https://smartapi.angelbroking.com/docs
- **Developer Portal:** https://smartapi.angelone.in/publisher-login
- **Forum:** https://smartapi.angelone.in/smartapi/forum
- **Official SDKs:** Python, Go, Java, NodeJS, C#, PHP available on GitHub

## Testing

```bash
# Integration tests
cargo test --test angel_one_integration -- --nocapture

# Specific tests
cargo test --test angel_one_integration test_login -- --nocapture
cargo test --test angel_one_integration test_get_price -- --nocapture
cargo test --test angel_one_integration test_get_balance -- --nocapture

# Unit tests
cargo test --lib angel_one
```

## Security

1. **Never commit credentials** - Use `.env` file (add to `.gitignore`)
2. **Protect TOTP secret** - Equivalent to full account access
3. **Rotate API key** - Regenerate in developer portal if compromised
4. **Monitor activity** - Check Angel One back-office for unusual trades

## License

Part of the NEMO trading system.
