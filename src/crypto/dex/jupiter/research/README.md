# Jupiter Aggregator API Research

Complete API documentation for implementing Jupiter V5 connector.

---

## Overview

**Jupiter** is Solana's leading DEX aggregator that consolidates liquidity from multiple automated market makers (AMMs) to provide optimal token swap rates.

**Protocol:** Solana (SPL tokens)
**Type:** DEX Aggregator
**API Style:** RESTful
**Authentication:** API Key (x-api-key header)

---

## Quick Start

### Base URLs

```
V6 Swap API:     https://quote-api.jup.ag/v6
Metis Swap API:  https://api.jup.ag/swap/v1  (requires API key)
Price API:       https://api.jup.ag/price/v3  (requires API key)
Tokens API:      https://api.jup.ag/tokens/v2 (requires API key)
```

### Authentication

```http
x-api-key: your-api-key-here
```

Get API key at: https://portal.jup.ag

### Basic Quote Request

```bash
GET https://quote-api.jup.ag/v6/quote?
  inputMint=So11111111111111111111111111111111111111112&
  outputMint=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v&
  amount=100000000&
  slippageBps=50
```

---

## Document Index

### [endpoints.md](./endpoints.md)
Complete endpoint reference including:
- Swap API (quote, swap, swap-instructions)
- Price API (get prices for multiple tokens)
- Tokens API (search, tags, categories, recent)
- Request/response formats
- Query parameters
- Symbol formatting (mint addresses)

### [authentication.md](./authentication.md)
Authentication and API tiers:
- API key setup and management
- Free tier (60 req/min)
- Pro tiers (100-5000 req/10s)
- Ultra tier (dynamic scaling)
- Payment options
- Migration from lite-api

### [response_formats.md](./response_formats.md)
Response structures and error codes:
- HTTP status codes (200, 400, 401, 404, 429, 500+)
- Quote response format
- Swap response format
- Swap instructions response
- Price response format
- Token metadata structure
- Program error codes
- TypeScript type definitions

### [symbols.md](./symbols.md)
Token identification on Solana:
- Mint address format (Base58 encoded)
- Common token mint addresses (SOL, USDC, USDT, JUP, etc.)
- Stablecoins and LSTs
- Token decimals and amount conversion
- Trading pairs (inputMint/outputMint)
- Token verification and discovery
- Symbol normalization

### [rate_limits.md](./rate_limits.md)
Rate limiting system:
- Fixed rate limits (Free: 60/min, Pro: 100-5000/10s)
- Dynamic rate limits (Ultra: scales with volume)
- Request buckets (Price, Studio, Default)
- Sliding window enforcement
- Rate limit headers
- Handling 429 errors
- Best practices (backoff, caching, batching)

### [websocket.md](./websocket.md)
Real-time data alternatives:
- Jupiter has NO native WebSocket
- Solana RPC WebSocket (on-chain monitoring)
- Third-party providers (bloXroute, Bitquery)
- Polling strategies
- Event-driven architecture
- Rate limit considerations

---

## Key Concepts

### Token Identification

Jupiter uses **Solana SPL token mint addresses** instead of symbols:

```
Symbol: SOL
Mint:   So11111111111111111111111111111111111111112
```

Always use mint addresses in API requests for precision.

### Amount Format

Amounts are in **raw units** (before decimals):

```
SOL (9 decimals):   1 SOL = 1,000,000,000
USDC (6 decimals):  1 USDC = 1,000,000
```

Formula: `raw_amount = human_amount × 10^decimals`

### Swap Flow

1. **GET /quote**: Request routing and pricing
2. **POST /swap**: Build transaction
3. **Sign & Submit**: Sign transaction and submit to Solana RPC

Or use **Ultra tier** for automatic execution.

### Rate Limits

Requests are counted per **account** (not per API key):
- Free: 60 requests / 60 seconds
- Pro I: 100 requests / 10 seconds
- Pro II: 500 requests / 10 seconds
- Pro III: 1,000 requests / 10 seconds
- Pro IV: 5,000 requests / 10 seconds
- Ultra: Dynamic (50-500+ based on volume)

---

## Implementation Checklist

### Required Components

- [ ] HTTP client (reqwest)
- [ ] API key management
- [ ] Mint address registry
- [ ] Amount conversion (decimals)
- [ ] Rate limiter
- [ ] Error handling (401, 429, 500)
- [ ] Quote endpoint
- [ ] Swap endpoint
- [ ] Price endpoint
- [ ] Token search endpoint

