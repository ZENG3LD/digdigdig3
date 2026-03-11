# Uniswap Token Addresses and Symbol Formatting

## Overview

Unlike centralized exchanges (CEX) that use symbol-based pairs (e.g., `BTC/USDT`), Uniswap uses:
- **Token Contract Addresses** for identification
- **ERC-20 Standard** for token interface
- **Pool Addresses** for trading pairs
- **Multiple Fee Tiers** per token pair

---

## 1. Token Address Format

### ERC-20 Token Addresses

All Uniswap tokens are identified by their Ethereum contract address.

**Format:**
- 20 bytes (40 hex characters)
- Prefixed with `0x`
- Case-sensitive (checksummed)

**Examples:**
```
WETH:  0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2
USDC:  0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48
USDT:  0xdAC17F958D2ee523a2206206994597C13D831ec7
DAI:   0x6B175474E89094C44Da98b954EedeAC495271d0F
UNI:   0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984
WBTC:  0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599
```

### Address Checksum

Ethereum addresses use EIP-55 mixed-case checksum encoding.

**Invalid (no checksum):**
```
0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2  // All lowercase
```

**Valid (checksummed):**
```
0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2  // Mixed case
```

**Rust Validation:**
```rust
use alloy::primitives::Address;

// This will validate checksum
let addr: Address = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".parse()?;

// Get checksummed string
let checksummed = format!("{:?}", addr);
```

---

## 2. Common Token Addresses (Ethereum Mainnet)

### Stablecoins

| Symbol | Name | Address | Decimals |
|--------|------|---------|----------|
| USDC | USD Coin | `0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48` | 6 |
| USDT | Tether USD | `0xdAC17F958D2ee523a2206206994597C13D831ec7` | 6 |
| DAI | Dai Stablecoin | `0x6B175474E89094C44Da98b954EedeAC495271d0F` | 18 |
| BUSD | Binance USD | `0x4Fabb145d64652a948d72533023f6E7A623C7C53` | 18 |
| FRAX | Frax | `0x853d955aCEf822Db058eb8505911ED77F175b99e` | 18 |

### Wrapped Native Tokens

| Symbol | Name | Address | Decimals |
|--------|------|---------|----------|
| WETH | Wrapped Ether | `0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2` | 18 |
| WBTC | Wrapped Bitcoin | `0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599` | 8 |

### DeFi Tokens

| Symbol | Name | Address | Decimals |
|--------|------|---------|----------|
| UNI | Uniswap | `0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984` | 18 |
| AAVE | Aave | `0x7Fc66500c84A76Ad7e9c93437bFc5Ac33E2DDaE9` | 18 |
| LINK | Chainlink | `0x514910771AF9Ca656af840dff83E8264EcF986CA` | 18 |
| MKR | Maker | `0x9f8F72aA9304c8B593d555F12eF6589cC3A579A2` | 18 |
| COMP | Compound | `0xc00e94Cb662C3520282E6f5717214004A7f26888` | 18 |
| SNX | Synthetix | `0xC011a73ee8576Fb46F5E1c5751cA3B9Fe0af2a6F` | 18 |

### Layer 2 & Scaling

| Symbol | Name | Address | Decimals |
|--------|------|---------|----------|
| MATIC | Polygon | `0x7D1AfA7B718fb893dB30A3aBc0Cfc608AaCfeBB0` | 18 |
| ARB | Arbitrum | `0xB50721BCf8d664c30412Cfbc6cf7a15145234ad1` | 18 |
| OP | Optimism | `0x4200000000000000000000000000000000000042` | 18 |

---

## 3. Pool Address Format

### Pool Identification

Uniswap V3 pools are identified by the combination of:
1. Token0 address (lower address value)
2. Token1 address (higher address value)
3. Fee tier

**Example Pool:**
```
Token0: USDC (0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48)
Token1: WETH (0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2)
Fee: 500 (0.05%)
Pool Address: 0x88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640
```

### Computing Pool Address

**Formula (CREATE2):**
```
pool_address = keccak256(
  0xff +
  factory_address +
  keccak256(token0, token1, fee) +
  init_code_hash
)[12:]
```

**Rust Implementation:**
```rust
use alloy::primitives::{Address, keccak256, B256};

fn compute_pool_address(
    factory: Address,
    token0: Address,
    token1: Address,
    fee: u32,
) -> Address {
    let (token0, token1) = if token0 < token1 {
        (token0, token1)
    } else {
        (token1, token0)
    };

    let salt = keccak256((token0, token1, fee).abi_encode());
    let init_code_hash = "0xe34f199b19b2b4f47f68442619d555527d244f78a3297ea89325f843f87b8b54";

    // CREATE2 computation
    // ...
}
```

