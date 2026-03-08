# Jupiter API Authentication

## Overview

Jupiter offers multiple API tiers with different authentication requirements and rate limits. As of December 2024, Jupiter has unified API access through a single portal and key-based authentication system.

## Authentication Method

### API Key Header

All authenticated endpoints require the `x-api-key` header:

```http
GET /price/v3?ids=So11111111111111111111111111111111111111112
Host: api.jup.ag
x-api-key: your-api-key-here
```

**Header Format:**
```
x-api-key: <your-api-key>
```

---

## API Tiers

Jupiter provides three service tiers with different base URLs and capabilities:

### 1. Free Tier

**Base URL:**
```
https://api.jup.ag/
```

**Features:**
- Fixed rate limits (60 requests per 60-second window)
- Access to all API endpoints
- Requires API key (free)
- No cost

**Setup:**
1. Visit https://portal.jup.ag
2. Connect via email
3. Generate free API key

---

### 2. Pro Tier

**Base URL:**
```
https://api.jup.ag/
```

**Features:**
- Fixed tiered rate limits (100-5000 requests per 10 seconds)
- Higher rate limits than Free tier
- Same data freshness as other tiers
- Paid subscription

**Pricing Plans:**

| Tier | Requests | Window | Use Case |
|------|----------|--------|----------|
| Pro I | 100 | 10 seconds | Small applications |
| Pro II | 500 | 10 seconds | Medium applications |
| Pro III | 1,000 | 10 seconds | Large applications |
| Pro IV | 5,000 | 10 seconds | High-volume applications |

**Payment Options:**
- **Helio**: USDC on Solana (manual renewal)
- **Coinflow**: Credit card (automatic subscription)

**Setup:**
1. Visit https://portal.jup.ag
2. Connect via email
3. Select Pro tier
4. Choose payment method
5. Generate API key

---

### 3. Ultra Tier (BETA)

**Base URL:**
```
https://api.jup.ag/ultra/
```

**Features:**
- Dynamic rate limits based on executed swap volume
- End-to-end swap execution (no RPC required)
- Automatic transaction submission
- Volume-based scaling

**Dynamic Rate Limiting:**

Adjusts every 10 minutes based on rolling 24-hour swap volume:

| Swap Volume (24h) | Rate Limit (requests/10s) |
|-------------------|---------------------------|
| $0 | 50 (base) |
| $10,000 | 51 |
| $100,000 | 61 |
| $1,000,000 | 165 |

**Setup:**
1. Visit https://portal.jup.ag
2. Connect via email
3. Select Ultra tier
4. Generate API key

**Note:** Ultra tier handles transaction execution, eliminating the need for your own RPC endpoint.

---

## Legacy Endpoints (No Authentication)

### Lite API (DEPRECATED)

**Base URL:**
```
https://lite-api.jup.ag
```

**Status:** Will be deprecated on **January 31, 2026**

**Features:**
- No API key required
- Limited rate limits
- Public access

**Migration Required:**
All users must migrate to `api.jup.ag` with API keys before the deprecation date.

---

## API Key Management

### Generating API Keys

1. Open Portal at https://portal.jup.ag
2. Connect via email authentication
3. Select your desired tier (Free/Pro/Ultra)
4. Click "Generate API Key"
5. Copy and securely store your key

### Key Properties

- **Universal**: Same API key works across all compatible endpoints
- **Account-Based**: Rate limits apply per account, not per key
- **Multiple Keys**: Can generate multiple keys, but they share the same rate limit pool
- **Rotation**: Can regenerate keys if compromised

### Security Best Practices

1. **Never Commit Keys**: Don't hardcode API keys in source code
2. **Environment Variables**: Store keys in `.env` files or secure vaults
3. **Rotate Regularly**: Generate new keys periodically
4. **Limit Exposure**: Use separate keys for different applications if needed
5. **Monitor Usage**: Check portal for usage statistics

---

## Endpoint Authentication Requirements

### Authenticated Endpoints

These endpoints **require** `x-api-key` header:

**Metis Swap API:**
- `POST https://api.jup.ag/swap/v1/quote`
- `POST https://api.jup.ag/swap/v1/swap`
- `POST https://api.jup.ag/swap/v1/swap-instructions`

**Price API V3:**
- `GET https://api.jup.ag/price/v3`

**Tokens API V2:**
- `GET https://api.jup.ag/tokens/v2/search`
- `GET https://api.jup.ag/tokens/v2/tag`
- `GET https://api.jup.ag/tokens/v2/{category}/{interval}`
- `GET https://api.jup.ag/tokens/v2/recent`

**Ultra API:**
- All endpoints under `https://api.jup.ag/ultra/`

### Public Endpoints (Transitioning)

These endpoints currently work without authentication but will require keys:

