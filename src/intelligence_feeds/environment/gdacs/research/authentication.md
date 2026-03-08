# GDACS Authentication

## Summary

**GDACS API requires NO authentication.** All endpoints are publicly accessible without API keys, tokens, or credentials.

## Authentication Details

### API Access
- **Type**: Public, open access
- **API Key**: Not required
- **OAuth**: Not supported
- **Bearer Token**: Not required
- **Basic Auth**: Not supported
- **Registration**: Not required

### HTTP Headers

**Minimal Headers**:
```http
GET /gdacsapi/api/events/geteventlist/SEARCH HTTP/1.1
Host: www.gdacs.org
Accept: application/json
User-Agent: YourAppName/1.0
```

**Recommended Headers**:
```http
GET /gdacsapi/api/events/geteventlist/SEARCH HTTP/1.1
Host: www.gdacs.org
Accept: application/json
Accept-Encoding: gzip, deflate
User-Agent: YourAppName/1.0 (contact@yourorg.org)
Connection: keep-alive
```

### User-Agent Best Practices

While not required, it's recommended to:
- Use a descriptive User-Agent string
- Include version and contact information
- Helps GDACS track API usage patterns
- May assist with troubleshooting

**Example**:
```
User-Agent: NemoTrading/1.0 GDACS-Monitor (https://github.com/yourorg/nemo)
```

## Rate Limiting

### Official Policy
**No documented rate limits** or authentication-based quotas.

### Recommended Usage Patterns

Based on RSS feed update frequency (6 minutes) and API characteristics:

**Conservative Approach**:
- Poll interval: 5-6 minutes
- Max requests per hour: 10-12
- Max concurrent connections: 1-2

**Moderate Approach**:
- Poll interval: 3-5 minutes
- Max requests per hour: 12-20
- Max concurrent connections: 2-3

**Avoid**:
- Sub-minute polling
- Hundreds of requests per hour
- Excessive concurrent connections
- Scraping historical data without pagination

### Self-Imposed Rate Limiting

```rust
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

pub struct RateLimiter {
    last_request: Mutex<Instant>,
    min_interval: Duration,
}

impl RateLimiter {
    pub fn new(min_interval: Duration) -> Self {
        Self {
            last_request: Mutex::new(Instant::now() - min_interval),
            min_interval,
        }
    }

    pub async fn wait_if_needed(&self) {
        let mut last = self.last_request.lock().await;
        let elapsed = last.elapsed();

        if elapsed < self.min_interval {
            let wait_time = self.min_interval - elapsed;
            tokio::time::sleep(wait_time).await;
        }

        *last = Instant::now();
    }
}

// Usage
let rate_limiter = RateLimiter::new(Duration::from_secs(300)); // 5 minutes

loop {
    rate_limiter.wait_if_needed().await;
    let events = fetch_gdacs_events().await;
    process_events(events).await;
}
```

## CORS (Cross-Origin Resource Sharing)

### Browser Access
**Not documented**, likely restricted for browser-based requests.

**Workaround for Web Applications**:
- Use server-side proxy
- CORS proxy services (for development only)
- Backend API that fetches GDACS data

### Native Applications
- No CORS restrictions for non-browser HTTP clients
- Rust connector will work without issues
- Mobile apps, desktop apps, servers: unrestricted

## IP Restrictions

**None documented**. API appears to be globally accessible without geographic or IP-based restrictions.

## Terms of Use

### Access Conditions

From GDACS Terms of Use (March 2025):
- Data is provided freely for public benefit
- No commercial restrictions explicitly stated
- Attribution required (see below)

### Attribution Requirements

**Required Attribution**:
```
Data provided by GDACS (Global Disaster Alert and Coordination System)
Source: https://www.gdacs.org/
```

**For Specific Disaster Types**:
- **Earthquakes**: "Data from USGS NEIC via GDACS"
- **Floods**: "Data from GLOFAS via GDACS"
- **Wildfires**: "Data from GWIS via GDACS"
- **Tropical Cyclones**: "Data from [Source Agency] via GDACS"
- **Volcanoes**: "Data from VAAC via GDACS"
- **Droughts**: "Data from GDO via GDACS"

### Usage Restrictions

**Allowed**:
- Real-time monitoring applications
- Research and analysis
- Humanitarian response tools
- Public awareness platforms
- Commercial applications (with attribution)
- Data aggregation and redistribution (with attribution)

**Important Disclaimer**:
From GDACS website:
> "This information is purely indicative and should not be used for any decision making without alternate sources of information."

**Best Practices**:
- Do not present GDACS data as definitive
- Recommend users verify with local authorities
- Include disclaimer in applications
- Cross-reference with other sources (USGS, local agencies)

## Rust Implementation

### Simple HTTP Client (No Auth)

```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct GdacsClient {
    client: Client,
    base_url: String,
}

impl GdacsClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("NemoTrading/1.0 GDACS-Monitor")
            .timeout(Duration::from_secs(30))
            .gzip(true)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            base_url: "https://www.gdacs.org/gdacsapi/api".to_string(),
        }
    }

    pub async fn get_events(&self, params: EventParams) -> Result<EventList, GdacsError> {
        let url = format!("{}/events/geteventlist/SEARCH", self.base_url);

        let response = self.client
            .get(&url)
            .query(&params) // Serialize params to query string
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(GdacsError::HttpError(response.status()));
        }

        let events = response.json::<EventList>().await?;
        Ok(events)
    }
}
```

