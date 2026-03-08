# KRX Connector Integration Test Guide

## Overview

This guide explains how to test the KRX connector once the codebase compiles and you have API credentials.

## Prerequisites

### 1. API Credentials

You need to register for KRX API keys:

#### Option A: Data Marketplace API (Recommended)
1. Visit: https://openapi.krx.co.kr/
2. Create an account
3. Navigate to "My Page" (마이페이지)
4. Request API key ("API 인증키 신청")
5. Apply for specific services:
   - Securities Daily Trading Information (유가증권 일별 거래정보)
   - KOSDAQ Daily Trading Information (코스닥 일별 거래정보)
6. Wait for approval (up to 1 business day)

#### Option B: Public Data Portal (Government API)
1. Visit: https://www.data.go.kr/
2. Create an account
3. Find "KRX Listed Info Service"
4. Request API key (serviceKey)
5. Approval is typically quick (same day)

### 2. Environment Variables

Set the following environment variables:

```bash
# Option A: Data Marketplace
export KRX_API_KEY="your_marketplace_api_key_here"

# Option B: Public Data Portal
export KRX_DATA_PORTAL_KEY="your_portal_service_key_here"

# Or both for full functionality
export KRX_API_KEY="your_marketplace_api_key_here"
export KRX_DATA_PORTAL_KEY="your_portal_service_key_here"
```

Windows PowerShell:
```powershell
$env:KRX_API_KEY="your_marketplace_api_key_here"
$env:KRX_DATA_PORTAL_KEY="your_portal_service_key_here"
```

## Running Tests

### Run All Tests

```bash
cd zengeld-terminal/crates/connectors/crates/v5
cargo test --lib stocks::korea::krx::tests -- --ignored --nocapture
```

### Run Individual Tests

```bash
# Test basic connectivity
cargo test --lib stocks::korea::krx::tests::test_ping -- --ignored --nocapture --exact

# Test getting stock price (Samsung Electronics)
cargo test --lib stocks::korea::krx::tests::test_get_price -- --ignored --nocapture --exact

# Test getting historical OHLCV data
cargo test --lib stocks::korea::krx::tests::test_get_klines -- --ignored --nocapture --exact

# Test getting stock information
cargo test --lib stocks::korea::krx::tests::test_get_stock_info -- --ignored --nocapture --exact

# Test investor trading data
cargo test --lib stocks::korea::krx::tests::test_get_investor_trading -- --ignored --nocapture --exact
```

### Run Unit Tests (No API Required)

```bash
# Test number parsing
cargo test --lib stocks::korea::krx::tests::test_parse_krx_number -- --nocapture --exact

# Test symbol formatting
cargo test --lib stocks::korea::krx::tests::test_symbol_formatting -- --nocapture --exact

# Test date formatting
cargo test --lib stocks::korea::krx::tests::test_date_formatting -- --nocapture --exact
```

## Manual Testing

Create a test file `test_krx.rs`:

```rust
use digdigdig3::stocks::korea::KrxConnector;
use digdigdig3::core::traits::{ExchangeIdentity, MarketData};
use digdigdig3::core::types::{AccountType, Symbol};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create connector with environment credentials
    let connector = KrxConnector::from_env();

    println!("Exchange: {}", connector.exchange_name());
    println!("Exchange ID: {:?}", connector.exchange_id());

    // Test ping
    println!("\n=== Testing Ping ===");
    match connector.ping().await {
        Ok(_) => println!("✓ Ping successful"),
        Err(e) => println!("✗ Ping failed: {}", e),
    }

    // Test get price for Samsung Electronics (ticker: 005930)
    println!("\n=== Testing Get Price ===");
    let symbol = Symbol::new("005930", "");
    match connector.get_price(symbol.clone(), AccountType::Spot).await {
        Ok(price) => println!("✓ Samsung Electronics price: {} KRW", price),
        Err(e) => println!("✗ Failed to get price: {}", e),
    }

    // Test get klines
    println!("\n=== Testing Get Klines ===");
    match connector.get_klines(symbol.clone(), "1d", Some(5), AccountType::Spot).await {
        Ok(klines) => {
            println!("✓ Retrieved {} klines", klines.len());
            if !klines.is_empty() {
                let latest = &klines[0];
                println!("  Latest: O={} H={} L={} C={} V={}",
                    latest.open, latest.high, latest.low, latest.close, latest.volume);
            }
        }
        Err(e) => println!("✗ Failed to get klines: {}", e),
    }

    // Test get ticker
    println!("\n=== Testing Get Ticker ===");
    match connector.get_ticker(symbol.clone(), AccountType::Spot).await {
        Ok(ticker) => {
            println!("✓ Ticker for {}", ticker.symbol);
            println!("  Price: {}", ticker.last_price);
            if let Some(high) = ticker.high_24h {
                println!("  24h High: {}", high);
            }
            if let Some(low) = ticker.low_24h {
                println!("  24h Low: {}", low);
            }
            if let Some(vol) = ticker.volume_24h {
                println!("  Volume: {}", vol);
            }
        }
        Err(e) => println!("✗ Failed to get ticker: {}", e),
    }

    // Test extended methods
    println!("\n=== Testing Extended Methods ===");
    match connector.get_stock_info("005930").await {
        Ok(info) => println!("✓ Stock info: {:#}", info),
        Err(e) => println!("✗ Failed to get stock info: {}", e),
    }

    Ok(())
}
```

