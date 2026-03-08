# NWS Alerts API - Authentication & Authorization

## Authentication Type

**Type**: NONE

The NWS Weather Alerts API is a **public, unauthenticated API**. No API keys, tokens, or credentials are required.

---

## Required Headers

### User-Agent (REQUIRED)

While no authentication is needed, a `User-Agent` header is **MANDATORY** for all requests.

**Header Name**: `User-Agent`

**Format**: Free-form string identifying your application

**Recommended Format**:
```
User-Agent: (myweatherapp.com, contact@myweatherapp.com)
```

**Examples**:
```
User-Agent: (ZengeldWeather/1.0, support@zengeld.com)
User-Agent: (MyWeatherApp, developer@example.com)
User-Agent: WeatherBot/2.1 (https://weatherbot.io)
User-Agent: Personal Weather Monitor (user@email.com)
```

**Purpose**:
- Application identification
- Contact point for NWS if issues arise
- Abuse prevention and tracking
- Security event isolation

**Uniqueness**:
The more unique your User-Agent, the less likely your application will be affected by rate limiting or security blocks targeting other applications.

**Consequences of Omission**:
Requests without a User-Agent header may be:
- Blocked by NWS servers
- Rate-limited more aggressively
- Unable to complete successfully

---

## Optional Headers

### Accept (Content Negotiation)

**Header Name**: `Accept`

**Purpose**: Specify desired response format

**Supported Values**:
- `application/geo+json` - GeoJSON format (default)
- `application/ld+json` - JSON-LD (linked data)
- `application/cap+xml` - CAP 1.2 XML format
- `application/atom+xml` - ATOM syndication format

**Example**:
```
Accept: application/geo+json
```

**Default**: If omitted, API returns GeoJSON format

---

### If-Modified-Since (Conditional Requests)

**Header Name**: `If-Modified-Since`

**Purpose**: Conditional request to avoid redundant data transfer

**Format**: HTTP date format
```
If-Modified-Since: Sat, 15 Feb 2026 12:00:00 GMT
```

**Expected Behavior**:
- `304 Not Modified` - Resource unchanged since specified time
- `200 OK` - Resource has been modified, new data returned

**Note**: NWS API support for this header may vary; test before relying on it for production caching

---

## Request Examples

### Minimal Valid Request

```http
GET /alerts/active HTTP/1.1
Host: api.weather.gov
User-Agent: (MyApp, contact@example.com)
```

### Full Request with Content Negotiation

```http
GET /alerts/active/area/TX HTTP/1.1
Host: api.weather.gov
User-Agent: (ZengeldTerminal/1.0, dev@zengeld.com)
Accept: application/geo+json
```

### Conditional Request with Caching

```http
GET /alerts/active HTTP/1.1
Host: api.weather.gov
User-Agent: (WeatherMonitor/2.1, admin@weather.com)
If-Modified-Since: Sun, 16 Feb 2026 08:00:00 GMT
Accept: application/geo+json
```

---

## Rust Implementation Example

### Basic Client Setup

```rust
use reqwest::Client;

pub struct NwsClient {
    client: Client,
    base_url: String,
    user_agent: String,
}

impl NwsClient {
    pub fn new(app_name: &str, contact: &str) -> Self {
        let user_agent = format!("({}, {})", app_name, contact);

        Self {
            client: Client::new(),
            base_url: "https://api.weather.gov".to_string(),
            user_agent,
        }
    }

    pub async fn fetch_active_alerts(&self) -> Result<Response, reqwest::Error> {
        self.client
            .get(&format!("{}/alerts/active", self.base_url))
            .header("User-Agent", &self.user_agent)
            .header("Accept", "application/geo+json")
            .send()
            .await
    }
}
```

### Usage

```rust
let client = NwsClient::new(
    "ZengeldTerminal/1.0",
    "support@zengeld.com"
);

let response = client.fetch_active_alerts().await?;
let alerts: AlertCollection = response.json().await?;
```

---

## Authorization Levels

**Public Access**: All endpoints are publicly accessible

