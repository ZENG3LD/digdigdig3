# GDACS WebSocket and Real-Time Capabilities

## Summary

**GDACS does NOT provide native WebSocket support.** The API is entirely REST-based with RSS/XML feeds as the primary real-time data delivery mechanism.

## Real-Time Data Delivery

### RSS Feed Polling (Primary Method)

**Update Frequency**: Every 6 minutes

**Mechanism**: HTTP polling of RSS/XML endpoints

**Recommended Polling Strategy**:
```
Poll interval: 5-6 minutes
Endpoint: https://www.gdacs.org/xml/rss_24h.xml (for all recent events)
```

**Advantages**:
- Simple to implement
- No connection management
- Standard RSS 2.0 format
- Multiple feed options (by type, severity, time)

**Disadvantages**:
- Not true real-time (6-minute lag)
- HTTP overhead for each poll
- Must check for duplicate events

### JSON API Polling (Alternative)

**Endpoint**: `https://www.gdacs.org/gdacsapi/api/events/geteventlist/SEARCH`

**Recommended Polling Strategy**:
```
Poll interval: 5-10 minutes
Parameters:
  - No date filter (gets latest events)
  - alertlevel=orange;red (exclude Green alerts)
  - pagesize=100 (max)
```

**Advantages**:
- Structured GeoJSON format
- Easier filtering
- Better for programmatic access
- Pagination support

**Disadvantages**:
- Same 6-minute update lag
- More parsing complexity than RSS

## Implementation Strategies

### 1. Periodic HTTP Polling (Recommended)

```rust
// Pseudo-code for Rust implementation
async fn poll_gdacs_events() {
    let mut last_event_ids = HashSet::new();

    loop {
        // Fetch latest events
        let events = fetch_events("https://www.gdacs.org/gdacsapi/api/events/geteventlist/SEARCH?alertlevel=orange;red").await;

        // Filter new events
        for event in events {
            if !last_event_ids.contains(&event.eventid) {
                // New event detected
                handle_new_event(event).await;
                last_event_ids.insert(event.eventid);
            }
        }

        // Cleanup old event IDs (keep last 1000)
        if last_event_ids.len() > 1000 {
            // Remove oldest
        }

        // Sleep for 5 minutes
        tokio::time::sleep(Duration::from_secs(300)).await;
    }
}
```

**Key Considerations**:
- Track event IDs to detect new events
- Use `eventid` + `episodeid` as unique key
- Check `datemodified` for updated events
- Implement exponential backoff on errors
- Cache recent event IDs to avoid reprocessing

### 2. RSS Feed Monitoring

```rust
// Pseudo-code for RSS monitoring
async fn monitor_rss_feed() {
    let feed_url = "https://www.gdacs.org/xml/rss_24h.xml";
    let mut last_build_date = None;

    loop {
        let rss = fetch_rss(feed_url).await;

        if rss.last_build_date != last_build_date {
            // Feed updated, parse new items
            for item in rss.items {
                if item.pub_date > last_build_date {
                    handle_new_event(item).await;
                }
            }
            last_build_date = rss.last_build_date;
        }

        tokio::time::sleep(Duration::from_secs(360)).await; // 6 minutes
    }
}
```

**RSS Fields for Change Detection**:
- `<lastBuildDate>`: Feed update timestamp
- `<pubDate>`: Individual item publication date
- `<guid>`: Unique event identifier

### 3. Webhook Integration (Third-Party Required)

GDACS does not provide webhooks directly. Options:

#### Option A: Use Third-Party Services
- **IFTTT**: May have GDACS RSS triggers
- **Zapier**: RSS feed monitoring
- **Integromat/Make**: RSS to webhook bridge

#### Option B: Self-Hosted RSS-to-Webhook Bridge
- Host a service that polls RSS feeds
- Converts RSS updates to webhook calls
- Examples: `rss-to-webhook`, custom Node.js/Python services

### 4. Event Change Detection

**Strategy**: Track event modifications

```rust
struct EventTracker {
    events: HashMap<(i64, i64), EventSnapshot>, // (eventid, episodeid) -> snapshot
}

impl EventTracker {
    fn check_for_changes(&mut self, new_events: Vec<Event>) -> Vec<EventChange> {
        let mut changes = Vec::new();

        for event in new_events {
            let key = (event.eventid, event.episodeid);

            match self.events.get(&key) {
                None => {
                    // New event
                    changes.push(EventChange::New(event.clone()));
                    self.events.insert(key, EventSnapshot::from(event));
                }
                Some(old) => {
                    // Check for modifications
                    if event.datemodified > old.datemodified {
                        changes.push(EventChange::Updated {
                            old: old.clone(),
                            new: event.clone(),
                        });
                        self.events.insert(key, EventSnapshot::from(event));
                    }

                    // Check for alert level changes
                    if event.alertlevel != old.alertlevel {
                        changes.push(EventChange::AlertChanged {
                            eventid: event.eventid,
                            old_level: old.alertlevel.clone(),
                            new_level: event.alertlevel.clone(),
                        });
                    }
                }
            }
        }

        changes
    }
}
```

**Key Change Indicators**:
- `datemodified`: Event data updated
- `alertlevel`: Severity changed
- `episodeid`: New episode of ongoing event
- `severitydata.severity`: Magnitude/impact changed
- `todate`: Event end time extended

