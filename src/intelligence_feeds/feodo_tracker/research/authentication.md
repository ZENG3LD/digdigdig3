# Feodo Tracker Authentication

## Authentication Required

**NO** - Feodo Tracker is a completely public API with no authentication requirements.

## Access Method

All endpoints are accessible via anonymous HTTP GET requests:

```http
GET https://feodotracker.abuse.ch/downloads/ipblocklist.json
```

No headers, tokens, or credentials needed.

## API Key

**Not Required** - No API key system exists.

## Rate Limiting Mechanism

No API key-based rate limiting. Access is controlled by:
1. Fair use expectations
2. Server-side abuse detection (if any)
3. Standard HTTP best practices

## Request Headers

### Required Headers

**None** - Minimum viable request:
```http
GET /downloads/ipblocklist.json HTTP/1.1
Host: feodotracker.abuse.ch
```

### Recommended Headers

For good HTTP client behavior:

```http
GET /downloads/ipblocklist.json HTTP/1.1
Host: feodotracker.abuse.ch
User-Agent: your-application/1.0.0
Accept: application/json
Accept-Encoding: gzip, deflate
If-Modified-Since: Wed, 15 Feb 2026 12:00:00 GMT
```

### User-Agent

While not required, it's good practice to identify your application:
```
User-Agent: nemo-trading/1.0.0 (Rust connector for Feodo Tracker)
```

## Conditional Requests

Use these headers to avoid re-downloading unchanged data:

### If-Modified-Since
```http
If-Modified-Since: Wed, 15 Feb 2026 12:00:00 GMT
```
Server returns `304 Not Modified` if data hasn't changed.

### If-None-Match
```http
If-None-Match: "abc123def456"
```
Use with ETag value from previous response.

## Response Headers to Track

### Last-Modified
```http
Last-Modified: Wed, 15 Feb 2026 12:05:00 GMT
```
Store this value for next request's `If-Modified-Since`.

### ETag
```http
ETag: "abc123def456"
```
Store this value for next request's `If-None-Match`.

## Rust Implementation

### HTTP Client Configuration

```rust
use reqwest::Client;

pub struct FeodoTrackerClient {
    client: Client,
    base_url: String,
    last_modified: Option<String>,
    etag: Option<String>,
}

impl FeodoTrackerClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("nemo-trading/1.0.0")
            .gzip(true)
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();

        Self {
            client,
            base_url: "https://feodotracker.abuse.ch".to_string(),
            last_modified: None,
            etag: None,
        }
    }

    pub async fn fetch_blocklist(&mut self) -> Result<Option<Vec<C2Entry>>> {
        let mut request = self.client
            .get(&format!("{}/downloads/ipblocklist.json", self.base_url));

        // Add conditional request headers
        if let Some(ref last_mod) = self.last_modified {
            request = request.header("If-Modified-Since", last_mod);
        }
        if let Some(ref etag) = self.etag {
            request = request.header("If-None-Match", etag);
        }

        let response = request.send().await?;

        // Handle 304 Not Modified
        if response.status() == StatusCode::NOT_MODIFIED {
            return Ok(None);
        }

        // Store new cache headers
        if let Some(last_mod) = response.headers().get("Last-Modified") {
            self.last_modified = Some(last_mod.to_str()?.to_string());
        }
        if let Some(etag) = response.headers().get("ETag") {
            self.etag = Some(etag.to_str()?.to_string());
        }

        // Parse response
        let data = response.json::<Vec<C2Entry>>().await?;
        Ok(Some(data))
    }
}
```

## Authentication Struct

For v5 connector architecture consistency, create minimal auth module:

```rust
// auth.rs
pub struct FeodoTrackerAuth;

impl FeodoTrackerAuth {
    pub fn new() -> Self {
        Self
    }

    // No-op: no signing needed
    pub fn sign_request(&self, _request: &mut reqwest::Request) {
        // Do nothing - public API
    }
}
```

## Security Considerations

### HTTPS
- All endpoints use HTTPS (TLS)
- Certificates validated by HTTP client
- No plaintext transmission

### No Credentials to Protect
- No API keys to secure
- No authentication tokens to rotate
- No secrets to manage

### Abuse Prevention
- Follow recommended polling intervals (5-15 minutes)
- Respect HTTP cache headers
- Use conditional requests to reduce bandwidth
- Identify your application with User-Agent

## Usage Limits

### Official Policy

From the terms of use:
- **Commercial Use**: Permitted without limitations
- **Non-Commercial Use**: Permitted without limitations
- **License**: CC0 (Creative Commons Zero)
- **Rate Limits**: None explicitly documented

### Best Practices

Even without enforced limits:
1. Poll no faster than every 5 minutes (matches generation frequency)
2. Use conditional requests to avoid unnecessary transfers
3. Implement exponential backoff on errors
4. Cache responses appropriately
5. Don't hammer the server on startup

## Error Handling

### HTTP Status Codes

- `200 OK` - Success, process response
- `304 Not Modified` - No change, use cached data
- `404 Not Found` - Invalid endpoint
- `429 Too Many Requests` - (Unlikely but handle gracefully)
- `500+ Server Error` - Retry with backoff

### Retry Strategy

```rust
// On error, exponential backoff
let retry_delays = [30, 60, 120, 300]; // seconds
for delay in retry_delays {
    tokio::time::sleep(Duration::from_secs(delay)).await;
    match fetch_blocklist().await {
        Ok(data) => return Ok(data),
        Err(e) => continue,
    }
}
```

## Summary

| Feature | Status |
|---------|--------|
| Authentication | None |
| API Key | Not Required |
| Bearer Token | Not Required |
| OAuth | Not Available |
| Rate Limiting | None (documented) |
| HTTPS | Required (default) |
| Headers Required | None (User-Agent recommended) |
| Caching | Use Last-Modified / ETag |
| Best Practice Polling | Every 5-15 minutes |
