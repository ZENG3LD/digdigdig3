# Hacker News Firebase API - Response Formats

## General Response Format

All endpoints return JSON. Successful responses have HTTP status 200 OK.

**Content-Type**: `application/json; charset=utf-8`

## Story List Responses

### Top Stories (`/v0/topstories.json`)

**Response Type**: JSON array of integers (story IDs)

**Example Response**:
```json
[
  39427470,
  39426838,
  39425902,
  39427134,
  39426521,
  39424718,
  39423981,
  39424992,
  39425513,
  39427213
]
```

**Characteristics**:
- Up to 500 IDs
- Ordered by HN ranking algorithm (score, time, comments)
- Updates every few minutes
- Response size: ~2-4 KB

### New Stories (`/v0/newstories.json`)

**Response Type**: JSON array of integers

**Example Response**:
```json
[
  39428156,
  39428144,
  39428132,
  39428118,
  39428099,
  39428077
]
```

**Characteristics**:
- Up to 500 IDs
- Ordered chronologically (newest first)
- Updates every 5-30 seconds
- Response size: ~2-4 KB

### Best Stories (`/v0/beststories.json`)

**Response Type**: JSON array of integers

**Example** (same format as topstories)

### Ask HN Stories (`/v0/askstories.json`)

**Response Type**: JSON array of integers

**Example Response**:
```json
[
  39427801,
  39427345,
  39426988,
  39426421
]
```

**Characteristics**:
- Up to 200 IDs
- Only "Ask HN" posts
- Response size: ~1-2 KB

### Show HN Stories (`/v0/showstories.json`)

**Response Type**: JSON array of integers

**Example** (same format as askstories)

### Job Stories (`/v0/jobstories.json`)

**Response Type**: JSON array of integers

**Example Response**:
```json
[
  39423456,
  39422134,
  39421890
]
```

**Characteristics**:
- Up to 200 IDs
- Only job postings
- Updates every 30-60 minutes
- Response size: ~1 KB

## Item Responses

### Story (`/v0/item/{id}.json` where type=story)

**Example Response** (Standard Story):
```json
{
  "by": "dhouston",
  "descendants": 71,
  "id": 8863,
  "kids": [
    8952,
    9224,
    8917,
    8884,
    8887,
    8943,
    8869,
    8958,
    9005,
    9671,
    8940,
    9067,
    8908,
    9055,
    8865,
    8881,
    8872,
    8873,
    8955,
    10403,
    8903,
    8928,
    9125,
    8998,
    8901,
    8902,
    8907,
    8894,
    8878,
    8870,
    8980,
    8934,
    8876
  ],
  "score": 111,
  "time": 1175714200,
  "title": "My YC app: Dropbox - Throw away your USB drive",
  "type": "story",
  "url": "http://www.getdropbox.com/u/2/screencast.html"
}
```

**Response Size**: ~500 bytes - 2 KB (depending on comment count)

**Example Response** (Ask HN Story):
```json
{
  "by": "tel",
  "descendants": 16,
  "id": 121003,
  "kids": [
    121016,
    121109,
    121168
  ],
  "score": 25,
  "text": "<i>or</i> HN: the Next Iteration<p>I get the impression that with Arc being released a lot of people who never had time for HN before are suddenly dropping in more often. (PG: what are the numbers on this? I'm envisioning a spike.)<p>Not to say that isn't great, but I'm wary of Diggification. Between links comparing programming to sex and a flurry of gratuitous, ostentatious  adjectives in the headlines it's a bit concerning.<p>...<p>I'd like to get a community dialog going on this. What do you think?",
  "time": 1203647620,
  "title": "Ask HN: The Arc Effect",
  "type": "ask"
}
```

**Note**: Ask HN stories have `text` field instead of `url`.

### Comment (`/v0/item/{id}.json` where type=comment)

