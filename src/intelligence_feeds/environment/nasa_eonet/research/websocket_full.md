# NASA EONET WebSocket Support

## Summary

**No WebSocket support available.**

EONET v3 API is a RESTful HTTP-only API. Real-time event streaming is not provided through WebSocket connections.

## Alternative Approaches for Real-time Updates

### 1. Polling Strategy

Since EONET tracks natural disasters that evolve over hours/days (not milliseconds), HTTP polling is appropriate:

```
Recommended polling intervals:
- Wildfires: 15-30 minutes
- Severe storms: 10-15 minutes
- Volcanoes: 30-60 minutes
- Floods: 30-60 minutes
- Earthquakes: 5-10 minutes (if monitoring this category)
- General monitoring: 30 minutes
```

**Efficient polling endpoint**:
```
GET /api/v3/events?status=open&days=1&limit=100
```

This queries only recent events, reducing response size.

### 2. RSS/ATOM Feeds

EONET provides RSS and ATOM feeds, but these are still pull-based (not push).

**RSS Feed Example**:
```
https://eonet.gsfc.nasa.gov/api/v3/events.rss?status=open&days=7
```

**ATOM Feed Example**:
```
https://eonet.gsfc.nasa.gov/api/v3/events.atom?status=open&days=7
```

### 3. Incremental Updates

Use date filtering to fetch only new events since last check:

```
GET /api/v3/events?start=2026-02-16T10:00:00Z&status=all
```

Track last sync timestamp and query events newer than that timestamp on each poll.

### 4. Event Change Detection

Compare event `geometry` arrays on each poll:
- New geometry entries indicate event updates (e.g., wildfire spreading)
- `closed` field changing from `null` to a date indicates event closure
- `magnitudeValue` changes indicate event growth/shrinkage

## Implementation Recommendations

For the Rust connector:

1. **Implement polling mechanism** with configurable intervals
2. **Cache event IDs and geometry counts** to detect changes
3. **Use `days` parameter** to limit response size (e.g., `days=1` or `days=7`)
4. **Handle rate limits** gracefully (respect `X-RateLimit-Remaining` header)
5. **Emit change events** when:
   - New event appears
   - Existing event geometry updated
   - Event status changes (closed)
   - Magnitude values change

## Pseudo-Implementation

```rust
// Polling loop (conceptual)
loop {
    let events = fetch_events("status=open&days=1").await?;

    for event in events {
        if !cache.contains(&event.id) {
            emit_new_event(event);
            cache.insert(event.id, event.geometry.len());
        } else if cache.get(&event.id) != event.geometry.len() {
            emit_event_updated(event);
            cache.update(event.id, event.geometry.len());
        }

        if event.closed.is_some() {
            emit_event_closed(event);
            cache.remove(&event.id);
        }
    }

    sleep(poll_interval).await;
}
```

## Why No WebSocket?

EONET events are curated from multiple sources with varying update frequencies. Events typically:
- Evolve over hours/days (not seconds)
- Are manually reviewed before publication
- Come from sources with their own update cadences

This makes polling (every 15-60 minutes) more appropriate than persistent WebSocket connections.

## Future Considerations

If NASA adds WebSocket support in future API versions, connection details would be:
- Likely base URL: `wss://eonet.gsfc.nasa.gov/api/v3/stream`
- Subscription topics by category: `subscribe: { category: "wildfires" }`
- Event types: `new_event`, `event_updated`, `event_closed`

**Current status**: Not available as of v3.0 (February 2026)
