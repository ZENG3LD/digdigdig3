# Hacker News Firebase API - Data Coverage and Limitations

## Data Coverage

### Historical Data

**Start Date**: The API provides access to all HN items since the site's inception.

- **Earliest Items**: Item IDs start at 1 (from 2006-10-09)
- **Current Max ID**: ~40 million items (as of February 2026)
- **Growth Rate**: ~15,000-25,000 new items per day

**Complete Archive**: The API exposes the entire Hacker News history:
- All stories since 2006
- All comments since 2006
- All user profiles (active and deleted)
- All polls and job postings

**Accessing Historical Data**:
```
Oldest item: GET /v0/item/1.json
Latest item: GET /v0/maxitem.json (returns current max ID)
Historical range: IDs 1 to maxitem (sequential, some IDs deleted/missing)
```

### Real-Time Data

**Update Frequency**:
- **New items**: Available immediately via `/v0/maxitem.json` (updates every 1-10 seconds)
- **Story rankings**: Updated every 1-5 minutes
- **Scores**: Updated within seconds of votes
- **Comments**: Appear immediately when posted

**Latency**: Near real-time (typically <5 seconds from user action to API update)

### Data Completeness

#### Included Data
- All stories (links, Ask HN, Show HN, text posts)
- All comments (top-level and nested)
- All job postings
- All polls and poll options
- User profiles (username, karma, about, submissions)
- Voting scores (stories, comments, polls)
- Comment counts (descendants)
- Timestamps (creation time for all items)

#### Excluded Data
- **Vote details**: Individual user votes (who voted on what)
- **Flagged status**: User-specific flags (you can see dead/killed items, but not who flagged)
- **Hidden stories**: User-specific hidden/favorites lists
- **Email addresses**: User emails are never exposed
- **IP addresses**: User IPs are never exposed
- **Edit history**: No revision history for edited comments/stories
- **Deleted content**: Text of deleted items is removed (only `id`, `type`, `deleted` remain)
- **Private messages**: HN doesn't have PMs, so N/A
- **Moderation logs**: Internal mod actions not exposed
- **Vote counts**: Only net score (upvotes - downvotes), not separate counts

## Content Types

### Supported Content Types

| Type | Description | API Support | Example Count |
|------|-------------|-------------|---------------|
| Stories | Standard link submissions | Full | ~1M active |
| Ask HN | Question posts | Full | ~100K active |
| Show HN | Project showcases | Full | ~80K active |
| Jobs | Job postings | Full | ~20K active |
| Polls | Multiple choice questions | Full | ~1K active |
| Poll Options | Individual poll choices | Full | ~3K active |
| Comments | User replies | Full | ~20M active |

**Total Items**: ~40 million (including deleted)

### Unsupported Content Types

- **Images**: HN doesn't host images; only URLs to external images
- **Videos**: HN doesn't host videos; only URLs to external videos
- **Attachments**: No file attachments supported
- **Rich Media**: No embedded media; plain HTML text only

## Geographic Coverage

**Scope**: Global

Hacker News is a worldwide platform with no geographic restrictions. The API provides equal access to content from all regions.

**Language**: Predominantly English, but the API returns content in any language posted by users (UTF-8 encoded).

## Temporal Coverage

### Update Frequency by Resource

| Resource | Update Frequency | Staleness | Best For |
|----------|------------------|-----------|----------|
| `/topstories.json` | 1-5 minutes | Low | Front page monitoring |
| `/newstories.json` | 5-30 seconds | Very low | Real-time feed |
| `/beststories.json` | 5-10 minutes | Low | Quality content |
| `/askstories.json` | 5-10 minutes | Low | Q&A monitoring |
| `/showstories.json` | 5-10 minutes | Low | Project tracking |
| `/jobstories.json` | 30-60 minutes | Medium | Job alerts |
| `/item/{id}.json` (score) | Seconds | Very low | Live score tracking |
| `/item/{id}.json` (kids) | Seconds | Very low | Comment monitoring |
| `/user/{id}.json` | Minutes | Low | Profile updates |
| `/maxitem.json` | 1-10 seconds | Very low | New item detection |
| `/updates.json` | 1-5 minutes | Low | Batch update checks |

### Data Retention

**Retention Policy**: Indefinite

- All items are stored permanently (unless deleted by user/mod)
- Deleted items remain in the database with `"deleted": true` but text removed
- User accounts persist even after deletion (submissions remain attributed)

