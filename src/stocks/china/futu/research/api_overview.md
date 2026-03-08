# Futu OpenAPI Overview

## Provider Information
- Full name: Futu Securities (富途證券) / moomoo
- Website: https://www.futuhk.com / https://www.futunn.com
- Documentation: https://openapi.futunn.com/futu-api-doc/en/
- Category: stocks/china
- Type: Licensed broker (Hong Kong, US, Singapore, Australia)

## API Type
- REST: No
- WebSocket: No (uses custom protocol)
- GraphQL: No
- gRPC: No
- Other protocols: **Custom TCP protocol via OpenD gateway**

### Architecture Details
Futu uses a unique architecture where:
1. **OpenD Gateway** runs locally or on cloud servers
2. Client applications connect to OpenD via custom TCP protocol
3. OpenD translates requests to Futu servers
4. Uses Protocol Buffers for serialization
5. SDK wrappers available for multiple languages

This is fundamentally different from typical REST/WebSocket APIs - OpenD acts as a local proxy/gateway.

## Base URLs
- Production: N/A (connects to local OpenD gateway, typically `127.0.0.1:11111`)
- OpenD Gateway: User configures (default port 11111 for trade, 11111 for quote)
- Testnet/Sandbox: No separate sandbox - uses paper trading accounts
- Regional endpoints: N/A (OpenD handles routing)
- API version: Current version v9.6 (as of documentation)

### OpenD Connection
```
Host: 127.0.0.1 (or remote server IP)
Port: 11111 (default, configurable)
Protocol: Custom TCP with Protocol Buffers
```

## Documentation Quality
- Official docs: https://openapi.futunn.com/futu-api-doc/en/
- Quality rating: **Excellent**
  - Comprehensive endpoint documentation
  - Multi-language examples (Python, Java, C#, C++, JavaScript)
  - Clear parameter descriptions
  - Protocol Buffer definitions included
  - Use case examples and best practices
- Code examples: Yes (all 5 languages with working examples)
- OpenAPI/Swagger spec: Not available (custom protocol)
- SDKs available:
  - Python: https://github.com/FutunnOpen/py-futu-api
  - Java, C#, C++, JavaScript: https://www.futunn.com/en/download/OpenAPI

## Licensing & Terms
- Free tier: Yes (with limitations)
- Paid tiers: Yes (quote cards for premium market data)
- Commercial use: Allowed (requires account opening and compliance questionnaire)
- Data redistribution: Prohibited (personal use only, subject to exchange rules)
- Terms of Service: https://www.futuhk.com/en/support/topic1_458
- Compliance: Must complete API Questionnaire and Agreements after first login

### Access Requirements
1. Open account with Futubull/moomoo app
2. Complete API compliance questionnaire
3. Download and configure OpenD gateway
4. Generate API connection key (if remote OpenD)

## Support Channels
- Email: api@futunn.com (implied from documentation)
- Discord/Slack: Not publicly available
- GitHub: https://github.com/FutunnOpen (official repositories)
- Status page: Not available
- Help Center: https://www.futuhk.com/en/support/ and https://support.futunn.com/en/
- Community: Forum at https://q.futunn.com/en/ (general trading, not API-specific)

## Unique Features
- **Fastest order execution**: "as fast as 0.0014s" (claimed performance)
- **No additional trading fees**: OpenAPI trading has same fees as mobile app
- **Multi-market support**: Single API for HK, US, A-shares, Singapore, Japan, Australia
- **Paper trading**: Simulated accounts with same API (TrdEnv.SIMULATE)
- **Real-time push architecture**: Subscription-based data push for low latency
- **OpenD flexibility**: Can run locally (low latency) or on cloud (24/7 operation)
- **Multi-language native support**: Not just REST wrappers, native SDKs
- **Comprehensive trading**: Full order lifecycle, complex order types, portfolio management

## Performance Characteristics
- Order latency: <0.0014s (advertised minimum)
- Quote latency: Real-time (depends on quote level and subscription)
- Connection: Persistent TCP (no HTTP overhead)
- Data push: Server-initiated (no polling required)

## Platforms Supported
- Windows (OpenD binary available)
- macOS (OpenD binary available)
- Linux: CentOS, Ubuntu (OpenD binary available)
- Docker: Possible to containerize OpenD

## Version History
Current documentation version: v9.6 (as of 2024-2025)
- Regular updates and improvements
- Backward compatibility maintained
- Check GitHub releases for version changelog