### With Rate Limiting

```rust
pub struct GdacsClient {
    client: Client,
    base_url: String,
    rate_limiter: Arc<RateLimiter>,
}

impl GdacsClient {
    pub fn new_with_rate_limit(min_interval: Duration) -> Self {
        let client = Client::builder()
            .user_agent("NemoTrading/1.0 GDACS-Monitor")
            .timeout(Duration::from_secs(30))
            .gzip(true)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            base_url: "https://www.gdacs.org/gdacsapi/api".to_string(),
            rate_limiter: Arc::new(RateLimiter::new(min_interval)),
        }
    }

    pub async fn get_events(&self, params: EventParams) -> Result<EventList, GdacsError> {
        // Wait if necessary to respect rate limit
        self.rate_limiter.wait_if_needed().await;

        let url = format!("{}/events/geteventlist/SEARCH", self.base_url);

        let response = self.client
            .get(&url)
            .query(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(GdacsError::HttpError(response.status()));
        }

        let events = response.json::<EventList>().await?;
        Ok(events)
    }
}
```

## Error Handling

### HTTP Status Codes

**Observed Responses**:
- `200 OK`: Successful request, data returned (may be empty array)
- `404 Not Found`: Invalid endpoint path
- `500 Internal Server Error`: Server error (rare)
- `503 Service Unavailable`: Maintenance or overload (rare)

**No Authentication Errors**:
- No `401 Unauthorized` responses (no auth required)
- No `403 Forbidden` responses (no access restrictions)
- No `429 Too Many Requests` (no documented rate limiting)

### Retry Strategy

```rust
use backoff::{ExponentialBackoff, backoff::Backoff};

pub async fn fetch_with_retry(
    client: &GdacsClient,
    params: EventParams,
) -> Result<EventList, GdacsError> {
    let mut backoff = ExponentialBackoff {
        max_elapsed_time: Some(Duration::from_secs(300)), // 5 minutes max
        ..Default::default()
    };

    loop {
        match client.get_events(params.clone()).await {
            Ok(events) => return Ok(events),
            Err(e) if e.is_retryable() => {
                if let Some(wait) = backoff.next_backoff() {
                    tracing::warn!(
                        error = ?e,
                        retry_after = ?wait,
                        "GDACS request failed, retrying"
                    );
                    tokio::time::sleep(wait).await;
                } else {
                    return Err(e);
                }
            }
            Err(e) => return Err(e),
        }
    }
}
```

## Testing

### Mock Server for Tests

```rust
#[cfg(test)]
mod tests {
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path};

    #[tokio::test]
    async fn test_no_auth_required() {
        let mock_server = MockServer::start().await;

        // Setup mock - no Authorization header check
        Mock::given(method("GET"))
            .and(path("/gdacsapi/api/events/geteventlist/SEARCH"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                json!({
                    "type": "FeatureCollection",
                    "features": []
                })
            ))
            .mount(&mock_server)
            .await;

        let client = GdacsClient::new_with_base_url(mock_server.uri());
        let result = client.get_events(EventParams::default()).await;

        assert!(result.is_ok());
    }
}
```

## Security Considerations

### Data Integrity
- **No encryption required**, but HTTPS available (`https://www.gdacs.org`)
- Use HTTPS to prevent MITM attacks
- Verify SSL certificates

### Privacy
- No user data transmitted (public API)
- No tracking or analytics headers required
- User-Agent is optional but recommended

### Abuse Prevention
- Implement client-side rate limiting
- Use caching to reduce API calls
- Don't expose API proxy without rate limits
- Monitor usage patterns

## Future Authentication Possibilities

### If GDACS Introduces Authentication

**Likely Scenarios**:
1. **API Key**: Query parameter or header
2. **OAuth 2.0**: For user-specific data (unlikely)
3. **Rate Limit Tiers**: Free vs. premium access

**Preparation**:
```rust
pub enum AuthMethod {
    None,
    ApiKey(String),
    Bearer(String),
}

pub struct GdacsClient {
    client: Client,
    base_url: String,
    auth: AuthMethod,
}

impl GdacsClient {
    fn apply_auth(&self, req: RequestBuilder) -> RequestBuilder {
        match &self.auth {
            AuthMethod::None => req,
            AuthMethod::ApiKey(key) => req.query(&[("api_key", key)]),
            AuthMethod::Bearer(token) => req.bearer_auth(token),
        }
    }
}
```

## Summary

- **No authentication required** - fully public API
- **No rate limits documented** - use 5-6 minute polling
- **HTTPS available** - recommended for data integrity
- **Attribution required** - include GDACS credit
- **No CORS for browsers** - use server-side proxy if needed
- **No IP restrictions** - globally accessible
- **Implement self-imposed rate limiting** - be respectful of public resource
