# Jupiter Token Symbols and Mint Addresses

## Overview

Jupiter operates on Solana and uses SPL token mint addresses instead of traditional symbol pairs. Each token is identified by its unique mint address (public key) rather than a simple ticker symbol.

---

## Token Identification

### Mint Address Format

**Structure:** Base58-encoded public key (32 bytes)
**Length:** 32-44 characters
**Character Set:** Base58 alphabet (no 0, O, I, l to avoid confusion)

**Examples:**
```
So11111111111111111111111111111111111111112  (SOL - Wrapped)
EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v  (USDC)
JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN  (JUP)
```

---

## Common Token Mint Addresses

### Major Tokens

| Symbol | Name | Mint Address | Decimals |
|--------|------|-------------|----------|
| SOL | Wrapped SOL | `So11111111111111111111111111111111111111112` | 9 |
| USDC | USD Coin | `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v` | 6 |
| USDT | Tether USD | `Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB` | 6 |
| JUP | Jupiter | `JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN` | 6 |
| RAY | Raydium | `4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R` | 6 |
| ORCA | Orca | `orcaEKTdK7LKz57vaAYr9QeNsVEPfiu6QeMU1kektZE` | 6 |

### Stablecoins

| Symbol | Name | Mint Address | Decimals |
|--------|------|-------------|----------|
| USDC | USD Coin | `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v` | 6 |
| USDT | Tether USD | `Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB` | 6 |
| USDH | USDH Stablecoin | `USDH1SM1ojwWUga67PGrgFWUHibbjqMvuMaDkRJTgkX` | 6 |
| UXD | UXD Stablecoin | `7kbnvuGBxxj8AG9qp8Scn56muWGaRaFqxg1FsRp3PaFT` | 6 |
| PAI | Parrot USD | `Ea5SjE2Y6yvCeW5dYTn7PYMuW5ikXkvbGdcmSnXeaLjS` | 6 |

### Liquid Staking Tokens (LST)

| Symbol | Name | Mint Address | Decimals |
|--------|------|-------------|----------|
| mSOL | Marinade Staked SOL | `mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So` | 9 |
| stSOL | Lido Staked SOL | `7dHbWXmci3dT8UFYWYZweBLXgycu7Y3iL6trKn1Y7ARj` | 9 |
| jitoSOL | Jito Staked SOL | `J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn` | 9 |
| bSOL | BlazeStake Staked SOL | `bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1` | 9 |

### DeFi Tokens

| Symbol | Name | Mint Address | Decimals |
|--------|------|-------------|----------|
| SRM | Serum | `SRMuApVNdxXokk5GT7XD5cUUgXMBCoAz2LHeuAoKWRt` | 6 |
| MNGO | Mango Markets | `MangoCzJ36AjZyKwVj3VnYU4GTonjfVEnJmvvWaxLac` | 6 |
| STEP | Step Finance | `StepAscQoEioFxxWGnh2sLBDFp9d8rvKz2Yp39iDpyT` | 9 |
| PORT | Port Finance | `PoRTjZMPXb9T7dyU7tpLEZRQj7e6ssfAE62j2oQuc6y` | 6 |

### Meme Tokens

| Symbol | Name | Mint Address | Decimals |
|--------|------|-------------|----------|
| BONK | Bonk | `DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263` | 5 |
| WIF | dogwifhat | `EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm` | 6 |
| SAMO | Samoyed Coin | `7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU` | 9 |

---

## Symbol vs Mint Address

### Why Mint Addresses?

Unlike centralized exchanges that use symbol pairs (e.g., "BTC/USDT"), Solana DEXs use mint addresses because:

1. **Uniqueness**: Multiple tokens can have same symbol
2. **No Namespace Conflicts**: Mint address is globally unique
3. **Immutable**: Address doesn't change; symbols might
4. **On-Chain Standard**: Solana's native identification method

### Symbol Collisions

**Problem:** Multiple tokens can share the same symbol.

**Example:**
```
Symbol: USDC
- Circle USDC: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
- Fake USDC:   USDCxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx

Symbol: BTC
- Wrapped BTC: 9n4nbM75f5Ui33ZbPYXn59EwSgE8CGsHtAeTH5YFeJ9E
- Portal BTC:  A9mUU4qviSctJVPJdBJWkb28deg915LYJKrzQ19ji3FM
```