### Optional Components

- [ ] Swap instructions endpoint
- [ ] Token categories/tags
- [ ] Caching layer
- [ ] WebSocket simulation (polling)
- [ ] Retry logic with exponential backoff
- [ ] Usage monitoring

---

## V5 Connector Structure

Following KuCoin reference pattern:

```
exchanges/jupiter/
├── mod.rs              # Module exports
├── endpoints.rs        # URL constants, endpoint enum
├── auth.rs             # API key authentication
├── parser.rs           # JSON parsing
├── connector.rs        # Trait implementations
└── research/           # This documentation
    ├── README.md
    ├── endpoints.md
    ├── authentication.md
    ├── response_formats.md
    ├── symbols.md
    ├── rate_limits.md
    └── websocket.md
```

### endpoints.rs

```rust
pub const BASE_URL: &str = "https://api.jup.ag";
pub const SWAP_BASE_URL: &str = "https://quote-api.jup.ag/v6";

pub enum JupiterEndpoint {
    Quote,
    Swap,
    SwapInstructions,
    Price,
    TokenSearch,
}

impl JupiterEndpoint {
    pub fn url(&self, base_url: &str) -> String {
        match self {
            Self::Quote => format!("{}/quote", SWAP_BASE_URL),
            Self::Swap => format!("{}/swap", SWAP_BASE_URL),
            Self::Price => format!("{}/price/v3", base_url),
            // ... etc
        }
    }
}
```

### auth.rs

```rust
use std::collections::HashMap;

pub struct JupiterAuth {
    api_key: String,
}

impl JupiterAuth {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }

    pub fn sign_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("x-api-key".to_string(), self.api_key.clone());
        headers
    }
}
```

### parser.rs

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct QuoteResponse {
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    #[serde(rename = "inAmount")]
    pub in_amount: String,
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    #[serde(rename = "outAmount")]
    pub out_amount: String,
    #[serde(rename = "slippageBps")]
    pub slippage_bps: u16,
    #[serde(rename = "routePlan")]
    pub route_plan: Vec<RoutePlan>,
}

#[derive(Debug, Deserialize)]
pub struct PriceResponse {
    #[serde(flatten)]
    pub prices: HashMap<String, Option<PriceData>>,
}

#[derive(Debug, Deserialize)]
pub struct PriceData {
    #[serde(rename = "usdPrice")]
    pub usd_price: f64,
    #[serde(rename = "blockId")]
    pub block_id: u64,
    pub decimals: u8,
    #[serde(rename = "priceChange24h")]
    pub price_change_24h: f64,
}
```

### connector.rs

```rust
pub struct JupiterConnector {
    client: reqwest::Client,
    base_url: String,
    swap_base_url: String,
    auth: JupiterAuth,
}

impl JupiterConnector {
    pub fn new(api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: BASE_URL.to_string(),
            swap_base_url: SWAP_BASE_URL.to_string(),
            auth: JupiterAuth::new(api_key),
        }
    }

    pub async fn get_quote(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        slippage_bps: u16,
    ) -> Result<QuoteResponse, ExchangeError> {
        let url = format!(
            "{}/quote?inputMint={}&outputMint={}&amount={}&slippageBps={}",
            self.swap_base_url, input_mint, output_mint, amount, slippage_bps
        );

        let response = self.client
            .get(&url)
            .send()
            .await?
            .json::<QuoteResponse>()
            .await?;

        Ok(response)
    }

    pub async fn get_prices(
        &self,
        mint_addresses: &[String],
    ) -> Result<PriceResponse, ExchangeError> {
        let ids = mint_addresses.join(",");
        let url = format!("{}/price/v3?ids={}", self.base_url, ids);

        let mut headers = self.auth.sign_headers();

        let response = self.client
            .get(&url)
            .headers(headers.into())
            .send()
            .await?
            .json::<PriceResponse>()
            .await?;

        Ok(response)
    }
}
```

---

## Common Use Cases

### 1. Get Token Price

```rust
let connector = JupiterConnector::new(api_key);

let prices = connector.get_prices(&[
    "So11111111111111111111111111111111111111112".to_string(),  // SOL
    "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
]).await?;

