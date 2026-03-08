# Hacker News Firebase API - Data Types and Structures

## Core Data Types

The API returns two primary resource types: **Items** and **Users**.

## Items

Items represent all content on Hacker News. There are 6 item types, each with a slightly different schema.

### Item Type: `story`

Standard story submission (link or text post).

**Fields**:

| Field | Type | Required | Mutable | Description |
|-------|------|----------|---------|-------------|
| `id` | integer | Yes | No | Unique item identifier |
| `type` | string | Yes | No | Always `"story"` |
| `by` | string | No | No | Username of author (missing if deleted) |
| `time` | integer | Yes | No | Unix timestamp of creation |
| `title` | string | Yes | No | Story title (may contain HTML entities) |
| `url` | string | No | No | Story URL (missing for text posts) |
| `text` | string | No | No | Story text (for Ask HN, Show HN, or text stories) |
| `score` | integer | Yes | Yes | Current score (upvotes - downvotes) |
| `descendants` | integer | No | Yes | Total comment count (recursively) |
| `kids` | array[integer] | No | Yes | Array of comment IDs (top-level only) |
| `dead` | boolean | No | Yes | True if killed by moderators or flags |
| `deleted` | boolean | No | Yes | True if deleted by author |

**Example**:
```json
{
  "by": "dhouston",
  "descendants": 71,
  "id": 8863,
  "kids": [8952, 9224, 8917, 8884, 8887, 8943, 8869, 8958, 9005, 9671, 8940, 9067, 8908, 9055, 8865, 8881, 8872, 8873, 8955, 10403, 8903, 8928, 9125, 8998, 8901, 8902, 8907, 8894, 8878, 8870, 8980, 8934, 8876],
  "score": 111,
  "time": 1175714200,
  "title": "My YC app: Dropbox - Throw away your USB drive",
  "type": "story",
  "url": "http://www.getdropbox.com/u/2/screencast.html"
}
```

### Item Type: `comment`

User comment on a story or another comment.

**Fields**:

| Field | Type | Required | Mutable | Description |
|-------|------|----------|---------|-------------|
| `id` | integer | Yes | No | Unique item identifier |
| `type` | string | Yes | No | Always `"comment"` |
| `by` | string | No | No | Username of author |
| `time` | integer | Yes | No | Unix timestamp of creation |
| `text` | string | No | No | Comment text (HTML, missing if deleted) |
| `parent` | integer | Yes | No | Parent story or comment ID |
| `kids` | array[integer] | No | Yes | Array of child comment IDs |
| `dead` | boolean | No | Yes | True if killed |
| `deleted` | boolean | No | Yes | True if deleted |

**Example**:
```json
{
  "by": "norvig",
  "id": 2921983,
  "kids": [2922097, 2922429, 2924562, 2922709, 2922573, 2922140, 2922141],
  "parent": 2921506,
  "text": "Aw shucks, guys ... you make me blush with your compliments.<p>Tell you what, Ill make a deal: I'll keep writing if you keep reading. K?",
  "time": 1314211127,
  "type": "comment"
}
```

### Item Type: `ask`

Ask HN story (question post).

**Schema**: Same as `story`, but:
- `type` is `"ask"`
- Always has `text` field (question body)
- Usually no `url` field

**Example**:
```json
{
  "by": "tel",
  "descendants": 16,
  "id": 121003,
  "kids": [121016, 121109, 121168],
  "score": 25,
  "text": "<i>or</i> HN: the Next Iteration<p>I get the impression that with Arc being released a lot of people who never had time for HN before are suddenly dropping in more often. (PG: what are the numbers on this? I'm envisioning a spike.)<p>Not to say that isn't great, but I'm wary of Diggification. Between links comparing programming to sex and a flurry of gratuitous, ostentatious  adjectives in the headlines it's a bit concerning.<p>...<p>I'd like to get a community dialog going on this. What do you think?",
  "time": 1203647620,
  "title": "Ask HN: The Arc Effect",
  "type": "ask"
}
```

### Item Type: `job`

Job posting.

**Fields**:

| Field | Type | Required | Mutable | Description |
|-------|------|----------|---------|-------------|
| `id` | integer | Yes | No | Unique item identifier |
| `type` | string | Yes | No | Always `"job"` |
| `by` | string | No | No | Username of poster |
| `time` | integer | Yes | No | Unix timestamp of creation |
| `title` | string | Yes | No | Job title |
| `text` | string | No | No | Job description (HTML) |
| `url` | string | No | No | Application URL |
| `score` | integer | No | No | Jobs don't have scores (always 0 or missing) |

