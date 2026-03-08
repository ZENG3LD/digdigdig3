# Hacker News Firebase API - Overview

## Provider Information

- **Name**: Hacker News API
- **Type**: Public News Aggregator Feed
- **Category**: Data Feeds (Social News)
- **Base URL**: `https://hacker-news.firebaseio.com/v0/`
- **Authentication**: None (Public API)
- **Protocol**: REST + Firebase Real-Time Streaming
- **Format**: JSON

## API Description

The Hacker News API is a Firebase-powered public API providing real-time access to Hacker News stories, comments, jobs, polls, and user data. Built in partnership with Firebase (now part of Google), it exposes HN's in-memory data structures through a simple REST interface with optional real-time streaming capabilities.

## Key Features

1. **No Authentication Required**: Completely public, no API keys or registration needed
2. **No Rate Limits**: Currently no enforced rate limiting
3. **Real-Time Updates**: Firebase SSE (Server-Sent Events) streaming support for live data
4. **Stable Versioning**: Only breaking changes involve removal of required fields or alteration of existing fields
5. **Multi-Platform**: Native Firebase SDKs available for Web, iOS, Android, and servers

## Data Model

The API exposes two primary resource types:

### Items
Items represent all content on HN:
- **Stories**: Standard submissions with URLs or text
- **Comments**: Replies to stories or other comments
- **Jobs**: Job postings
- **Ask HN**: Text-based questions
- **Show HN**: Project showcases
- **Polls**: Voting questions with multiple options
- **Poll Options**: Individual poll choices

### Users
User profiles with public activity, karma, and submission history.

## Primary Use Cases

1. **News Aggregation**: Building HN readers and aggregators
2. **Real-Time Monitoring**: Tracking trending stories and discussions
3. **Analytics**: Analyzing HN content, voting patterns, user behavior
4. **Notifications**: Alerting on new stories, comments, or keywords
5. **Archival**: Historical data collection and research

## API Design Philosophy

According to the official documentation, the API is "essentially a dump of our in-memory data structures." This means:
- No pre-computed aggregations (e.g., comment counts require tree traversal)
- Raw data exposure with minimal transformation
- Client-side processing expected for complex queries
- Simple, predictable structure

## Versioning

- **Current Version**: v0
- **Stability**: Breaking changes limited to required field removal or existing field modification
- **Forward Compatibility**: Clients should gracefully handle unexpected new fields

## Official Resources

- **Documentation**: https://github.com/HackerNews/API
- **Base Endpoint**: https://hacker-news.firebaseio.com/v0/
- **Support Email**: api@ycombinator.com
- **Firebase Blog**: https://firebase.blog/posts/2014/10/hacker-news-now-has-api-its-firebase/

## Rate Limits & Performance

- **Rate Limit**: None currently enforced
- **Response Time**: Typically <100ms for individual items
- **Concurrency**: Recommended limit of 10 concurrent requests to prevent fan-out issues
- **Caching**: Items are immutable once created (except scores/descendants)

## Data Freshness

- **Update Frequency**: Near real-time (seconds)
- **Max Item ID**: Updated continuously as new items are posted
- **Story Lists**: Top/new/best lists updated every few minutes
- **Scores**: Update as users vote

## Limitations

1. **No Search**: No built-in search functionality (use Algolia HN Search API separately)
2. **No Filtering**: Must fetch and filter client-side
3. **Tree Traversal**: Comment threads require recursive fetching
4. **No Batch Endpoints**: Must fetch items individually
5. **Limited Historical Access**: Only current state, no edit history

## Integration Considerations

For a Rust connector:
- Use reqwest for HTTP/REST calls
- Implement SSE streaming with eventsource-client or similar for real-time updates
- Consider caching immutable item fields (title, url, text)
- Implement concurrent fetching with tokio for story lists
- Handle missing/deleted items gracefully (return None instead of error)
