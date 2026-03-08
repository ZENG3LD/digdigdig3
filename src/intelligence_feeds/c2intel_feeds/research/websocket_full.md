# C2IntelFeeds WebSocket Capabilities

## Real-Time Support

**Status**: ❌ Not Available

C2IntelFeeds does not provide real-time WebSocket or streaming capabilities.

## Data Access Method

**Type**: Static file hosting via GitHub raw file URLs
**Protocol**: HTTPS only
**Update Mechanism**: Automated batch updates (GitHub Actions/CI pipeline)

## Alternative Real-Time Strategies

### 1. Polling

Since feeds are updated automatically but without WebSocket support, consumers must implement HTTP polling:

```
Recommended polling interval:
- High-frequency use cases: Every 5-15 minutes
- Standard monitoring: Every 30-60 minutes
- Low-frequency: Every 6-12 hours
```

**Considerations**:
- GitHub raw file rate limits apply (see tiers_and_limits.md)
- Use conditional requests (If-Modified-Since header) to minimize bandwidth
- Track Last-Modified header to detect changes

### 2. GitHub Webhooks (Repository-Level)

For advanced users with GitHub API access:
- Monitor the repository for commit events
- Subscribe to repository push notifications
- Trigger feed downloads only when repository updates

**Requires**:
- GitHub API token
- Webhook endpoint for receiving events
- Repository watch/subscription

### 3. GitHub API (Commits Endpoint)

Poll GitHub API to check latest commits:

```
GET https://api.github.com/repos/drb-ra/C2IntelFeeds/commits?path=feeds/IPC2s-30day.csv&per_page=1
```

**Advantages**:
- Lower bandwidth than downloading full feed
- Provides commit SHA for change detection
- Rate limit: 60 req/hour (unauthenticated), 5000/hour (authenticated)

## Change Detection Strategies

### HTTP Conditional Requests

```http
GET /drb-ra/C2IntelFeeds/master/feeds/IPC2s-30day.csv HTTP/1.1
Host: raw.githubusercontent.com
If-Modified-Since: Sat, 15 Feb 2026 12:00:00 GMT
```

**Response codes**:
- `200 OK`: Feed has been updated (download new version)
- `304 Not Modified`: No changes since last check (skip download)

### ETag Support

GitHub raw files support ETags:

```http
GET /drb-ra/C2IntelFeeds/master/feeds/IPC2s-30day.csv HTTP/1.1
Host: raw.githubusercontent.com
If-None-Match: "abc123def456..."
```

### Content Hashing

For detecting changes without HTTP headers:
1. Download feed
2. Compute SHA-256 hash
3. Compare with previous hash
4. Process only if hash changed

## Feed Update Frequency

**Documented**: "Updated automatically"
**Observed patterns**: Likely daily or more frequent updates based on Censys data ingestion

**Note**: Exact update schedule not published in repository documentation.

## Summary

- **No WebSocket support**: Feeds are static CSV files on GitHub
- **Polling required**: Implement HTTP polling with conditional requests
- **Rate limits apply**: GitHub raw file limits (see tiers_and_limits.md)
- **Change detection**: Use If-Modified-Since, ETags, or GitHub API commit tracking
- **Recommended interval**: 15-60 minutes for most use cases