if let Some(sol_price) = prices.prices.get("So11111111111111111111111111111111111111112") {
    println!("SOL Price: ${}", sol_price.usd_price);
}
```

### 2. Get Swap Quote

```rust
let quote = connector.get_quote(
    "So11111111111111111111111111111111111111112",  // SOL
    "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC
    100_000_000,  // 0.1 SOL (9 decimals)
    50,           // 0.5% slippage
).await?;

println!("Input: {} SOL", quote.in_amount);
println!("Output: {} USDC", quote.out_amount);
println!("Price Impact: {}%", quote.price_impact_pct);
```

### 3. Search Tokens

```rust
let tokens = connector.search_tokens("SOL").await?;

for token in tokens {
    println!("{} ({}): {}", token.name, token.symbol, token.id);
}
```

### 4. Get Verified Tokens

```rust
let verified = connector.get_tokens_by_tag("verified").await?;

for token in verified {
    println!("{}: Organic Score {}", token.symbol, token.organic_score);
}
```

---

## Important Notes

1. **No Signing Required**: Unlike CEXs, Jupiter only needs API key in header
2. **Mint Addresses**: Always use mint addresses, not symbols
3. **Amount Format**: Raw units (multiply by 10^decimals)
4. **Rate Limits**: Per-account, not per-key
5. **No WebSocket**: Use polling or Solana RPC for real-time data
6. **Slippage in BPS**: 50 bps = 0.5%, 100 bps = 1%
7. **Batch Requests**: Use Price API to get up to 50 prices at once
8. **Deprecation**: lite-api.jup.ag deprecated Jan 31, 2026

---

## Testing

### Testnet

Jupiter operates on Solana mainnet-beta. For testing:
- Use devnet/testnet Solana RPC
- Use small amounts on mainnet
- No separate testnet API

### Common Mint Addresses for Testing

```rust
pub const SOL: &str = "So11111111111111111111111111111111111111112";
pub const USDC: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
pub const USDT: &str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";
pub const JUP: &str = "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN";
```

---

## Error Handling

### Common Errors

| Code | Error | Action |
|------|-------|--------|
| 400 | Bad Request | Check parameters |
| 401 | Unauthorized | Verify API key |
| 404 | Not Found | Check endpoint URL |
| 429 | Rate Limited | Implement backoff |
| 500 | Server Error | Retry with backoff |

### Retry Strategy

```rust
pub async fn retry_request<F, T>(
    mut request_fn: F,
    max_retries: u32,
) -> Result<T, Error>
where
    F: FnMut() -> BoxFuture<'static, Result<T, Error>>,
{
    let mut retry_count = 0;
    let base_delay = Duration::from_millis(100);

    loop {
        match request_fn().await {
            Ok(response) => return Ok(response),
            Err(e) if should_retry(&e) && retry_count < max_retries => {
                let delay = base_delay * 2u32.pow(retry_count);
                tokio::time::sleep(delay).await;
                retry_count += 1;
            }
            Err(e) => return Err(e),
        }
    }
}
```

---

## Resources

### Official Documentation
- Developer Docs: https://dev.jup.ag
- API Reference: https://dev.jup.ag/docs/api
- Portal: https://portal.jup.ag

### Community
- Discord: https://discord.gg/jup
- Twitter: @JupiterExchange
- GitHub: https://github.com/jup-ag

### Related APIs
- Solana RPC: https://docs.solana.com/api
- bloXroute: https://docs.bloxroute.com/solana
- Bitquery: https://docs.bitquery.io/docs/blockchain/Solana

---

## Version History

- **V6**: Current Swap API (quote-api.jup.ag/v6)
- **V3**: Current Price API (api.jup.ag/price/v3)
- **V2**: Tokens API (api.jup.ag/tokens/v2)
- **Deprecated**: lite-api.jup.ag (EOL: Jan 31, 2026)

---

## Sources

- [Jupiter Developers](https://dev.jup.ag/api-reference)
- [Jupiter Hub Docs](https://hub.jup.ag/docs/apis/swap-api)
- [API Rate Limiting](https://dev.jup.ag/docs/api-rate-limit)
- [API Key Setup](https://dev.jup.ag/docs/api-setup)
- [Price API V3](https://dev.jup.ag/docs/price/v3)
- [Tokens API V2](https://dev.jup.ag/docs/tokens/v2/token-information)
- [Quote API Reference](https://dev.jup.ag/api-reference/swap/quote)
- [Swap API Reference](https://dev.jup.ag/api-reference/swap/swap)
