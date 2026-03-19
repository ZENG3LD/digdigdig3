# Interactive Brokers Connector

V5 connector for Interactive Brokers Client Portal Web API.

## Features

- **Multi-Asset Trading**: Stocks, Forex, Futures, Options, Bonds
- **Global Markets**: Access to 150+ exchanges worldwide
- **Real-Time Data**: Market data snapshots and historical candles
- **WebSocket Streaming**: Real-time market data and order updates (coming soon)
- **Account Management**: Positions, balances, P&L tracking

## Authentication

### Gateway (Individual Accounts) - Recommended for Testing

1. **Download Client Portal Gateway**:
   - Go to [IBKR Client Portal Gateway](https://www.interactivebrokers.com/en/trading/ib-api.php)
   - Download and extract the gateway package

2. **Start Gateway**:
   ```bash
   cd clientportal.gw
   bin/run.sh root/conf.yaml  # Linux/Mac
   bin\run.bat root\conf.yaml  # Windows
   ```

3. **Authenticate via Browser**:
   - Open `https://localhost:5000` in your browser
   - Accept the self-signed certificate warning (expected)
   - Login with your IBKR username and password
   - Complete two-factor authentication

4. **Verify Authentication**:
   ```bash
   curl -k https://localhost:5000/v1/api/iserver/auth/status
   # Should return: {"authenticated": true, "connected": true, "competing": false}
   ```

### OAuth 2.0 (Enterprise Accounts) - Not Yet Implemented

OAuth 2.0 support for enterprise clients will be added in a future update.

## Usage

### Basic Example

```rust
use digdigdig3::aggregators::ib::IBConnector;
use digdigdig3::core::traits::{ExchangeIdentity, MarketData};
use digdigdig3::core::types::{Symbol, AccountType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create connector (assumes Gateway is running and authenticated)
    let connector = IBConnector::from_gateway(
        "https://localhost:5000/v1/api",
        "DU12345"  // Your account ID
    ).await?;

    // Get current price
    let symbol = Symbol::new("AAPL", "USD");
    let price = connector.get_price(symbol.clone(), AccountType::Spot).await?;
    println!("AAPL price: ${}", price);

    // Get 24h ticker data
    let ticker = connector.get_ticker(symbol.clone(), AccountType::Spot).await?;
    println!("AAPL ticker: {:?}", ticker);

    // Get historical candles
    let klines = connector.get_klines(
        symbol,
        "1h",
        Some(24),
        AccountType::Spot
    ).await?;
    println!("Retrieved {} candles", klines.len());

    Ok(())
}
```

### Advanced Example: Account Management

```rust
use digdigdig3::aggregators::ib::IBConnector;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let connector = IBConnector::from_gateway(
        "https://localhost:5000/v1/api",
        "DU12345"
    ).await?;

    // Get account summary
    let summary = connector.get_account_summary().await?;
    println!("Net Liquidation: ${:.2}", summary.net_liquidation);
    println!("Buying Power: ${:.2}", summary.buying_power);
    println!("Unrealized P&L: ${:.2}", summary.unrealized_pnl);

    // Get current positions
    let positions = connector.get_positions().await?;
    for pos in positions {
        println!("{}: {} shares @ ${:.2} (P&L: ${:.2})",
            pos.symbol,
            pos.position,
            pos.avg_price,
            pos.unrealized_pnl
        );
    }

    Ok(())
}
```

## Running Tests

### Prerequisites

1. **Gateway Running**: Client Portal Gateway must be running on localhost:5000
2. **Authenticated**: Must have active authenticated session via browser
3. **Account ID**: Set `IB_ACCOUNT_ID` environment variable
4. **Market Data**: Active market data subscriptions for tested symbols

### Run Integration Tests

```bash
# Set your account ID
export IB_ACCOUNT_ID="DU12345"  # Paper trading account recommended

# Run tests (one at a time to respect rate limits)
cargo test --test ib_integration_tests -- --test-threads=1 --ignored

# Run specific test
cargo test --test ib_integration_tests test_get_price -- --ignored --nocapture
```

### Expected Test Behavior

- Tests will **skip** if Gateway is not running or not authenticated
- Some tests may **warn** if market data subscriptions are missing
- Tests use **AAPL** as default symbol (requires US stock market data subscription)
- Tests respect **rate limits** (10 req/s for Gateway)

## Configuration

### Gateway Configuration

Default Gateway configuration (`conf.yaml`):

```yaml
listenPort: 5000
listenSsl: true
sslCert: root/cacert.pem
sslPwd: ""
ips:
  allow:
    - 127.0.0.1
  deny: []
```

To change the port:

1. Edit `conf.yaml`: `listenPort: 5001`
2. Restart Gateway
3. Update connector URL: `https://localhost:5001/v1/api`

## Rate Limits

- **Gateway (Individual)**: 10 requests per second
- **OAuth (Enterprise)**: 50 requests per second
- **Specific Endpoints**:
  - `/tickle` (keep-alive): 1 req/s
  - `/iserver/account/{accountId}/orders`: 1 req/5s
  - `/iserver/marketdata/snapshot`: 10 req/s
  - `/iserver/marketdata/history`: 5 concurrent max

**Rate Limit Violations**:
- HTTP 429 error
- IP banned for 15 minutes
- Repeated violations may result in permanent ban

## Contract ID (conid) Resolution

IB uses Contract IDs (conid) instead of symbols for all operations. The connector automatically:

1. **Searches** for the contract using the provided symbol
2. **Caches** the conid for future requests
3. **Reuses** cached conids to avoid repeated searches

Example:
- Symbol: `AAPL` → Contract Search → conid: `265598` → Cached
- Future requests for `AAPL` use cached conid `265598`

## Session Management

Gateway sessions timeout after ~6 minutes of inactivity. To maintain the session:

1. The connector calls `/tickle` periodically (handled internally)
2. Manual tickle: `curl -k https://localhost:5000/v1/api/tickle`

Session constraints:
- **Single session per username**: Only one active session at a time
- **Maximum duration**: 24 hours
- **Auto-reset**: Midnight in account timezone

## Supported Asset Types

The connector currently maps all IB asset types to `AccountType::Spot`:

- **Stocks (STK)**: Regular stocks (AAPL, MSFT, etc.)
- **Forex (CASH)**: FX pairs (will be supported in future update)
- **Futures (FUT)**: Commodity/index futures (will be supported in future update)
- **Options (OPT)**: Stock/index options (will be supported in future update)

Future updates will add proper support for derivatives with specific account types.

## Limitations

### Current Implementation

1. **Manual Authentication**: Individual accounts require manual browser login (cannot be automated)
2. **Read-Only**: Trading operations not yet implemented (only market data and account queries)
3. **WebSocket**: Full WebSocket support pending
4. **Limited Asset Types**: Only stocks tested; forex/futures/options coming soon

### IB API Limitations

1. **Market Data**: Requires active subscriptions (typically 100 concurrent)
2. **Geographic**: Canadian residents cannot trade Canadian exchanges programmatically
3. **Single Session**: Only one session per username
4. **Order Types**: Not all TWS API order types available in Client Portal API

## Troubleshooting

### "Not authenticated" Error

**Cause**: Gateway session not established or expired

**Solutions**:
1. Open browser to `https://localhost:5000` and login
2. Check auth status: `curl -k https://localhost:5000/v1/api/iserver/auth/status`
3. Initialize session: `curl -X POST -k https://localhost:5000/v1/api/iserver/auth/ssodh/init`

### "Symbol not found" Error

**Cause**: Symbol search returned no results

**Solutions**:
1. Verify symbol is correct (use IB's symbol, not generic ticker)
2. Specify correct security type (currently defaults to STK)
3. Check if you have market data subscription for that exchange

### "Market data unavailable" Error

**Cause**: No active market data subscription

**Solutions**:
1. Ensure you have subscribed to market data for the exchange
2. Sign required market data agreements in Account Management
3. Use delayed data if real-time is not needed

### SSL Certificate Errors

**Cause**: Self-signed Gateway certificate

**Solution**: This is expected for localhost Gateway. The connector automatically disables SSL verification for localhost.

### Rate Limit Exceeded (HTTP 429)

**Cause**: Exceeded 10 req/s (Gateway) or 50 req/s (OAuth)

**Solutions**:
1. Add delays between requests
2. Implement request batching
3. Wait 15 minutes for IP ban to lift

## Resources

- [Client Portal Web API Documentation](https://interactivebrokers.github.io/cpwebapi/)
- [IBKR Campus API Page](https://www.interactivebrokers.com/campus/ibkr-api-page/cpapi-v1/)
- [WebSocket Streaming Guide](https://www.interactivebrokers.com/campus/trading-lessons/websockets/)
- [Gateway Setup Guide](https://www.interactivebrokers.com/campus/trading-lessons/launching-and-authenticating-the-gateway/)
- [Order Types Reference](https://www.interactivebrokers.com/campus/ibkr-api-page/order-types/)

## Contributing

When extending the IB connector:

1. **Follow V5 patterns**: Reference `src/exchanges/kucoin/` for structure
2. **Parse IB field IDs**: Map numeric field IDs to domain types
3. **Handle confirmation flow**: Some orders require explicit confirmation
4. **Cache conids**: Always cache symbol → conid mappings
5. **Respect rate limits**: Implement proper rate limiting
6. **Test with paper trading**: Use paper trading account (DU prefix) for testing

## License

See project root LICENSE file.
