# Feodo Tracker Tiers and Limits

## Pricing Tiers

**FREE TIER ONLY** - Feodo Tracker is a completely free service with no paid tiers.

### Free Tier Features
- Unlimited access to all blocklists
- All formats (JSON, CSV, TXT, rules)
- All historical data
- Commercial use permitted
- Non-commercial use permitted
- No registration required
- No API key needed

## Rate Limits

### Official Rate Limits

**None Explicitly Documented** - No hard rate limits are published in the documentation.

### Recommended Usage

According to official documentation:

**Update Frequency**:
```
Generation Interval: Every 5 minutes
Recommended Poll Rate: Every 5-15 minutes minimum
Optimal Poll Rate: Every 5 minutes
```

### Practical Limits

While no enforced rate limits exist, respect these guidelines:

1. **Minimum Poll Interval**: 5 minutes
   - Data only updates every 5 minutes
   - Polling faster provides no benefit
   - Wastes bandwidth and server resources

2. **Maximum Request Rate**: No documented limit
   - Use conditional requests (If-Modified-Since)
   - Implement client-side caching
   - Don't abuse the free service

3. **Concurrent Connections**: Not specified
   - Use single connection per client instance
   - No need for concurrent requests to same endpoint

## Usage Quotas

### Daily Limits
- **Requests per Day**: Unlimited (no documented limit)
- **Data Transfer**: Unlimited (no documented limit)
- **Endpoints Accessible**: All endpoints

### Monthly Limits
- **Requests per Month**: Unlimited
- **Cost**: Free

## HTTP Status Codes for Limits

If rate limiting is implemented server-side (undocumented):

- `429 Too Many Requests` - Theoretical but not observed
- `503 Service Unavailable` - Temporary overload

**Handling**: Implement exponential backoff for any 4xx/5xx errors.

## File Size Limits

### Download Sizes

When datasets are populated (estimated):
- JSON blocklist: 10-100 KB
- CSV blocklist: 10-100 KB
- TXT blocklist: 5-50 KB
- Aggressive blocklist: 50-200 KB
- Rules files: 50-500 KB

**Current Size**: All datasets empty (0 KB) as of February 2026.

### No Upload Capability

Feodo Tracker is read-only. No POST/PUT/DELETE endpoints exist.

## Data Freshness Guarantees

### Update Schedule
- **Generation Frequency**: Every 5 minutes
- **Maximum Staleness**: 5 minutes (if polling continuously)
- **Typical Staleness**: 5-15 minutes (with recommended polling)

### SLA / Uptime
- **Uptime Guarantee**: None (best effort)
- **Support**: Community forum / abuse.ch contact
- **Incident Notifications**: Not provided

## Commercial Use Restrictions

**NONE** - Explicitly allowed under CC0 license.

### Permitted Uses
- Commercial products
- Proprietary systems
- Revenue-generating services
- Resale/redistribution
- Integration into paid offerings

### Attribution
- **Required**: No
- **Appreciated**: Yes
- **Recommendation**: Credit abuse.ch/Feodo Tracker in documentation

## License Terms

**CC0 (Creative Commons Zero)**:
- Public domain dedication
- No rights reserved
- No attribution required
- No warranty provided

### Liability Disclaimer

From terms of use:
- Data provided "as is" on best effort basis
- No guarantee of accuracy
- No liability for false positives/negatives
- Use at your own risk

## Fair Use Policy

While no explicit fair use policy is documented, follow these principles:

### Acceptable Use
- Automated polling every 5-15 minutes
- Conditional requests (If-Modified-Since/ETag)
- Client-side caching
- Bulk downloads for offline analysis
- Integration into security products

### Discouraged Use
- Polling faster than every 5 minutes (wasteful)
- Ignoring cache headers
- Concurrent connections to same endpoint
- Distributed scraping from many IPs
- Intentional server stress testing

## Alternative Access Methods

### Spamhaus BCL (Paid Alternative)

For organizations needing:
- True real-time updates
- SLA guarantees
- Commercial support
- Higher data volume
- BGP/DNS integration

**Contact**: Spamhaus directly (not through abuse.ch)

### Self-Hosting

Data can be downloaded and cached locally:
- Mirror JSON files on your infrastructure
- Serve to internal clients
- Reduce external requests
- Control update frequency
- Add custom enrichment

## Connector Implementation Limits

For the v5 Rust connector, implement these safety limits:

```rust
pub struct FeodoTrackerLimits {
    pub min_poll_interval: Duration,      // 5 minutes
    pub request_timeout: Duration,        // 30 seconds
    pub max_retries: u32,                 // 3
    pub backoff_base: Duration,           // 30 seconds
    pub backoff_max: Duration,            // 5 minutes
}

impl Default for FeodoTrackerLimits {
    fn default() -> Self {
        Self {
            min_poll_interval: Duration::from_secs(5 * 60),
            request_timeout: Duration::from_secs(30),
            max_retries: 3,
            backoff_base: Duration::from_secs(30),
            backoff_max: Duration::from_secs(5 * 60),
        }
    }
}
```

## Error Rate Handling

No documented error rate limits, but implement:

### Client-Side Protections
1. **Request Timeout**: 30 seconds
2. **Connection Timeout**: 10 seconds
3. **Retry Logic**: Max 3 attempts
4. **Backoff**: Exponential (30s, 60s, 120s)
5. **Circuit Breaker**: After 5 consecutive failures, wait 5 minutes

### Health Checks
- Track success/failure rate
- Alert on sustained failures
- Fallback to cached data if available

## Burst Limits

**Not Applicable** - No burst limiting documented or observed.

You can request multiple endpoints simultaneously if needed:
- `/downloads/ipblocklist.json`
- `/downloads/ipblocklist.csv`
- `/downloads/feodotracker.rules`

However, for the connector, only poll the JSON endpoint.

## Summary

| Limit Type | Value |
|------------|-------|
| Pricing | Free (CC0 License) |
| API Key Required | No |
| Rate Limit (Requests/Min) | None documented |
| Rate Limit (Requests/Day) | None documented |
| Recommended Poll Interval | 5-15 minutes |
| Data Update Frequency | Every 5 minutes |
| Commercial Use | Allowed |
| Attribution | Not required |
| SLA / Uptime | None (best effort) |
| Support | Community |
| Maximum Dataset Size | ~100 KB (when populated) |
| Concurrent Requests | Not limited |
| Bulk Downloads | Allowed |
| Redistribution | Allowed |
