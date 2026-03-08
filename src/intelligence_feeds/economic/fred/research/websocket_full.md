# FRED - WebSocket Documentation

## Availability: No

The FRED API **does not support WebSocket connections**.

## Reasons for No WebSocket Support

1. **Data Nature**: Economic data from FRED is typically updated on scheduled intervals (daily, weekly, monthly, quarterly, annual) rather than requiring real-time streaming.

2. **API Architecture**: FRED uses a traditional REST architecture with HTTP/HTTPS requests only.

3. **Update Frequency**: Most economic indicators are released at fixed times (e.g., employment data on first Friday of month, GDP quarterly), making polling-based approaches sufficient.

## Alternative Approaches for Real-Time Updates

### Polling Strategy

Since economic data updates are predictable and infrequent:

1. **Check /fred/series/updates endpoint** periodically to discover recently updated series
   - Use `start_time` and `end_time` parameters to filter by update window
   - This endpoint shows series sorted by last observation update time

2. **Monitor Release Dates** via /fred/releases/dates
   - Economic releases follow published calendars
   - Check for new release dates and fetch data after scheduled release times

3. **Use /fred/series/vintagedates** for revision tracking
   - Track when data values are revised (important for ALFRED archival data)
   - Some series are revised weeks or months after initial release

### Recommended Polling Intervals

| Data Type | Recommended Polling | Endpoint |
|-----------|---------------------|----------|
| Daily indicators | Every 1-4 hours | /fred/series/observations |
| Weekly indicators | Daily (morning) | /fred/series/observations |
| Monthly indicators | Daily (post-release) | /fred/series/observations |
| Quarterly indicators | Weekly | /fred/series/observations |
| Check for updates | Every 15-30 minutes | /fred/series/updates |

### Rate Limit Considerations

With 120 requests/minute limit:
- You can monitor 120 series per minute
- For 1000 series: Full refresh every ~8-9 minutes
- Use /fred/series/updates to minimize unnecessary requests

## Third-Party Real-Time Alternatives

If you need actual real-time economic data streaming:

1. **Bloomberg Terminal** - Professional real-time economic data
2. **Refinitiv/Reuters** - Real-time economic news and data feeds
3. **Trading Economics API** - Some real-time economic indicators
4. **Econoday** - Economic calendar with real-time release notifications

**Note**: FRED remains the authoritative free source, but data appears with slight delay after official release times.

## Push Notification Workarounds

While FRED doesn't offer WebSockets, you could build:

1. **Email Alerts**: Some economic data providers send email alerts on releases
2. **RSS Feeds**: Monitor economic calendars via RSS
3. **Custom Polling Service**: Build a service that polls FRED and pushes to your app via WebSocket/SSE
4. **GitHub Actions/Cron Jobs**: Scheduled checks for new data

## Conclusion

FRED API is REST-only with no WebSocket support. The nature of economic data (scheduled releases, infrequent updates) makes this acceptable. Use intelligent polling based on release schedules and the /fred/series/updates endpoint.