Run with:
```bash
cargo run --example test_krx
```

## Expected Results

### Success Scenario

```
Exchange: krx
Exchange ID: Krx

=== Testing Ping ===
✓ Ping successful

=== Testing Get Price ===
✓ Samsung Electronics price: 76200 KRW

=== Testing Get Klines ===
✓ Retrieved 5 klines
  Latest: O=75000 H=76500 L=74800 C=76200 V=12345678

=== Testing Get Ticker ===
✓ Ticker for 005930
  Price: 76200
  24h High: 76500
  24h Low: 74800
  Volume: 12345678

=== Testing Extended Methods ===
✓ Stock info: {
  "srtnCd": "005930",
  "isinCd": "KR7005930003",
  "itmsNm": "삼성전자",
  "mrktCtg": "KOSPI"
}
```

### Common Errors

#### 1. Authentication Error (401)

```
Error: API key not authorized for this service
```

**Solution**: Ensure your API key is approved for the specific service. Visit the service list and apply for required APIs.

#### 2. Rate Limit (429)

```
Error: Rate limit exceeded
```

**Solution**: Wait before retrying. Consider upgrading to paid tier for higher limits.

#### 3. No Credentials

```
Error: Request failed: ...
```

**Solution**: Set the `KRX_API_KEY` or `KRX_DATA_PORTAL_KEY` environment variable.

#### 4. Invalid Symbol

```
Error: Stock 'INVALID' not found
```

**Solution**: Use valid KRX ticker codes (e.g., "005930" for Samsung, "000660" for SK Hynix).

## Test Stocks

Common Korean stocks to test with:

| Ticker | ISIN | Name | Market |
|--------|------|------|--------|
| 005930 | KR7005930003 | Samsung Electronics | KOSPI |
| 000660 | KR7000660001 | SK Hynix | KOSPI |
| 035420 | KR7035420009 | NAVER | KOSPI |
| 051910 | KR7051910008 | LG Chem | KOSPI |
| 035720 | KR7035720002 | Kakao | KOSPI |

## Debugging

### Enable Detailed Logging

```bash
export RUST_LOG=digdigdig3=debug
cargo test --lib stocks::korea::krx::tests::test_get_price -- --ignored --nocapture
```

### Check API Response Manually

```bash
# Data Marketplace API
curl -X POST "http://data.krx.co.kr/comm/bldAttendant/getJsonData.cmd" \
  -H "Accept: application/json, text/javascript, */*; q=0.01" \
  -H "Content-Type: application/x-www-form-urlencoded; charset=UTF-8" \
  -H "X-API-Key: YOUR_API_KEY" \
  -d "bld=dbms/MDC/STAT/standard/MDCSTAT01501" \
  -d "locale=ko_KR" \
  -d "mktId=STK" \
  -d "trdDd=20260120"

# Public Data Portal API
curl "https://apis.data.go.kr/1160100/service/GetKrxListedInfoService/getItemInfo?serviceKey=YOUR_KEY&resultType=json&numOfRows=10&pageNo=1&likeSrtnCd=005930"
```

## Notes

- All KRX data is delayed by 1 business day minimum
- Data updates at 1:00 PM KST on business days
- Only daily candles are available (no intraday data)
- Numeric values in responses are comma-formatted strings
- Most field names and data are in Korean
- Trading operations are not supported (data provider only)