**Easier: Query from Factory Contract**

```rust
// Call factory.getPool(token0, token1, fee)
let pool_address = factory.getPool(token0, token1, fee).call().await?;
```

---

## 4. Symbol Formatting for API Requests

### Trading API Format

**Quote Request:**
```json
{
  "tokenIn": "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
  "tokenOut": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
}
```

**NOT symbol-based:**
```json
// ❌ WRONG - symbols not used
{
  "tokenIn": "WETH",
  "tokenOut": "USDC"
}
```

### Subgraph Format

**Query by Address:**
```graphql
{
  token(id: "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2") {
    symbol
    name
    decimals
  }
}
```

**Query by Symbol (search):**
```graphql
{
  tokens(where: { symbol: "WETH" }) {
    id
    symbol
    name
  }
}
```

---

## 5. Native ETH vs WETH

### Important Distinction

Uniswap V3 **does not support native ETH directly**. All trading uses WETH.

**Native ETH:**
- Address: N/A (not an ERC-20 token)
- Can't be used directly in Uniswap pools
- Must be wrapped first

**Wrapped ETH (WETH):**
- Address: `0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2`
- ERC-20 token (1:1 with ETH)
- Used in all Uniswap pools

### Wrapping ETH

**Manual Wrap:**
```solidity
// WETH contract
function deposit() external payable;
function withdraw(uint256 amount) external;
```

**Auto-Wrap via Router:**
The Uniswap router can automatically wrap ETH when `msg.value > 0`:

```json
{
  "tokenIn": "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
  "amount": "1000000000000000000",
  "transaction": {
    "value": "1000000000000000000"  // ETH sent, auto-wrapped
  }
}
```

---

## 6. Fee Tiers and Pool Variants

### Multiple Pools per Pair

Unlike CEX, Uniswap has **multiple pools** for the same token pair, each with different fee tiers.

**Example: USDC/WETH Pools**

| Fee Tier | Fee % | Pool Address | Use Case |
|----------|-------|--------------|----------|
| 100 | 0.01% | `0x...` | Stablecoin-like pairs |
| 500 | 0.05% | `0x88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640` | Low volatility |
| 3000 | 0.30% | `0x8ad599c3A0ff1De082011EFDDc58f1908eb6e6D8` | Standard |
| 10000 | 1.00% | `0x...` | High volatility/exotic |

### Fee Tier Selection

**When Creating Pools:**
```rust
factory.createPool(
    token0,
    token1,
    500  // Fee tier (0.05%)
)
```

**When Swapping (specify pool):**
```rust
router.exactInputSingle(ExactInputSingleParams {
    tokenIn: weth,
    tokenOut: usdc,
    fee: 500,  // Choose fee tier
    recipient: trader,
    // ...
})
```

**When Quoting (API chooses best):**
```json
{
  "tokenIn": "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
  "tokenOut": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
  "routingPreference": "BEST_PRICE"
}
```

The routing API will find the best fee tier automatically.

---

## 7. Multi-Chain Token Addresses

### Same Token, Different Chains

Token symbols may be the same across chains, but **addresses differ**.

**USDC on Different Chains:**

| Chain | Chain ID | USDC Address | Symbol |
|-------|----------|--------------|--------|
| Ethereum | 1 | `0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48` | USDC |
| Polygon | 137 | `0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174` | USDC |
| Arbitrum | 42161 | `0xFF970A61A04b1cA14834A43f5dE4533eBDDB5CC8` | USDC |
| Optimism | 10 | `0x7F5c764cBc14f9669B88837ca1490cCa17c31607` | USDC |
| Base | 8453 | `0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913` | USDC |

### Bridged vs Native Tokens

**Native USDC:**
- Issued directly by Circle
- Example: Ethereum mainnet

**Bridged USDC:**
- Wrapped version from bridges
- Example: USDC.e on Polygon (bridged from Ethereum)

**Symbol Variants:**
- `USDC` - Native
- `USDC.e` - Bridged (Ethereum-bridged)
- `USDbC` - Base-bridged USDC

---

## 8. Token Metadata Retrieval

### ERC-20 Standard Methods

**Get Token Info:**
```solidity
function name() external view returns (string);
function symbol() external view returns (string);
function decimals() external view returns (uint8);
function totalSupply() external view returns (uint256);
```

