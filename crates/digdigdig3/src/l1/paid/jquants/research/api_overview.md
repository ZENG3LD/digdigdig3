# JQuants API Overview

## Provider Information
- Full name: J-Quants API (Japan Exchange Group Quantitative Data API)
- Website: https://jpx-jquants.com/en
- Documentation: https://jpx.gitbook.io/j-quants-en/
- Category: stocks/japan
- Provider: JPX Market Innovation & Research, Inc. (JPXI)
- Parent organization: Japan Exchange Group (JPX)

## API Type
- REST: Yes (base URL: https://api.jquants.com)
- WebSocket: No (data-only API, no real-time streaming)
- GraphQL: No
- gRPC: No
- Other protocols: CSV/SFTP bulk downloads (J-Quants Pro only)

## Base URLs
- Production: https://api.jquants.com/v1/ (legacy, being phased out)
- Production V2: https://api.jquants.com/v2/ (current)
- Pro API: https://api.jquants-pro.com/v2/ (enterprise version)
- Testnet/Sandbox: Not available
- Regional endpoints: None (Japan-focused service)
- API version: v2 (current), v1 (legacy, being deprecated)

## Documentation Quality
- Official docs: https://jpx.gitbook.io/j-quants-en/
- Quality rating: Good
- Code examples: Yes (languages: Python, R, Rust, Julia, Java, Common Lisp)
- OpenAPI/Swagger spec: Not publicly available
- SDKs available:
  - Python: https://github.com/J-Quants/jquants-api-client-python
  - R: https://github.com/J-Quants/JQuantsR
  - Rust: https://crates.io/crates/jquants-api-client
  - Julia: JQuants.jl package
  - Java: https://github.com/J-Quants/jquants-api-jvm
  - Common Lisp: https://github.com/minisoba/cl-jquants-api
- Quick start: https://github.com/J-Quants/jquants-api-quick-start

## Licensing & Terms
- Free tier: Yes (12-week delayed data, 2 years history)
- Paid tiers: Yes (Light, Standard, Premium)
- Commercial use: Allowed (varies by tier)
- Data redistribution: Prohibited (except with specific licensing)
- Terms of Service: https://jpx-jquants.com/en (see registration page)
- Data source: Official exchange data from Tokyo Stock Exchange (TSE), Osaka Exchange (OSE)

## Support Channels
- Email: Support available through user portal
- Discord/Slack: Not publicly available
- GitHub: https://github.com/J-Quants (official client libraries)
- Status page: Not publicly available
- User portal: https://jpx-jquants.com/en (after registration)

## Special Features
- Official Japanese stock market data provider
- Data-only API (NO trading capabilities)
- Covers Tokyo Stock Exchange (TSE/JPX) exclusively
- Recently added (January 2026): CSV bulk downloads and minute/tick data
- Two product lines:
  - J-Quants API: For individual investors
  - J-Quants Pro: For corporate users (API + SFTP + Snowflake integration)

## Recent Updates (2026)
- **January 19, 2026**: CSV-format bulk delivery and minute bar/tick data added
- **December 2025**: API V2 released with API key-based authentication (replacing token-based)
- V1 is being phased out, users encouraged to migrate to V2

## Important Limitations
- **Data-only service**: No order placement or trading capabilities
- **Japan-focused**: Only Tokyo Stock Exchange data
- **No real-time WebSocket**: All data via REST API polling
- **Free tier delay**: 12-week delay on all data
- **Geographical restriction**: May have regional access restrictions
