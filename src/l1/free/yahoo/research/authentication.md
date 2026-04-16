# yahoo - Authentication

## Public Endpoints

- Public endpoints exist: Yes
- Require authentication: No (most endpoints)
- Rate limits without auth: ~2000 req/hr (IP-based)

## API Key

### Required For
- All endpoints: No
- Paid tier only: N/A (no official paid tiers from Yahoo)
- Rate limit increase: No (unofficial API has no API key mechanism)
- Specific endpoints: Historical download endpoint requires cookie/crumb (not API key)

### How to Obtain
**Not Applicable** - Yahoo Finance unofficial API does not use API keys for authentication.

### API Key Format
**Not Applicable** - No API key system exists.

### Multiple Keys
**Not Applicable** - No API key system exists.

## OAuth (if applicable)

### OAuth 2.0
- Supported: No
- Grant types: N/A
- Scopes: N/A
- Token endpoint: N/A
- Authorization endpoint: N/A

**Note:** While Yahoo Developer Network supports OAuth for other Yahoo services, Yahoo Finance API does not implement OAuth.

## Cookie/Crumb Authentication (Yahoo Finance Specific)

**CRITICAL:** Yahoo Finance uses a unique cookie-crumb authentication system for certain endpoints, specifically the historical data download endpoint.

### What is it?
- **Cookie**: Browser session cookie stored after visiting Yahoo Finance website
- **Crumb**: Alphanumeric token paired with the cookie
- **Purpose**: Anti-scraping mechanism to verify requests come from a browser session

### Required For
- `/v7/finance/download/{symbol}` endpoint (historical CSV download)
- Optional for other endpoints (work without it, but may have higher rate limits with it)

### How Cookie/Crumb Works

1. **Visit Yahoo Finance website** to establish session
2. **Extract cookie** from response headers
3. **Obtain crumb** by calling `/v1/test/getcrumb` endpoint with the cookie
4. **Use both** in subsequent requests

### Obtaining Cookie

#### Method 1: Manual Browser
1. Open browser and visit `https://finance.yahoo.com/`
2. Open DevTools → Network tab
3. Find request with cookies
4. Copy cookie value

#### Method 2: Programmatic (Python)
```python
import requests

session = requests.Session()
response = session.get("https://finance.yahoo.com/")
cookies = session.cookies.get_dict()
cookie_string = "; ".join([f"{k}={v}" for k, v in cookies.items()])
```

#### Method 3: Programmatic (JavaScript)
```javascript
const axios = require('axios');

const response = await axios.get('https://finance.yahoo.com/', {
  headers: {
    'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36'
  }
});

const cookies = response.headers['set-cookie'];
```

### Obtaining Crumb

Once you have a valid cookie, obtain the crumb:

**Endpoint:** `https://query1.finance.yahoo.com/v1/test/getcrumb`

**Method:** GET

**Headers:**
```
Cookie: <your-cookie-string>
User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36
```

**Response:**
```
AbCdEfGhIjK
```
(Plain text, single line, alphanumeric string)

**Example (Python):**
```python
import requests

cookies = {"B": "abc123", "A3": "def456"}  # From previous step
headers = {
    "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"
}

response = requests.get(
    "https://query1.finance.yahoo.com/v1/test/getcrumb",
    cookies=cookies,
    headers=headers
)

crumb = response.text
print(f"Crumb: {crumb}")
```

**Example (cURL):**
```bash
curl -H "Cookie: B=abc123; A3=def456" \
     -H "User-Agent: Mozilla/5.0" \
     https://query1.finance.yahoo.com/v1/test/getcrumb
```

### Using Cookie and Crumb

**Example: Download historical data**

```python
import requests

symbol = "AAPL"
period1 = 1609459200  # Jan 1, 2021
period2 = 1640995200  # Jan 1, 2022
interval = "1d"

url = f"https://query1.finance.yahoo.com/v7/finance/download/{symbol}"
params = {
    "period1": period1,
    "period2": period2,
    "interval": interval,
    "events": "history",
    "crumb": "AbCdEfGhIjK"  # Your crumb
}

cookies = {"B": "abc123", "A3": "def456"}  # Your cookies
headers = {
    "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"
}

response = requests.get(url, params=params, cookies=cookies, headers=headers)

# Response is CSV format
csv_data = response.text
print(csv_data)
```

**Example (cURL):**
```bash
curl -H "Cookie: B=abc123; A3=def456" \
     -H "User-Agent: Mozilla/5.0" \
     "https://query1.finance.yahoo.com/v7/finance/download/AAPL?period1=1609459200&period2=1640995200&interval=1d&events=history&crumb=AbCdEfGhIjK"
```

### Cookie/Crumb Lifetime

- **Cookie expiration**: Varies (typically 1-24 hours)
- **Crumb validity**: Tied to cookie (expires when cookie expires)
- **Renewal**: Must obtain new cookie+crumb when expired

### Error: Invalid Cookie

**Error Response:**
```json
{
  "finance": {
    "error": {
      "code": "Unauthorized",
      "description": "Invalid Cookie"
    }
  }
}
```

**HTTP Status:** 401 Unauthorized

**Resolution:**
1. Obtain fresh cookie by visiting Yahoo Finance
2. Get new crumb with the new cookie
3. Retry request

### Error: Invalid Crumb

**Error Response:**
```json
{
  "finance": {
    "error": {
      "code": "Unauthorized",
      "description": "Invalid Crumb"
    }
  }
}
```

