# Raydium Symbol/Token Formatting Research

**Research Date**: 2026-01-20

This document covers token addressing, symbol formatting, and pair representation in Raydium DEX.

---

## Table of Contents

- [Critical Difference from CEX](#critical-difference-from-cex)
- [Token Identification on Solana](#token-identification-on-solana)
- [Symbol Format](#symbol-format)
- [Pool/Pair Representation](#poolpair-representation)
- [Common Token Addresses](#common-token-addresses)
- [Token Decimals](#token-decimals)
- [Comparison with CEX Symbol Format](#comparison-with-cex-symbol-format)

---

## Critical Difference from CEX

### Centralized Exchange Symbol Format

**Examples** (KuCoin, Binance):
- Spot: `BTC-USDT`, `ETH-BTC`
- Futures: `XBTUSDTM`, `ETHUSDM`

**Characteristics**:
- Human-readable string identifiers
- Exchange-specific format
- Same asset has different symbols on different exchanges
- No on-chain representation

### DEX Symbol Format (Solana/Raydium)

**Examples**:
- Token: `So11111111111111111111111111111111111111112` (SOL)
- Token: `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v` (USDC)
- Pool: `AVs9TA4nWDzfPJE9gGVNJMVhcQy3V9PGazuz33BfG2RA` (SOL-USDC pool)

**Characteristics**:
- **Mint address** (Solana pubkey) is the universal identifier
- Same mint address across ALL Solana applications
- Human-readable symbols (like "SOL", "USDC") are metadata, not identifiers
- On-chain address is the source of truth

**Key Insight**: On Solana, you don't query by symbol. You query by **mint address**.

---

## Token Identification on Solana

### Mint Address (Primary Identifier)

**Definition**: A Solana public key representing the SPL token mint account.

**Format**: Base58-encoded 32-byte public key

**Length**: 32-44 characters (Base58 encoding introduces variability)

**Examples**:
```
So11111111111111111111111111111111111111112  # SOL (wrapped)
EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v # USDC
Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB # USDT
4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R # RAY
```

**Properties**:
- **Unique**: Each token has exactly one mint address
- **Immutable**: Mint address never changes
- **Universal**: Same address on all Solana programs and dApps
- **Verifiable**: Can be verified on Solana blockchain

### Symbol (Human-Readable Label)

**Definition**: A short string representing the token (e.g., "SOL", "USDC", "RAY")

**Format**: 2-10 characters, uppercase recommended

**Examples**:
```
SOL   -> So11111111111111111111111111111111111111112
USDC  -> EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
USDT  -> Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB
RAY   -> 4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R
```

**Properties**:
- **Not unique**: Multiple tokens can have same symbol (e.g., many "USDT" fakes)
- **Mutable**: Can change in token metadata
- **Display-only**: Never used as identifier in smart contracts or APIs
- **Unverified**: Anyone can create token with any symbol

**Critical Warning**: NEVER use symbol as primary identifier. Always use mint address.

### Symbol Collisions

**Problem**: Multiple tokens can have identical symbols.

**Example**:
```
Symbol: "USDT"
├── Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB (real USDT)
├── XYZ123...ABC (fake USDT #1)
├── DEF456...GHI (fake USDT #2)
└── ... (hundreds of fake USDT tokens)
```

**Solution**: Always validate mint address against verified token lists:
- [Solana Token List](https://github.com/solana-labs/token-list)
- Raydium token list endpoint: `GET /mint/list`
- CoinGecko verified tokens

---

## Symbol Format

### Token Metadata Structure

**From Raydium API**:
```json
{
  "address": "So11111111111111111111111111111111111111112",
  "chainId": 101,
  "symbol": "SOL",
  "name": "Wrapped SOL",
  "decimals": 9,
  "logoURI": "https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/So11111111111111111111111111111111111111112/logo.png",
  "tags": ["wrapped", "native"],
  "extensions": {
    "coingeckoId": "solana"
  }
}
```

**Fields**:
- `address`: Mint address (primary key)
- `symbol`: Short ticker (display only)
- `name`: Full token name
- `decimals`: Number of decimal places
- `logoURI`: Logo image URL
- `tags`: Category tags
- `extensions`: Additional metadata (CoinGecko ID, etc.)

### Symbol Naming Conventions

**Standard Symbols**:
- Major tokens: `SOL`, `USDC`, `USDT`, `RAY`, `SRM`
- Uppercase: Recommended
- Short: 2-5 characters typical
- Alphanumeric: A-Z, 0-9

**LP Token Symbols**:
- Format: `{TOKEN_A}-{TOKEN_B}`
- Example: `SOL-USDC` (LP token for SOL/USDC pool)
- Note: This is just metadata, not the mint address

**Wrapped vs Native**:
- Native SOL: `11111111111111111111111111111111` (system program)
- Wrapped SOL: `So11111111111111111111111111111111111111112` (SPL token)
- For trading on Raydium: Always use wrapped SOL address

---

## Pool/Pair Representation

### Pool Identification

**Pool ID (Primary Identifier)**:
```
AVs9TA4nWDzfPJE9gGVNJMVhcQy3V9PGazuz33BfG2RA
```

**Format**: Solana public key (Base58-encoded)

**Properties**:
- Unique identifier for liquidity pool
- Derived from pool account address on-chain
- Never changes for a given pool

### Token Pair Representation

**API Response**:
```json
{
  "id": "AVs9TA4nWDzfPJE9gGVNJMVhcQy3V9PGazuz33BfG2RA",
  "mintA": {
    "address": "So11111111111111111111111111111111111111112",
    "symbol": "SOL",
    "decimals": 9
  },
  "mintB": {
    "address": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
    "symbol": "USDC",
    "decimals": 6
  },
  "price": 145.67
}
```

**Pair Notation**:
- **mintA**: First token in pair (base)
- **mintB**: Second token in pair (quote)
- **price**: Price of mintA in terms of mintB

**Display Format**:
```
SOL/USDC  # SOL is base, USDC is quote
145.67 USDC per SOL
```

### Finding Pools by Token Pair

**API Query**:
```http
GET /pools/info/mint?mint1=So11111111111111111111111111111111111111112&mint2=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
```

**Parameters**:
- `mint1`: First token mint address (required)
- `mint2`: Second token mint address (optional)

**Response**: All pools containing the specified token(s)

**Note**: Order of mint1/mint2 doesn't matter. API returns pools with tokens in either order.

---

## Common Token Addresses

### Native Solana Tokens

| Symbol | Mint Address | Decimals | Description |
|--------|--------------|----------|-------------|
| **SOL** (wrapped) | `So11111111111111111111111111111111111111112` | 9 | Wrapped SOL for SPL token standard |
| **USDC** | `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v` | 6 | USD Coin (Circle) |
| **USDT** | `Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB` | 6 | Tether USD |
| **RAY** | `4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R` | 6 | Raydium Token |
| **SRM** | `SRMuApVNdxXokk5GT7XD5cUUgXMBCoAz2LHeuAoKWRt` | 6 | Serum Token |
| **MNGO** | `MangoCzJ36AjZyKwVj3VnYU4GTonjfVEnJmvvWaxLac` | 6 | Mango Markets |
| **ORCA** | `orcaEKTdK7LKz57vaAYr9QeNsVEPfiu6QeMU1kektZE` | 6 | Orca DEX Token |

### Stablecoins

| Symbol | Mint Address | Decimals | Issuer |
|--------|--------------|----------|--------|
| **USDC** | `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v` | 6 | Circle |
| **USDT** | `Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB` | 6 | Tether |
| **USDC.e** | `A9mUU4qviSctJVPJdBJWkb28deg915LYJKrzQ19ji3FM` | 6 | Wormhole USDC |
| **BUSD** | `AJ1W9A9N9dEMdVyoDiam2rV44gnBm2csrPDP7xqcapgX` | 6 | Binance USD |
| **DAI** | `EjmyN6qEC1Tf1JxiG1ae7UTJhUxSwk1TCWNWqxWV4J6o` | 8 | Dai Stablecoin |

### DEX Tokens

| Symbol | Mint Address | Decimals | DEX |
|--------|--------------|----------|-----|
| **RAY** | `4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R` | 6 | Raydium |
| **SRM** | `SRMuApVNdxXokk5GT7XD5cUUgXMBCoAz2LHeuAoKWRt` | 6 | Serum (deprecated) |
| **ORCA** | `orcaEKTdK7LKz57vaAYr9QeNsVEPfiu6QeMU1kektZE` | 6 | Orca |
| **BONK** | `DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263` | 5 | Bonk |

### Popular Meme Coins

| Symbol | Mint Address | Decimals | Name |
|--------|--------------|----------|------|
| **BONK** | `DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263` | 5 | Bonk |
| **WIF** | `EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm` | 6 | dogwifhat |
| **SAMO** | `7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU` | 9 | Samoyed Coin |

**Warning**: Meme coins are highly volatile and risky. Many fake tokens exist with similar symbols.

---

## Token Decimals

### Decimal Precision by Asset Class

| Asset Type | Typical Decimals | Example |
|------------|------------------|---------|
| **Native SOL** | 9 | 1 SOL = 1,000,000,000 lamports |
| **Stablecoins** | 6 | 1 USDC = 1,000,000 micro-USDC |
| **Most SPL tokens** | 6-9 | Varies |
| **LP tokens** | 6-9 | Matches underlying tokens |

### Conversion Examples

**SOL (9 decimals)**:
```
Human-readable: 1.5 SOL
Base units: 1,500,000,000 lamports
Formula: 1.5 * 10^9 = 1,500,000,000
```

**USDC (6 decimals)**:
```
Human-readable: 145.67 USDC
Base units: 145,670,000 micro-USDC
Formula: 145.67 * 10^6 = 145,670,000
```

**RAY (6 decimals)**:
```
Human-readable: 2.5 RAY
Base units: 2,500,000
Formula: 2.5 * 10^6 = 2,500,000
```

### Importance of Decimals

**Critical for**:
1. **Amount Parsing**: Converting API responses to human-readable
2. **Amount Encoding**: Building transactions with correct base units
3. **Price Calculations**: Ensuring decimal point alignment

**Example Error**:
```rust
// WRONG: Treats USDC as having 9 decimals (like SOL)
let amount_wrong = 145.67 * 10_f64.powi(9); // 145,670,000,000 (wrong!)

// CORRECT: Uses actual USDC decimals (6)
let decimals = 6; // from token metadata
let amount_correct = 145.67 * 10_f64.powi(decimals); // 145,670,000 (correct!)
```

### Querying Decimals

**From Token List API**:
```json
{
  "address": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
  "symbol": "USDC",
  "decimals": 6  // <-- Use this
}
```

**On-Chain Query** (if not in token list):
```rust
use solana_client::rpc_client::RpcClient;
use spl_token::state::Mint;

let rpc = RpcClient::new("https://api.mainnet-beta.solana.com");
let mint_data = rpc.get_account_data(&mint_pubkey)?;
let mint = Mint::unpack(&mint_data)?;
let decimals = mint.decimals; // u8
```

---

## Comparison with CEX Symbol Format

### CEX (e.g., KuCoin) Symbol Format

**Spot Format**:
```
BTC-USDT
ETH-BTC
SOL-USDC
```

**Pattern**: `{BASE}-{QUOTE}`
- Separator: Hyphen (`-`)
- Case: Uppercase
- Human-readable symbols

**Futures Format**:
```
XBTUSDTM  # BTC/USDT perpetual
ETHUSDM   # ETH/USD perpetual
```

**Pattern**: `{BASE}{QUOTE}M`
- No separator
- Special mappings (BTC → XBT)

**API Query Example**:
```http
GET /api/v1/market/stats?symbol=BTC-USDT
```

**Rust Implementation**:
```rust
pub fn format_symbol(base: &str, quote: &str) -> String {
    format!("{}-{}", base, quote)
}

// Usage
let symbol = format_symbol("BTC", "USDT"); // "BTC-USDT"
```

---

### DEX (Raydium) Symbol Format

**Token Format**:
```
So11111111111111111111111111111111111111112  # SOL
EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v # USDC
```

**Pattern**: Base58-encoded Solana public key
- No separator (single address)
- Case-sensitive Base58
- Cryptographic identifier

**Pair Query Example**:
```http
GET /pools/info/mint?mint1=So11111111111111111111111111111111111111112&mint2=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
```

**Rust Implementation**:
```rust
pub fn query_pool(mint_a: &str, mint_b: &str) -> String {
    format!(
        "/pools/info/mint?mint1={}&mint2={}",
        mint_a, mint_b
    )
}

// Usage
let url = query_pool(
    "So11111111111111111111111111111111111111112",
    "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
);
```

---

### Key Differences Summary

| Aspect | CEX (KuCoin) | DEX (Raydium) |
|--------|--------------|---------------|
| **Identifier** | Human-readable symbol | Mint address (cryptographic) |
| **Format** | `BTC-USDT` | `So11111...112` + `EPjF...1v` |
| **Uniqueness** | Exchange-specific | Universal (Solana-wide) |
| **Collision Risk** | Low (exchange controls) | High (anyone can create) |
| **Query Method** | By symbol string | By mint address(es) |
| **Validation** | Simple string check | Base58 decode + length check |
| **Length** | 5-10 characters | 32-44 characters per address |
| **Case Sensitivity** | Usually case-insensitive | Case-sensitive (Base58) |
| **Separator** | Hyphen for pairs | None (separate parameters) |

---

## Symbol Validation

### Validating Mint Address

**Rust Example**:
```rust
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

fn validate_mint_address(address: &str) -> Result<Pubkey, String> {
    Pubkey::from_str(address)
        .map_err(|e| format!("Invalid mint address: {}", e))
}

// Usage
match validate_mint_address("So11111111111111111111111111111111111111112") {
    Ok(pubkey) => println!("Valid: {}", pubkey),
    Err(e) => eprintln!("Error: {}", e),
}
```

**Validation Rules**:
1. Must be valid Base58 string
2. Must decode to exactly 32 bytes
3. Characters: `[1-9A-HJ-NP-Za-km-z]` (Base58 alphabet)
4. No `0`, `O`, `I`, `l` (excluded from Base58)

### Verifying Against Token List

**Check if token is legitimate**:
```rust
async fn is_verified_token(mint: &str) -> Result<bool> {
    let token_list = fetch_token_list().await?;
    Ok(token_list.iter().any(|token| token.address == mint))
}
```

**Raydium Token List Endpoint**:
```http
GET /mint/list
```

**Response**: Array of verified tokens recognized by Raydium.

---

## Symbol-to-Mint Lookup

### Problem

User inputs human-readable symbol (e.g., "SOL", "USDC"), but API requires mint address.

### Solution: Symbol Mapping

**Build Lookup Table**:
```rust
use std::collections::HashMap;

struct TokenRegistry {
    by_symbol: HashMap<String, String>, // symbol -> mint
    by_mint: HashMap<String, TokenInfo>, // mint -> full info
}

impl TokenRegistry {
    async fn load_from_api() -> Result<Self> {
        let response = reqwest::get("https://api-v3.raydium.io/mint/list").await?;
        let tokens: Vec<TokenInfo> = response.json().await?;

        let mut by_symbol = HashMap::new();
        let mut by_mint = HashMap::new();

        for token in tokens {
            by_symbol.insert(token.symbol.clone(), token.address.clone());
            by_mint.insert(token.address.clone(), token);
        }

        Ok(Self { by_symbol, by_mint })
    }

    fn symbol_to_mint(&self, symbol: &str) -> Option<&String> {
        self.by_symbol.get(symbol)
    }

    fn mint_to_info(&self, mint: &str) -> Option<&TokenInfo> {
        self.by_mint.get(mint)
    }
}
```

**Usage**:
```rust
let registry = TokenRegistry::load_from_api().await?;

// Convert symbol to mint
let sol_mint = registry.symbol_to_mint("SOL").unwrap();
// "So11111111111111111111111111111111111111112"

// Get full token info
let token_info = registry.mint_to_info(sol_mint).unwrap();
println!("Decimals: {}", token_info.decimals);
```

**Collision Handling**:
```rust
// If multiple tokens have same symbol, use first verified one
// OR prompt user to select from list
// OR prefer by TVL/liquidity ranking
```

---

## Implementation Recommendations

### For Connector Development

**Store Mint Addresses, Not Symbols**:
```rust
pub struct Pool {
    pub id: String,            // Pool address
    pub mint_a: String,        // First token mint
    pub mint_b: String,        // Second token mint
    pub symbol_a: String,      // Display only
    pub symbol_b: String,      // Display only
    pub decimals_a: u8,        // For amount conversion
    pub decimals_b: u8,        // For amount conversion
}
```

**Query by Mint Address**:
```rust
impl RaydiumConnector {
    pub async fn get_pool(&self, mint_a: &str, mint_b: &str) -> Result<Pool> {
        let url = format!(
            "{}/pools/info/mint?mint1={}&mint2={}",
            self.base_url, mint_a, mint_b
        );
        // ...
    }
}
```

**Display to User**:
```rust
fn display_pool(pool: &Pool) -> String {
    format!("{}/{}", pool.symbol_a, pool.symbol_b)
}

// Shows: "SOL/USDC" (user-friendly)
// But internally uses:
// mint_a: "So11111111111111111111111111111111111111112"
// mint_b: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
```

### Don't Hardcode Symbols

**Bad**:
```rust
const SOL_MINT: &str = "So11111111111111111111111111111111111111112";
const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

fn get_sol_usdc_pool() -> Pool {
    query_pool(SOL_MINT, USDC_MINT)
}
```

**Good**:
```rust
async fn get_token_by_symbol(symbol: &str) -> Result<String> {
    let registry = load_token_registry().await?;
    registry.symbol_to_mint(symbol)
        .ok_or(Error::TokenNotFound(symbol.to_string()))
}

async fn get_pool_by_symbols(symbol_a: &str, symbol_b: &str) -> Result<Pool> {
    let mint_a = get_token_by_symbol(symbol_a).await?;
    let mint_b = get_token_by_symbol(symbol_b).await?;
    query_pool(&mint_a, &mint_b).await
}
```

**Why**: Token lists update. New tokens added. Hardcoding becomes stale.

---

## Sources

Research compiled from the following sources:

- [Raydium API Documentation](https://docs.raydium.io/raydium/for-developers/api)
- [Solana Token List Standard](https://github.com/solana-labs/token-list)
- [Solana Documentation - Tokens](https://docs.solana.com/developing/programming-model/tokens)
- [SPL Token Program](https://spl.solana.com/token)
- [Raydium SDK V2 GitHub](https://github.com/raydium-io/raydium-sdk-V2)
- [Base58 Encoding Specification](https://en.bitcoin.it/wiki/Base58Check_encoding)

---

**Document Version**: 1.0
**Last Updated**: 2026-01-20
**Research Completed By**: Claude Code Research Agent
