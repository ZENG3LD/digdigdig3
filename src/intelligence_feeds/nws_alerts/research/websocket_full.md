# NWS Alerts API - WebSocket & Real-Time Capabilities

## WebSocket Support

**Status**: NOT AVAILABLE

The NWS Weather Alerts API does not provide WebSocket or Server-Sent Events (SSE) endpoints for real-time push notifications.

---

## Real-Time Data Strategy

### Polling Approach (Recommended)

Since WebSocket is not available, real-time alert monitoring requires HTTP polling.

**Recommended Polling Interval**: 30-60 seconds
- Aligns with NWS rate limit guidance (1 request per 30 seconds)
- Balances timeliness with server load
- Appropriate for weather alert lifecycle (alerts typically active for minutes/hours)

**Endpoint for Polling**:
```
GET /alerts/active
```

Or with geographic filtering:
```
GET /alerts/active/area/{state}
GET /alerts/active/zone/{zoneId}
GET /alerts/active?point=lat,lon
```

---

## Polling Implementation Patterns

### Pattern 1: Simple Interval Polling

```rust
// Pseudocode
loop {
    let alerts = fetch_active_alerts().await?;
    process_new_alerts(alerts);
    sleep(Duration::from_secs(30)).await;
}
```

**Pros**: Simple, predictable load
**Cons**: May miss very short-lived alerts

### Pattern 2: Cache-Based Polling

```rust
// Pseudocode
let mut last_modified = None;
loop {
    let response = fetch_with_headers(if_modified_since: last_modified).await?;
    if response.status == 304 {
        // No changes, skip processing
    } else {
        last_modified = response.headers.get("Last-Modified");
        process_alerts(response.body);
    }
    sleep(Duration::from_secs(30)).await;
}
```

**Pros**: Reduces processing when no changes
**Cons**: NWS may not support If-Modified-Since consistently

### Pattern 3: ID-Based Deduplication

```rust
// Pseudocode
let mut seen_alert_ids = HashSet::new();
loop {
    let alerts = fetch_active_alerts().await?;
    let new_alerts = alerts.filter(|a| !seen_alert_ids.contains(&a.id));
    for alert in new_alerts {
        seen_alert_ids.insert(alert.id);
        notify_user(alert);
    }
    // Cleanup expired alerts from set
    cleanup_old_ids(&mut seen_alert_ids, alerts);
    sleep(Duration::from_secs(30)).await;
}
```

**Pros**: Only processes truly new alerts
**Cons**: Requires state management

---

## Change Detection Strategies

### 1. Alert ID Tracking

Each alert has a unique `id` field:
```json
{
  "id": "https://api.weather.gov/alerts/urn:oid:2.49.0.1.840.0...",
  "properties": {
    "id": "urn:oid:2.49.0.1.840.0..."
  }
}
```

**Strategy**: Store set of active alert IDs, compare on each poll
**Detects**: New alerts, removed alerts (no longer in active set)

### 2. Message Type Detection

Alerts have `messageType` field:
- `Alert` - New alert issued
- `Update` - Existing alert updated
- `Cancel` - Alert cancelled

**Strategy**: Process all alerts, handle based on message type
**Detects**: Updates and cancellations to existing alerts

### 3. Sent Timestamp Comparison

Each alert has `sent` timestamp:
```json
{
  "properties": {
    "sent": "2026-02-16T02:01:00-07:00"
  }
}
```

**Strategy**: Track last poll time, only process alerts sent after that time
**Detects**: New alerts issued since last check

### 4. References Field for Updates

Updated/cancelled alerts reference previous versions:
```json
{
  "properties": {
    "references": [
      {
        "identifier": "urn:oid:...",
        "sender": "w-nws.webmaster@noaa.gov",
        "sent": "2026-02-16T00:00:00-07:00"
      }
    ]
  }
}
```

**Strategy**: Follow reference chains to track alert lifecycle
**Detects**: Alert evolution, superseded warnings

---

## Alert Lifecycle Events

### Events to Track

1. **New Alert Issued**
   - `messageType: "Alert"`
   - No `references` field
   - First appearance in active set

2. **Alert Updated**
   - `messageType: "Update"`
   - Contains `references` to previous version
   - Same event type, updated content

3. **Alert Cancelled**
   - `messageType: "Cancel"`
   - Contains `references` to cancelled alert
   - May still appear in active set briefly

4. **Alert Expired**
   - `expires` timestamp passed
   - Removed from `/alerts/active` endpoint
   - No longer appears in polling results

---

## Efficient Polling Techniques

### Geographic Filtering

Instead of polling all US alerts, filter by area of interest:

```
# State-level (reduces response size significantly)
GET /alerts/active/area/TX

# Zone-level (even smaller responses)
GET /alerts/active/zone/TXZ253

# Point-level (smallest, most targeted)
GET /alerts/active?point=29.4241,-98.4936
```

**Benefits**:
- Smaller payloads (faster transfer)
- Less processing required
- Lower bandwidth usage
- Reduced rate limit impact

