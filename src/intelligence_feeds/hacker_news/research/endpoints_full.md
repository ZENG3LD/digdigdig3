# Hacker News Firebase API - Complete Endpoint Reference

## Base URL

```
https://hacker-news.firebaseio.com/v0/
```

All endpoints return JSON. Append `.json` to each path.

## Story List Endpoints

### Top Stories
**Endpoint**: `/v0/topstories.json`
**Method**: GET
**Description**: Up to 500 top stories (ranked by HN algorithm)
**Response**: Array of story IDs (integers)
**Update Frequency**: Every few minutes
**Typical Size**: ~500 IDs

**Example Response**:
```json
[39427470, 39426838, 39425902, ...]
```

### New Stories
**Endpoint**: `/v0/newstories.json`
**Method**: GET
**Description**: Up to 500 newest stories (chronological)
**Response**: Array of story IDs
**Update Frequency**: Every few seconds

### Best Stories
**Endpoint**: `/v0/beststories.json`
**Method**: GET
**Description**: Up to 500 best stories (quality-ranked)
**Response**: Array of story IDs
**Update Frequency**: Every few minutes

### Ask HN Stories
**Endpoint**: `/v0/askstories.json`
**Method**: GET
**Description**: Up to 200 latest "Ask HN" stories
**Response**: Array of story IDs
**Update Frequency**: Every few minutes

### Show HN Stories
**Endpoint**: `/v0/showstories.json`
**Method**: GET
**Description**: Up to 200 latest "Show HN" stories
**Response**: Array of story IDs
**Update Frequency**: Every few minutes

### Job Stories
**Endpoint**: `/v0/jobstories.json`
**Method**: GET
**Description**: Up to 200 latest job postings
**Response**: Array of job IDs
**Update Frequency**: Every few minutes

## Item Endpoints

### Get Item by ID
**Endpoint**: `/v0/item/{id}.json`
**Method**: GET
**Parameters**:
- `{id}` (path): Item ID (integer)

**Description**: Fetch a single item (story, comment, job, poll, or pollopt)
**Response**: Item object or `null` if deleted/not found

**Example**: `/v0/item/8863.json`

**Response Schema** (varies by type - see data_types.md for full details):
```json
{
  "by": "dhouston",
  "descendants": 71,
  "id": 8863,
  "kids": [8952, 9224, 8917, ...],
  "score": 111,
  "time": 1175714200,
  "title": "My YC app: Dropbox - Throw away your USB drive",
  "type": "story",
  "url": "http://www.getdropbox.com/u/2/screencast.html"
}
```

### Notes on Item Fetching
- **Immutable Fields**: `id`, `type`, `by`, `time`, `text`, `url`, `title`, `parent`, `poll`, `parts` never change after creation
- **Mutable Fields**: `score`, `descendants`, `kids`, `deleted`, `dead` can change
- **Deleted Items**: Return `null` or have `"deleted": true`
- **Dead Items**: Have `"dead": true` (killed by moderators or flagged)

## User Endpoints

### Get User by ID
**Endpoint**: `/v0/user/{id}.json`
**Method**: GET
**Parameters**:
- `{id}` (path): Username (case-sensitive string)

**Description**: Fetch user profile with public activity
**Response**: User object or `null` if not found

**Example**: `/v0/user/jl.json`

**Response Schema**:
```json
{
  "about": "This is a test",
  "created": 1173923446,
  "id": "jl",
  "karma": 2937,
  "submitted": [8265435, 8168423, 8090946, ...]
}
```

**Fields**:
- `id` (string, required): Unique username
- `created` (integer): Account creation timestamp (Unix time)
- `karma` (integer): User karma score
- `about` (string, optional): Self-description (HTML)
- `submitted` (array): All submission IDs (stories, comments, polls, poll options)

## Discovery Endpoints

### Max Item ID
**Endpoint**: `/v0/maxitem.json`
**Method**: GET
**Description**: Current largest item ID in the system
**Response**: Single integer
**Use Case**: Starting point for backward iteration to discover all items

**Example Response**:
```json
39427999
```

### Changed Items and Profiles (Updates)
**Endpoint**: `/v0/updates.json`
**Method**: GET
**Description**: Recently changed items and user profiles
**Response**: Object with two arrays