**No Expiration**: There is no TTL or auto-deletion of old content.

## Rate Limits Impact on Coverage

### No Enforced Limits

Since there are no official rate limits, coverage is theoretically unlimited:
- Can fetch all 500 top stories
- Can fetch all 500 new stories
- Can fetch all historical items (1 to maxitem)
- Can poll for updates continuously

### Practical Limits

**Recommended Coverage Strategies**:

1. **Real-Time Monitoring**: Poll `/v0/maxitem.json` every 10 seconds, fetch new items
2. **Front Page Feed**: Poll `/v0/topstories.json` every 5 minutes, fetch first 30 items
3. **Historical Archive**: Batch fetch items 1 to maxitem with concurrent requests (limit to 10 concurrent)
4. **User Activity**: Fetch user profiles on-demand, cache for 10 minutes

**Example Coverage** (30 stories, 5-minute poll):
- Requests per hour: 12 (story list) + 12 * 30 (items) = 372 requests/hour
- Total data: ~180 KB/hour uncompressed

## Data Quality

### Accuracy

- **Scores**: Exact at time of fetch, but can change rapidly
- **Timestamps**: Exact (Unix seconds)
- **IDs**: Permanent, unique identifiers
- **Text**: Exact as posted (HTML entities encoded)
- **Deleted Items**: May return `null` or `{"deleted": true}` (inconsistent)

### Consistency

- **Eventual Consistency**: Firebase is eventually consistent; rare delays (<1s) possible
- **Read-After-Write**: New items may not appear immediately in story lists
- **Score Lag**: Scores may lag by a few seconds during high voting activity

### Data Integrity

- **No Validation**: API returns data as-is (URLs not validated, HTML not sanitized)
- **User Input**: Text fields contain user-generated HTML (potential XSS risk if not sanitized)
- **Encoding**: UTF-8 encoded, but may contain broken HTML or malformed entities

## Gaps and Limitations

### Known Gaps

1. **Deleted Item Text**: Text content removed permanently (only metadata remains)
2. **Killed Item Visibility**: Dead items still returned but marked `"dead": true`
3. **Sequential ID Gaps**: Some IDs missing (deleted items, test posts, etc.)
4. **No Search**: Cannot search by keyword, author, date range (use Algolia HN Search API separately)
5. **No Filtering**: Cannot filter by score, comment count, domain, etc. (must fetch and filter client-side)
6. **No Aggregations**: Cannot get "top stories this week" or similar aggregations (must compute client-side)

### Deleted vs Missing IDs

**Deleted Items**:
```json
GET /v0/item/12345.json
Response: {"deleted": true, "id": 12345, "type": "comment"}
```

**Missing IDs** (never existed or test data):
```json
GET /v0/item/99999999.json
Response: null
```

**Handling**: Treat both as "not available" in your application.

### Incomplete Data Scenarios

#### Missing `kids` Field
Some stories/comments have no `kids` field even if they should have comments. This can happen if:
- Comments are still being posted (eventual consistency)
- All child comments were deleted
- API bug (rare)

**Workaround**: Check `descendants` count; if >0 but no `kids`, retry after a delay.

#### Missing `by` Field
Items may lack a `by` field if:
- User deleted their account
- Item was posted anonymously (very rare, legacy items)

**Handling**: Display as "deleted" or "unknown" user.

#### Missing `score` Field
Jobs typically don't have scores (or have score=0). Polls and stories always have scores.

## Comparison with Other Data Sources

### Algolia HN Search API

**Algolia API**: https://hn.algolia.com/api

**Differences**:
| Feature | Firebase API | Algolia API |
|---------|--------------|-------------|
| Search | No | Yes (full-text) |
| Filtering | No | Yes (by date, score, type) |
| Aggregations | No | Yes (facets) |
| Real-Time | Yes | Slightly delayed (~1 min) |
| Historical | Full access | Full access |
| Rate Limit | None | 10,000 req/hour |
| Auth | None | API key required |

**Recommendation**: Use Firebase for real-time feeds, Algolia for search/filtering.

### Official HN Website

**Website**: https://news.ycombinator.com

The API exposes the same data as the website, but:
- Website has pagination, API returns raw ID arrays
- Website has "More" button, API returns up to 500 items
- Website shows user-specific data (favorites, hidden), API does not
- Website renders HTML, API returns raw HTML strings

