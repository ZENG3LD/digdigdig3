# KRX - Authentication

## Public Endpoints

- Public endpoints exist: Limited
- Require authentication: Most endpoints YES (as of January 2026)
- Rate limits without auth: Severely restricted or blocked
- Recent change: KRX moved from open access to API-key-required model in early 2026

## API Key

### Required For

- All endpoints: Yes (Data Marketplace and Open API portal)
- Paid tier only: No (free tier available with registration)
- Rate limit increase: Yes (paid tiers have higher limits)
- Specific endpoints: All Data Marketplace endpoints require authentication

### How to Obtain

#### KRX Open API Portal
1. Sign up: https://openapi.krx.co.kr/
2. Create account (Korean or international)
3. Navigate to "My Page" (마이페이지)
4. Find "API 인증키 신청" (API Authentication Key Application)
5. Request new API key
6. **Approval time:** Up to 1 business day

#### Service-Specific Registration
After obtaining API key, apply for specific services:
- Securities Daily Trading Information (유가증권 일별 거래정보)
- KOSDAQ Daily Trading Information (코스닥 일별 거래정보)
- KONEX Daily Trading Information (코넥스 일별 거래정보)
- Securities Basic Information (유가증권 기본정보)
- KOSDAQ Basic Information (코스닥 기본정보)
- KONEX Basic Information (코넥스 기본정보)

**Service List:** https://openapi.krx.co.kr/contents/OPP/INFO/service/OPPINFO004.cmd

**Status:** Applications show as "승인대기" (Pending Approval) until approved

#### Public Data Portal API Key
1. Sign up: https://www.data.go.kr/
2. Navigate to KRX Listed Info Service
3. Request API key (serviceKey)
4. Immediate or quick approval (typically same day)
5. Free tier: 100,000 requests/day

### API Key Format

#### Data Marketplace (openapi.krx.co.kr)
**Method 1: Cookie-based (after login)**
```http
GET /api/endpoint HTTP/1.1
Host: openapi.krx.co.kr
Cookie: JSESSIONID=xxx; authToken=yyy
```

**Method 2: Header-based**
```http
POST /comm/bldAttendant/getJsonData.cmd HTTP/1.1
Host: data.krx.co.kr
X-API-Key: your_api_key_here
Referer: http://data.krx.co.kr/contents/MDC/MDI/mdiLoader/index.cmd
```

**Note:** Exact header name may vary; documentation is limited. Authentication often happens via session cookies after web login.

#### Public Data Portal
**Query parameter:**
```http
GET /1160100/service/GetKrxListedInfoService/getItemInfo?serviceKey=YOUR_KEY&... HTTP/1.1
Host: apis.data.go.kr
```

### Multiple Keys

- Multiple keys allowed: Yes (per service application)
- Rate limits per key: Yes
- Use cases for multiple keys:
  - Different applications
  - Load distribution
  - Service-specific access
  - Development vs production

## OAuth

### OAuth 2.0
- Supported: No
- KRX uses simpler API key authentication
- No OAuth flows implemented

## Signature/HMAC

### Required: No

KRX does NOT use HMAC signatures for authentication. This is typical for data-only providers (unlike trading exchanges like Binance).

Authentication is straightforward:
- Public Data Portal: API key in query parameter
- Data Marketplace: Session cookies or API key header
- No cryptographic signatures needed

## Authentication Examples

### REST with Public Data Portal API Key

```bash
# Basic stock information
curl "https://apis.data.go.kr/1160100/service/GetKrxListedInfoService/getItemInfo?serviceKey=YOUR_KEY&resultType=json&numOfRows=10&pageNo=1"
```

```python
import requests

service_key = "YOUR_SERVICE_KEY"
base_url = "https://apis.data.go.kr/1160100/service/GetKrxListedInfoService/getItemInfo"

params = {
    'serviceKey': service_key,
    'resultType': 'json',
    'numOfRows': 100,
    'pageNo': 1,
    'basDt': '20260120'  # YYYYMMDD format
}

response = requests.get(base_url, params=params)
data = response.json()
```

### REST with Data Marketplace

```bash
# Historical OHLCV data
curl -X POST "http://data.krx.co.kr/comm/bldAttendant/getJsonData.cmd" \
  -H "Accept: application/json, text/javascript, */*; q=0.01" \
  -H "Content-Type: application/x-www-form-urlencoded; charset=UTF-8" \
  -H "Referer: http://data.krx.co.kr/contents/MDC/MDI/mdiLoader/index.cmd" \
  -H "User-Agent: Mozilla/5.0" \
  -d "bld=dbms/MDC/STAT/standard/MDCSTAT01701" \
  -d "locale=ko_KR" \
  -d "isuCd=KR7005930003" \
  -d "strtDd=20260101" \
  -d "endDd=20260120" \
  -d "csvxls_isNo=false"
```

