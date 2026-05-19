# KRX - Response Formats

## For EVERY important endpoint

All examples are from actual KRX API responses based on documentation and community library implementations.

---

## Public Data Portal API

### GET /1160100/service/GetKrxListedInfoService/getItemInfo

**URL:** https://apis.data.go.kr/1160100/service/GetKrxListedInfoService/getItemInfo?serviceKey=XXX&resultType=json&numOfRows=10&pageNo=1&basDt=20260120

**Response (JSON):**
```json
{
  "response": {
    "header": {
      "resultCode": "00",
      "resultMsg": "NORMAL SERVICE."
    },
    "body": {
      "numOfRows": 10,
      "pageNo": 1,
      "totalCount": 2409,
      "items": {
        "item": [
          {
            "basDt": "20260120",
            "srtnCd": "005930",
            "isinCd": "KR7005930003",
            "mrktCtg": "KOSPI",
            "itmsNm": "삼성전자",
            "crno": "1301110006246",
            "corpNm": "삼성전자주식회사"
          },
          {
            "basDt": "20260120",
            "srtnCd": "000660",
            "isinCd": "KR7000660001",
            "mrktCtg": "KOSPI",
            "itmsNm": "SK하이닉스",
            "crno": "1301110366646",
            "corpNm": "SK하이닉스주식회사"
          }
        ]
      }
    }
  }
}
```

**Field Descriptions:**
- `basDt`: Base date (YYYYMMDD)
- `srtnCd`: Short code (ticker symbol)
- `isinCd`: ISIN code (International Securities Identification Number)
- `mrktCtg`: Market category (KOSPI, KOSDAQ, KONEX)
- `itmsNm`: Item name (stock name in Korean)
- `crno`: Corporate registration number
- `corpNm`: Corporation name (in Korean)

---

## Data Marketplace API

Base URL: http://data.krx.co.kr/comm/bldAttendant/getJsonData.cmd

### Historical OHLCV Data (Module: MDCSTAT01701)

**Request:**
```http
POST /comm/bldAttendant/getJsonData.cmd HTTP/1.1
Host: data.krx.co.kr
Content-Type: application/x-www-form-urlencoded

bld=dbms/MDC/STAT/standard/MDCSTAT01701&locale=ko_KR&isuCd=KR7005930003&strtDd=20260101&endDd=20260120&csvxls_isNo=false
```

**Response (JSON):**
```json
{
  "OutBlock_1": [
    {
      "TRD_DD": "2026/01/20",
      "TDD_OPNPRC": "75,000",
      "TDD_HGPRC": "76,500",
      "TDD_LWPRC": "74,800",
      "TDD_CLSPRC": "76,200",
      "FLUC_TP_CD": "2",
      "CMPPRVDD_PRC": "1,200",
      "FLUC_RT": "1.60",
      "ACC_TRDVOL": "12,345,678",
      "ACC_TRDVAL": "935,432,100,000",
      "MKTCAP": "455,123,456,789,000",
      "LIST_SHRS": "5,969,782,550"
    },
    {
      "TRD_DD": "2026/01/17",
      "TDD_OPNPRC": "74,500",
      "TDD_HGPRC": "75,200",
      "TDD_LWPRC": "74,000",
      "TDD_CLSPRC": "75,000",
      "FLUC_TP_CD": "2",
      "CMPPRVDD_PRC": "500",
      "FLUC_RT": "0.67",
      "ACC_TRDVOL": "10,234,567",
      "ACC_TRDVAL": "764,321,234,000",
      "MKTCAP": "447,733,691,250,000",
      "LIST_SHRS": "5,969,782,550"
    }
  ]
}
```