**Example**:
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

### Item Type: `poll`

Poll post with multiple choice options.

**Fields**:

| Field | Type | Required | Mutable | Description |
|-------|------|----------|---------|-------------|
| `id` | integer | Yes | No | Unique item identifier |
| `type` | string | Yes | No | Always `"poll"` |
| `by` | string | No | No | Username of author |
| `time` | integer | Yes | No | Unix timestamp of creation |
| `title` | string | Yes | No | Poll question |
| `text` | string | No | No | Poll description (HTML) |
| `score` | integer | Yes | Yes | Poll score |
| `descendants` | integer | No | Yes | Total comment count |
| `kids` | array[integer] | No | Yes | Comment IDs (not poll options) |
| `parts` | array[integer] | Yes | No | Array of pollopt IDs (the choices) |
| `dead` | boolean | No | Yes | True if killed |
| `deleted` | boolean | No | Yes | True if deleted |

**Example**:
```json
{
  "by": "pg",
  "descendants": 54,
  "id": 126809,
  "kids": [126822, 126823, 126993, 126824, 126934, 127411, 126888, 127681, 126818, 126816, 126854, 127095, 126861, 127313, 127299, 126859, 126852, 126882, 126832, 127072, 127217, 126889, 127535, 126917, 126875],
  "parts": [126810, 126811, 126812],
  "score": 46,
  "text": "",
  "time": 1204403652,
  "title": "Poll: What would happen if News.YC had explicit support for polls?",
  "type": "poll"
}
```

### Item Type: `pollopt`

Individual poll option (choice).

**Fields**:

| Field | Type | Required | Mutable | Description |
|-------|------|----------|---------|-------------|
| `id` | integer | Yes | No | Unique item identifier |
| `type` | string | Yes | No | Always `"pollopt"` |
| `by` | string | No | No | Username (same as poll author) |
| `time` | integer | Yes | No | Unix timestamp (same as poll) |
| `text` | string | Yes | No | Option text |
| `poll` | integer | Yes | No | Parent poll ID |
| `score` | integer | Yes | Yes | Number of votes for this option |

**Example**:
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

## Users

User profiles with public information.

**Fields**:

| Field | Type | Required | Mutable | Description |
|-------|------|----------|---------|-------------|
| `id` | string | Yes | No | Username (case-sensitive, unique) |
| `created` | integer | Yes | No | Account creation Unix timestamp |
| `karma` | integer | Yes | Yes | User karma score |
| `about` | string | No | Yes | User bio (HTML) |
| `submitted` | array[integer] | Yes | Yes | Array of submitted item IDs (stories, comments, polls, pollopts) |

**Example**:
```json
{
  "about": "This is a test",
  "created": 1173923446,
  "id": "jl",
  "karma": 2937,
  "submitted": [8265435, 8168423, 8090946, 8090326, 7699907, 7637962, 7596179, 7596163, 7594569, 7562135, 7562111, 7494708, 7494171, 7488093, 7444860, 7327817, 7280290, 7278694, 7097557, 7097546, 7097254, 7052857, 7039484, 6987273, 6649999, 6649706, 6629560, 6609127, 6327951, 6225810, 6111999, 5580079, 5112008, 4907948, 4901821, 4700469, 4678919, 3779193, 3711380, 3701405, 3627981, 3473004, 3473000, 3457006, 3422158, 3136701, 2943046, 2794646, 2482737, 2425640, 2411925, 2408077, 2407992, 2407940, 2278689, 2220295, 2144918, 2144852, 1875323, 1875295, 1857397, 1839737, 1809010, 1788048, 1780681, 1721745, 1676227, 1654023, 1651449, 1641019, 1631985, 1618759, 1522978, 1499641, 1441290, 1440993, 1436440, 1430510, 1430208, 1385525, 1384917, 1370453, 1346118, 1309968, 1305415, 1305037, 1304247, 1303960, 1303815, 1303793, 1303772, 1303755, 1303745, 1303740, 1303724, 1303718, 1303702, 1303689, 1303671, 1303666, 1303665, 1303662, 1303658, 1303656, 1303637]
}
```

## Special Response Values

### `null`
Returned when:
- Item has been deleted
- Item ID doesn't exist
- User doesn't exist

**Example**:
```http
GET /v0/item/999999999.json HTTP/1.1
Response: null
```

### `dead` flag
Items killed by moderators or community flags have `"dead": true`. These items still return data but are marked as dead.

### `deleted` flag
Items deleted by the original author have `"deleted": true` and usually have most fields removed (only `id`, `type`, `deleted` remain).

