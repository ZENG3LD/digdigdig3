# SEC EDGAR Connector

Implementation of the SEC EDGAR API connector following the FRED pattern.

## Structure

```
sec_edgar/
├── mod.rs         - Module exports
├── endpoints.rs   - API endpoints and URL formatting
├── auth.rs        - User-Agent header authentication
├── parser.rs      - JSON response parsers and domain types
└── connector.rs   - Main connector implementation
```

## Features

### Company Data
- `get_company_filings(cik)` - All filings for a company
- `get_company_facts(cik)` - XBRL financial data
- `get_company_concept(cik, taxonomy, tag)` - Specific financial concept

### Search
- `search_filings(query, forms, date_from, date_to)` - Full-text search

### Bulk Data
- `get_company_tickers()` - All companies with CIKs
- `get_mutual_fund_tickers()` - All mutual funds with CIKs

### XBRL Frames
- `get_xbrl_frames(taxonomy, tag, unit, period)` - Aggregate data across all filers

### Convenience Methods
- `get_10k_filings(cik)` - Annual reports
- `get_10q_filings(cik)` - Quarterly reports
- `get_insider_trades(cik)` - Form 4 filings
- `get_13f_filings(cik)` - Institutional holdings
- `get_revenue(cik)` - Revenue data (us-gaap/Revenues)
- `get_net_income(cik)` - Net income data (us-gaap/NetIncomeLoss)
- `get_total_assets(cik)` - Total assets (us-gaap/Assets)

## Authentication

SEC EDGAR requires a User-Agent header with company name and email:
- Environment variable: `SEC_EDGAR_USER_AGENT`
- Format: "CompanyName email@example.com"
- Default: "NemoTrading contact@example.com"

## Usage Example

```rust
use digdigdig3::data_feeds::sec_edgar::SecEdgarConnector;

// Create connector (uses env var or default)
let connector = SecEdgarConnector::new();

// Or with custom User-Agent
let connector = SecEdgarConnector::with_user_agent("MyCompany contact@mycompany.com");

// Get company filings (Apple's CIK: 320193)
let filings = connector.get_company_filings("320193").await?;

// Get financial data
let facts = connector.get_company_facts("320193").await?;

// Get revenue data
let revenues = connector.get_revenue("320193").await?;

// Search for filings
let results = connector.search_filings("apple", Some("10-K"), None, None).await?;

// Get all companies
let companies = connector.get_company_tickers().await?;
```

## CIK Numbers

CIK numbers must be zero-padded to 10 digits:
- Input: "320193" → Padded: "0000320193" (Apple)
- Input: "1318605" → Padded: "0001318605" (Tesla)

The `pad_cik()` function in `endpoints.rs` handles this automatically.

## Rate Limits

SEC EDGAR enforces 10 requests per second. No built-in rate limiting is implemented.

## Data Types

### CompanyFiling
Company filing metadata including CIK, entity type, SIC, name, tickers, exchanges, and nested filings data.

### CompanyFacts
XBRL financial data with CIK, entity name, and nested facts organized by taxonomy.

### CompanyConcept
Specific financial concept data with time series by unit (e.g., USD).

### XbrlFrame
Aggregate data across all filers for a specific financial concept.

### CompanyTicker
Company ticker information with CIK, ticker symbol, and title.

### SearchResult
Search result with CIK, company name, form type, filing date, and accession number.

## References

- Pattern: `data_feeds/fred/`
- Spec: Request requirements (User-Agent header, CIK padding, rate limits)