**Example Response**:
```json
{
  "by": "norvig",
  "id": 2921983,
  "kids": [
    2922097,
    2922429,
    2924562,
    2922709,
    2922573,
    2922140,
    2922141
  ],
  "parent": 2921506,
  "text": "Aw shucks, guys ... you make me blush with your compliments.<p>Tell you what, Ill make a deal: I'll keep writing if you keep reading. K?",
  "time": 1314211127,
  "type": "comment"
}
```

**Response Size**: ~200 bytes - 5 KB (depending on text length and child count)

**Example Response** (Nested Comment):
```json
{
  "by": "username",
  "id": 12345678,
  "parent": 12345670,
  "text": "This is a reply to another comment.",
  "time": 1609459200,
  "type": "comment"
}
```

**Note**: Nested comments have no `kids` field if they're leaf nodes.

### Job (`/v0/item/{id}.json` where type=job)

**Example Response**:
```json
{
  "by": "justin",
  "id": 192327,
  "score": 6,
  "text": "Justin.tv is the biggest live video site online. We serve hundreds of thousands of video streams a day, and have supported up to 50k live concurrent viewers. Our site is growing every week, and we just added a 10 gbps line to our colo. Our unique visitors are up 900% since January.<p>There are a lot of pieces that fit together to make Justin.tv work: our video cluster, IRC server, our web app, and our monitoring and search services, to name a few. A lot of our website is dependent on Flash, and we're looking for talented Flash Engineers who know AS2 and AS3 very well who want to be leaders in the development of our Flash.<p>Responsibilities<p>* Contribute to product design and implementation discussions<p>* Implement applications that handle million of events per day<p>* Write clean, well-tested, performant code<p>...",
  "time": 1210981217,
  "title": "Justin.tv is looking for a Lead Flash Engineer!",
  "type": "job",
  "url": ""
}
```

**Response Size**: ~1-5 KB (job descriptions can be long)

### Poll (`/v0/item/{id}.json` where type=poll)

**Example Response**:
```json
{
  "by": "pg",
  "descendants": 54,
  "id": 126809,
  "kids": [
    126822,
    126823,
    126993,
    126824,
    126934,
    127411,
    126888,
    127681,
    126818,
    126816,
    126854,
    127095,
    126861,
    127313,
    127299,
    126859,
    126852,
    126882,
    126832,
    127072,
    127217,
    126889,
    127535,
    126917,
    126875
  ],
  "parts": [
    126810,
    126811,
    126812
  ],
  "score": 46,
  "text": "",
  "time": 1204403652,
  "title": "Poll: What would happen if News.YC had explicit support for polls?",
  "type": "poll"
}
```

**Response Size**: ~500 bytes - 2 KB

**Note**: `parts` array contains pollopt IDs (the choices), `kids` contains comment IDs.

### Poll Option (`/v0/item/{id}.json` where type=pollopt)

**Example Response**:
```json
{
  "by": "pg",
  "id": 126810,
  "poll": 126809,
  "score": 335,
  "text": "Yes, ban them; I'm tired of seeing Valleywag stories on News.YC.",
  "time": 1204403652,
  "type": "pollopt"
}
```

**Response Size**: ~100-300 bytes

### Deleted Item

**Example Response**:
```json
{
  "deleted": true,
  "id": 8863,
  "type": "comment"
}
```

**Or**:
```json
null
```

**Response Size**: ~50 bytes or 4 bytes (`null`)

### Dead Item

**Example Response**:
```json
{
  "by": "username",
  "dead": true,
  "id": 1234567,
  "time": 1609459200,
  "title": "Spam story",
  "type": "story"
}
```

**Note**: Dead items still return data but have `"dead": true`.

## User Responses

### User Profile (`/v0/user/{id}.json`)

