# Interactive Brokers Client Portal Web API - Overview

## Provider Information

**Name:** Interactive Brokers (IBKR)
**Category:** Aggregator (Multi-Asset Broker)
**API Name:** Client Portal Web API (cpwebapi)
**Documentation:** https://interactivebrokers.github.io/cpwebapi/
**Official Docs:** https://www.interactivebrokers.com/campus/ibkr-api-page/cpapi-v1/

## Overview

Interactive Brokers' Client Portal Web API is a comprehensive REST and WebSocket API that provides real-time access to Interactive Brokers' trading functionality, including live market data, market scanners, intra-day portfolio updates, order management, and account information. The API enables clients to communicate directly with IBKR infrastructure both synchronously using HTTP endpoints and asynchronously via WebSocket connections.

## Asset Classes Supported

Interactive Brokers is a **multi-asset broker** supporting trading across:

- **Stocks (Equities)** - Global stocks on 150+ markets
- **Forex (FX/CASH)** - Spot FX trading with IDEALPRO and Virtual Forex
- **Futures** - Commodity, index, currency futures
- **Options** - Equity options, index options, futures options
- **Bonds** - Government and corporate bonds
- **Mutual Funds** - Mutual funds and ETFs
- **Cryptocurrencies** - Crypto trading (limited availability)
- **CFDs** - Contracts for difference (non-US accounts)
- **Warrants** - European warrants
- **Structured Products** - Various structured instruments

## Trading vs Market Data Provider

**Trading Support:** YES - Full trading capabilities
**Market Data Support:** YES - Real-time and historical market data

This is a **full-service broker with both trading and market data capabilities**.

## Key Features

### Trading Capabilities
- Order placement, modification, and cancellation
- Support for 10+ order types (Market, Limit, Stop, Trailing Stop, etc.)
- Bracket orders for risk management
- Combo/spread orders for complex strategies
- Algorithmic order parameters (limited)
- Order preview (What-If orders)
- Real-time order status updates
- Execution reports and fill notifications

### Market Data Capabilities
- Real-time market data snapshots (top-of-book)
- Level 2 market data (depth of book)
- Historical market data (bars/candles)
- Multiple timeframes (1 second to monthly bars)
- Market scanners for opportunity discovery
- Contract search and discovery
- Options analytics (CBOE data)
- Event contracts trading

### Account Management
- Multi-account support
- Real-time portfolio positions
- Account summary and P&L
- Margin and equity monitoring
- Account ledger history
- Financial Advisor (FA) account allocation groups
- Sub-account management

### Additional Features
- Watchlist management
- Custom alerts (price, time, margin, trade, volume)
- Notifications system (FYI)
- Portfolio analytics
- Flex Web Service for reporting
- WebSocket streaming for real-time updates

## API Architecture

### Communication Protocols
- **REST API:** HTTPS-based synchronous requests
- **WebSocket:** WSS-based asynchronous streaming

### Authentication Methods
- **Client Portal Gateway** (Java-based, for individual accounts)
- **OAuth 2.0** (for enterprise clients)
- **OAuth 1.0a Extended** (legacy support)
- **SSO (Single Sign-On)** (for institutional clients)

### Base URLs
- **Gateway (Local):** `https://localhost:5000/v1/api/`
- **Production:** `https://api.ibkr.com/v1/api/`
- **WebSocket (Local):** `wss://localhost:5000/v1/api/ws`
- **WebSocket (Production):** `wss://api.ibkr.com/v1/api/ws`

## Account Requirements

### Prerequisites
- Active IBKR account (must be IBKR Pro for API access)
- Funded account
- Fully activated account
- Two-factor authentication (mandatory)
- Market data subscriptions (for real-time data)
- Signed market data agreements

### Restrictions
- **Canadian Residents:** Programmatic trading on Canadian exchanges is prohibited
- **Geographic:** Some features restricted based on account location
- **Concurrent Sessions:** Single username can only be signed in once at any given time
- **Account Type:** Must be IBKR Pro (not IBKR Lite)

## API Versions and Stability

### Current Version
- **Version:** v1.0 (as of 2024)
- **Status:** Active development, documented as "in beta and subject to change"
- **Legacy Support:** Previous API versions remain accessible

### Unified Web API Initiative
IBKR is merging their web-based API products into a single, comprehensive IBKR Web API:
- Client Portal Web API (Trading)
- Digital Account Management (Account operations)
- Flex Web Service (Reporting)

All unified under **OAuth 2.0 authorization**.

## Rate Limits

