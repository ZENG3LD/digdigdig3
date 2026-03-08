# Hacker News Firebase API - Authentication

## Authentication Overview

**Authentication Type**: None
**API Keys**: Not required
**Registration**: Not required
**Public Access**: All endpoints are publicly accessible

## Summary

The Hacker News API is completely open and does not require any form of authentication. All data exposed through the API is public information already visible on the Hacker News website (https://news.ycombinator.com).

## No Authentication Required

### REST Endpoints
All REST endpoints can be accessed without any credentials:

```http
GET /v0/topstories.json HTTP/1.1
Host: hacker-news.firebaseio.com
```

No headers, query parameters, or authentication tokens are needed.

### SSE Streaming
Firebase SSE streaming typically requires an `auth` query parameter for authenticated databases, but Hacker News does not enforce this:

```http
GET /v0/maxitem.json HTTP/1.1
Host: hacker-news.firebaseio.com
Accept: text/event-stream
```

No `auth` parameter needed.

## Read-Only Access

The API is read-only. Write operations (POST, PUT, PATCH, DELETE) are not supported, even if authenticated:

- **Cannot**: Post stories, submit comments, vote, create accounts, modify items
- **Can**: Read all public stories, comments, user profiles, job postings

Users who want to post content must use the Hacker News web interface directly.

## User Privacy

### Public Data
The API exposes only information that is already publicly visible on HN:
- Usernames
- Public submissions (stories, comments)
- Karma scores
- User "about" descriptions

### Private Data
The API does not expose:
- Email addresses
- IP addresses
- Voting history (who voted on what)
- Flagged status of items (except dead/deleted flags)
- Hidden stories (from user's perspective)

## Firebase Security Rules

Behind the scenes, the Hacker News Firebase database uses Firebase security rules to enforce read-only public access:

**Conceptual Rules** (not exposed to API users):
```json
{
  "rules": {
    ".read": true,
    ".write": false
  }
}
```

This means:
- All data is readable by anyone (`.read: true`)
- No data is writable through the API (`.write: false`)

API consumers don't need to know these rules, but they explain why no authentication is required.

## CORS (Cross-Origin Resource Sharing)

The API supports **CORS**, allowing browser-based JavaScript applications to call the API from any domain:

**CORS Headers** (sent by Firebase):
```http
Access-Control-Allow-Origin: *
Access-Control-Allow-Methods: GET, OPTIONS
Access-Control-Allow-Headers: Content-Type, Accept
```

This means web applications can use the API directly from client-side JavaScript without proxy servers.

## HTTPS

All API requests must use HTTPS:

- **Valid**: `https://hacker-news.firebaseio.com/v0/item/8863.json`
- **Invalid**: `http://hacker-news.firebaseio.com/v0/item/8863.json` (HTTP will auto-redirect to HTTPS)

Firebase enforces TLS/SSL for all connections.

## Rate Limiting & Abuse Prevention

Since there's no authentication, rate limiting is typically enforced by:

### IP-Based Throttling (Hypothetical)
Firebase may throttle excessive requests from a single IP address, though the official documentation states "there is currently no rate limit."

**Best Practices**:
- Limit concurrent requests to ~10
- Use reasonable polling intervals (30s-5min)
- Cache immutable data (stories, comments don't change once posted)
- Respect server load

### No API Key Bans
Since there are no API keys, there's nothing to ban. Abuse would be handled at the IP/network level by Firebase infrastructure.

## Comparison with Other APIs

| Feature | Hacker News API | Typical API |
|---------|-----------------|-------------|
| Authentication | None | API key, OAuth, JWT |
| Registration | No | Yes (sign up for keys) |
| Rate Limiting | None (officially) | Yes (per key/tier) |
| Write Access | No | Often yes (for owned resources) |
| CORS | Enabled | Often restricted |
| Cost | Free | Often tiered pricing |

## Implementation Considerations

### Rust Connector
Since no authentication is required, the Rust connector implementation is simplified:

**No Need For**:
- API key storage/configuration
- Request signing (HMAC, etc.)
- OAuth flows
- Token refresh logic
- Credential management

**Simplified HTTP Client**:
```rust
use reqwest;

pub struct HackerNewsClient {
    base_url: String,
    client: reqwest::Client,
}

impl HackerNewsClient {
    pub fn new() -> Self {
        Self {
            base_url: "https://hacker-news.firebaseio.com/v0".to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_item(&self, id: u64) -> Result<Item, Error> {
        let url = format!("{}/item/{}.json", self.base_url, id);
        let response = self.client.get(&url).send().await?;
        // No auth headers needed
        Ok(response.json().await?)
    }
}
```

### Configuration
The connector config can omit all authentication fields:

```rust
pub struct HackerNewsConfig {
    pub base_url: Option<String>,  // Optional override (default: official URL)
    pub timeout: Option<Duration>,
    pub max_concurrent: Option<usize>,
}

// No api_key, secret, token, etc.
```

### Testing
No need for test credentials or API key rotation in CI/CD pipelines. Integration tests can run against the live API directly:

```rust
#[tokio::test]
async fn test_fetch_top_stories() {
    let client = HackerNewsClient::new();
    let stories = client.get_top_stories().await.unwrap();
    assert!(!stories.is_empty());
}
```

## Security Considerations

### No Credential Leakage Risk
Since there are no credentials, there's no risk of:
- Accidentally committing API keys to version control
- Exposing secrets in logs
- Stolen credentials being abused

### Rate Limit Best Practices
Even without enforced rate limits, implement client-side throttling:
- Use `tokio::time::sleep()` between bursts
- Implement semaphore for concurrent request limiting
- Cache aggressively to reduce load

### User-Agent Header
Consider setting a descriptive User-Agent to identify your application:

```rust
let client = reqwest::Client::builder()
    .user_agent("my-hn-reader/1.0")
    .build()?;
```

This helps HN/Firebase track API usage patterns and contact you if issues arise.

## Future Changes

The API documentation does not mention any plans to add authentication. However, if abuse becomes a problem, Firebase/YC could:

1. **Add IP-based rate limiting** (most likely)
2. **Require API keys** for higher rate limits (possible)
3. **Implement CAPTCHA** for browser clients (unlikely)
4. **Restrict CORS** to approved domains (unlikely, breaks use cases)

Monitor the official documentation (https://github.com/HackerNews/API) and announcement blog (https://blog.ycombinator.com) for any changes.

## Summary

- **No authentication required**: Public read-only access
- **No API keys**: No registration needed
- **No rate limits** (officially): Use responsibly
- **HTTPS enforced**: All requests use TLS
- **CORS enabled**: Works from browser JavaScript
- **Simplified implementation**: No credential management in code
- **Testing-friendly**: No test credentials needed

This makes the Hacker News API one of the simplest public APIs to integrate, with no authentication overhead.
