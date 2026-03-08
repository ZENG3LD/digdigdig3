# Alpaca Authentication Setup Guide

## Problem: 401 Unauthorized Error

All tests return 401 errors because **API credentials are not set** in the environment.

## Root Cause Analysis

### 1. Authentication Implementation (✓ CORRECT)

**File: `auth.rs`**
- Header format: ✓ Correct (`APCA-API-KEY-ID`, `APCA-API-SECRET-KEY`)
- Implementation: ✓ Simple header-based auth (no HMAC needed)
- Environment variable lookup: ✓ Supports both `ALPACA_*` and `APCA_*` prefixes

### 2. Endpoints Configuration (✓ CORRECT)

**File: `endpoints.rs`**
- Paper trading base URL: `https://paper-api.alpaca.markets` ✓
- Live trading base URL: `https://api.alpaca.markets` ✓
- Market data base URL: `https://data.alpaca.markets` ✓
- Default: Paper trading (safe for testing) ✓

### 3. Test Configuration (✓ CORRECT)

**File: `tests/alpaca_integration.rs`**
- Uses `AlpacaConnector::from_env()` to load credentials
- Expects environment variables to be set
- Tests will fail with 401 if credentials are missing

## Exact Requirements

### Environment Variable Names

The connector supports **BOTH** naming conventions:

**Option 1 (Recommended):**
```bash
ALPACA_API_KEY_ID=your_key_id_here
ALPACA_API_SECRET_KEY=your_secret_key_here
```

**Option 2 (Alternative):**
```bash
APCA_API_KEY_ID=your_key_id_here
APCA_API_SECRET_KEY=your_secret_key_here
```

### Exact Header Format

When making requests, headers must be:
```http
APCA-API-KEY-ID: your_key_id
APCA-API-SECRET-KEY: your_secret_key
```

**IMPORTANT:**
- No `Authorization: Bearer` prefix
- No HMAC signature required
- Just plain API key headers

## How to Get FREE Paper Trading API Keys

### Step 1: Sign Up

1. Go to: https://app.alpaca.markets/signup
2. Enter your email address (no credit card required)
3. Complete email verification
4. **No US residency required for paper trading!**

### Step 2: Access Dashboard

1. Login to: https://app.alpaca.markets/
2. Navigate to: **Account → API Keys** (left sidebar)
3. You will see your paper trading API keys immediately

### Step 3: Generate Keys

Paper trading keys are **automatically generated** when you create an account.

You should see:
- **API Key ID** (public): ~20 alphanumeric characters (e.g., `PKXYZ123ABC456DEF789`)
- **API Secret Key** (private): ~40 alphanumeric characters (e.g., `abcdef1234567890abcdef1234567890abcdef12`)

### Step 4: Export Environment Variables

**On Linux/macOS (bash/zsh):**
```bash
export ALPACA_API_KEY_ID="PKXYZ123ABC456DEF789"
export ALPACA_API_SECRET_KEY="abcdef1234567890abcdef1234567890abcdef12"
```

**On Windows (PowerShell):**
```powershell
$env:ALPACA_API_KEY_ID="PKXYZ123ABC456DEF789"
$env:ALPACA_API_SECRET_KEY="abcdef1234567890abcdef1234567890abcdef12"
```

**On Windows (CMD):**
```cmd
set ALPACA_API_KEY_ID=PKXYZ123ABC456DEF789
set ALPACA_API_SECRET_KEY=abcdef1234567890abcdef1234567890abcdef12
```

**Make Permanent (add to `.bashrc`, `.zshrc`, or Windows environment variables):**
```bash
# Add to ~/.bashrc or ~/.zshrc
export ALPACA_API_KEY_ID="your_key_id"
export ALPACA_API_SECRET_KEY="your_secret_key"
```

## Testing Authentication

### Test 1: Verify Environment Variables Are Set

```bash
echo "Key ID: $ALPACA_API_KEY_ID"
echo "Secret: ${ALPACA_API_SECRET_KEY:0:10}..." # Show only first 10 chars
```

Expected output:
```
Key ID: PKXYZ123ABC456DEF789
Secret: abcdef1234...
```

### Test 2: Manual curl Test

**Test account endpoint:**
```bash
curl -H "APCA-API-KEY-ID: $ALPACA_API_KEY_ID" \
     -H "APCA-API-SECRET-KEY: $ALPACA_API_SECRET_KEY" \
     "https://paper-api.alpaca.markets/v2/account"
```

**Expected success response (HTTP 200):**
```json
{
  "id": "...",
  "account_number": "...",
  "status": "ACTIVE",
  "cash": "100000.00",
  "portfolio_value": "100000.00",
  "buying_power": "400000.00",
  ...
}
```

**If you get 401 error:**
```json
{
  "code": 40110000,
  "message": "access key verification failed"
}
```

This means:
- API key ID is incorrect
- API secret key is incorrect
- Using live keys with paper endpoint (or vice versa)
- Keys have been revoked/regenerated

### Test 3: Test market data endpoint

```bash
curl -H "APCA-API-KEY-ID: $ALPACA_API_KEY_ID" \
     -H "APCA-API-SECRET-KEY: $ALPACA_API_SECRET_KEY" \
     "https://data.alpaca.markets/v2/stocks/snapshots?symbols=AAPL&feed=iex"
```

**Expected response:**
```json
{
  "AAPL": {
    "latestTrade": {
      "t": "2024-01-01T12:34:56Z",
      "p": 185.45,
      ...
    },
    ...
  }
}
```

### Test 4: Run Rust Integration Tests

