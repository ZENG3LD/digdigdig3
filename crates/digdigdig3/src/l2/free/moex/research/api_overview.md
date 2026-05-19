# MOEX API Overview

## Provider Information
- Full name: Moscow Exchange (Московская Биржа, MOEX)
- Website: https://www.moex.com
- Documentation: https://www.moex.com/a2193
- Category: stocks/russia
- Type: Data Provider (Market Data Only - NO TRADING)

## API Type
- REST: Yes (base URL: https://iss.moex.com/iss)
- WebSocket: Yes (URL: wss://iss.moex.com/infocx/v3/websocket)
- GraphQL: No
- gRPC: No
- Other protocols: STOMP over WebSocket for real-time data

## Base URLs
- Production REST: https://iss.moex.com/iss
- Production WebSocket: wss://iss.moex.com/infocx/v3/websocket
- Testnet/Sandbox: Not available
- Regional endpoints: None (Russia-based)
- API version: ISS v0.14.1 (Informational & Statistical Server)

## Documentation Quality
- Official docs: https://iss.moex.com/iss/reference/
- Quality rating: Good (comprehensive endpoint reference, Russian language dominant)
- Code examples: Yes (Python and Visual Basic .NET examples available)
- OpenAPI/Swagger spec: Not available
- SDKs available:
  - JavaScript: https://github.com/timmson/moex-api
  - Python: https://github.com/panychek/moex (aiomoex on PyPI)
  - R: moexer package https://github.com/x1o/moexer
  - Go: https://github.com/Ruvad39/go-moex-iss
  - PHP: https://packagist.org/packages/panychek/moex

## Licensing & Terms
- Free tier: Yes (delayed data - 15 minutes delay)
- Paid tiers: Yes (real-time data requires subscription)
- Commercial use: Requires license and subscription
- Data redistribution: Prohibited without contract with Moscow Exchange
- Terms of Service: Data is for informational purposes only, cannot be used for profit or third-party services without contract
- Key restriction: "The information obtained from ISS is available for informational purposes only"

## Support Channels
- Email: help@moex.com
- Phone: +7 (495) 733-9507 (technical questions)
- Discord/Slack: Not available
- GitHub: Community-maintained client libraries
- Status page: Not available
- Client support: Contact personal manager for API access approval

## Trading Systems (Engines)
MOEX operates 11 distinct trading engines:
1. **Stock** (id: 1) - Фондовый рынок и рынок депозитов (Equities and deposits)
2. **State** (id: 2) - Рынок ГЦБ (размещение) (Government securities placement)
3. **Currency** (id: 3) - Валютный рынок (Foreign exchange)
4. **Futures** (id: 4) - Срочный рынок (Derivatives market)
5. **Commodity** (id: 5) - Товарный рынок (Commodity market)
6. **Interventions** (id: 6) - Товарные интервенции (Commodity interventions)
7. **Offboard** (id: 7) - ОТС-система (OTC system)
8. **Agro** (id: 9) - Агро (Agricultural products)
9. **OTC** (id: 1012) - ОТС с ЦК (OTC with central counterparty)
10. **Quotes** (id: 1282) - Квоты (Quote systems)
11. **Money** (id: 1326) - Денежный рынок (Money market)

## Market Structure
- 120+ markets across all engines
- 500+ trading boards with granular trading parameters
- Settlement models: T+, T0, addressable
- Currencies: RUB, USD, EUR, CNY, HKD, KZT, BYN
- 90+ security types cataloged
- Primary focus: Russian equities, bonds, and derivatives

## Additional APIs
MOEX offers several specialized APIs:
- **ISS (Informational & Statistical Server)** - REST/WebSocket market data (this research)
- **FIX Protocol** - FIX 4.4 for trading (fastest transactional interface)
- **ASTS Bridge** - Native API for Equities, Bonds, FX, Money markets (trading + clearing)
- **PLAZA II** - Native API for Derivatives market (trading + clearing)
- **WebAPI** - Trading API with OAuth 2.0 authentication

**Note**: This research focuses on ISS for market data only. Trading capabilities require separate APIs (FIX/ASTS/PLAZA II).