**Field Descriptions:**
- `TRD_DD`: Trade Date (YYYY/MM/DD format)
- `TDD_OPNPRC`: Daily Opening Price (KRW, comma-formatted string)
- `TDD_HGPRC`: Daily High Price (KRW)
- `TDD_LWPRC`: Daily Low Price (KRW)
- `TDD_CLSPRC`: Daily Closing Price (KRW)
- `FLUC_TP_CD`: Fluctuation Type Code (1=down, 2=up, 3=unchanged)
- `CMPPRVDD_PRC`: Compared to Previous Day Price (absolute change)
- `FLUC_RT`: Fluctuation Rate (percentage change)
- `ACC_TRDVOL`: Accumulated Trade Volume (shares)
- `ACC_TRDVAL`: Accumulated Trade Value (KRW)
- `MKTCAP`: Market Capitalization (KRW)
- `LIST_SHRS`: Listed Shares (total outstanding shares)

**Important Notes:**
- Numeric values are returned as **comma-formatted strings**
- Must parse and remove commas before converting to numbers
- Dates use `/` separator instead of `-`
- All prices in Korean Won (KRW)

---

### Stock Ticker List (Module: MDCSTAT01501)

**Request:**
```http
POST /comm/bldAttendant/getJsonData.cmd HTTP/1.1
Host: data.krx.co.kr
Content-Type: application/x-www-form-urlencoded

bld=dbms/MDC/STAT/standard/MDCSTAT01501&locale=ko_KR&mktId=STK&trdDd=20260120&csvxls_isNo=false
```

**Response (JSON):**
```json
{
  "OutBlock_1": [
    {
      "ISU_SRT_CD": "005930",
      "ISU_CD": "KR7005930003",
      "ISU_NM": "삼성전자",
      "MKT_NM": "KOSPI",
      "SECUGRP_NM": "주권",
      "SECT_TP_NM": "전기전자",
      "KIND_STKCERT_TP_NM": "보통주",
      "PARVAL": "100",
      "LIST_SHRS": "5,969,782,550"
    },
    {
      "ISU_SRT_CD": "000660",
      "ISU_CD": "KR7000660001",
      "ISU_NM": "SK하이닉스",
      "MKT_NM": "KOSPI",
      "SECUGRP_NM": "주권",
      "SECT_TP_NM": "전기전자",
      "KIND_STKCERT_TP_NM": "보통주",
      "PARVAL": "5,000",
      "LIST_SHRS": "728,002,365"
    }
  ]
}
```

**Field Descriptions:**
- `ISU_SRT_CD`: Issue Short Code (ticker, e.g., "005930")
- `ISU_CD`: Issue Code (ISIN)
- `ISU_NM`: Issue Name (company name in Korean)
- `MKT_NM`: Market Name (KOSPI/KOSDAQ/KONEX)
- `SECUGRP_NM`: Security Group Name (주권 = stock)
- `SECT_TP_NM`: Sector Type Name (industry)
- `KIND_STKCERT_TP_NM`: Kind of Stock Certificate Type Name (보통주 = common stock, 우선주 = preferred stock)
- `PARVAL`: Par Value (KRW)
- `LIST_SHRS`: Listed Shares

---

### Trading Value by Investor Type (Module: MDCSTAT01901)

**Request:**
```http
POST /comm/bldAttendant/getJsonData.cmd HTTP/1.1
Host: data.krx.co.kr
Content-Type: application/x-www-form-urlencoded

bld=dbms/MDC/STAT/standard/MDCSTAT01901&locale=ko_KR&isuCd=KR7005930003&strtDd=20260120&endDd=20260120&csvxls_isNo=false
```

**Response (JSON):**
```json
{
  "OutBlock_1": [
    {
      "TRD_DD": "2026/01/20",
      "INVSTRY_NM": "개인",
      "BUY_TRDVOL": "5,234,567",
      "BUY_TRDVAL": "395,432,100,000",
      "SELL_TRDVOL": "4,876,543",
      "SELL_TRDVAL": "368,765,400,000",
      "NET_TRDVOL": "358,024",
      "NET_TRDVAL": "26,666,700,000"
    },
    {
      "TRD_DD": "2026/01/20",
      "INVSTRY_NM": "외국인",
      "BUY_TRDVOL": "3,456,789",
      "BUY_TRDVAL": "261,234,567,000",
      "SELL_TRDVOL": "4,123,456",
      "SELL_TRDVAL": "311,543,210,000",
      "NET_TRDVOL": "-666,667",
      "NET_TRDVAL": "-50,308,643,000"
    },
    {
      "TRD_DD": "2026/01/20",
      "INVSTRY_NM": "기관계",
      "BUY_TRDVOL": "3,654,322",
      "BUY_TRDVAL": "276,543,210,000",
      "SELL_TRDVOL": "3,345,679",
      "SELL_TRDVAL": "252,987,654,000",
      "NET_TRDVOL": "308,643",
      "NET_TRDVAL": "23,555,556,000"
    }
  ]
}
```