```python
import requests

url = "http://data.krx.co.kr/comm/bldAttendant/getJsonData.cmd"

headers = {
    'Accept': 'application/json, text/javascript, */*; q=0.01',
    'Content-Type': 'application/x-www-form-urlencoded; charset=UTF-8',
    'Referer': 'http://data.krx.co.kr/contents/MDC/MDI/mdiLoader/index.cmd',
    'User-Agent': 'Mozilla/5.0'
}

data = {
    'bld': 'dbms/MDC/STAT/standard/MDCSTAT01701',
    'locale': 'ko_KR',
    'isuCd': 'KR7005930003',  # Samsung Electronics
    'strtDd': '20260101',
    'endDd': '20260120',
    'csvxls_isNo': 'false'
}

response = requests.post(url, headers=headers, data=data)
json_data = response.json()
```

### OTP-Based Download

```python
import requests

# Step 1: Generate OTP
otp_url = "http://data.krx.co.kr/comm/fileDn/GenerateOTP/generate.cmd"
otp_params = {
    'mktId': 'ALL',
    'trdDd': '20260120',
    'csvxls_isNo': 'false',
    'name': 'fileDown',
    'url': 'dbms/MDC/STAT/standard/MDCSTAT01501'
}

otp_response = requests.get(otp_url, params=otp_params)
otp_code = otp_response.text

# Step 2: Download with OTP
download_url = "http://data.krx.co.kr/comm/fileDn/download_csv/download.cmd"
download_data = {'code': otp_code}

csv_response = requests.post(download_url, data=download_data)
csv_content = csv_response.content
```

## Error Codes

| Code | Description | Resolution |
|------|-------------|------------|
| 401 | Unauthorized API Call | API key not approved for requested service |
| 403 | Forbidden | Check permissions or upgrade tier |
| 429 | Rate Limit Exceeded | Wait or upgrade to higher tier |
| 500 | Internal Server Error | Contact KRX support or retry later |
| 400 | Bad Request | Check parameters (dates, symbol codes, etc.) |

### Common 401 Error Scenarios

**Cause 1: Service Not Applied**
- You have an API key but haven't applied for the specific service
- Solution: Visit service list and apply for each required API

**Cause 2: Pending Approval**
- Service application is "승인대기" (Pending Approval)
- Solution: Wait for approval (up to 1 business day)

**Cause 3: Expired Session**
- Session cookie expired for Data Marketplace
- Solution: Re-login or refresh session

**Cause 4: Invalid API Key**
- API key is incorrect or revoked
- Solution: Verify key in "My Page" or request new one

## Rate Limiting Behavior

### Public Data Portal
- Header included: `X-RateLimit-Limit: 100000`
- Header included: `X-RateLimit-Remaining: 99500`
- Resets: Daily at 00:00 KST
- On limit exceeded: HTTP 429 with retry-after header

### Data Marketplace
- Rate limits exist but not publicly documented
- Likely per-API-key basis
- Commercial tiers have higher limits
- No public rate limit headers observed

## Best Practices

### Security
1. **Never commit API keys to version control**
2. **Use environment variables**:
   ```bash
   export KRX_API_KEY="your_key_here"
   export KRX_DATA_PORTAL_KEY="your_other_key"
   ```
3. **Rotate keys periodically**
4. **Use separate keys for dev/staging/production**

### Error Handling
```python
def make_krx_request(url, **kwargs):
    """Wrapper with proper error handling"""
    try:
        response = requests.post(url, **kwargs, timeout=30)

        if response.status_code == 401:
            raise Exception("API key not authorized for this service")
        elif response.status_code == 429:
            raise Exception("Rate limit exceeded - wait before retry")
        elif response.status_code == 500:
            raise Exception("KRX server error - retry later")

        response.raise_for_status()
        return response.json()

    except requests.exceptions.Timeout:
        raise Exception("Request timeout - KRX server slow")
    except requests.exceptions.RequestException as e:
        raise Exception(f"Request failed: {e}")
```

### Respect Rate Limits
```python
import time
from datetime import datetime, timedelta

class RateLimiter:
    def __init__(self, max_requests=100, window_seconds=60):
        self.max_requests = max_requests
        self.window_seconds = window_seconds
        self.requests = []

    def wait_if_needed(self):
        now = datetime.now()
        cutoff = now - timedelta(seconds=self.window_seconds)
        self.requests = [r for r in self.requests if r > cutoff]

        if len(self.requests) >= self.max_requests:
            sleep_time = (self.requests[0] - cutoff).total_seconds()
            time.sleep(sleep_time)

        self.requests.append(now)
```

## Authentication Flow Summary

```
1. Register → 2. Get API Key → 3. Apply for Services → 4. Wait for Approval → 5. Make Requests
```

**Timeline:**
- Registration: Immediate
- API Key Generation: Immediate (but may need approval)
- Service Application: Requires manual approval
- Approval Wait: Up to 1 business day
- API Access: Immediate after approval