**Solution:** Always use mint addresses for precision.

---

## Token Discovery

### Using Tokens API

#### Search by Symbol
```http
GET /tokens/v2/search?query=SOL
```

**Response:**
```json
[
  {
    "id": "So11111111111111111111111111111111111111112",
    "symbol": "SOL",
    "name": "Wrapped SOL",
    "decimals": 9,
    "isVerified": true
  }
]
```

#### Search by Mint Address
```http
GET /tokens/v2/search?query=So11111111111111111111111111111111111111112
```

#### Get Verified Tokens
```http
GET /tokens/v2/tag?query=verified
```

#### Get Liquid Staking Tokens
```http
GET /tokens/v2/tag?query=lst
```

---

## Token Decimals

### Common Decimal Counts

| Decimals | Common For | Examples |
|----------|-----------|----------|
| 0 | NFTs, Unique Items | - |
| 2 | Shares, Units | - |
| 5 | BONK | BONK |
| 6 | Most tokens, Stables | USDC, USDT, JUP, RAY |
| 9 | SOL and LSTs | SOL, mSOL, stSOL |

### Amount Conversion

**Formula:**
```
raw_amount = human_amount × 10^decimals
human_amount = raw_amount / 10^decimals
```

**Examples:**

SOL (9 decimals):
```
1 SOL = 1,000,000,000 raw units
0.1 SOL = 100,000,000 raw units
0.001 SOL = 1,000,000 raw units
```

USDC (6 decimals):
```
1 USDC = 1,000,000 raw units
0.1 USDC = 100,000 raw units
0.001 USDC = 1,000 raw units
```

BONK (5 decimals):
```
1 BONK = 100,000 raw units
0.1 BONK = 10,000 raw units
```

---

## Trading Pairs

### Pair Representation

Jupiter uses separate `inputMint` and `outputMint` parameters:

**Traditional CEX:**
```
SOL/USDC  (base/quote)
```

**Jupiter:**
```
inputMint: So11111111111111111111111111111111111111112  (SOL)
outputMint: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v (USDC)
```

### No Market Direction

Unlike order books with BID/ASK sides, Jupiter is directional:
- **Input**: What you're selling
- **Output**: What you're buying

**Example - Selling SOL for USDC:**
```json
{
  "inputMint": "So11111111111111111111111111111111111111112",
  "outputMint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
  "amount": "100000000"
}
```

**Example - Buying SOL with USDC:**
```json
{
  "inputMint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
  "outputMint": "So11111111111111111111111111111111111111112",
  "amount": "15000000"
}
```

---

## Token Verification

### Verified Status

Jupiter uses a "Jupiter Verify" system (V3 token list) based on:
- Organic score (community support)
- Smart likes
- Trading volume
- Liquidity
- Holder distribution

**Check via API:**
```http
GET /tokens/v2/tag?query=verified
```

**Response Field:**
```json
{
  "isVerified": true
}
```

### Trust Indicators

When evaluating tokens, check:

1. **Verification**: `isVerified` flag
2. **Organic Score**: 0-100 (higher = better)
3. **Audit Info**:
   - Authority status (disabled = better)
   - Holder concentration (lower = better)
   - Freezable/Mintable (false = better)
4. **Liquidity**: Higher = more reliable pricing
5. **CEX Listings**: Listed on major CEXs
6. **Holder Count**: More holders = more adoption

---

## Symbol Normalization

### Converting Symbols to Mint Addresses

**Implementation Pattern:**

```rust
use std::collections::HashMap;

pub struct TokenRegistry {
    symbol_to_mint: HashMap<String, String>,
}

impl TokenRegistry {
    pub fn new() -> Self {
        let mut registry = HashMap::new();

        // Verified tokens only
        registry.insert("SOL".to_string(),
            "So11111111111111111111111111111111111111112".to_string());
        registry.insert("USDC".to_string(),
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string());
        registry.insert("USDT".to_string(),
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string());

        Self {
            symbol_to_mint: registry,
        }
    }

    pub fn get_mint(&self, symbol: &str) -> Option<&String> {
        self.symbol_to_mint.get(symbol)
    }

    pub async fn search_symbol(&self, symbol: &str) -> Result<Vec<Token>, Error> {
        // Use Jupiter API to search
        let url = format!(
            "https://api.jup.ag/tokens/v2/search?query={}",
            symbol
        );
        // ... fetch and parse
    }
}
```