### Global Rate Limits
- **Default:** 10 requests per second per authenticated session
- **Client Portal Gateway Users:** 10 requests per second
- **OAuth Users:** 50 requests per second per authenticated username

### Endpoint-Specific Limits
- `/tickle` (keep-alive): 1 request per second
- `/sso/validate`: 1 request per minute
- `/iserver/account/{accountId}/orders` (POST/GET): 1 request per 5 seconds
- `/iserver/marketdata/snapshot`: 10 requests per second
- `/iserver/marketdata/history`: 5 concurrent requests maximum
- `/iserver/scanner/params`: 1 request per 15 minutes
- `/iserver/scanner/run`: 1 request per second
- `/fyi/*` (notification endpoints): 1 request per second
- `/pa/*` (portfolio analyst): 1 request per 15 minutes
- All other `/iserver/*` endpoints: Follow global limit (10 req/s)

### Rate Limit Violations
- **Response:** HTTP 429 Too Many Requests
- **Penalty:** IP address penalized for 15 minutes
- **Repeat Offenders:** Permanent IP block possible

## Session Management

### Session Types
1. **Read-Only Session:** Outer prerequisite session for non-`/iserver` endpoints
2. **Brokerage Session:** Required for `/iserver` endpoints (trading, market data)

### Session Timeouts
- **Idle Timeout:** ~6 minutes without requests
- **Maximum Session:** 24 hours
- **Session Reset:** Midnight Eastern Time (New York), Central European Time (Zug), or Hong Kong Time
- **Keep-Alive:** Use `/tickle` endpoint at max 1 req/sec to maintain session

### Session Authentication
Individual accounts must authenticate via browser on the same machine as Client Portal Gateway. There is **no mechanism to automate brokerage session authentication** for individual accounts.

## Data Format

### Request Format
- **Content-Type:** `application/json`
- **Encoding:** UTF-8
- **POST/PATCH Bodies:** JSON payload

### Response Format
- **Content-Type:** `application/json`
- **Status Codes:** Standard HTTP status codes
- **Error Format:** JSON with error messages and codes

### Common Headers
- `Host`
- `User-Agent`
- `Accept`
- `Connection`
- `Content-Length` (for POST/PUT/PATCH)
- `Content-Type: application/json`

## Contract Identification

### Contract ID (conid)
IBKR uses unique **Contract IDs (conid)** to identify instruments:
- Static identifiers that never change
- Example: `265598` = Apple Inc. (AAPL) stock
- Required for: Market data, trading, contract details

### Contract Search
Multiple methods to retrieve conids:
- `/iserver/secdef/search` - Symbol search by security type
- `/iserver/secdef/info` - Detailed contract information
- `/iserver/contract/{conid}` - Contract details by conid
- `/iserver/contract/search` (POST) - Advanced contract search

### Security Types (secType)
- `STK` - Stocks
- `OPT` - Options
- `FUT` - Futures
- `CASH` - Forex
- `BOND` - Bonds
- `CFD` - Contracts for Difference
- `WAR` - Warrants
- `IND` - Indices
- `FUND` - Mutual Funds

## Market Data Subscriptions

### Limitations
- **Concurrent Subscriptions:** Typically 100 simultaneous market data lines for standard accounts
- **Market Data Permissions:** Must have active market data subscriptions
- **Data Agreements:** Must sign data agreements for each exchange
- **Streaming Allocation:** WebSocket subscriptions consume market data lines

### Available Data
- Top-of-book quotes (Bid, Ask, Last)
- Market depth (Level 2)
- Historical bars (from 1 second to monthly)
- Options chains
- Implied volatility
- Greeks (for options)
- Volume and VWAP
- Real-time updates via WebSocket

## Client Portal Gateway

### Purpose
Java-based local proxy that:
- Routes requests between local client and IBKR backend
- Handles brokerage session authentication
- Manages SSL/TLS certificates
- Provides local HTTPS endpoint

### Configuration
- **Default Port:** 5000
- **Configurable:** Via `conf.yaml` in gateway root directory
- **SSL Certificate:** Customizable via `sslCert` and `sslPwd` fields
- **Listen Port:** Modifiable via `listenPort` field

### Deployment
- **Requirement:** Must run on same machine as API client
- **Browser Authentication:** User must authenticate via browser on same machine
- **No Automation:** No automated login for individual accounts
- **TWO_FA:** Two-factor authentication required during login

## Known Limitations

### Individual Account Restrictions
- No automated authentication (manual browser login required)
- Single concurrent session per username
- Must use Client Portal Gateway (Java dependency)
- Canadian residents cannot trade Canadian exchanges programmatically

