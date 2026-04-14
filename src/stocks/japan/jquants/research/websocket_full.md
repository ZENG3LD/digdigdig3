# JQuants - WebSocket Documentation

## Availability: No

JQuants API does **NOT** provide WebSocket streaming capabilities.

## Why No WebSocket?

JQuants is a **data-only API** focused on historical and delayed data delivery for individual investors and researchers. The service design principles:

1. **Data Archive Focus**: Provides historical data for analysis, backtesting, and research
2. **Delayed Data Model**: Free tier has 12-week delay; even paid tiers focus on end-of-day data
3. **REST API Polling**: All data accessed via REST endpoints with daily/weekly update schedules
4. **Target Audience**: Individual investors, researchers, quants doing historical analysis

## Real-time Data Limitations

- **No tick-by-tick streaming**: Even with the new tick data add-on (Jan 2026), data is accessed via REST API, not WebSocket
- **No real-time order book**: No L2 depth streaming
- **No trade stream**: Trade data available via REST API only
- **Daily updates**: Most data updated once per day at scheduled times (see update schedule below)

## Data Update Schedule (REST API Polling)

Since there's no WebSocket, clients must poll REST endpoints. Update timing:

| Data Type | Update Time (JST) | Polling Recommendation |
|-----------|-------------------|------------------------|
| Daily stock prices | 16:30 | Poll once after 16:30 |
| Morning session prices | 12:00 | Poll once after 12:00 (Premium only) |
| Financial statements | 18:00 (prelim), 24:30 (final) | Poll twice daily |
| Indices | 16:30 | Poll once after 16:30 |
| Futures/Options | 27:00 (3:00 AM next day) | Poll once after 03:00 |
| Short selling data | 16:30 | Poll once after 16:30 |
| Trading by investor type | Thursday 18:00 | Poll weekly |
| Margin trading | Tuesday 16:30 | Poll weekly |
| Dividend announcements | 12:00-19:00 hourly | Poll hourly during window |
| Earnings calendar | ~19:00 | Poll daily evening |

## Alternative for Real-time Data

For real-time Japanese stock market data, consider:

1. **Direct exchange feeds**: Tokyo Stock Exchange (TSE) official market data feeds (enterprise)
2. **Bloomberg/Reuters**: Professional data terminals
3. **Japanese brokers**: Some provide real-time quotes via their APIs (e.g., Rakuten Securities, SBI Securities)
4. **Market data vendors**: Providers specializing in Asia-Pacific real-time feeds

## JQuants Use Cases

Given no WebSocket, JQuants is ideal for:

- Historical data analysis
- Backtesting trading strategies
- Fundamental analysis using financial statements
- Research projects
- Learning quantitative finance
- End-of-day trading systems
- Weekly/monthly portfolio rebalancing

## Not Suitable For

- High-frequency trading (HFT)
- Intraday algorithmic trading requiring real-time data
- Real-time market monitoring dashboards
- Order book analysis requiring streaming depth
- Latency-sensitive applications

## Future Possibilities

As of January 2026, no WebSocket support announced. The recent API V2 update (December 2025) focused on:
- CSV bulk downloads
- Minute/tick data via REST API
- Simplified authentication

No indication of WebSocket streaming in the roadmap.

## Workaround: REST API Polling Strategy

If you need near-real-time data from JQuants:

```rust
// Example polling strategy for daily price updates
async fn poll_daily_prices(client: &JQuantsClient) {
    // Wait until 16:30 JST (market close + 30 minutes)
    let update_time = "16:30 JST";

    // Poll once per day after update time
    let prices = client.get_daily_quotes(params).await?;

    // Process data
}

// For minute bars (add-on plan)
async fn poll_minute_bars(client: &JQuantsClient) {
    // Respect 60 req/min rate limit
    let interval = Duration::from_secs(1); // 1 second between requests

    loop {
        let bars = client.get_minute_bars(params).await?;
        tokio::time::sleep(interval).await;
    }
}
```

## Recommendation

For a V5 connector implementation:
- Implement REST-only client
- No need for WebSocket module
- Focus on efficient polling with rate limit handling
- Cache daily data (updates only once per day)
- Implement update schedule awareness to avoid unnecessary polling
