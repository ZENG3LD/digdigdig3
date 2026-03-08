# Hacker News Firebase API - Real-Time Streaming (WebSocket/SSE)

## Overview

The Hacker News API is built on Firebase Realtime Database, which supports real-time data synchronization through multiple protocols. While the API can be accessed via standard REST calls, Firebase also provides streaming capabilities for live updates.

## Streaming Protocols

### Server-Sent Events (SSE)
Firebase REST API supports **Server-Sent Events (EventSource)** for streaming changes to Firebase locations.

**Protocol**: SSE over HTTP
**Direction**: Server → Client (unidirectional)
**Use Case**: Receiving live updates when HN data changes

### WebSocket (via Firebase SDKs)
Firebase native SDKs (Web, iOS, Android) use WebSocket connections under the hood for bidirectional real-time sync.

**Protocol**: WebSocket
**Direction**: Bidirectional
**Use Case**: Full-featured real-time applications with presence and offline support

## REST Streaming with SSE

### Endpoint Format
Any Firebase REST endpoint can be streamed by changing the HTTP headers:

```
GET https://hacker-news.firebaseio.com/v0/{resource}.json
Accept: text/event-stream
```

### Connection Setup

**Requirements**:
1. Set `Accept` header to `text/event-stream`
2. Respect HTTP redirects (especially 307 Temporary Redirect)
3. Include `auth` query parameter if authentication required (not needed for HN)

**Example Request**:
```http
GET /v0/item/8863.json HTTP/1.1
Host: hacker-news.firebaseio.com
Accept: text/event-stream
```

### Event Types

Firebase SSE streams send named events as data changes:

#### 1. `put` Event
Sent when data is set or updated at the location.

**Format**:
```
event: put
data: {"path": "/", "data": {"by": "dhouston", "id": 8863, "score": 111, ...}}
```

**Fields**:
- `path`: JSON path within the resource that changed (usually `/` for full object)
- `data`: The new data value

**Example** (story score update):
```
event: put
data: {"path": "/score", "data": 115}
```

#### 2. `patch` Event
Sent when a subset of data changes (partial update).

**Format**:
```
event: patch
data: {"path": "/", "data": {"score": 120, "descendants": 45}}
```

#### 3. `keep-alive` Event
Periodic heartbeat to keep connection alive.

**Format**:
```
event: keep-alive
data: null
```

Sent every ~30 seconds of inactivity.

#### 4. `cancel` Event
Sent if the client no longer has permission to read the location (not applicable to HN).

**Format**:
```
event: cancel
data: null
```

### Streaming Individual Items

**Use Case**: Monitor a specific story's score and comment count in real-time.

**Endpoint**:
```
GET /v0/item/{id}.json
Accept: text/event-stream
```

**Example** (monitoring story 8863):
```
GET /v0/item/8863.json
Accept: text/event-stream
```

**Response Stream**:
```
event: put
data: {"path": "/", "data": {"by": "dhouston", "descendants": 71, "id": 8863, "kids": [8952, 9224], "score": 111, "time": 1175714200, "title": "My YC app: Dropbox", "type": "story", "url": "http://www.getdropbox.com/u/2/screencast.html"}}

event: put
data: {"path": "/score", "data": 115}

event: put
data: {"path": "/descendants", "data": 72}

event: keep-alive
data: null
```

### Streaming Story Lists

**Use Case**: Get notified immediately when new stories appear on the front page.

**Endpoint**:
```
GET /v0/topstories.json
Accept: text/event-stream
```

**Response Stream**:
```
event: put
data: {"path": "/", "data": [39427470, 39426838, 39425902, ...]}

event: put
data: {"path": "/", "data": [39428001, 39427470, 39426838, ...]}
```

The entire array is replaced each time the ranking changes.

### Streaming Max Item ID

**Use Case**: Detect new items as soon as they're posted.

**Endpoint**:
```
GET /v0/maxitem.json
Accept: text/event-stream
```

**Response Stream**:
```
event: put
data: {"path": "/", "data": 39427999}

event: put
data: {"path": "/", "data": 39428000}

event: put
data: {"path": "/", "data": 39428001}
```

Increments by 1 each time a new item is created (story, comment, job, poll, etc.).

## Real-Time Update Frequency

### Story Lists
- **Top Stories**: Updates every 1-5 minutes (algorithm re-ranking)
- **New Stories**: Updates every 5-30 seconds (as items are posted)
- **Best Stories**: Updates every 5-10 minutes

### Individual Items
- **Score**: Updates within seconds of voting
- **Comments (`kids`)**: Updates immediately when new comments posted
- **Descendants**: Updates immediately when comment count changes
- **Deleted/Dead**: Updates immediately when moderation occurs