### Conditional Requests

Use HTTP caching headers if supported:

```
GET /alerts/active
If-Modified-Since: Sat, 15 Feb 2026 12:00:00 GMT
```

**Expected Response**:
- `304 Not Modified` - No changes, skip processing
- `200 OK` - New data, process response

**Note**: NWS API caching behavior may vary; test before relying on this

### Batch Processing

If monitoring multiple states/zones:

```rust
// BAD - Sequential requests
for state in states {
    fetch_alerts(state).await;  // Slow, serial
}

// GOOD - Parallel requests (respecting rate limits)
let futures = states.map(|s| fetch_alerts(s));
let results = join_all(futures).await;
```

**Caution**: Ensure total request rate stays under 1 request per 30 seconds

---

## Notification Triggers

### Severity-Based

Trigger notifications based on severity level:
- `Extreme` - Immediate push notification, alarm
- `Severe` - High-priority notification
- `Moderate` - Standard notification
- `Minor` - Low-priority or background update

### Urgency-Based

Trigger based on time sensitivity:
- `Immediate` - Instant alert (life-threatening)
- `Expected` - Alert within the hour
- `Future` - Advance notice
- `Past` - Historical/informational only

### Event Type Based

Different handling for different events:
- Tornado Warning - Highest priority
- Severe Thunderstorm Warning - High priority
- Winter Storm Watch - Advance planning
- Special Weather Statement - Informational

### Geographic Relevance

Only notify if alert affects user's location:
1. Parse `geocode.UGC` or `geocode.SAME` arrays
2. Check if user's zone/county is listed
3. Or check if user's lat/lon is within alert geometry (if present)

---

## Alternative Real-Time Sources

While the NWS REST API doesn't support WebSocket, other options exist:

### 1. ATOM Feeds

NWS provides ATOM syndication feeds:
```
GET /alerts/active
Accept: application/atom+xml
```

**Use**: RSS readers, feed aggregators
**Update Frequency**: Poll every 1-5 minutes

### 2. CAP XML Feeds

Legacy XML format still available:
```
GET /alerts/active
Accept: application/cap+xml
```

**Use**: Emergency management systems expecting CAP XML
**Update Frequency**: Same as JSON endpoints

### 3. NOAA Weather Wire Service (NWWS)

**Separate Service**: Not part of api.weather.gov
**Protocol**: XMPP-based push messaging
**Access**: Requires separate registration
**Use Case**: Emergency operations centers requiring true real-time

**More Info**: https://www.weather.gov/nwws/

### 4. Third-Party Aggregators

Some commercial services aggregate NWS data and provide WebSocket/push:
- Weather Underground
- Weather.com
- OpenWeatherMap (has NWS integration)

**Caution**: Not official, may have delays or costs

---

## Rate Limiting Considerations

### Official Guidance
- **Max Rate**: 1 request per 30 seconds
- **Burst**: May be allowed briefly, but not sustainable
- **Penalty**: Rate-limiting firewall blocks, 5-second retry

### Conservative Polling
For production systems:
- **Interval**: 60 seconds (safer margin)
- **Geographic**: Filter to specific areas (reduces load)
- **Caching**: Implement local caching to reduce duplicate requests
- **User-Agent**: Set descriptive User-Agent to avoid blanket blocks

### Handling Rate Limits

```rust
// Pseudocode
match fetch_alerts().await {
    Err(RateLimited) => {
        sleep(Duration::from_secs(5)).await;
        retry_fetch().await?
    }
    Ok(data) => process(data),
}
```

---

## Polling vs WebSocket Comparison

### Why NWS Doesn't Offer WebSocket

1. **Government Infrastructure**: Legacy systems, procurement constraints
2. **Scalability**: HTTP + CDN easier to scale than WebSocket for millions of users
3. **Caching**: Alerts naturally cacheable, WebSocket bypasses cache
4. **Alert Lifecycle**: Weather events evolve slowly (minutes/hours), not milliseconds

### When Polling is Acceptable

Weather alerts are appropriate for polling because:
- **Update Frequency**: Alerts change on minute-scale, not second-scale
- **Criticality**: 30-60 second delay acceptable for non-tornado events
- **Stateless**: HTTP polling simpler to implement and debug

### When Real-Time is Critical

For applications requiring <5 second latency:
- Consider NOAA Weather Wire Service (NWWS) XMPP feed
- Implement aggressive polling (15-second intervals) with careful rate limit handling
- Use third-party services with WebSocket support

---

## Recommended Implementation

For a Rust data feed connector:

1. **Use tokio interval timer** for 30-second polling
2. **Filter by geography** to minimize payload
3. **Track alert IDs** in HashSet for deduplication
4. **Parse `messageType`** to detect updates/cancellations
5. **Respect rate limits** with exponential backoff
6. **Cache responses** locally to detect changes
7. **Provide callbacks** for new/updated/cancelled alerts
8. **Monitor `expires`** field to auto-remove stale alerts

This gives near-real-time updates (30-60 sec latency) while respecting NWS infrastructure limits.
