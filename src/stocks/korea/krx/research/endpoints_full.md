# KRX - Complete Endpoint Reference

## Category: Public Data Portal - Stock Information

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /1160100/service/GetKrxListedInfoService/getItemInfo | Listed stock information | Yes | API Key | 100k/day | Basic company info by date or ISIN |

### GET /1160100/service/GetKrxListedInfoService/getItemInfo

**Base URL:** https://apis.data.go.kr

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| serviceKey | string | Yes | - | Portal-issued authentication key |
| numOfRows | int | No | 10 | Results per page |
| pageNo | int | No | 1 | Page number |
| resultType | string | No | xml | Response format (xml, json) |
| basDt | string | No | - | Base date (YYYYMMDD) - exact match |
| beginBasDt | string | No | - | Date range start (>=) |
| endBasDt | string | No | - | Date range end (<=) |
| likeSrtnCd | string | No | - | Abbreviated code substring search |
| isinCd | string | No | - | ISIN code - exact match |
| likeIsinCd | string | No | - | ISIN code substring search |
| itmsNm | string | No | - | Stock name - exact match |
| likeItmsNm | string | No | - | Stock name substring search |
| crno | string | No | - | Corporate registration number |
| corpNm | string | No | - | Company name - exact match |
| likeCorpNm | string | No | - | Company name substring search |

## Category: Data Marketplace - Market Data

### Base Endpoint
**POST** `http://data.krx.co.kr/comm/bldAttendant/getJsonData.cmd`

All Data Marketplace endpoints use this single URL with different `bld` module parameters.

| Module (bld parameter) | Description | Free? | Auth? | Rate Limit | Notes |
|------------------------|-------------|-------|-------|------------|-------|
| dbms/MDC/STAT/standard/MDCSTAT01701 | Historical OHLCV data | Yes | API Key | TBD | Stock price history |
| dbms/MDC/STAT/standard/MDCSTAT01501 | Stock ticker list | Yes | API Key | TBD | All listed stocks |
| dbms/MDC/STAT/standard/MDCSTAT01901 | Trading value by date | Yes | API Key | TBD | Investor type breakdown |
| dbms/MDC/STAT/standard/MDCSTAT02001 | Market capitalization | Yes | API Key | TBD | Market cap data |
| dbms/MDC/STAT/standard/MDCSTAT03001 | Index data | Yes | API Key | TBD | KOSPI, KOSDAQ indices |
| dbms/MDC/STAT/standard/MDCSTAT04001 | Sector information | Yes | API Key | TBD | Industry classification |
| dbms/MDC/STAT/standard/MDCSTAT05001 | Short selling data | Yes | API Key | TBD | Short position data |

### POST /comm/bldAttendant/getJsonData.cmd

**Common Headers:**
```
Accept: application/json, text/javascript, */*; q=0.01
Content-Type: application/x-www-form-urlencoded; charset=UTF-8
Referer: http://data.krx.co.kr/contents/MDC/MDI/mdiLoader/index.cmd
User-Agent: Mozilla/5.0
```

**Module: Historical OHLCV (MDCSTAT01701)**

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| bld | string | Yes | - | Module path: dbms/MDC/STAT/standard/MDCSTAT01701 |
| locale | string | No | ko_KR | Language (ko_KR, en_US) |
| isuCd | string | Yes | - | Stock code (e.g., KR7005930003) |
| strtDd | string | Yes | - | Start date (YYYYMMDD) |
| endDd | string | Yes | - | End date (YYYYMMDD) |
| csvxls_isNo | string | No | false | CSV/Excel export flag |

**Module: Stock Ticker List (MDCSTAT01501)**

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| bld | string | Yes | - | Module path: dbms/MDC/STAT/standard/MDCSTAT01501 |
| locale | string | No | ko_KR | Language (ko_KR, en_US) |
| mktId | string | Yes | - | Market ID: STK (KOSPI), KSQ (KOSDAQ), KNX (KONEX), ALL |
| trdDd | string | Yes | - | Trading date (YYYYMMDD) |
| csvxls_isNo | string | No | false | CSV/Excel export flag |