## Data Freshness Guarantees

### No SLA

Firebase/YC does not provide an SLA (Service Level Agreement) for the API:
- No guaranteed uptime
- No guaranteed update frequency
- No guaranteed response time

### Observed Performance

Based on community observations:
- **Uptime**: >99.9% (very rare outages)
- **Latency**: p50 < 100ms, p95 < 500ms, p99 < 2s
- **Update Lag**: Typically <5 seconds from user action to API update

### Monitoring Recommendations

To detect stale data:
1. Track `/v0/maxitem.json` increments (should increase every 10-60 seconds during active hours)
2. Compare story scores over time (should generally increase for trending stories)
3. Alert if no new items appear for >5 minutes during peak hours (9am-5pm PT)

## Historical Data Access

### Archiving Strategy

To build a complete HN archive:

**Step 1: Determine Range**
```
GET /v0/maxitem.json -> 39427999
Archive items 1 to 39427999
```

**Step 2: Batch Fetch**
```rust
for chunk in (1..=max_id).chunks(1000) {
    let tasks: Vec<_> = chunk.iter()
        .map(|id| client.get_item(*id))
        .collect();

    let items = join_all(tasks).await; // Concurrent, limited by semaphore

    // Store in database
    for item in items {
        db.insert(item);
    }

    // Rate limit: small delay between chunks
    sleep(Duration::from_millis(500)).await;
}
```

**Estimated Time**:
- 40 million items
- 10 concurrent requests
- ~500ms per batch of 10 items
- ~2,000,000 batches
- Total time: ~12 days (continuous)

**Recommended**: Parallelize across multiple machines or throttle to run over weeks.

### Incremental Updates

To keep archive up-to-date:

**Strategy 1: Poll Max Item**
```rust
let mut last_id = db.get_max_id();

loop {
    let current_id = client.get_max_item().await?;

    if current_id > last_id {
        for id in (last_id + 1)..=current_id {
            let item = client.get_item(id).await?;
            db.insert(item);
        }
        last_id = current_id;
    }

    sleep(Duration::from_secs(10)).await;
}
```

**Strategy 2: Poll Updates**
```rust
loop {
    let updates = client.get_updates().await?;

    for id in updates.items {
        let item = client.get_item(id).await?;
        db.update(item); // Update existing record
    }

    for username in updates.profiles {
        let user = client.get_user(username).await?;
        db.update_user(user);
    }

    sleep(Duration::from_secs(300)).await; // 5 minutes
}
```

## Regional and Temporal Patterns

### Activity Patterns

**Hourly**: HN activity peaks during US business hours (9am-5pm PT)
- Peak new item rate: 2-3 items/second (7200-10800/hour)
- Off-peak rate: 0.5-1 items/second (1800-3600/hour)

**Daily**: Weekdays see higher activity than weekends
- Weekday average: ~20,000 items/day
- Weekend average: ~12,000 items/day

**Timezone Considerations**: Most users are US-based, but the API serves global traffic with no regional restrictions.

## Future Coverage Changes

### API Stability

The API has been stable since 2014 with minimal breaking changes. Future changes are unlikely to remove data, but may add fields.

**Forward Compatibility**: Always handle unexpected fields gracefully:
```rust
#[derive(Deserialize)]
#[serde(deny_unknown_fields = false)] // Allow unexpected fields
pub struct Item { ... }
```

### Monitoring for Changes

- Subscribe to HN API updates: api@ycombinator.com
- Watch GitHub repo: https://github.com/HackerNews/API
- Monitor Firebase blog: https://firebase.blog

## Summary

- **Historical Coverage**: Complete since 2006 (40M+ items)
- **Real-Time Coverage**: Near real-time (<5s latency)
- **Update Frequency**: Seconds (scores, comments) to minutes (rankings)
- **Data Retention**: Indefinite (no expiration)
- **Gaps**: Deleted item text, individual votes, edit history
- **Quality**: High accuracy, eventual consistency
- **Rate Limits**: None (use responsibly)
- **Comparison**: Use Firebase for real-time feeds, Algolia for search
- **Archival**: Possible to archive all 40M items (takes ~12 days)
- **Incremental Updates**: Poll `/v0/maxitem.json` every 10 seconds

The API provides comprehensive coverage of all public HN data with minimal limitations, making it suitable for real-time feeds, historical analysis, and complete archival projects.
