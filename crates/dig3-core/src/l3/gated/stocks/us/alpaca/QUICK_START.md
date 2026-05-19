# Alpaca Quick Start - Fix 401 Error

## TL;DR

**Problem:** 401 Unauthorized
**Cause:** Missing API credentials
**Solution:** Get free API keys and set environment variables

## 3-Minute Setup

### 1. Get API Keys (2 minutes)

```
https://app.alpaca.markets/signup
→ Sign up (email only, no credit card)
→ Verify email
→ Login → Account → API Keys
→ Copy both keys
```

### 2. Set Environment Variables (30 seconds)

**Linux/macOS:**
```bash
export ALPACA_API_KEY_ID="PKXYZ123..."
export ALPACA_API_SECRET_KEY="abcdef123..."
```

**Windows PowerShell:**
```powershell
$env:ALPACA_API_KEY_ID="PKXYZ123..."
$env:ALPACA_API_SECRET_KEY="abcdef123..."
```

### 3. Test (30 seconds)

```bash
# Quick curl test
curl -H "APCA-API-KEY-ID: $ALPACA_API_KEY_ID" \
     -H "APCA-API-SECRET-KEY: $ALPACA_API_SECRET_KEY" \
     "https://paper-api.alpaca.markets/v2/account"

# Should return JSON with account info
# If you see "cash": "100000.00" - SUCCESS!
```

## Run Tests

```bash
cd zengeld-terminal/crates/connectors/crates/v5

# Run all Alpaca tests
cargo test --test alpaca_integration -- --nocapture

# Quick ping test
cargo test --test alpaca_integration test_ping -- --nocapture
```

## What You Need to Know

### Environment Variables (EXACT names)

```bash
ALPACA_API_KEY_ID       # OR: APCA_API_KEY_ID
ALPACA_API_SECRET_KEY   # OR: APCA_API_SECRET_KEY
```

Both naming conventions work. Pick one and stick with it.

### Header Format (for curl/HTTP requests)

```http
APCA-API-KEY-ID: your_key_id
APCA-API-SECRET-KEY: your_secret_key
```

**NOT:**
- ❌ `Authorization: Bearer ...`
- ❌ Query parameters `?api_key=...`
- ❌ Any HMAC signatures

Just simple headers!

### Base URLs

```
Paper Trading (testing):  https://paper-api.alpaca.markets
Live Trading (real $$$):  https://api.alpaca.markets
Market Data:              https://data.alpaca.markets
```

Default connector uses **paper trading** (safe).

## Common Errors

### "401 Unauthorized"
- **Missing env vars** → Set `ALPACA_API_KEY_ID` and `ALPACA_API_SECRET_KEY`
- **Wrong keys** → Copy fresh from dashboard
- **Typo** → Keys are case-sensitive

### "403 Forbidden"
- **Wrong environment** → Paper keys only work with paper URL
- **Account inactive** → Check dashboard status

### "422 Unprocessable"
- **Invalid format** → Check no spaces in keys
- **Expired keys** → Regenerate in dashboard

## Verification Checklist

```bash
# 1. Check env vars are set
echo "Key: $ALPACA_API_KEY_ID"
echo "Secret: ${ALPACA_API_SECRET_KEY:0:10}..."

# 2. Test account endpoint
curl -H "APCA-API-KEY-ID: $ALPACA_API_KEY_ID" \
     -H "APCA-API-SECRET-KEY: $ALPACA_API_SECRET_KEY" \
     "https://paper-api.alpaca.markets/v2/account" | jq .

# 3. Test market data
curl -H "APCA-API-KEY-ID: $ALPACA_API_KEY_ID" \
     -H "APCA-API-SECRET-KEY: $ALPACA_API_SECRET_KEY" \
     "https://data.alpaca.markets/v2/stocks/snapshots?symbols=AAPL&feed=iex" | jq .

# 4. Run Rust ping test
cargo test --test alpaca_integration test_ping -- --nocapture
```

All 4 should succeed if credentials are correct.

## Paper Trading Details

- **Free forever**
- **Global access** (no US residency required)
- **$100,000 virtual cash**
- **Full API access** (trading, market data, WebSocket)
- **Same API as live trading** (just different keys/URL)
- **IEX real-time data** (free tier)

Perfect for testing!

## Implementation Status

✅ Authentication: Correct
✅ Endpoints: Correct
✅ Header format: Correct
✅ Tests: Correct

**No code changes needed!**

Just need API keys from Alpaca.

## Links

- Sign up: https://app.alpaca.markets/signup
- Dashboard: https://app.alpaca.markets/
- API Docs: https://docs.alpaca.markets/
- Full setup guide: See `AUTHENTICATION_SETUP.md` in this directory

## Need Help?

1. **Read full guide:** `AUTHENTICATION_SETUP.md`
2. **Check auth docs:** `research/authentication.md`
3. **Verify implementation:** `auth.rs`, `endpoints.rs`
4. **Review tests:** `tests/alpaca_integration.rs`