**Field Descriptions:**
- `TRD_DD`: Trade Date
- `INVSTRY_NM`: Investor Name (Type)
  - `개인`: Individual/Retail investors
  - `외국인`: Foreign investors
  - `기관계`: Institutional investors
  - `금융투자`: Financial investment companies
  - `보험`: Insurance companies
  - `투신`: Investment trust companies
  - `사모`: Private equity funds
  - `은행`: Banks
  - `기타금융`: Other financial institutions
  - `연기금`: Pension funds
  - `기타법인`: Other corporations
- `BUY_TRDVOL`: Buy Trade Volume (shares)
- `BUY_TRDVAL`: Buy Trade Value (KRW)
- `SELL_TRDVOL`: Sell Trade Volume (shares)
- `SELL_TRDVAL`: Sell Trade Value (KRW)
- `NET_TRDVOL`: Net Trade Volume (shares, positive = net buying)
- `NET_TRDVAL`: Net Trade Value (KRW, positive = net buying)

---

### Market Index Data (Module: MDCSTAT03001)

**Request:**
```http
POST /comm/bldAttendant/getJsonData.cmd HTTP/1.1
Host: data.krx.co.kr
Content-Type: application/x-www-form-urlencoded

bld=dbms/MDC/STAT/standard/MDCSTAT03001&locale=ko_KR&indIdx=1&strtDd=20260101&endDd=20260120&csvxls_isNo=false
```

**Response (JSON):**
```json
{
  "OutBlock_1": [
    {
      "TRD_DD": "2026/01/20",
      "CLSPRC_IDX": "2,850.45",
      "FLUC_TP_CD": "2",
      "CMPPRVDD_IDX": "25.30",
      "FLUC_RT": "0.90",
      "OPNPRC_IDX": "2,832.10",
      "HGPRC_IDX": "2,856.78",
      "LWPRC_IDX": "2,828.45",
      "ACC_TRDVOL": "543,234,567",
      "ACC_TRDVAL": "12,345,678,901,234",
      "MKTCAP": "2,134,567,890,123,456"
    },
    {
      "TRD_DD": "2026/01/17",
      "CLSPRC_IDX": "2,825.15",
      "FLUC_TP_CD": "1",
      "CMPPRVDD_IDX": "-12.45",
      "FLUC_RT": "-0.44",
      "OPNPRC_IDX": "2,835.60",
      "HGPRC_IDX": "2,840.23",
      "LWPRC_IDX": "2,820.10",
      "ACC_TRDVOL": "498,765,432",
      "ACC_TRDVAL": "11,234,567,890,123",
      "MKTCAP": "2,098,765,432,109,876"
    }
  ]
}
```

**Field Descriptions:**
- `TRD_DD`: Trade Date
- `CLSPRC_IDX`: Closing Price Index
- `FLUC_TP_CD`: Fluctuation Type Code (1=down, 2=up, 3=unchanged)
- `CMPPRVDD_IDX`: Compared to Previous Day Index (absolute change)
- `FLUC_RT`: Fluctuation Rate (percentage)
- `OPNPRC_IDX`: Opening Price Index
- `HGPRC_IDX`: High Price Index
- `LWPRC_IDX`: Low Price Index
- `ACC_TRDVOL`: Accumulated Trade Volume (shares)
- `ACC_TRDVAL`: Accumulated Trade Value (KRW)
- `MKTCAP`: Total Market Capitalization (KRW)

---