**Example**:
```json
{
  "deleted": true,
  "id": 1234567,
  "type": "comment"
}
```

## Field Type Details

### Timestamps (`time`, `created`)
- **Format**: Unix timestamp (seconds since epoch)
- **Type**: Integer
- **Example**: `1175714200` → 2007-04-04 20:23:20 UTC
- **Rust Conversion**: `std::time::UNIX_EPOCH + Duration::from_secs(time)`

### HTML Content (`text`, `about`, `title`)
- **Format**: HTML entities encoded (e.g., `<p>`, `&quot;`, `&#x27;`)
- **Type**: String
- **Rendering**: Must parse/sanitize HTML for display
- **Example**: `"This is <i>italic</i> and this is <a href=\"...\">a link</a>."`

### IDs (`id`, `parent`, `poll`, `kids`, `parts`, `submitted`)
- **Format**: Unsigned integers
- **Range**: 1 to ~40 million (as of 2026)
- **Uniqueness**: Globally unique across all items (stories, comments, jobs, polls, pollopts)
- **Sequential**: IDs are monotonically increasing (newer items have higher IDs)

### Usernames (`by`, `id` in User objects)
- **Format**: Alphanumeric string, case-sensitive
- **Example**: `"pg"`, `"dhouston"`, `"tptacek"`
- **Missing**: Field absent if user deleted account or item

### URLs (`url`)
- **Format**: String (not validated by API)
- **Protocol**: May be HTTP or HTTPS
- **Encoding**: URL-encoded
- **Example**: `"http://www.getdropbox.com/u/2/screencast.html"`

### Scores (`score`)
- **Format**: Integer (can be negative for heavily downvoted content)
- **Range**: 0 to ~8000 (typical max for top stories)
- **Calculation**: Approximately upvotes - downvotes (exact algorithm not public)
- **Mutable**: Changes as users vote

### Descendants Count (`descendants`)
- **Format**: Integer
- **Calculation**: Total count of all comments recursively (children + grandchildren + ...)
- **Mutable**: Increases when comments are added
- **Note**: Requires traversal of comment tree; not real-time

## Type Hierarchies

### Item Type Relationships

```
Item (abstract)
├── story
│   ├── standard story (has url)
│   ├── ask (has text, no url)
│   └── show (has text or url)
├── comment
├── job
├── poll
└── pollopt
```

### Data Relationships

```
Story
├── kids → Comment[]
│   └── kids → Comment[] (recursive)
└── by → User

Poll
├── parts → PollOpt[]
├── kids → Comment[]
└── by → User

User
└── submitted → Item[] (stories, comments, pollopts)
```

## Nullability Summary

| Field | Can be null | Can be absent | Conditions |
|-------|-------------|---------------|------------|
| `id` | No | No | Always present |
| `type` | No | No | Always present |
| `by` | No | Yes | Absent if user deleted |
| `time` | No | No | Always present |
| `title` | No | Yes | Only for stories, jobs, polls |
| `text` | No | Yes | Optional for stories, may be deleted |
| `url` | No | Yes | Only for stories with links |
| `score` | No | Yes | Not applicable for jobs |
| `descendants` | No | Yes | Only for stories/polls with comments |
| `kids` | No | Yes | Only if item has children |
| `parent` | No | Yes | Only for comments |
| `parts` | No | Yes | Only for polls |
| `poll` | No | Yes | Only for pollopts |
| `dead` | No | Yes | Only present if true |
| `deleted` | No | Yes | Only present if true |

## Rust Type Mapping

Recommended Rust structs for the connector:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum Item {
    Story(Story),
    Comment(Comment),
    Ask(Story), // Same as Story
    Job(Job),
    Poll(Poll),
    PollOpt(PollOpt),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Story {
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub by: Option<String>,
    pub time: u64,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    pub score: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub descendants: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kids: Option<Vec<u64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dead: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Comment {
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub by: Option<String>,
    pub time: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    pub parent: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kids: Option<Vec<u64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dead: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Job {
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub by: Option<String>,
    pub time: u64,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Poll {
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub by: Option<String>,
    pub time: u64,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    pub score: i32,
    pub parts: Vec<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub descendants: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kids: Option<Vec<u64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dead: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PollOpt {
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub by: Option<String>,
    pub time: u64,
    pub text: String,
    pub poll: u64,
    pub score: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct User {
    pub id: String,
    pub created: u64,
    pub karma: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub about: Option<String>,
    pub submitted: Vec<u64>,
}
```

This schema handles all nullable fields and type variations in the HN API.