### Max Item ID
- Updates every 1-10 seconds (as users post stories, comments, etc.)
- Fastest-changing resource in the API

## Connection Management

### Connection Lifecycle
1. **Initial Snapshot**: First event is always a `put` with current data
2. **Live Updates**: Subsequent events reflect changes
3. **Heartbeat**: `keep-alive` events every ~30 seconds
4. **Reconnection**: If connection drops, client must reconnect and handle new snapshot

### Reconnection Strategy
- Use exponential backoff: 1s, 2s, 4s, 8s, 16s, max 30s
- Handle 307 redirects (Firebase load balancing)
- Reset backoff on successful connection lasting >60s

### Connection Pooling
- Recommend max 10 concurrent SSE connections
- Reuse connections for multiple resource subscriptions if possible
- Firebase may throttle excessive connections from single IP

## Rust Implementation Considerations

### SSE Libraries
Recommended Rust crates:
- `eventsource-client`: Async SSE client with reconnection logic
- `reqwest-eventsource`: SSE support built on reqwest
- Manual implementation with `reqwest::get()` and streaming response body

### Example Pattern (Conceptual)
```rust
use eventsource_client as es;

let client = es::ClientBuilder::for_url("https://hacker-news.firebaseio.com/v0/maxitem.json")
    .header("Accept", "text/event-stream")?
    .build();

client
    .stream()
    .try_for_each(|event| async move {
        match event {
            es::SSE::Event(ev) => {
                match ev.event_type.as_str() {
                    "put" => {
                        let data: serde_json::Value = serde_json::from_str(&ev.data)?;
                        // Handle new max item ID
                    }
                    "keep-alive" => { /* Heartbeat */ }
                    _ => {}
                }
            }
            es::SSE::Comment(_) => { /* Ignore */ }
        }
        Ok(())
    })
    .await?;
```

## Limitations

### No Binary Protocol
Firebase Realtime Database does not use binary WebSocket protocols (like MessagePack or Protobuf). All data is JSON text.

### No Multiplexing
Each SSE connection streams a single resource. To monitor multiple items:
- Option 1: Open multiple SSE connections (up to ~10 recommended)
- Option 2: Poll `/v0/updates.json` periodically and fetch changed items via REST

### No Filtering
SSE streams send all updates to the resource. Cannot filter server-side (e.g., "only score > 100"). Must filter client-side.

### No Historical Events
SSE streams only send current state + future changes. Past events before connection are not replayed.

## Comparison: SSE vs REST Polling

| Feature | SSE Streaming | REST Polling |
|---------|---------------|--------------|
| Latency | Near real-time (<1s) | Polling interval (e.g., 30s) |
| Overhead | Persistent connection | Repeated HTTP handshakes |
| Data Efficiency | Only changes sent | Full object each time |
| Complexity | Higher (event handling, reconnection) | Lower (simple loop) |
| Connection Limit | ~10 concurrent recommended | Unlimited |
| Use Case | Live monitoring | Periodic updates |

## Recommended Strategy for HN Connector

### For Live Monitoring
Use SSE streaming for:
- `/v0/maxitem.json` (detect new items immediately)
- Individual story monitoring (if user pins specific stories)

### For Bulk Operations
Use REST polling for:
- `/v0/topstories.json` (fetch top 30-100 stories every 5 minutes)
- `/v0/newstories.json` (fetch new stories every 30 seconds)
- Individual item details (concurrent batch fetch)

### Hybrid Approach
1. Stream `/v0/maxitem.json` to detect new items in real-time
2. Fetch new item details via REST: `GET /v0/item/{new_id}.json`
3. Poll story lists every 5 minutes via REST
4. Cache item details (immutable fields never change)

## Alternative: Firebase SDK

For full real-time features (presence, offline, transactions), use the official Firebase SDK instead of raw SSE. However, this adds dependency complexity and is overkill for simple HN data feeds.

**Trade-off**:
- **SSE/REST**: Lightweight, simple, sufficient for read-only HN feed
- **Firebase SDK**: Full-featured, but heavy dependency (JavaScript runtime or native bindings)

## Summary

- **Protocol**: SSE over HTTP (REST streaming)
- **Endpoint Format**: Same as REST, add `Accept: text/event-stream` header
- **Event Types**: `put`, `patch`, `keep-alive`, `cancel`
- **Update Frequency**: Near real-time (seconds for most resources)
- **Rust Implementation**: Use `eventsource-client` or `reqwest-eventsource`
- **Recommended Usage**: Stream `/v0/maxitem.json`, poll story lists, REST fetch items
- **Connection Limit**: ~10 concurrent SSE connections recommended