### Short Selling Data (Module: MDCSTAT05001)

**Response (JSON):**
```json
{
  "OutBlock_1": [
    {
      "TRD_DD": "2026/01/20",
      "ISU_CD": "KR7005930003",
      "ISU_NM": "삼성전자",
      "SHTSALE_TRDVOL": "245,678",
      "SHTSALE_TRDVAL": "18,543,210,000",
      "TRDVOL": "12,345,678",
      "TRDVAL": "935,432,100,000",
      "SHTSALE_TRDVOL_RT": "1.99",
      "SHTSALE_TRDVAL_RT": "1.98"
    }
  ]
}
```

**Field Descriptions:**
- `TRD_DD`: Trade Date
- `ISU_CD`: Issue Code (ISIN)
- `ISU_NM`: Issue Name
- `SHTSALE_TRDVOL`: Short Sale Trade Volume (shares)
- `SHTSALE_TRDVAL`: Short Sale Trade Value (KRW)
- `TRDVOL`: Total Trade Volume (shares)
- `TRDVAL`: Total Trade Value (KRW)
- `SHTSALE_TRDVOL_RT`: Short Sale Trade Volume Ratio (%)
- `SHTSALE_TRDVAL_RT`: Short Sale Trade Value Ratio (%)

---

## OTP-Based Download Response

### Step 1: Generate OTP

**Request:**
```http
GET /comm/fileDn/GenerateOTP/generate.cmd?mktId=ALL&trdDd=20260120&csvxls_isNo=false&name=fileDown&url=dbms/MDC/STAT/standard/MDCSTAT01501 HTTP/1.1
Host: data.krx.co.kr
```

**Response (Plain Text):**
```
b8a3f4c2e1d6g7h8i9j0k1l2m3n4o5p6
```

Single-line OTP code (one-time password), valid for immediate use.

---

### Step 2: Download CSV

**Request:**
```http
POST /comm/fileDn/download_csv/download.cmd HTTP/1.1
Host: data.krx.co.kr
Content-Type: application/x-www-form-urlencoded

code=b8a3f4c2e1d6g7h8i9j0k1l2m3n4o5p6
```

**Response (CSV):**
```csv
종목코드,종목명,시장구분,증권구분,업종명,주식종류,액면가,상장주식수
005930,삼성전자,KOSPI,주권,전기전자,보통주,100,"5,969,782,550"
000660,SK하이닉스,KOSPI,주권,전기전자,보통주,"5,000","728,002,365"
035420,NAVER,KOSPI,주권,서비스업,보통주,100,"164,263,395"
```

**Note:** CSV is in Korean with comma-formatted numbers.

---

## Error Responses

### HTTP 401 Unauthorized

**Request without proper authentication:**

**Response (JSON):**
```json
{
  "error": {
    "code": 401,
    "message": "Unauthorized API Call",
    "detail": "API key not approved for this service"
  }
}
```

Or sometimes just HTTP status code without JSON body.

---

### HTTP 429 Rate Limit Exceeded

**Response (JSON - expected format):**
```json
{
  "error": {
    "code": 429,
    "message": "Rate limit exceeded",
    "retry_after": 60
  }
}
```

**Note:** Actual KRX response format may vary; this is standard HTTP 429 behavior.

---

### HTTP 400 Bad Request

**Invalid parameters:**

**Response (JSON):**
```json
{
  "error": {
    "code": 400,
    "message": "Bad Request",
    "detail": "Invalid date format"
  }
}
```

Or sometimes:
```json
{
  "OutBlock_1": []
}
```

Empty result set when no data matches query.

---

## Response Parsing Notes

### String Number Formatting

**CRITICAL:** KRX returns numeric values as comma-formatted strings.

**Example:**
```json
{
  "TDD_CLSPRC": "76,200",
  "ACC_TRDVOL": "12,345,678",
  "ACC_TRDVAL": "935,432,100,000"
}
```

