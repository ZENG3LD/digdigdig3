# Whale Alert API Overview

## Provider Information
- Full name: Whale Alert
- Website: https://whale-alert.io/
- Documentation: https://developer.whale-alert.io/documentation/
- Category: data_feeds

## API Type
- REST: Yes (base URL: https://leviathan.whale-alert.io for Enterprise, https://api.whale-alert.io/v1 for Developer API)
- WebSocket: Yes (URL: wss://leviathan.whale-alert.io/ws)
- GraphQL: No
- gRPC: No
- Other protocols: None

## Base URLs
- Production (Enterprise REST API): https://leviathan.whale-alert.io
- Production (Developer API v1 - Deprecated): https://api.whale-alert.io/v1
- WebSocket (Custom Alerts): wss://leviathan.whale-alert.io/ws?api_key=YOUR_API_KEY
- WebSocket (Priority Alerts): wss://leviathan.whale-alert.io/ws?api_key=YOUR_API_KEY (same endpoint, different tier)
- Testnet/Sandbox: Not available
- Regional endpoints: None
- API version: v1 (Developer API), v2 (Enterprise API)

## Documentation Quality
- Official docs: https://developer.whale-alert.io/documentation/
- Quality rating: Good
- Code examples: Yes (languages: Go, JavaScript/Node.js examples available in GitHub)
- OpenAPI/Swagger spec: Not available
- SDKs available: No official SDKs, community implementations (Python, Go examples in GitHub)

## Licensing & Terms
- Free tier: Yes (Developer API v1 - deprecated but still functional)
- Paid tiers: Yes (Custom Alerts, Priority Alerts, Quantitative, Historical)
- Commercial use: Allowed with appropriate tier
- Data redistribution: Prohibited without permission
- Terms of Service: https://whale-alert.io/ (referenced in documentation)

## Support Channels
- Email: Contact available through developer portal
- Discord/Slack: Not publicly listed
- GitHub: https://github.com/whale-alert/ (example repositories)
- Status page: Not publicly available

## Additional Notes
- Whale Alert specializes in tracking large blockchain transactions ("whale" movements)
- Provides real-time alerts for significant cryptocurrency transactions across major blockchains
- Offers address attribution data for over 400 entities (exchanges, wallets, etc.)
- Data is scientifically validated and used by institutional traders
- Services track transactions across 11+ blockchains including Bitcoin, Ethereum, Solana, Tron, Polygon, etc.