### Via Smart Contract Call

```rust
use alloy::contract::SolCall;

// Define ERC-20 interface
#[derive(SolInterface)]
interface IERC20 {
    function symbol() external view returns (string memory);
    function decimals() external view returns (uint8);
}

// Call contract
let symbol = IERC20::symbolCall::new(())
    .call(provider, token_address)
    .await?;
```

### Via Subgraph

```graphql
{
  token(id: "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48") {
    id
    symbol
    name
    decimals
    totalSupply
  }
}
```

**Response:**
```json
{
  "data": {
    "token": {
      "id": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
      "symbol": "USDC",
      "name": "USD Coin",
      "decimals": "6",
      "totalSupply": "32000000000000000"
    }
  }
}
```

### Via Trading API

```bash
GET /swappable_tokens
```

**Response:**
```json
{
  "tokens": [
    {
      "chainId": 1,
      "address": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
      "symbol": "USDC",
      "name": "USD Coin",
      "decimals": 6,
      "logoURI": "https://..."
    }
  ]
}
```

---

## 9. Symbol Normalization for Connector

### Storage Format

**Recommended structure:**
```rust
pub struct TokenInfo {
    pub address: Address,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub chain_id: u64,
}

pub struct TradingPair {
    pub token0: TokenInfo,
    pub token1: TokenInfo,
    pub fee_tier: u32,
    pub pool_address: Address,
}
```

### Symbol to Address Mapping

**Build lookup table:**
```rust
use std::collections::HashMap;

lazy_static! {
    static ref SYMBOL_TO_ADDRESS: HashMap<&'static str, Address> = {
        let mut m = HashMap::new();
        m.insert("WETH", "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".parse().unwrap());
        m.insert("USDC", "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".parse().unwrap());
        m.insert("USDT", "0xdAC17F958D2ee523a2206206994597C13D831ec7".parse().unwrap());
        m.insert("DAI", "0x6B175474E89094C44Da98b954EedeAC495271d0F".parse().unwrap());
        // ... more tokens
        m
    };
}

fn symbol_to_address(symbol: &str) -> Option<Address> {
    SYMBOL_TO_ADDRESS.get(symbol).copied()
}
```

### User Input Handling

**Accept both formats:**
```rust
fn parse_token_identifier(input: &str) -> Result<Address> {
    // Try parsing as address first
    if let Ok(addr) = input.parse::<Address>() {
        return Ok(addr);
    }

    // Try as symbol
    if let Some(addr) = symbol_to_address(input) {
        return Ok(addr);
    }

    Err(Error::InvalidToken(input.to_string()))
}
```

**Example:**
```rust
// Both should work
parse_token_identifier("WETH")?;  // → 0xC02aaA3...
parse_token_identifier("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2")?;  // → 0xC02aaA3...
```

---

## 10. Pair Formatting Examples

### CEX Format (NOT used by Uniswap)
```
BTC/USDT
ETH-USD
btcusdt
```

### Uniswap Format

**API Request:**
```json
{
  "tokenIn": "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
  "tokenOut": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
}
```

**Contract Call:**
```rust
router.exactInputSingle(ExactInputSingleParams {
    tokenIn: Address::from_str("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2")?,
    tokenOut: Address::from_str("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")?,
    fee: 500,
    // ...
})
```

**Subgraph Query:**
```graphql
{
  pool(id: "0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640") {
    token0 { symbol }
    token1 { symbol }
    # Result: USDC/WETH
  }
}
```

### Human-Readable Display

**Format for UI:**
```rust
fn format_pair(token0: &TokenInfo, token1: &TokenInfo, fee: u32) -> String {
    let fee_pct = fee as f64 / 10000.0;
    format!("{}/{} ({:.2}%)", token0.symbol, token1.symbol, fee_pct)
}

// Example output: "USDC/WETH (0.05%)"
```

---

## Summary

| Concept | Format | Example |
|---------|--------|---------|
| Token ID | Address (checksummed) | `0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2` |
| Symbol | String (metadata) | `WETH` |
| Pair | token0 + token1 + fee | `USDC/WETH 500` |
| Pool | Address (computed) | `0x88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640` |
| Amount | uint256 (smallest unit) | `1000000000000000000` (1 ETH) |

**Key Differences from CEX:**
1. Use addresses, not symbols
2. Multiple pools per pair (different fees)
3. Native ETH must be wrapped to WETH
4. Addresses differ across chains
5. No centralized "symbol" registry