## Latency Characteristics

### Data Source to GDACS
- **Earthquakes**: Near real-time (USGS NEIC feed, seconds to minutes)
- **Tropical Cyclones**: Updated every 6 hours (forecasts)
- **Floods**: Variable (GLOFAS model runs + manual validation)
- **Volcanoes**: Manual input (minutes to hours)
- **Wildfires**: Daily (GWIS updates)
- **Droughts**: Weekly/monthly (GDO updates)

### GDACS to API Consumer
- **RSS Feed Update**: Every 6 minutes
- **API Endpoint Refresh**: Assumed similar (6 minutes)
- **HTTP Polling Overhead**: 100-500ms per request

### Total Latency
- **Best Case**: 6-7 minutes (earthquake detected → RSS update → poll → notification)
- **Typical**: 10-15 minutes (including processing time)
- **Manual Events**: Hours to days (FL, VO)

## Scalability Considerations

### Single Consumer
- Poll every 5-6 minutes
- Track 100-200 events concurrently
- Minimal bandwidth (10-50 KB per poll)

### Multiple Consumers
- Implement shared cache layer
- Single poller feeds multiple clients via internal pub/sub
- Use WebSocket internally to distribute to clients

### High-Volume Applications
```
GDACS API (HTTP)
    ↓ (5-min poll)
Redis Cache + Pub/Sub
    ↓ (internal WebSocket)
Multiple Clients
```

**Architecture**:
1. Background service polls GDACS API
2. Detects changes, publishes to Redis
3. WebSocket server subscribes to Redis
4. Clients connect via WebSocket for instant updates

## Alternative Real-Time Sources

### Third-Party Aggregators
- **EONET (Earth Observatory Natural Event Tracker)**: NASA API with some GDACS data
- **Copernicus EMS**: European emergency management data
- **USGS Earthquake API**: Direct earthquake data (faster than GDACS for EQ)

### Complementary Services
- **Twitter/X API**: Monitor @GDACS account for alerts
- **Telegram**: GDACS bot may exist
- **Email Alerts**: GDACS may offer email subscriptions (check website)

## Recommendations for Rust Connector

### Implementation Approach
```rust
pub struct GdacsRealTimeClient {
    http_client: reqwest::Client,
    event_cache: Arc<RwLock<EventCache>>,
    poll_interval: Duration,
}

impl GdacsRealTimeClient {
    pub async fn start_monitoring(&self, tx: mpsc::Sender<EventUpdate>) {
        let mut interval = tokio::time::interval(self.poll_interval);

        loop {
            interval.tick().await;

            match self.fetch_and_compare_events().await {
                Ok(updates) => {
                    for update in updates {
                        let _ = tx.send(update).await;
                    }
                }
                Err(e) => {
                    eprintln!("Poll error: {}", e);
                    // Implement exponential backoff
                }
            }
        }
    }

    async fn fetch_and_compare_events(&self) -> Result<Vec<EventUpdate>> {
        // Fetch latest events
        let events = self.fetch_events().await?;

        // Compare with cache
        let mut cache = self.event_cache.write().await;
        let updates = cache.detect_changes(events);

        Ok(updates)
    }
}
```

### Configuration
```rust
pub struct RealTimeConfig {
    pub poll_interval: Duration,           // 5-6 minutes
    pub alert_levels: Vec<String>,         // ["orange", "red"]
    pub disaster_types: Vec<String>,       // ["EQ", "TC", "FL"]
    pub max_cached_events: usize,          // 1000
    pub retry_backoff: Duration,           // 30 seconds
    pub max_retries: u32,                  // 3
}
```

## Testing Real-Time Functionality

### Simulation
```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_event_change_detection() {
        let mut tracker = EventTracker::new();

        // First poll - new event
        let event1 = create_test_event(1, 1, "Orange");
        let changes = tracker.check_for_changes(vec![event1.clone()]);
        assert_eq!(changes.len(), 1);
        assert!(matches!(changes[0], EventChange::New(_)));

        // Second poll - alert level change
        let mut event2 = event1.clone();
        event2.alertlevel = "Red".to_string();
        event2.datemodified = Utc::now();

        let changes = tracker.check_for_changes(vec![event2]);
        assert_eq!(changes.len(), 2); // Updated + AlertChanged
    }
}
```

## Monitoring and Observability

### Metrics to Track
- Poll success/failure rate
- Average response time
- New events detected per hour
- Event update frequency
- Alert level distribution
- Latency (event timestamp → detection)

### Logging
```rust
tracing::info!(
    poll_duration_ms = ?duration.as_millis(),
    new_events = updates.len(),
    "GDACS poll completed"
);

tracing::warn!(
    error = ?e,
    retry_count = retries,
    "GDACS poll failed, retrying"
);
```

## Conclusion

While GDACS lacks native WebSocket support, effective real-time monitoring can be achieved through:

1. **5-6 minute HTTP polling** (matches RSS update frequency)
2. **Event change detection** using ID and modification timestamps
3. **Internal WebSocket layer** for distributing updates to multiple clients
4. **Exponential backoff** and error handling for reliability
5. **Caching** to reduce API load and improve response time

The 6-minute update cycle is inherent to GDACS infrastructure and cannot be improved without using direct data sources (e.g., USGS for earthquakes).
