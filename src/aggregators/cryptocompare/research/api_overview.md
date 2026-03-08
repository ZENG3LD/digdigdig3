# CryptoCompare API Overview

## Provider Information
- Full name: CryptoCompare (now part of CoinDesk/CCData)
- Website: https://www.cryptocompare.com
- Documentation: https://min-api.cryptocompare.com/documentation
- Category: aggregators
- Status: Active, acquired by CoinDesk (now operates under CCData brand)

## API Type
- REST: Yes (base URL: https://min-api.cryptocompare.com)
- WebSocket: Yes (URL: wss://streamer.cryptocompare.com/v2)
- GraphQL: No
- gRPC: No
- Other protocols: None

## Base URLs
- Production REST: https://min-api.cryptocompare.com
- Alternative REST: https://data-api.cryptocompare.com (newer endpoints)
- WebSocket: wss://streamer.cryptocompare.com/v2
- Testnet/Sandbox: Not available
- Regional endpoints: None (global)
- API version: v1 (REST), v2 (WebSocket)

## Documentation Quality
- Official docs: https://min-api.cryptocompare.com/documentation
- Quality rating: Good (comprehensive but some redirects to CoinDesk)
- Code examples: Yes (JavaScript, Python examples in community SDKs)
- OpenAPI/Swagger spec: Not publicly available
- SDKs available:
  - JavaScript/Node.js (community: ExodusMovement/cryptocompare)
  - Python (community: ttsteiger/cryptocompy)
  - C# (community: trakx/cryptocompare-api-client, LadislavBohm/cryptocompare-streamer)
  - Elixir (community SDK available)
- Note: Official documentation has migrated to CoinDesk Developers portal for some sections

## Licensing & Terms
- Free tier: Yes (requires sign-up and API key)
- Paid tiers: Yes (Starter ~$80/mo, Professional ~$200/mo, Enterprise custom)
- Commercial use: Allowed with paid tiers
- Data redistribution: Prohibited without license (attribution required for free tier)
- Terms of Service: https://www.cryptocompare.com/terms-of-use/
- Attribution: Free tier users MUST credit CryptoCompare in their applications

## Support Channels
- Email: Available for paid customers
- Discord/Slack: Community forums available
- GitHub: https://github.com/CryptoCompareLTD (official)
- Status page: Not publicly available
- Developer portal: https://developers.coindesk.com (after CoinDesk acquisition)
- API key management: https://www.cryptocompare.com/cryptopian/api-keys

## Key Features
- 5,700+ cryptocurrencies coverage
- 170+ exchanges aggregated (some sources report 316 exchanges)
- 260,000+ trading pairs
- Real-time and historical data
- CCCAGG (CryptoCompare Aggregate Index) - proprietary volume-weighted index
- Social media metrics (Reddit, Twitter, Facebook, GitHub)
- News aggregation
- Blockchain data
- WebSocket streaming for real-time updates
- Up to 40,000 API calls per second burst capacity (enterprise)

## Notes
- CryptoCompare was acquired by CoinDesk and now operates under the CCData brand
- Some documentation URLs redirect to developers.coindesk.com
- API keys from CryptoCompare continue to work
- Authentication system has been migrated to Auth0 (visible in CoinDesk integration)
