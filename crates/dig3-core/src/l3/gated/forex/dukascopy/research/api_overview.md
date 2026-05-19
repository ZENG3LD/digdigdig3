# Dukascopy API Overview

## Provider Information
- Full name: Dukascopy Bank SA (Swiss forex and CFD provider)
- Website: https://www.dukascopy.com
- Documentation: https://www.dukascopy.com/wiki/en/development/strategy-api/
- Category: forex
- Headquarters: Geneva, Switzerland
- Specialty: Historical tick data, forex data provider

## API Type
- REST: No official REST API (third-party wrappers available)
- WebSocket: No official WebSocket API (third-party implementations exist)
- GraphQL: No
- gRPC: No
- Other protocols:
  - **JForex SDK (Java)**: Primary official API - Java-based SDK for trading and data access
  - **FIX 4.4 Protocol**: Professional trading API
  - **Binary Data Files**: Direct access to historical tick data via .bi5 files

## Base URLs

### JForex SDK
- Production: Requires SDK integration (not REST-based)
- Demo: Available via ITesterClient interface
- Maven Repository: https://www.dukascopy.com/maven/

### FIX API
- Trading Gateway: SSL port 10443
- Data Feed: SSL port 9443
- Protocol: FIX 4.4
- Connection: SSL-encrypted TCP sockets

### Historical Data (Direct Download)
- Base URL: `https://datafeed.dukascopy.com/datafeed/`
- Structure: `{BASE_URL}/{SYMBOL}/{YYYY}/{MM}/{DD}/{HH}h_ticks.bi5`
- Example: `https://datafeed.dukascopy.com/datafeed/EURUSD/2024/02/15/14h_ticks.bi5`
- Format: LZMA-compressed binary files (.bi5)
- Timezone: UTC+0

### Third-Party REST/WebSocket (Unofficial)
- GitHub: https://github.com/ismailfer/dukascopy-api-websocket
- REST Port: 7080
- WebSocket Port: 7081
- Note: Community-built wrapper around JForex SDK

## Documentation Quality
- Official docs: https://www.dukascopy.com/wiki/en/development/strategy-api/
- Quality rating: Good (comprehensive for JForex SDK, limited for other access methods)
- Code examples: Yes (Java primarily)
- OpenAPI/Swagger spec: Not available
- SDKs available:
  - Official: JForex SDK (Java)
  - Community: Python (dukascopy-python, TickVault), Node.js (dukascopy-node), Dart, .NET (Dukas.Net), Go (go-duka)
- Javadocs: https://www.dukascopy.com/client/javadoc3/com/dukascopy/api/
- FIX API Documentation: PDF available (Revision 8.8.1)

## Licensing & Terms
- Free tier: Yes (demo account for JForex SDK)
  - Demo account: Extended validity available (no expiration for data access)
  - Historical data: Free access via demo account
  - Tick data downloads: Free via public datafeed URLs
- Paid tiers: Yes (live trading accounts)
  - Minimum deposit for FIX API: USD 100,000
- Commercial use:
  - Personal/Non-commercial: Free with standard license
  - Commercial: Requires signed supplementary agreement
  - License: "Non-exclusive, non-transferable, worldwide license for personal, non-commercial use"
- Data redistribution: Restricted (requires agreement)
- Terms of Service: https://www.dukascopy.com/swiss/english/home/terms-of-use/

## Support Channels
- Email: support@dukascopy.com
- Discord/Slack: Not available
- GitHub: Community projects available (unofficial)
- Status page: Not available
- Community: Forex Factory, various trading forums
- Wiki: https://www.dukascopy.com/wiki/

## Rate Limits & Restrictions
- JForex SDK: No explicit rate limits documented (fair use policy)
- FIX API:
  - Connection attempts: 5 per minute per server per IP
  - Max orders per second: 16
  - Max open positions: 100
- Historical data downloads: Rate limiting in place (since Oct 12, 2018)
  - Specific limits not publicly documented
  - Community reports suggest throttling after bulk downloads
- IP Registration: Required for FIX API connections

## Data Access Methods Summary

1. **JForex SDK (Recommended for live/historical data)**
   - Full API access
   - Historical data via IHistory interface
   - Real-time data via live feeds
   - Requires Java

2. **FIX API (Professional trading)**
   - Real-time market data
   - Order management
   - High minimum deposit requirement

3. **Direct Binary Downloads (Best for bulk historical data)**
   - Free tick data access
   - Hourly .bi5 files
   - Requires LZMA decompression
   - No authentication needed

4. **Third-Party Wrappers (Community solutions)**
   - REST/WebSocket interfaces
   - Multiple language support
   - Unofficial, may break with updates