**HTTP Status:** 401 Unauthorized

**Resolution:**
1. Verify crumb matches current cookie
2. Obtain new crumb from `/v1/test/getcrumb`
3. Ensure crumb is URL-encoded if contains special characters

## User-Agent Requirement

**CRITICAL:** Most endpoints require a valid User-Agent header to avoid detection as a bot.

### Recommended User-Agent
```
Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36
```

### Why Required
- Yahoo Finance blocks requests with missing or suspicious User-Agent
- Helps avoid 403 Forbidden errors
- Makes requests look like legitimate browser traffic

### Example Headers
```python
headers = {
    "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
    "Accept": "application/json",
    "Accept-Language": "en-US,en;q=0.9",
    "Referer": "https://finance.yahoo.com/"
}
```

## RapidAPI Authentication (Third-Party)

For those seeking an "official" authenticated API, RapidAPI hosts Yahoo Finance proxies.

### Access
- Sign up at: https://rapidapi.com/apidojo/api/yahoo-finance1
- Get API key from RapidAPI dashboard

### Authentication Method
- Header: `X-RapidAPI-Key: your_api_key_here`
- Header: `X-RapidAPI-Host: yahoo-finance1.p.rapidapi.com`

### Example Request (RapidAPI)
```python
import requests

url = "https://yahoo-finance1.p.rapidapi.com/v8/finance/chart/AAPL"
params = {"period1": "1609459200", "period2": "1640995200", "interval": "1d"}

headers = {
    "X-RapidAPI-Key": "your_rapidapi_key",
    "X-RapidAPI-Host": "yahoo-finance1.p.rapidapi.com"
}

response = requests.get(url, headers=headers, params=params)
data = response.json()
```

### RapidAPI Tiers
- Free: 500 requests/month, 5 req/sec
- Basic: $10/month, 10,000 requests/month
- Pro: Higher limits (check RapidAPI for current pricing)

## Error Codes

| Code | Description | Resolution |
|------|-------------|------------|
| 401 | Invalid Cookie | Obtain fresh cookie from Yahoo Finance |
| 401 | Invalid Crumb | Get new crumb with valid cookie |
| 403 | Forbidden | Add User-Agent header, check IP not blocked |
| 429 | Too Many Requests | Reduce request rate, wait before retrying |
| 404 | Not Found | Verify endpoint URL and symbol format |
| 500 | Internal Server Error | Yahoo server issue, retry later |

## Best Practices

### Session Management
```python
import requests

# Create session to reuse cookies
session = requests.Session()
session.headers.update({
    "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"
})

# Initialize session
session.get("https://finance.yahoo.com/")

# Get crumb
crumb_response = session.get("https://query1.finance.yahoo.com/v1/test/getcrumb")
crumb = crumb_response.text

# Now make requests with session (cookies automatic)
data = session.get(f"https://query1.finance.yahoo.com/v7/finance/download/AAPL?crumb={crumb}&...")
```

### Crumb Caching
```python
import time
import requests

class YahooFinanceAuth:
    def __init__(self):
        self.session = requests.Session()
        self.session.headers.update({
            "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"
        })
        self.crumb = None
        self.crumb_timestamp = 0

    def get_crumb(self, force_refresh=False):
        # Cache crumb for 30 minutes
        if self.crumb and not force_refresh and (time.time() - self.crumb_timestamp < 1800):
            return self.crumb

        # Initialize session
        self.session.get("https://finance.yahoo.com/")

        # Get fresh crumb
        response = self.session.get("https://query1.finance.yahoo.com/v1/test/getcrumb")
        self.crumb = response.text
        self.crumb_timestamp = time.time()

        return self.crumb

    def download_history(self, symbol, period1, period2, interval="1d"):
        crumb = self.get_crumb()
        url = f"https://query1.finance.yahoo.com/v7/finance/download/{symbol}"
        params = {
            "period1": period1,
            "period2": period2,
            "interval": interval,
            "events": "history",
            "crumb": crumb
        }
        response = self.session.get(url, params=params)
        return response.text
```

### Error Retry Logic
```python
import time
import requests

def get_with_retry(url, params=None, cookies=None, headers=None, max_retries=3):
    for attempt in range(max_retries):
        try:
            response = requests.get(url, params=params, cookies=cookies, headers=headers)

            if response.status_code == 200:
                return response
            elif response.status_code == 401:
                # Cookie/crumb expired, need to refresh
                print("Auth failed, refresh cookie/crumb")
                return None
            elif response.status_code == 429:
                # Rate limited, exponential backoff
                wait_time = (2 ** attempt) * 1
                print(f"Rate limited, waiting {wait_time}s")
                time.sleep(wait_time)
            else:
                print(f"Error {response.status_code}: {response.text}")
                return None

        except Exception as e:
            print(f"Request failed: {e}")
            time.sleep(2 ** attempt)

    return None
```

## Summary

| Authentication Type | Endpoints | Required | Complexity |
|---------------------|-----------|----------|------------|
| None | Most REST endpoints | No | Low |
| Cookie + Crumb | Historical download | Yes | Medium |
| User-Agent | All endpoints (recommended) | Recommended | Low |
| RapidAPI Key | RapidAPI proxy | Yes | Low |

**Recommendation for Production:**
- Use session management to handle cookies automatically
- Cache crumb for 30 minutes
- Always include User-Agent header
- Implement retry logic for 401/429 errors
- Consider RapidAPI for guaranteed rate limits and support
