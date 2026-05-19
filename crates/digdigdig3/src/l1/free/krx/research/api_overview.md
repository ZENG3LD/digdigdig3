# KRX API Overview

## Provider Information
- Full name: Korea Exchange (한국거래소)
- Website: https://global.krx.co.kr/
- Documentation: https://openapi.krx.co.kr/
- Data Marketplace: https://data.krx.co.kr/
- Category: stocks/korea
- Type: Official stock exchange data provider (DATA ONLY - NO TRADING)

## API Type
- REST: Yes (base URL: http://data.krx.co.kr/comm/bldAttendant/getJsonData.cmd)
- WebSocket: No (not available for public API)
- GraphQL: No
- gRPC: No
- Other protocols: OTP-based download system for bulk data

## Base URLs

### Data Marketplace API
- Production: http://data.krx.co.kr/comm/bldAttendant/getJsonData.cmd
- OTP Generation: http://data.krx.co.kr/comm/fileDn/GenerateOTP/generate.cmd
- OTP Download: http://data.krx.co.kr/comm/fileDn/download_csv/download.cmd

### Open API Portal
- Production: https://openapi.krx.co.kr/
- API version: Not explicitly versioned (uses module-based paths like MDCSTAT01701)
- Regional endpoints: None (Korea only)
- Testnet/Sandbox: Not available

### Public Data Portal API (Government)
- HTTP: http://apis.data.go.kr/1160100/service/GetKrxListedInfoService/getItemInfo
- HTTPS: https://apis.data.go.kr/1160100/service/GetKrxListedInfoService/getItemInfo
- API version: v1 (embedded in service name)

## Documentation Quality
- Official docs: https://openapi.krx.co.kr/ (Korean language, requires registration)
- Data Marketplace: https://data.krx.co.kr/contents/MDC/MAIN/main/index.cmd?locale=en (English available)
- Quality rating: Adequate (Korean-focused, limited English documentation)
- Code examples: Limited (mostly available through community libraries)
- OpenAPI/Swagger spec: Not available
- SDKs available:
  - Python (pykrx - community library: https://github.com/sharebook-kr/pykrx)
  - Node.js (krx-stock-api: https://github.com/Shin-JaeHeon/krx-stock-api)
  - Go (go-krx: https://github.com/dojinkimm/go-krx)
  - R (tqk: https://github.com/mrchypark/tqk)

## Licensing & Terms
- Free tier: Yes (with API key approval required)
- Paid tiers: Yes (commercial data packages available)
- Commercial use: Requires proper licensing through KRX Data Marketplace
- Data redistribution: Prohibited without explicit permission
- Attribution required: Yes
- Terms of Service: Available on openapi.krx.co.kr (Korean)
- Important: API key approval can take up to 1 business day
- Update frequency: Daily (data updated at 1:00 PM business day following reference date)

## Support Channels
- Email: Available through Data Marketplace contact form
- Discord/Slack: Not available
- GitHub: No official repository (community libraries only)
- Status page: Not available
- Registration required: Yes (both for Open API and Public Data Portal)

## Important Notes

### Recent Changes (January 2026)
- KRX now requires API authentication for most data access
- Previous direct data.krx.co.kr access without authentication has been deprecated
- API key application requires approval for specific services:
  - Securities Daily Trading Information
  - KOSDAQ Daily Trading Information
  - KONEX Daily Trading Information
  - Securities Basic Information
  - KOSDAQ Basic Information
  - KONEX Basic Information

### Data Access Limitations
- Real-time data: Not available through free API (delayed by 1 business day)
- Historical data: Available (depth varies by data type)
- Update timing: 1:00 PM KST on business day following data reference date
- Rate limits: Apply (see tiers_and_limits.md)

### Alternative Access Methods
- Third-party providers (ICE, Twelve Data) offer real-time KRX data
- Community libraries (pykrx) provide easier API access patterns
- Government Open Data Portal offers basic company information