**Usage:**
```rust
let registry = TokenRegistry::new();

// Direct lookup for known tokens
if let Some(mint) = registry.get_mint("SOL") {
    println!("SOL mint: {}", mint);
}

// API search for unknown tokens
let tokens = registry.search_symbol("WIF").await?;
for token in tokens {
    println!("{}: {}", token.symbol, token.id);
}
```

---

## Native SOL vs Wrapped SOL

### Distinction

**Native SOL:**
- Solana's native token
- Not an SPL token
- Used for transaction fees
- Not tradeable directly in DEXs

**Wrapped SOL (wSOL):**
- Mint: `So11111111111111111111111111111111111111112`
- SPL token representation of SOL
- Tradeable in DEXs
- Automatically wrapped/unwrapped by Jupiter

### Auto Wrap/Unwrap

Jupiter handles conversion automatically when `wrapAndUnwrapSol: true` (default):

**Trading SOL → USDC:**
1. User provides native SOL
2. Jupiter wraps to wSOL
3. Swaps wSOL → USDC
4. Returns USDC to user

**Trading USDC → SOL:**
1. User provides USDC
2. Swaps USDC → wSOL
3. Jupiter unwraps to native SOL
4. Returns native SOL to user

---

## Token List Updates

### Dynamic Token List

Jupiter doesn't maintain a static token list. Tokens are discovered dynamically:

1. **Automatic Discovery**: New pools are detected automatically
2. **Quality Scoring**: Organic score calculated based on metrics
3. **Verification**: High-quality tokens earn "verified" status
4. **Real-Time**: No manual listing process

### Fetching Current Tokens

**Top Traded:**
```http
GET /tokens/v2/toptraded/24h?limit=100
```

**Recently Added:**
```http
GET /tokens/v2/recent?limit=50
```

**Trending:**
```http
GET /tokens/v2/toptrending/6h?limit=50
```

---

## Rust Implementation Example

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMint {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
}

impl TokenMint {
    /// Parse raw amount to human-readable
    pub fn to_human(&self, raw_amount: u64) -> f64 {
        raw_amount as f64 / 10f64.powi(self.decimals as i32)
    }

    /// Parse human-readable to raw amount
    pub fn to_raw(&self, human_amount: f64) -> u64 {
        (human_amount * 10f64.powi(self.decimals as i32)) as u64
    }

    /// Validate mint address format
    pub fn is_valid_mint(address: &str) -> bool {
        // Base58 alphabet
        const BASE58: &str = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

        // Check length (32-44 chars for Solana addresses)
        if address.len() < 32 || address.len() > 44 {
            return false;
        }

        // Check all chars are valid base58
        address.chars().all(|c| BASE58.contains(c))
    }
}

// Usage
let sol = TokenMint {
    address: "So11111111111111111111111111111111111111112".to_string(),
    symbol: "SOL".to_string(),
    name: "Wrapped SOL".to_string(),
    decimals: 9,
};

// Convert 1.5 SOL to raw
let raw = sol.to_raw(1.5);  // 1500000000

// Convert raw back to human
let human = sol.to_human(1500000000);  // 1.5
```

---

## Special Cases

### Pump.fun Tokens

Pump.fun is a token launchpad on Solana. New tokens:
- Often have low liquidity initially
- May not be verified
- Can have high volatility
- Usually 6 decimals

**Discovery:**
```http
GET /tokens/v2/recent?limit=100
```

### Migrated Tokens

Some tokens have migrated to new mint addresses:
- Old mint may still exist but deprecated
- Jupiter automatically routes to new mint
- Check token's official sources for current mint

---

## Notes

1. **Always Use Mint Addresses**: Never rely on symbols alone
2. **Verify Tokens**: Check `isVerified` and `organicScore`
3. **Check Decimals**: Different tokens have different decimal places
4. **Handle Precision**: Use proper decimal arithmetic in code
5. **SOL Wrapping**: Jupiter handles automatically
6. **Token Discovery**: Use Tokens API for dynamic discovery
7. **No Delisting**: Tokens aren't removed, but quality scores update
8. **Case Sensitive**: Mint addresses are case-sensitive
