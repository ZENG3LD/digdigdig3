# KRX - WebSocket Documentation

## Availability: No

## Status

**WebSocket streams are NOT available through the public KRX API.**

KRX (Korea Exchange) does not provide a public WebSocket API for real-time market data streaming. All data access is through:

1. **REST API (Data Marketplace)** - http://data.krx.co.kr/
2. **Public Data Portal API** - https://apis.data.go.kr/
3. **OTP-based bulk downloads** - For CSV/Excel data export

## Real-Time Data Access

For real-time market data from KRX, the following options exist:

### Third-Party Data Providers

**ICE (Intercontinental Exchange)**
- URL: https://developer.ice.com/fixed-income-data-services/catalog/korea-exchange-krx
- Provides: Streaming data for KRX equities, bonds, commodities, indices, futures, options
- Features: Level 1 and Level 2 pricing, full market depth, settlement values
- Access: Commercial license required
- Delivery: APIs and WebSocket connections available

**Twelve Data**
- Institutional-grade low latency (170ms) data streaming via WebSocket
- Commercial service
- Supports KRX exchange data

**EODHD (EOD Historical Data)**
- WebSocket-based real-time data
- Transport latency < 50ms
- Commercial service

### KRX Direct Market Data Feed

KRX offers direct market data feeds for institutional clients:
- Requires exchange membership or approved data vendor status
- High-frequency trading (HFT) capable
- Direct connection to exchange infrastructure
- Contact: KRX Data Sales Division

### KOSCOM Trading System

KOSCOM provides the IT infrastructure for KRX:
- Real-time market data available through KOSCOM
- Requires commercial agreement
- Access through trading platforms and APIs

## Data Latency via Public API

| Method | Latency | Use Case |
|--------|---------|----------|
| REST API (Data Marketplace) | +1 business day | Historical analysis, backtesting |
| Public Data Portal | +1 business day | Basic company information |
| OTP Bulk Download | +1 business day | Batch data processing |

**Update Schedule:** Data becomes available at 1:00 PM KST on the business day following the reference date.

## Alternative: Polling Pattern

Since WebSocket is not available, real-time-like updates can be achieved through:

### High-Frequency REST Polling
```python
# Example polling pattern (not recommended for production)
import time
import requests

def poll_krx_data(symbol, interval_seconds=60):
    """
    Poll KRX API at regular intervals
    Note: Respect rate limits, data is delayed anyway
    """
    while True:
        response = requests.post(
            'http://data.krx.co.kr/comm/bldAttendant/getJsonData.cmd',
            data={
                'bld': 'dbms/MDC/STAT/standard/MDCSTAT01701',
                'isuCd': symbol,
                # ... other params
            }
        )
        # Process response
        time.sleep(interval_seconds)
```

**Limitations:**
- Data is already delayed by 1+ business day
- Rate limits apply
- Not suitable for real-time trading
- Inefficient compared to WebSocket

## Recommended Approach

For applications requiring real-time KRX data:

1. **Use third-party providers** (ICE, Twelve Data) with WebSocket support
2. **Contact KRX directly** for institutional market data feed
3. **Use KOSCOM services** if you're a registered broker/trader
4. **Accept delayed data** for non-critical applications (historical analysis, research)

## Future Considerations

As of January 2026, KRX has been modernizing their API infrastructure:
- Recent introduction of API key authentication system
- Increased focus on Open API portal (openapi.krx.co.kr)
- Possible future WebSocket support (not announced)

Monitor https://openapi.krx.co.kr/ for updates on new API features.

## WebSocket Support Summary

| Feature | Status | Notes |
|---------|--------|-------|
| Public WebSocket API | Not Available | - |
| Real-time data via REST | Not Available | +1 day delay |
| Third-party WebSocket | Available | Commercial providers only |
| Direct exchange feed | Available | Institutional only |
| Future public WebSocket | Unknown | Not announced |
