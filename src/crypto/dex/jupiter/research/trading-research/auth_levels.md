# Jupiter API Authentication & Rate Limits

Source: https://dev.jup.ag/ (official Jupiter developer documentation)
Research date: 2026-03-11

---

## 1. Authentication Method

### 1A. API Key (HTTP Header)

All Jupiter REST API endpoints (Ultra Swap, Metis Swap, Trigger, Recurring) authenticate via a single HTTP header:

```
x-api-key: <your-api-key>
```

**How to get an API key:**
1. Go to https://portal.jup.ag/
2. Log in via email
3. Generate a free API key
4. Allow 2-5 minutes for activation

**Key properties:**
- Universal: the same key works across ALL Jupiter API families (Ultra, Swap, Trigger, Recurring, Price, Studio)
- Free to generate
- Rate limits are enforced per account, not per individual API key
- Multiple keys under the same account share the same quota; generating extra keys does NOT increase rate limits

---

### 1B. Wallet Signature (Transaction Authentication)

Jupiter never holds custody of user funds. For all trading operations, the user must sign the Solana transaction with their private key before it can be submitted. This is not an API authentication mechanism — it is a Solana blockchain requirement.

**Flow for all order types:**
1. Call Jupiter API endpoint to get an unsigned transaction (base64-encoded)
2. Deserialize the transaction
3. Sign with user's Solana keypair (private key, never sent to Jupiter)
4. Re-serialize to base64
5. Submit signed transaction either:
   - To Jupiter's `/execute` endpoint (Ultra/Trigger/Recurring)
   - Directly to Solana RPC (Metis Swap path)

Jupiter never receives, stores, or has access to private keys.

---

### 1C. Quote Endpoints — No Auth Required?

The public `lite-api.jup.ag` mirror exists and is referenced in some documentation:
- `https://lite-api.jup.ag/ultra/v1/order`
- `https://lite-api.jup.ag/swap/v1/quote`
- `https://lite-api.jup.ag/trigger/v1/createOrder`
- `https://lite-api.jup.ag/recurring/v1/createOrder`

The lite-api endpoint is subject to free-tier rate limits. The `api.jup.ag` endpoint requires an API key for reliable access. Official documentation recommends always using an API key for production integrations.

---

## 2. Rate Limits

### 2A. Rate Limit System Structure

Jupiter uses two separate rate-limiting systems:

| System | Applies To | Window |
|---|---|---|
| Fixed Rate Limit | Free tier, Pro tiers (not Ultra Swap) | Sliding window |
| Dynamic Rate Limit | Ultra Swap API only | Rolling 10-second window |

Rate limits apply **per account** (based on API key). Generating multiple API keys does not multiply quota.

---

### 2B. Fixed Rate Limits (Free and Pro Tiers)

#### Free Tier

| Metric | Value |
|---|---|
| Cost | Free |
| Requests per window | 60 |
| Window | 60 seconds |
| Base URL | `https://api.jup.ag/` |

#### Pro Tiers (paid)

| Tier | Requests per Window | Window | Notes |
|---|---|---|---|
| Pro I | 100 | 10 seconds | ~600 req/min |
| Pro II | 500 | 10 seconds | ~3,000 req/min |
| Pro IV | 5,000 | 10 seconds | ~30,000 req/min |

Payment options: USDC on Solana (manual renewal) or credit card subscription.

**Pro tier does NOT automatically extend rate limits to the Ultra Swap API.** Pro and Ultra are separate systems and can be combined.

---

### 2C. Dynamic Rate Limits (Ultra Swap API)

The Ultra Swap API uses a dynamic rate limit that scales with the account's swap execution volume on `ultra/v1/execute`.

| Metric | Value |
|---|---|
| Base quota | 50 requests per 10 seconds |
| Scaling | Increases based on rolling-day swap volume |
| Max (example) | 165+ requests per 10 seconds at sufficient volume |
| Window | Rolling 10-second window |
| Volume source | Aggregated from `/execute` on Ultra for current rolling day |

**Implication for V5 design:** The Ultra API is self-reinforcing — higher swap throughput unlocks higher API quota, making it suitable for high-frequency bot usage that actually executes trades.

---

### 2D. Rate Limit Buckets (Independent per API family)

Three separate rate-limit buckets exist. Each bucket enforces its own sliding window independently:

| Bucket | Endpoints | Notes |
|---|---|---|
| Default | All except Price API and Studio API | Covers Swap, Trigger, Recurring, Ultra |
| Price API | `/price/v3/` | Separate quota |
| Studio API | `/studio/` | Separate quota |

A Pro II subscriber gets 500 requests per 10 seconds per bucket, meaning 500 for Default + 500 for Price + 500 for Studio simultaneously.

---

### 2E. Rate Limit Errors and Headers

- Exceeding limits: HTTP `429 Too Many Requests`
- Response includes `Retry-After` header
- Recommended client behavior: exponential backoff

---

## 3. API Tier Comparison Table

| Feature | Free | Pro | Ultra (Dynamic) |
|---|---|---|---|
| Cost | Free | Paid (USDC or card) | Free (volume-scaled) |
| Rate limit | 60 req/60s | 100–5,000 req/10s | 50+ req/10s (scales) |
| API key required | Yes | Yes | Yes |
| Data freshness | Same | Same | Same |
| Latency | Same | Same | Same |
| Execute via Jupiter | Yes (Ultra) | Yes (Ultra) | Yes (Ultra native) |
| Custom RPC required | For Metis | For Metis | No (Ultra handles it) |
| CPI support | Metis only | Metis only | No |

No differences in data quality or freshness between tiers — only rate limits differ.

---

## 4. Perps Authentication

Jupiter Perpetuals (Perps) does not use REST API authentication. All interaction is via direct Solana blockchain transactions:

- No API key needed to read position data (public on-chain accounts)
- Transaction signing uses standard Solana wallet keypair
- No special authentication beyond standard Solana transaction signing

---

## 5. Summary for V5 Trait Design

### What needs wallet private key:
- All swap execution (Ultra `/execute`, Metis manual RPC)
- Trigger order creation and cancellation
- Recurring order creation and cancellation
- Perps position management

### What does NOT need private key:
- Getting quotes (`/swap/v1/quote`)
- Getting order data (`/order`, `/getTriggerOrders`, `/getRecurringOrders`)
- Getting token holdings (`/holdings/{address}`)
- Reading Perps data from chain

### API key management:
- Single `x-api-key` header for all REST endpoints
- Free tier sufficient for development and low-volume bots
- Pro tier needed for high-frequency REST calls (non-Ultra)
- Ultra Dynamic tier is self-funding (no cost, scales with volume)

---

## Sources

- [API Key Setup](https://dev.jup.ag/docs/api-setup)
- [Dynamic Rate Limit](https://dev.jup.ag/portal/rate-limit)
- [API FAQ](https://dev.jup.ag/docs/api-faq)
- [Ultra Swap API Overview](https://dev.jup.ag/docs/ultra)
- [Jupiter Developer Portal](https://portal.jup.ag/)