**Example Response**:
```json
{
  "about": "This is a test",
  "created": 1173923446,
  "id": "jl",
  "karma": 2937,
  "submitted": [
    8265435,
    8168423,
    8090946,
    8090326,
    7699907,
    7637962,
    7596179,
    7596163,
    7594569,
    7562135,
    7562111,
    7494708,
    7494171,
    7488093,
    7444860,
    7327817,
    7280290,
    7278694,
    7097557,
    7097546,
    7097254,
    7052857,
    7039484,
    6987273,
    6649999,
    6649706,
    6629560,
    6609127,
    6327951,
    6225810,
    6111999,
    5580079,
    5112008,
    4907948,
    4901821,
    4700469,
    4678919,
    3779193,
    3711380,
    3701405,
    3627981,
    3473004,
    3473000,
    3457006,
    3422158,
    3136701,
    2943046,
    2794646,
    2482737,
    2425640,
    2411925,
    2408077,
    2407992,
    2407940,
    2278689,
    2220295,
    2144918,
    2144852,
    1875323,
    1875295,
    1857397,
    1839737,
    1809010,
    1788048,
    1780681,
    1721745,
    1676227,
    1654023,
    1651449,
    1641019,
    1631985,
    1618759,
    1522978,
    1499641,
    1441290,
    1440993,
    1436440,
    1430510,
    1430208,
    1385525,
    1384917,
    1370453,
    1346118,
    1309968,
    1305415,
    1305037,
    1304247,
    1303960,
    1303815,
    1303793,
    1303772,
    1303755,
    1303745,
    1303740,
    1303724,
    1303718,
    1303702,
    1303689,
    1303671,
    1303666,
    1303665,
    1303662,
    1303658,
    1303656,
    1303637
  ]
}
```

**Response Size**: ~1-10 KB (depending on submission count)

**Example Response** (User with no bio):
```json
{
  "created": 1234567890,
  "id": "newuser",
  "karma": 1,
  "submitted": [
    12345678
  ]
}
```

**Example Response** (Deleted User):
```json
null
```

## Discovery Responses

### Max Item ID (`/v0/maxitem.json`)

**Example Response**:
```json
39427999
```

**Response Type**: Single integer (not in an array)

**Response Size**: ~8 bytes

### Updates (`/v0/updates.json`)

**Example Response**:
```json
{
  "items": [
    8423305,
    8420805,
    8423379,
    8422862,
    8423103,
    8423450,
    8423455,
    8423479,
    8423478,
    8423493
  ],
  "profiles": [
    "thefox",
    "mdda",
    "plinkplonk",
    "nostrademons",
    "beza1e1",
    "yarapavan",
    "edw519",
    "dhouston"
  ]
}
```

**Response Size**: ~2-5 KB

**Note**: `items` contains up to ~500 recently changed item IDs, `profiles` contains ~50 recently updated usernames.

## Error Responses

### Item Not Found

**HTTP Status**: 200 OK
**Response Body**: `null`

**Example**:
```http
GET /v0/item/999999999.json HTTP/1.1

HTTP/1.1 200 OK
Content-Type: application/json

null
```

### User Not Found

**HTTP Status**: 200 OK
**Response Body**: `null`

**Example**:
```http
GET /v0/user/nonexistentuser.json HTTP/1.1

HTTP/1.1 200 OK
Content-Type: application/json

null
```

### Server Error

**HTTP Status**: 503 Service Unavailable
**Response Body**: Firebase error page (HTML, not JSON)

**Example** (rare):
```http
GET /v0/topstories.json HTTP/1.1

HTTP/1.1 503 Service Unavailable
Content-Type: text/html

<html>Firebase temporarily unavailable</html>
```

## SSE Streaming Responses

### Initial Snapshot (`event: put`)

**Example Response**:
```
event: put
data: {"path": "/", "data": {"by": "dhouston", "descendants": 71, "id": 8863, "kids": [8952, 9224], "score": 111, "time": 1175714200, "title": "My YC app: Dropbox", "type": "story", "url": "http://www.getdropbox.com/u/2/screencast.html"}}

```

