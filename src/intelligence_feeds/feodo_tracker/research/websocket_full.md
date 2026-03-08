# Feodo Tracker WebSocket Capabilities

## Real-Time Stream Support

**Status**: NOT AVAILABLE

Feodo Tracker does not provide WebSocket or streaming APIs. All data access is through HTTP downloads only.

## Data Delivery Method

**Polling-Based Architecture**:
- Static file generation every 5 minutes
- Clients must poll HTTP endpoints for updates
- No push notifications or event streams

## Recommended Polling Strategy

### Polling Interval
```
Recommended: Every 5-15 minutes
Minimum:     Every 15 minutes (per official docs)
Maximum:     Every 5 minutes (matches generation frequency)
```

### Implementation Approach

Since no real-time API exists, implement polling with:

1. **Timer-Based Polling**
   ```rust
   // Example pattern
   let interval = Duration::from_secs(5 * 60); // 5 minutes
   loop {
       fetch_blocklist().await;
       tokio::time::sleep(interval).await;
   }
   ```

2. **Conditional Requests**
   - Use `If-Modified-Since` HTTP header
   - Cache `Last-Modified` response header
   - Avoid re-downloading unchanged data

3. **ETag Support**
   - Check for `ETag` header in responses
   - Use `If-None-Match` in subsequent requests
   - Server returns 304 Not Modified if unchanged

## Alternative: Spamhaus Real-Time Feed

For organizations requiring true real-time updates, abuse.ch recommends contacting Spamhaus for their Botnet Controller List (BCL):

**Spamhaus BCL Features**:
- Real-time botnet C2 intelligence
- Multiple delivery formats (BGP, DNS, API)
- 800-2,500 active entries maintained
- Status re-evaluation multiple times per day
- Up to 50 new detections every 24 hours

**Access**: Requires contacting Spamhaus directly (not free/public)

## Why No WebSocket?

Feodo Tracker's design rationale for static files:
1. **Simplicity** - No client-side WebSocket library needed
2. **Reliability** - HTTP is universally supported
3. **Low Server Load** - Static file serving scales better
4. **Firewall Friendly** - No persistent connections required
5. **Caching** - CDN and HTTP cache friendly

## Connector Implementation

For the v5 Rust connector:

### Primary Interface
```rust
// No WebSocket struct needed
// Use REST client for polling
```

### Polling Service
```rust
pub struct FeodoTrackerPoller {
    client: reqwest::Client,
    interval: Duration,
    last_modified: Option<String>,
}

impl FeodoTrackerPoller {
    pub async fn poll_once(&mut self) -> Result<Option<Vec<C2Entry>>> {
        // Check if data changed using Last-Modified header
        // Return None if 304 Not Modified
        // Return Some(data) if new data available
    }
}
```

### Change Detection
```rust
// Store last_modified timestamp
// Only process new data when timestamp changes
// Reduces unnecessary parsing and memory allocation
```

## Data Freshness

Despite being polling-based:
- **Update Lag**: Maximum 5 minutes behind current state
- **Generation**: Automated every 5 minutes
- **Acceptable Delay**: For blocklist purposes, 5-15 minute lag is acceptable

## Event Types (Logical, Not Streamed)

While no streaming exists, logical event types can be derived from polling:

1. **C2_ADDED** - New IP appears in dataset
2. **C2_REMOVED** - IP no longer in dataset
3. **STATUS_CHANGED** - Status changes from online to offline
4. **METADATA_UPDATED** - last_online timestamp updated

Implementation: Compare current fetch with previous fetch to detect changes.

## Summary

| Feature | Status |
|---------|--------|
| WebSocket API | Not Available |
| Server-Sent Events | Not Available |
| Long Polling | Not Available |
| Webhooks | Not Available |
| HTTP Polling | ONLY METHOD |
| Recommended Interval | 5-15 minutes |
| Change Detection | HTTP headers (Last-Modified, ETag) |
| Real-time Alternative | Spamhaus BCL (paid) |

## Connector Decision

**websocket.rs**: NOT REQUIRED for Feodo Tracker connector

The connector will be REST-only with optional polling service implementation.