**No Tiered Access**: Unlike many APIs, NWS does not have:
- Premium tiers
- Authenticated vs unauthenticated rate limits
- Feature gating based on API keys
- Usage quotas per user

**Government Open Data**: As a US government service, all data is public domain

---

## Rate Limiting Without Authentication

Since there's no authentication, rate limiting is based on:

1. **IP Address**: Source IP of requests
2. **User-Agent**: Application identification string
3. **Request Pattern**: Frequency, burst behavior

**Implications**:
- Shared IPs (corporate networks, NAT) may hit limits faster
- Unique User-Agent helps isolate your app from others
- Polite request patterns (30-60 sec intervals) reduce limit risk

---

## Security Considerations

### HTTPS Only

**Protocol**: HTTPS (TLS 1.2+)

**HTTP Requests**: Automatically upgraded to HTTPS

**Certificate Validation**: Use standard OS certificate store

```rust
// reqwest handles HTTPS by default
let client = Client::new();  // TLS enabled, cert validation on
```

### User-Agent Spoofing

**Risk**: Using generic or common User-Agent strings

**Problem**: If other apps using same User-Agent misbehave, your app may be blocked

**Solution**: Use unique, descriptive User-Agent with your contact info

### Data Integrity

**Consideration**: No request signing or authentication means:
- You cannot verify response authenticity cryptographically
- Rely on HTTPS/TLS for transport security
- Trust NWS domain certificate

**Mitigation**:
- Validate response structure and data types
- Check for reasonable alert content
- Monitor for anomalies in response patterns

---

## API Key Migration Path

**Current State**: No API keys

**Future-Proofing**: If NWS ever introduces API keys:

```rust
// Extensible client design
pub struct NwsClient {
    client: Client,
    base_url: String,
    user_agent: String,
    api_key: Option<String>,  // Future-proof
}

impl NwsClient {
    async fn fetch(&self, endpoint: &str) -> Result<Response> {
        let mut request = self.client.get(endpoint)
            .header("User-Agent", &self.user_agent);

        // Add API key if configured
        if let Some(key) = &self.api_key {
            request = request.header("X-API-Key", key);
        }

        request.send().await
    }
}
```

---

## Error Responses Related to Authentication/Headers

### 400 Bad Request

**Cause**: Malformed request, possibly invalid User-Agent

**Example**:
```json
{
  "correlationId": "abc123",
  "title": "Bad Request",
  "type": "https://api.weather.gov/problems/BadRequest",
  "status": 400,
  "detail": "Invalid request"
}
```

### 403 Forbidden

**Cause**: Blocked User-Agent or IP address (abuse detection)

**Action**:
- Review User-Agent format
- Check request rate
- Contact nco.ops@noaa.gov if legitimate traffic blocked

### 429 Too Many Requests

**Cause**: Rate limit exceeded

**Action**:
- Wait 5 seconds
- Reduce request frequency
- Implement exponential backoff

**Not Related to Authentication**: This is based on behavior, not credentials

---

## Comparison to Authenticated APIs

### Advantages of No Authentication

1. **Simplicity**: No credential management
2. **No Expiration**: No token refresh logic
3. **No Signup**: Immediate access
4. **No Quota Tracking**: No per-user limits
5. **Open Access**: True open data

### Disadvantages

1. **No Usage Analytics**: Can't track your specific usage
2. **No Priority Support**: Can't identify yourself for support
3. **Shared Rate Limits**: IP-based limits affect all users on same network
4. **No SLA**: No service level guarantees tied to your identity

---

## Contact Points for Issues

If you experience authentication-related issues (blocked User-Agent, IP bans):

**Operational Issues**:
- Email: nco.ops@noaa.gov
- Phone: 301-713-0902

**Technical Questions**:
- Email: sdb.support@noaa.gov

**General Inquiries**:
- Email: mike.gerber@noaa.gov

**Community Discussion**:
- GitHub: https://github.com/weather-gov/api

When contacting, provide:
- Your User-Agent string
- Source IP address (if known)
- Timestamp of blocked requests
- Example request URLs