**Module: Trading Value by Investor Type (MDCSTAT01901)**

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| bld | string | Yes | - | Module path: dbms/MDC/STAT/standard/MDCSTAT01901 |
| locale | string | No | ko_KR | Language (ko_KR, en_US) |
| isuCd | string | Yes | - | Stock code |
| strtDd | string | Yes | - | Start date (YYYYMMDD) |
| endDd | string | Yes | - | End date (YYYYMMDD) |
| csvxls_isNo | string | No | false | CSV/Excel export flag |

## Category: OTP-Based Bulk Download

### Step 1: Generate OTP
**GET** `http://data.krx.co.kr/comm/fileDn/GenerateOTP/generate.cmd`

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| mktId | string | Yes | Market ID: STK, KSQ, KNX, ALL |
| trdDd | string | Yes | Trading date (YYYYMMDD) |
| csvxls_isNo | string | No | false for CSV |
| name | string | Yes | fileDown |
| url | string | Yes | Module path (e.g., dbms/MDC/STAT/standard/MDCSTAT01501) |

**Returns:** One-time password string

### Step 2: Download Data
**POST** `http://data.krx.co.kr/comm/fileDn/download_csv/download.cmd`

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| code | string | Yes | OTP from step 1 |

**Returns:** CSV file download

## Category: Open API Portal - Authentication Services

| Method | Endpoint | Description | Free? | Auth? | Notes |
|--------|----------|-------------|-------|-------|-------|
| POST | /api/auth/login | Login to Open API portal | Yes | Credentials | Returns session |
| GET | /api/user/apikey | Get API key info | Yes | Session | API key details |
| POST | /api/service/apply | Apply for API service | Yes | Session | Service registration |
| GET | /api/service/status | Check service status | Yes | API Key | Approval status |

**Note:** Exact endpoint paths for Open API portal are not publicly documented. These are inferred from usage patterns.

## Category: Metadata

| Module | Description | Free? | Auth? | Notes |
|--------|-------------|-------|-------|-------|
| Market info | Exchange information | Yes | API Key | Trading hours, holidays |
| Calendar | Trading calendar | Yes | API Key | Market holidays |
| Symbol list | All listed stocks | Yes | API Key | KOSPI, KOSDAQ, KONEX |

## Category: Historical Data

All historical data access uses the Data Marketplace API with module-based endpoints.

**Available data types:**
- Daily OHLCV (Open, High, Low, Close, Volume)
- Trading value by investor type (institutional, individual, foreign)
- Market capitalization
- Financial ratios
- Dividend history
- Stock splits
- Corporate actions

**Historical depth:**
- Stock data: Available from listing date
- Maximum query range: Varies by module (typically 1-5 years per request)
- Granularity: Daily only (no intraday data through public API)

## Category: Market Statistics

| Module | Description | Coverage |
|--------|-------------|----------|
| Sector stats | Industry performance | All sectors |
| Market indices | KOSPI, KOSDAQ, KRX100, etc. | Major indices |
| Foreign ownership | Foreign investor holdings | Individual stocks |
| Short selling | Short position data | Individual stocks |
| Program trading | Institutional program trading | Market-wide |

## Category: Fundamental Data

Access through specialized modules:

- Financial statements (quarterly, annual)
- Corporate governance
- Disclosure information
- Earnings announcements
- Dividend payments
- Stock splits and consolidations

**Note:** Detailed fundamental data may require separate DART (Data Analysis, Retrieval and Transfer System) API access.

## Notes on API Structure

### Module-Based System
KRX uses a module-based system where all requests go to a single endpoint with different `bld` (build/module) parameters. Each module represents a specific data type or report.

### Authentication
- Public Data Portal: API key in query parameter (`serviceKey=xxx`)
- Data Marketplace: API key in header or cookie (requires registration)
- Open API: Session-based authentication after login

### Rate Limits
- Not explicitly documented in public materials
- Likely per-API-key basis
- Commercial users have higher limits

### Data Freshness
- All data delayed by 1 business day minimum
- Updates occur at 1:00 PM KST
- Real-time data not available through public API

### Common Issues
- 401 Unauthorized: API key not approved for service
- Module paths may change without notice
- Korean language responses common (locale parameter may help)
- Some modules require specific market participation status
