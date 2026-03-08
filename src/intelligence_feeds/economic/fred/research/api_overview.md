# FRED API Overview

## Provider Information
- Full name: Federal Reserve Economic Data (FRED)
- Website: https://fred.stlouisfed.org
- Documentation: https://fred.stlouisfed.org/docs/api/fred/
- Category: data_feeds
- Provider: Federal Reserve Bank of St. Louis - Economic Research Division

## API Type
- REST: Yes (base URL: https://api.stlouisfed.org)
- WebSocket: No
- GraphQL: No
- gRPC: No
- Other protocols: None - Standard HTTPS REST architecture only

## Base URLs
- Production: https://api.stlouisfed.org
- Testnet/Sandbox: No sandbox environment available
- Regional endpoints: None - single global endpoint
- API version: v2 (current), v1 (legacy incremental data)

## Documentation Quality
- Official docs: https://fred.stlouisfed.org/docs/api/fred/
- Quality rating: Good
- Code examples: Yes (Python, R, JavaScript via community SDKs)
- OpenAPI/Swagger spec: Available (community-maintained: https://github.com/armanobosyan/FRED-OpenAPI-specification)
- SDKs available:
  - Python: fredapi (https://pypi.org/project/fredapi/)
  - R: fredr (https://cran.r-project.org/web/packages/fredr/)
  - JavaScript/Node.js: node-fred-api (https://github.com/pastorsj/node-fred-api)
  - Ruby: fredric (https://github.com/DannyBen/fredric)

## Data Coverage
- Total time series: 840,000+ economic time series
- Data sources: 118 sources
- Categories: 80+ major categories
- Update frequency: Varies by series (real-time, daily, weekly, monthly, quarterly, annual)
- Historical depth: Extensive (some series date back to 1700s, most from 1900s-present)

## Licensing & Terms
- Free tier: Yes (completely free for non-commercial use)
- Paid tiers: No paid tiers - entirely free service
- Commercial use: Restricted - requires special permission from Federal Reserve Bank of St. Louis
- Data redistribution: Prohibited - cannot cache, store, or redistribute data
- Terms of Service: https://fred.stlouisfed.org/docs/api/terms_of_use.html

### Key Restrictions:
1. **AI/ML Training Prohibited**: Cannot use for training machine learning models, LLMs, or any AI systems
2. **No Caching/Archiving**: Cannot store, cache, or archive any FRED data
3. **No Wholesale Downloads**: Cannot engage in bulk downloading
4. **Non-commercial Use Only**: Free tier is for educational, personal, non-commercial use
5. **Third-party Data Rights**: Many series are owned by third parties and subject to copyright
6. **Attribution Required**: Must display: "This product uses the FRED® API but is not endorsed or certified by the Federal Reserve Bank of St. Louis."

## Support Channels
- Email: Not publicly listed (contact via website forms)
- Discord/Slack: None
- GitHub: Community SDKs have GitHub repos with issues/discussions
- Status page: None publicly available
- Help Center: https://fred.stlouisfed.org/docs/
- User Account: https://fredaccount.stlouisfed.org/ (required for API key management)

## Rate Limits
- Free tier: 120 requests per minute
- No burst allowance documented
- Rate limit can be adjusted by contacting FRED if needed
- No documented daily/hourly limits beyond per-minute restriction

## Notable Features
- Real-time period support: Access historical revisions of data (vintages)
- ALFRED integration: Archival FRED data for research on data revisions
- Extensive filtering: By category, release, source, tags, geographic area
- Multiple output formats: XML (default), JSON, CSV, XLSX
- No authentication for browsing, but API key required for programmatic access