```bash
cd zengeld-terminal/crates/connectors/crates/v5

# Run all Alpaca tests
cargo test --test alpaca_integration -- --nocapture

# Run specific test
cargo test --test alpaca_integration test_ping -- --nocapture
```

**Expected output (if credentials are correct):**
```
running 1 test
✓ Ping successful
test test_ping ... ok
```

**If credentials are missing:**
```
⚠ Ping failed: Api { code: 401, message: "..." }
  This may be due to:
  - Missing API credentials (set ALPACA_API_KEY_ID and ALPACA_API_SECRET_KEY)
```

## Common Issues & Solutions

### Issue 1: "No Alpaca environment variables found"

**Solution:**
```bash
export ALPACA_API_KEY_ID="your_key"
export ALPACA_API_SECRET_KEY="your_secret"
```

### Issue 2: 401 Error with correct environment variables

**Possible causes:**
1. **Wrong environment** - Using paper keys with live endpoint (or vice versa)
   - Paper keys only work with `https://paper-api.alpaca.markets`
   - Live keys only work with `https://api.alpaca.markets`

2. **Keys revoked/regenerated**
   - Go to dashboard and check if keys are still active
   - Regenerate keys if needed

3. **Typo in keys**
   - Keys are case-sensitive
   - No spaces before/after keys
   - Copy-paste directly from dashboard

### Issue 3: Tests pass but real trading fails

**Check:**
```bash
# Verify you're using paper trading (testnet)
cargo test --test alpaca_integration test_exchange_identity -- --nocapture
```

Expected:
```
Is Testnet: true
```

If false, you're using LIVE trading keys! Switch to paper keys for testing.

### Issue 4: Market data works but trading fails

**Possible causes:**
1. **Account not approved for trading**
   - Check account status in dashboard
   - Paper accounts should be approved immediately

2. **Market closed**
   - US stock market hours: 9:30 AM - 4:00 PM ET, Monday-Friday
   - Paper trading still requires market to be open for some operations

3. **Insufficient balance**
   - Paper accounts start with $100,000 virtual cash
   - Check balance: `cargo test test_get_balance`

## Paper Trading vs Live Trading

### Paper Trading (Free, Global Access)
- **Purpose:** Testing and development
- **Account type:** Virtual money ($100,000 starting balance)
- **Access:** Anyone worldwide
- **API keys:** Separate from live trading
- **Base URL:** `https://paper-api.alpaca.markets`
- **Data feed:** Same as live (IEX free, SIP paid)

### Live Trading (US Residents Only)
- **Purpose:** Real money trading
- **Account type:** Real brokerage account
- **Access:** US residents only (KYC required)
- **API keys:** Separate from paper trading
- **Base URL:** `https://api.alpaca.markets`
- **WARNING:** Real money at risk!

## Security Best Practices

1. **Never commit API keys to git**
   ```bash
   # Add to .gitignore
   .env
   .env.local
   ```

2. **Use separate keys for different environments**
   - Development: Paper keys
   - Testing: Paper keys
   - Production: Live keys (if needed)

3. **Rotate keys periodically**
   - Go to dashboard → API Keys → Regenerate
   - Update environment variables immediately

4. **Monitor API usage**
   - Check dashboard for unusual activity
   - Free tier: 200 API calls/minute
   - Paid tier: Unlimited (fair use)

5. **Never share keys**
   - Don't paste in chat/email
   - Don't share screenshots with keys visible
   - Use OAuth for third-party integrations

## Data Feed Options

### IEX Feed (FREE)
- Coverage: ~2.5% of US market volume
- Cost: Free with any account
- Real-time data: Yes
- Sufficient for: Testing, basic strategies

### SIP Feed (PAID - $99/mo)
- Coverage: 100% of US market volume
- Cost: $99/month (Algo Trader Plus subscription)
- Real-time data: Yes
- Best for: Production trading, high-frequency strategies

**To change feed in code:**
```rust
use digdigdig3::stocks::us::alpaca::{AlpacaConnector, DataFeed};

let connector = AlpacaConnector::from_env()
    .with_feed(DataFeed::SIP);  // Use paid SIP feed
```

## Demo Keys Available?

**No official demo keys** - but paper trading is effectively "demo":
- Free forever
- No credit card required
- Instant API key generation
- Full API access (except live trading)
- Virtual $100,000 balance

## Quick Start Checklist

- [ ] Sign up at https://app.alpaca.markets/signup
- [ ] Verify email
- [ ] Login to dashboard
- [ ] Go to Account → API Keys
- [ ] Copy API Key ID
- [ ] Copy API Secret Key
- [ ] Export environment variables:
  ```bash
  export ALPACA_API_KEY_ID="your_key_id"
  export ALPACA_API_SECRET_KEY="your_secret_key"
  ```
- [ ] Test with curl:
  ```bash
  curl -H "APCA-API-KEY-ID: $ALPACA_API_KEY_ID" \
       -H "APCA-API-SECRET-KEY: $ALPACA_API_SECRET_KEY" \
       "https://paper-api.alpaca.markets/v2/account"
  ```
- [ ] Run Rust tests:
  ```bash
  cd zengeld-terminal/crates/connectors/crates/v5
  cargo test --test alpaca_integration test_ping -- --nocapture
  ```

## Summary

**Code is correct** - the 401 errors are expected because:
1. No API credentials are set in environment variables
2. This is intentional security design (credentials not hardcoded)
3. Solution: Get free paper trading API keys from Alpaca
4. Export them as environment variables
5. Tests will pass once credentials are configured

**No code changes needed** - just need to set up API keys!