**V6 Swap API (Public):**
- `GET https://quote-api.jup.ag/v6/quote`
- `POST https://quote-api.jup.ag/v6/swap`
- `POST https://quote-api.jup.ag/v6/swap-instructions`

**Note:** Public V6 endpoints are being phased out in favor of authenticated Metis API.

---

## Implementation Examples

### Rust

```rust
use reqwest::header::{HeaderMap, HeaderValue};

pub struct JupiterClient {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl JupiterClient {
    pub fn new(api_key: String, tier: JupiterTier) -> Self {
        let base_url = match tier {
            JupiterTier::Free | JupiterTier::Pro => "https://api.jup.ag".to_string(),
            JupiterTier::Ultra => "https://api.jup.ag/ultra".to_string(),
        };

        Self {
            client: reqwest::Client::new(),
            base_url,
            api_key,
        }
    }

    fn build_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-api-key",
            HeaderValue::from_str(&self.api_key).unwrap()
        );
        headers
    }

    pub async fn get_quote(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
    ) -> Result<QuoteResponse, Error> {
        let url = format!(
            "{}/swap/v1/quote?inputMint={}&outputMint={}&amount={}",
            self.base_url, input_mint, output_mint, amount
        );

        let response = self.client
            .get(&url)
            .headers(self.build_headers())
            .send()
            .await?;

        response.json().await
    }
}

pub enum JupiterTier {
    Free,
    Pro,
    Ultra,
}
```

### JavaScript/TypeScript

```javascript
class JupiterClient {
  constructor(apiKey, tier = 'free') {
    this.apiKey = apiKey;
    this.baseUrl = tier === 'ultra'
      ? 'https://api.jup.ag/ultra'
      : 'https://api.jup.ag';
  }

  async getQuote(inputMint, outputMint, amount) {
    const url = `${this.baseUrl}/swap/v1/quote?` +
      `inputMint=${inputMint}&outputMint=${outputMint}&amount=${amount}`;

    const response = await fetch(url, {
      headers: {
        'x-api-key': this.apiKey,
      },
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    return await response.json();
  }

  async getPrices(mintAddresses) {
    const url = `${this.baseUrl}/price/v3?ids=${mintAddresses.join(',')}`;

    const response = await fetch(url, {
      headers: {
        'x-api-key': this.apiKey,
      },
    });

    return await response.json();
  }
}

// Usage
const client = new JupiterClient(process.env.JUPITER_API_KEY, 'pro');
const quote = await client.getQuote(
  'So11111111111111111111111111111111111111112',
  'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v',
  100000000
);
```

---

## Rate Limit Scope

### Per-Account Limits

- Rate limits apply to the **account**, not individual API keys
- Generating multiple API keys **does not increase** your rate limit
- All keys under one account share the same rate limit pool

### Request Buckets

Requests are distributed across three independent buckets:

1. **Price API Bucket** (`/price/v3/*`)
   - Separate limits from other endpoints
   - Dedicated bucket for price queries

2. **Studio API Bucket** (`/studio/*`)
   - Pro: 10 requests per 10 seconds
   - Free: 100 requests per 5 minutes

3. **Default Bucket**
   - All other API endpoints
   - Main rate limit applies

---

## Error Handling

### Authentication Errors

**401 Unauthorized:**
```json
{
  "error": "Invalid API key"
}
```

**Cause:** Invalid or missing API key

**Resolution:**
1. Verify API key is correct
2. Check `x-api-key` header is set
3. Regenerate key if necessary

---

## Migration from Lite API

### Before January 31, 2026

If currently using `lite-api.jup.ag`:

1. **Register for API Key:**
   - Visit https://portal.jup.ag
   - Create free account
   - Generate API key

2. **Update Base URLs:**
   ```diff
   - https://lite-api.jup.ag/v6/quote
   + https://api.jup.ag/swap/v1/quote
   ```

3. **Add Authentication Header:**
   ```diff
   const response = await fetch(url, {
   +  headers: {
   +    'x-api-key': process.env.JUPITER_API_KEY,
   +  },
   });
   ```

4. **Test Integration:**
   - Verify all endpoints work with new URL and auth
   - Monitor rate limits in portal

---

## Important Notes

1. **No Signing Required**: Unlike CEX APIs, Jupiter doesn't require request signing with HMAC
2. **Simple Header Auth**: Only `x-api-key` header is needed
3. **No Nonce/Timestamp**: No timestamp or nonce parameters required
4. **No IP Whitelisting**: API keys work from any IP address
5. **Ultra Auto-Execution**: Ultra tier submits transactions automatically
6. **Free Tier Available**: No cost barrier to start using the API
7. **Same Data**: All tiers access same data; only rate limits differ

---

## Support

For authentication issues:
- Join Jupiter Discord: https://discord.gg/jup
- Check documentation: https://dev.jup.ag
- Portal support: https://portal.jup.ag