### API Feature Limitations
- Many TWS API order types not available in Client Portal API
- Limited algorithmic order support
- No support for certain advanced order types (Auction, Block, Discretionary, Market-to-Limit)
- Market data subscriptions required for real-time data
- Some endpoints require specific account permissions

### Technical Constraints
- JavaScript required for documentation website
- WebSocket SSL verification typically disabled in examples
- Historical data: <30 second bars only available for 6 months
- Flex Query execution reports delayed 5-10 minutes (not real-time)

## Use Cases

### Ideal For
- Algorithmic trading across multiple asset classes
- Portfolio management and rebalancing
- Multi-strategy trading systems
- Options trading and analysis
- Market scanning and opportunity discovery
- Integration with custom trading applications
- Professional traders and institutions
- Quantitative research and backtesting (via historical data)

### Not Ideal For
- High-frequency trading (rate limits too restrictive)
- Fully automated retail systems (requires manual authentication)
- Crypto-focused trading (limited crypto support)
- Canadian exchange trading (geographic restriction)

## Comparison with Other IBKR APIs

### Client Portal Web API vs TWS API
- **Client Portal:** REST/WebSocket, modern architecture, limited order types
- **TWS API:** Socket-based, more features, more complex, desktop app required

### Client Portal Web API vs FIX API
- **Client Portal:** Easier integration, rate-limited, for individual/small institutional
- **FIX:** Ultra low-latency, full features, for large institutional clients

## Security Considerations

### SSL/TLS
- All communication over HTTPS/WSS
- Self-signed certificates common with Gateway (local development)
- Production OAuth endpoints use valid SSL certificates

### Authentication Security
- Two-factor authentication mandatory
- OAuth tokens for enterprise clients
- HMAC-SHA256 signature for OAuth 1.0a
- Private key JWT (RFC 7521/7523) for OAuth 2.0
- No client secret in OAuth 2.0 (more secure)

### Best Practices
- Store credentials securely (never in code)
- Use environment variables for sensitive data
- Implement proper error handling for authentication failures
- Monitor session timeouts and re-authenticate gracefully
- Respect rate limits to avoid IP bans
- Validate all user input before sending to API

## Documentation Quality

### Strengths
- Comprehensive endpoint reference
- Interactive API documentation (ReDoc-based)
- Code examples in multiple languages
- Detailed tutorials and lessons on IBKR Campus
- Active community support

### Weaknesses
- Documentation marked as "beta and subject to change"
- JavaScript required for docs website (not static)
- Some endpoints lack detailed examples
- Field ID definitions scattered across docs
- OAuth 2.0 documentation less detailed than OAuth 1.0a

## Community and Support

### Official Resources
- **IBKR Campus:** https://www.interactivebrokers.com/campus/ibkr-api-page/
- **GitHub:** https://github.com/interactivebrokers/
- **Release Notes:** https://ibkrguides.com/releasenotes/api/cp-web/

### Community Libraries
- **Python:** Multiple wrappers (EasyIB, ibind, interactive-broker-python-api)
- **Go:** ibclient
- **Elixir:** ibkr_api
- **Rust:** Various community implementations

### Support Channels
- Official IBKR support tickets
- API forums (twsapi@groups.io)
- GitHub issues on official repos
- IBKR Campus Q&A sections

## Future Developments

### Unified Web API
IBKR is actively working on unifying all web APIs:
- Single OAuth 2.0 authentication
- Consistent endpoint structure
- Combined documentation
- Currently in beta (as of 2024-2026)

### Expected Improvements
- More order type support in Client Portal API
- Enhanced WebSocket features
- Better OAuth 2.0 documentation
- Automated authentication options for individual accounts (requested feature)

## Conclusion

Interactive Brokers Client Portal Web API is a powerful, feature-rich API suitable for professional algorithmic trading and portfolio management across multiple asset classes. While it has some limitations (rate limits, manual authentication for individuals, limited order types compared to TWS API), it provides a modern REST/WebSocket interface with comprehensive market data and trading capabilities.

**Best suited for:** Medium-frequency trading strategies, portfolio management, multi-asset trading systems, and professional traders who need programmatic access to global markets.

**Key strengths:** Multi-asset support, comprehensive market data, robust account management, good documentation, active development.

**Key weaknesses:** Manual authentication for individuals, rate limits, limited advanced order types, geographic restrictions for Canadians.

---

**Research Date:** 2026-01-26
**API Version Documented:** v1.0
**Status:** Active, Production-Ready (marked as beta but widely used)