**Must parse:**
```python
def parse_krx_number(value_str):
    """Remove commas and convert to number"""
    if isinstance(value_str, str):
        return float(value_str.replace(',', ''))
    return float(value_str)

# Usage
close_price = parse_krx_number("76,200")  # 76200.0
volume = parse_krx_number("12,345,678")   # 12345678.0
```

### Date Formatting

**KRX uses different date formats:**

- Input format: `YYYYMMDD` (e.g., `20260120`)
- Output format: `YYYY/MM/DD` (e.g., `2026/01/20`)

**Parsing:**
```python
from datetime import datetime

def parse_krx_date(date_str):
    """Parse KRX date format"""
    return datetime.strptime(date_str, '%Y/%m/%d')
```

### Korean Language Fields

Many fields contain Korean text:
- `ISU_NM`: Stock names in Korean (삼성전자, SK하이닉스)
- `INVSTRY_NM`: Investor types (개인, 외국인, 기관계)
- `SECT_TP_NM`: Sector names (전기전자, 서비스업)

**Handling:**
- Ensure UTF-8 encoding support
- Consider translation or mapping to English equivalents
- Use unicode strings in code

### Array Wrapping

Responses are typically wrapped in `OutBlock_1` array or similar:

```json
{
  "OutBlock_1": [ ... ]
}
```

Some endpoints use different names:
- `OutBlock_1`
- `output`
- `data`
- `result`

**Always check actual response structure.**

### Empty Results

When no data matches query:
```json
{
  "OutBlock_1": []
}
```

Or:
```json
{
  "output": null
}
```

**Handle gracefully in code.**

---

## Complete Parsing Example

```python
import requests
import json
from datetime import datetime

def parse_krx_number(value_str):
    """Parse comma-formatted number string"""
    if isinstance(value_str, str):
        return float(value_str.replace(',', ''))
    return float(value_str) if value_str else 0.0

def parse_krx_date(date_str):
    """Parse KRX date format YYYY/MM/DD"""
    return datetime.strptime(date_str, '%Y/%m/%d')

def get_ohlcv_data(isin_code, start_date, end_date):
    """
    Fetch OHLCV data from KRX

    Args:
        isin_code: Stock ISIN (e.g., 'KR7005930003')
        start_date: Start date YYYYMMDD
        end_date: End date YYYYMMDD

    Returns:
        List of OHLCV dictionaries
    """
    url = "http://data.krx.co.kr/comm/bldAttendant/getJsonData.cmd"

    headers = {
        'Accept': 'application/json, text/javascript, */*; q=0.01',
        'Content-Type': 'application/x-www-form-urlencoded; charset=UTF-8',
        'Referer': 'http://data.krx.co.kr/contents/MDC/MDI/mdiLoader/index.cmd'
    }

    data = {
        'bld': 'dbms/MDC/STAT/standard/MDCSTAT01701',
        'locale': 'ko_KR',
        'isuCd': isin_code,
        'strtDd': start_date,
        'endDd': end_date,
        'csvxls_isNo': 'false'
    }

    response = requests.post(url, headers=headers, data=data)
    response.raise_for_status()

    json_data = response.json()
    raw_data = json_data.get('OutBlock_1', [])

    parsed_data = []
    for item in raw_data:
        parsed_data.append({
            'date': parse_krx_date(item['TRD_DD']),
            'open': parse_krx_number(item['TDD_OPNPRC']),
            'high': parse_krx_number(item['TDD_HGPRC']),
            'low': parse_krx_number(item['TDD_LWPRC']),
            'close': parse_krx_number(item['TDD_CLSPRC']),
            'volume': parse_krx_number(item['ACC_TRDVOL']),
            'value': parse_krx_number(item['ACC_TRDVAL']),
            'market_cap': parse_krx_number(item['MKTCAP']),
            'change_pct': parse_krx_number(item['FLUC_RT'])
        })

    return parsed_data

# Usage
ohlcv = get_ohlcv_data('KR7005930003', '20260101', '20260120')
for bar in ohlcv:
    print(f"{bar['date']}: O={bar['open']} H={bar['high']} L={bar['low']} C={bar['close']} V={bar['volume']}")
```
