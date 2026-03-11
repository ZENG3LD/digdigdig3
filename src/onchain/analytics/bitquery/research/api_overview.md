# Bitquery API Overview

## Provider Information
- Full name: Bitquery
- Website: https://bitquery.io/
- Documentation: https://docs.bitquery.io/
- Category: data_feeds
- Type: Blockchain data provider (NO trading/exchange functionality)

## API Type
- REST: No (uses GraphQL exclusively)
- WebSocket: Yes (for GraphQL subscriptions - real-time streaming)
- GraphQL: Yes (primary interface)
  - V1 endpoint: https://graphql.bitquery.io
  - V2 endpoint (recommended): https://streaming.bitquery.io/graphql
- gRPC: Yes (Solana-specific)
- Other protocols: Kafka streams, SQL access, Cloud integrations

## Base URLs

### Production Endpoints
- **GraphQL V2 (Recommended)**: https://streaming.bitquery.io/graphql
- **GraphQL V1 (Legacy)**: https://graphql.bitquery.io
- **WebSocket (Subscriptions)**: wss://streaming.bitquery.io/graphql?token=YOUR_TOKEN
- **WebSocket EAP**: wss://streaming.bitquery.io/eap?token=YOUR_TOKEN
- **GraphQL IDE**: https://ide.bitquery.io

### Regional Endpoints
- No regional endpoints - single global endpoint

### API Versions
- **V2**: Current, production-ready (Streaming APIs)
- **V1**: Legacy, still supported but V2 recommended

### Testnet/Sandbox
- No separate testnet endpoint
- Free Developer plan with 10K trial points for testing

## Documentation Quality

### Official Documentation
- V2 Docs: https://docs.bitquery.io/
- V1 Docs: https://docs.bitquery.io/v1/
- Quality rating: **Excellent**
- Comprehensive guides for GraphQL query structure
- Extensive examples for all major use cases
- Well-organized by blockchain and data type

### Code Examples
- Yes - Multiple languages available:
  - Python (comprehensive)
  - JavaScript/TypeScript (comprehensive)
  - Rust (WebSocket examples available)
  - cURL (REST-like GraphQL calls)
- Example location: https://docs.bitquery.io/docs/subscriptions/examples/

### Interactive Tools
- **GraphQL IDE**: https://ide.bitquery.io
  - Browser-based query builder
  - Auto-completion (Ctrl+Space)
  - Schema explorer
  - Query history and saved queries
  - Real-time execution

### OpenAPI/Swagger Spec
- Not applicable (GraphQL uses introspection instead)
- GraphQL schema available via introspection queries
- Schema visible in IDE documentation panel

### Official SDKs
- No official SDK packages
- GraphQL client libraries recommended:
  - Python: `gql`, `requests`
  - JavaScript: `graphql-request`, `apollo-client`
  - Rust: `graphql_client`, `reqwest`

### Community Resources
- GitHub: https://github.com/bitquery/documentation
- GraphQL IDE GitHub: https://github.com/bitquery/graphql-ide
- Streaming Demo: https://github.com/bitquery/streaming-demo-view

## Licensing & Terms

### Free Tier
- Yes - Developer Plan
- Features:
  - 10,000 trial points (first month)
  - 10 rows per request limit
  - 10 requests/minute rate limit
  - 2 simultaneous streams for testing
  - Access to all 40+ blockchains
  - Public Telegram support

### Paid Tiers
- Yes - Commercial Plan (custom pricing)
- Enterprise features:
  - Custom point allocation
  - Unlimited rows per request
  - Scalable API calls (no throttling)
  - Unlimited simultaneous streams
  - 24/7 engineering team access
  - Priority Slack/Telegram support
  - Custom SLA
  - Multiple data interfaces (SQL, Cloud, Kafka)

### Commercial Use
- Free tier: Development and testing only
- Commercial use: Requires Commercial Plan
- Pricing: Contact sales (usage-based, pay-as-you-go)

### Data Redistribution
- Prohibited on free tier
- Commercial plan: Allowed with proper licensing
- Attribution: Not explicitly required but recommended
- Data resale: Requires enterprise agreement

### Terms of Service
- Terms URL: https://bitquery.io/privacy (includes Terms & Privacy)
- Key restrictions:
  - Fair use policy on free tier
  - No abuse/scraping
  - Commercial use requires paid plan
  - Point-based usage limits

## Support Channels

### Community Support (Free Tier)
- Public Telegram: Available
- Community forum: https://community.bitquery.io/
- GitHub Issues: For documentation/bug reports

### Paid Support (Commercial Plan)
- Email: support@bitquery.io
- Ticket system: https://support.bitquery.io
- Priority Slack channel: Dedicated channel
- Priority Telegram: Direct access
- 24/7 Engineering team: For Commercial customers

### Status & Uptime
- Status page: Not explicitly mentioned
- Monitoring: Available in account dashboard
- SLA: Custom SLA for enterprise customers

### Response Times
- Free tier: Community-driven (best effort)
- Commercial: 24/7 support with SLA guarantees
- Onboarding: Dedicated onboarding for Commercial plan

## Additional Features

### Data Export Formats
- JSON (default for GraphQL)
- Protocol Buffers (Kafka streams)
- Parquet (data warehouse exports)
- Custom formats available on request

### Cloud Integrations
- AWS (Redshift, S3)
- Google Cloud (BigQuery)
- Microsoft Azure
- Snowflake
- Databricks
- Tencent Cloud

### Real-time Capabilities
- GraphQL Subscriptions (WebSocket)
- Sub-second latency for realtime dataset
- Kafka streaming for continuous pipelines
- Live data across 40+ blockchains

## Use Cases
- DeFi analytics platforms
- NFT marketplaces and trackers
- Blockchain explorers
- Trading bots (data only - no execution)
- Portfolio trackers
- On-chain analytics
- Smart contract monitoring
- Token holder analysis
- DEX aggregation
- Mempool monitoring

## Key Differentiators
1. **Multi-chain coverage**: 40+ blockchains including EVM and non-EVM
2. **GraphQL-first**: Flexible querying with exact data selection
3. **Real-time streaming**: WebSocket subscriptions for live data
4. **Historical depth**: Complete data from blockchain genesis
5. **Enterprise integrations**: SQL, Kafka, cloud data warehouses
6. **No trading functionality**: Pure data provider (not an exchange)