**Response Schema**:
```json
{
  "items": [8423305, 8420805, 8423379, ...],
  "profiles": ["thefox", "mdda", "plinkplonk", ...]
}
```

**Fields**:
- `items` (array): Item IDs that changed recently (scores, new comments, etc.)
- `profiles` (array): Usernames with recent profile updates

**Update Frequency**: Every few minutes
**Typical Size**: ~500 items, ~50 profiles

## URL Patterns & Conventions

### Standard Pattern
All endpoints follow Firebase URL conventions:
```
https://hacker-news.firebaseio.com/v0/{resource}.json
```

### Path Parameter Notation
Item and user lookups use path parameters:
```
/v0/item/{id}.json
/v0/user/{username}.json
```

### No Query Parameters
The API does not use query parameters for filtering, pagination, or sorting. All filtering must be done client-side.

## HTTP Methods

- **GET**: Only HTTP method supported
- **POST/PUT/DELETE/PATCH**: Not available (read-only API)

## Response Format

- **Content-Type**: `application/json`
- **Character Encoding**: UTF-8
- **Success Status**: 200 OK
- **Not Found**: Returns `null` with 200 OK (not 404)
- **Error Status**: Rare; usually 503 Service Unavailable if Firebase is down

## Pagination

The API does not support traditional pagination. Instead:
1. Fetch story list (returns up to 500 IDs)
2. Slice the array client-side for desired page size
3. Fetch individual items concurrently

Example for "page 1, 30 items":
```
GET /v0/topstories.json -> [id1, id2, ..., id500]
Take first 30 IDs: [id1..id30]
GET /v0/item/id1.json, /v0/item/id2.json, ... (concurrent)
```

## Common Usage Patterns

### Fetching Top Stories (First Page)
1. `GET /v0/topstories.json` -> `[39427470, 39426838, ...]`
2. Take first 30 IDs
3. Concurrent fetch: `GET /v0/item/39427470.json`, etc.
4. Filter out `null` responses (deleted items)

### Monitoring New Content
1. `GET /v0/maxitem.json` -> `39427999`
2. Periodically poll, incrementing from last known max
3. `GET /v0/item/39428000.json`, etc.
4. Or use `/v0/updates.json` for recently changed items

### Building Comment Threads
1. Fetch story: `GET /v0/item/{story_id}.json`
2. Extract `kids` array (top-level comment IDs)
3. Recursively fetch each comment: `GET /v0/item/{comment_id}.json`
4. Continue recursively for nested `kids`

### User Activity Timeline
1. `GET /v0/user/{username}.json`
2. Extract `submitted` array
3. Fetch recent items: `GET /v0/item/{id}.json` for each
4. Filter by `type` (stories vs comments)

## Error Handling

- **Item Not Found**: Returns `null` (not an error)
- **User Not Found**: Returns `null`
- **Invalid ID**: Returns `null`
- **Network Errors**: Standard HTTP errors (503, timeout, etc.)
- **Deleted Items**: May return `null` or `{"deleted": true, "id": 123}`

## Rate Limiting & Throttling

- **Official Limit**: None currently enforced
- **Recommended**: Limit concurrent requests to ~10 to prevent fan-out issues
- **Best Practice**: Cache immutable fields, poll changed items via `/v0/updates.json`

## Endpoint Summary Table

| Endpoint | Resource | Typical Size | Update Frequency | Mutable |
|----------|----------|--------------|------------------|---------|
| `/topstories.json` | Story IDs | ~500 | Minutes | Yes |
| `/newstories.json` | Story IDs | ~500 | Seconds | Yes |
| `/beststories.json` | Story IDs | ~500 | Minutes | Yes |
| `/askstories.json` | Story IDs | ~200 | Minutes | Yes |
| `/showstories.json` | Story IDs | ~200 | Minutes | Yes |
| `/jobstories.json` | Job IDs | ~200 | Minutes | Yes |
| `/item/{id}.json` | Item object | ~500 bytes | Varies | Partially |
| `/user/{id}.json` | User object | ~1KB | Rarely | Yes |
| `/maxitem.json` | Integer | 8 bytes | Seconds | Yes |
| `/updates.json` | Items + Profiles | ~550 IDs | Minutes | Yes |