**Format**:
- `event: put` (event type)
- `data: {...}` (JSON payload)
- Blank line (separator)

### Partial Update (`event: patch`)

**Example Response**:
```
event: patch
data: {"path": "/score", "data": 115}

```

**Note**: Only changed fields are included.

### Keep-Alive

**Example Response**:
```
event: keep-alive
data: null

```

**Frequency**: Every ~30 seconds of inactivity.

## Response Size Summary

| Endpoint | Typical Size | Max Size | Compression |
|----------|--------------|----------|-------------|
| `/topstories.json` | 3 KB | 4 KB | Gzip supported |
| `/newstories.json` | 3 KB | 4 KB | Gzip supported |
| `/beststories.json` | 3 KB | 4 KB | Gzip supported |
| `/askstories.json` | 1.5 KB | 2 KB | Gzip supported |
| `/showstories.json` | 1.5 KB | 2 KB | Gzip supported |
| `/jobstories.json` | 1 KB | 2 KB | Gzip supported |
| `/item/{id}.json` (story) | 1 KB | 5 KB | Gzip supported |
| `/item/{id}.json` (comment) | 500 bytes | 10 KB | Gzip supported |
| `/item/{id}.json` (job) | 2 KB | 10 KB | Gzip supported |
| `/user/{id}.json` | 3 KB | 20 KB | Gzip supported |
| `/maxitem.json` | 8 bytes | 10 bytes | Minimal |
| `/updates.json` | 3 KB | 5 KB | Gzip supported |

## Content Encoding

All endpoints support gzip compression. To enable:

**Request Header**:
```
Accept-Encoding: gzip
```

**Response Header**:
```
Content-Encoding: gzip
```

Compression typically reduces response sizes by 60-70%.

## Character Encoding

All responses use UTF-8 encoding:

**Response Header**:
```
Content-Type: application/json; charset=utf-8
```

## Parsing Considerations

### HTML in Text Fields
- `text`, `title`, `about` fields may contain HTML tags and entities
- Examples: `<p>`, `<i>`, `<a href="...">`, `&quot;`, `&#x27;`
- Must sanitize or render as HTML, not plain text

### Null vs Absent Fields
- Missing fields (e.g., no `kids` field): Field does not exist in JSON
- `null` response: Entire object is `null` (item/user not found)

### Integer Overflow
- Item IDs are u64 (currently ~40 million, max 2^64)
- Timestamps are u64 (Unix seconds)
- Scores are i32 (can be negative)
- Karma is i32

## Example: Fetching Top 10 Stories with Details

**Step 1**: Fetch story IDs
```http
GET /v0/topstories.json
Response: [39427470, 39426838, 39425902, ...]
```

**Step 2**: Take first 10 IDs
```
[39427470, 39426838, 39425902, 39427134, 39426521, 39424718, 39423981, 39424992, 39425513, 39427213]
```

**Step 3**: Fetch each story (concurrent)
```http
GET /v0/item/39427470.json
Response: {"by": "user1", "id": 39427470, "score": 150, "title": "Example Story", "type": "story", "url": "..."}

GET /v0/item/39426838.json
Response: {"by": "user2", "id": 39426838, "score": 120, "title": "Another Story", "type": "story", "url": "..."}

...
```

**Total Requests**: 11 (1 list + 10 items)
**Total Data**: ~15 KB uncompressed, ~5 KB gzipped

## Summary

- **Story Lists**: Arrays of integers (IDs)
- **Items**: Objects with type-specific fields
- **Users**: Objects with profile data
- **Max Item**: Single integer
- **Updates**: Object with `items` and `profiles` arrays
- **Errors**: `null` for not found, 503 for server errors
- **SSE Streams**: `put`, `patch`, `keep-alive` events
- **Encoding**: UTF-8 JSON with optional gzip
- **HTML**: Present in text fields, must sanitize

All responses are well-structured JSON (except `null` for missing resources), making parsing straightforward with serde in Rust.
