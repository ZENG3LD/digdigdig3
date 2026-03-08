# Raydium API Authentication Research

**Research Date**: 2026-01-20

This document details authentication mechanisms for Raydium DEX. Unlike centralized exchanges, Raydium as a DEX operates fundamentally differently regarding authentication.

---

## Table of Contents

- [Critical Understanding: DEX vs CEX](#critical-understanding-dex-vs-cex)
- [API Authentication (None Required)](#api-authentication-none-required)
- [On-Chain Transaction Authentication](#on-chain-transaction-authentication)
- [Wallet Signature Process](#wallet-signature-process)
- [Comparison with CEX Authentication](#comparison-with-cex-authentication)

---

## Critical Understanding: DEX vs CEX

### Centralized Exchange (CEX) Model

**Examples**: Binance, KuCoin, Coinbase

**Authentication Flow**:
1. User creates account with exchange
2. Exchange generates API key + secret
3. User signs API requests with HMAC-SHA256
4. Exchange validates signature and executes on user's behalf
5. Exchange maintains custody of funds

**Key Point**: Exchange acts as **trusted intermediary** holding user funds.

### Decentralized Exchange (DEX) Model

**Example**: Raydium

**Authentication Flow**:
1. User owns Solana wallet (private key never shared)
2. User builds transaction locally
3. User signs transaction with ed25519 private key
4. User submits signed transaction to Solana blockchain
5. Raydium smart contract validates signature and executes
6. User maintains custody of funds

**Key Point**: User interacts **directly with blockchain**. No intermediary holds funds.

---

## API Authentication (None Required)

### REST API (api-v3.raydium.io)

**Authentication**: ❌ **NONE**

**Reason**: All API endpoints are **public and read-only**.

**What You Can Access Without Auth**:
- Pool data (liquidity, TVL, APR)
- Token lists and prices
- Farm information
- Historical data
- Quote calculations

**What You CANNOT Do Without Auth**:
- Nothing is restricted (all endpoints are public)

**No Headers Required**:
```http
GET /pools/info/list HTTP/1.1
Host: api-v3.raydium.io
Accept: application/json

# No API-KEY header
# No signature header
# No timestamp header
# No authentication whatsoever
```

**Response Format** (same for all endpoints):
```json
{
  "id": "unique-request-id",
  "success": true,
  "data": {
    // endpoint-specific data
  }
}
```

### Trade API (transaction-v1.raydium.io)

**Authentication**: ❌ **NONE**

**Reason**: Trade API only **serializes transactions**. It does NOT execute trades.

**No Auth Required Because**:
1. API just calculates quotes (mathematical computation)
2. API just builds unsigned transactions (data serialization)
3. API never touches user funds
4. Actual execution happens on-chain after user signature

**Example Request (No Auth)**:
```http
GET /compute/swap-base-in?inputMint=So11111...&outputMint=EPjF...&amount=1000000000&slippageBps=50&txVersion=V0 HTTP/1.1
Host: transaction-v1.raydium.io
Accept: application/json

# No authentication required
```

---

## On-Chain Transaction Authentication

### How Swaps Are Actually Authenticated

When executing a swap on Raydium:

**Step 1: Get Quote (No Auth)**
```typescript
const quote = await fetch('https://transaction-v1.raydium.io/compute/swap-base-in?' + params);
```

**Step 2: Serialize Transaction (No Auth)**
```typescript
const { transaction } = await fetch('https://transaction-v1.raydium.io/transaction/swap-base-in', {
  method: 'POST',
  body: JSON.stringify({
    swapResponse: quote.data,
    wallet: userPublicKey,
    // ...
  })
});
```

**Step 3: Sign Transaction (User's Private Key)**
```typescript
// This happens CLIENT-SIDE with user's wallet
const signedTx = await wallet.signTransaction(transaction);
```

**Step 4: Submit to Blockchain**
```typescript
// Submit to Solana RPC, NOT to Raydium API
const signature = await connection.sendRawTransaction(signedTx.serialize());
```

**Step 5: Blockchain Validates**
- Solana validators verify ed25519 signature
- Raydium program checks if signature matches transaction signer
- If valid, program executes swap
- If invalid, transaction reverts

### Signature Algorithm: ed25519

**Algorithm**: Ed25519 (EdDSA signature scheme)

**Not HMAC-SHA256 like CEX**:
- CEX uses HMAC-SHA256 with shared secret
- Solana uses Ed25519 with keypair (asymmetric cryptography)
- Public key is on-chain (address)
- Private key never leaves user's device

**Signature Process**:
```
signature = ed25519_sign(transaction_bytes, private_key)
verification = ed25519_verify(signature, transaction_bytes, public_key)
```

**Key Properties**:
- 64-byte signature
- 32-byte public key
- 32-byte private key (seed)
- Deterministic (same input → same signature)
- Fast verification on-chain

---

## Wallet Signature Process

### Wallet Types

**Software Wallets**:
- Phantom
- Solflare
- Backpack
- Sollet

**Hardware Wallets**:
- Ledger Nano S/X
- Trezor (with Solana support)

**Programmatic Wallets**:
- Keypair generated from seed phrase
- Used in SDK/bot implementations

### Transaction Signing Flow

**1. User Initiates Swap**:
```typescript
const swap = await raydium.swap({
  inputMint: "SOL",
  outputMint: "USDC",
  amount: 1000000000, // 1 SOL in lamports
  slippage: 0.5,
});
```

**2. SDK Builds Transaction**:
```typescript
const transaction = await swap.buildTransaction({
  feePayer: userPublicKey,
  recentBlockhash: blockhash,
});
```

**3. Wallet Signs Transaction**:
```typescript
// Browser wallet (Phantom, Solflare)
const signedTx = await window.solana.signTransaction(transaction);

// Or programmatic wallet
const keypair = Keypair.fromSecretKey(secretKeyBytes);
transaction.sign(keypair);
```

**4. Transaction Contains**:
- Instructions to Raydium program
- User's public key (signer)
- Ed25519 signature
- Recent blockhash (nonce)
- Fee payer designation

**5. Submit to Blockchain**:
```typescript
const signature = await connection.sendTransaction(signedTx);
await connection.confirmTransaction(signature);
```

### Signature Verification On-Chain

**Solana Validators Check**:
1. Is signature valid for this public key? (ed25519_verify)
2. Does transaction include this public key as signer?
3. Is blockhash recent (not expired)?
4. Does user have sufficient SOL for fees?
5. Does Raydium program validate the swap parameters?

**If all checks pass**: Transaction executes
**If any check fails**: Transaction reverts, fees still charged

---

## Comparison with CEX Authentication

### KuCoin/Binance (CEX) Authentication

**Endpoint**: `POST /api/v1/orders` (place order)

**Headers Required**:
```http
KC-API-KEY: your-api-key
KC-API-SIGN: base64(hmac_sha256(timestamp + method + endpoint + body, api_secret))
KC-API-TIMESTAMP: 1737379200000
KC-API-PASSPHRASE: encrypted_passphrase
KC-API-KEY-VERSION: 2
Content-Type: application/json
```

**Signature String**:
```
sign_string = timestamp + method + endpoint + body
signature = base64(HMAC-SHA256(sign_string, api_secret))
```

**Process**:
1. Exchange generates API key/secret pair
2. User stores secret locally
3. For each request, user computes HMAC signature
4. Exchange verifies signature with stored secret
5. Exchange executes order on user's behalf
6. Exchange maintains custody of funds

**Trust Model**: User trusts exchange to execute correctly and hold funds securely.

---

### Raydium (DEX) Authentication

**Endpoint**: N/A (no trading endpoints)

**Headers Required**: ❌ **NONE**

**Process**:
1. User generates Solana wallet keypair
2. User stores private key locally (or in hardware wallet)
3. User builds transaction with SDK
4. User signs transaction with ed25519 private key
5. User submits signed transaction to Solana blockchain
6. Raydium program validates signature on-chain
7. User maintains custody of funds throughout

**Trust Model**: User trusts only the blockchain and cryptography. No intermediary.

---

### Key Differences Summary

| Aspect | CEX (KuCoin) | DEX (Raydium) |
|--------|--------------|---------------|
| **Auth Method** | HMAC-SHA256 API signature | Ed25519 transaction signature |
| **Secret Type** | API secret (symmetric) | Private key (asymmetric) |
| **Who Signs** | User signs API request | User signs blockchain transaction |
| **Who Validates** | Exchange backend | Blockchain validators |
| **Who Executes** | Exchange on user's behalf | Smart contract autonomously |
| **Custody** | Exchange holds funds | User holds funds |
| **Trust Required** | Trust exchange | Trust only blockchain |
| **Revocation** | Revoke API key | Generate new keypair |
| **Passphrase** | Yes (additional layer) | No (seed phrase for wallet) |
| **Timestamp** | Required (prevents replay) | Blockhash (prevents replay) |
| **Headers** | 5-6 headers | None (data in transaction) |

---

## No API Keys Concept

### Why Raydium Doesn't Have API Keys

**1. No User Accounts**:
- Raydium doesn't store user data
- No email/password system
- No account management API

**2. No Authorization Needed**:
- All APIs are public read-only
- No sensitive user data to protect
- No rate limits tied to users

**3. On-Chain Authorization**:
- Authorization happens on Solana blockchain
- Wallet signature proves ownership
- Smart contract enforces rules

**4. Decentralization Philosophy**:
- API keys imply centralized control
- DEX aims to be permissionless
- Anyone can build apps using Raydium

### Accessing Raydium APIs

**No Registration Required**:
```typescript
// Just start calling APIs
const pools = await fetch('https://api-v3.raydium.io/pools/info/list');

// No API key needed
const quote = await fetch('https://transaction-v1.raydium.io/compute/swap-base-in?...');
```

**No SDK Authentication**:
```typescript
import { Raydium } from '@raydium-io/raydium-sdk-v2';

const raydium = await Raydium.load({
  cluster: 'mainnet',
  // No API key parameter exists
});

// All methods work immediately
const tokenList = await raydium.api.getTokenList();
```

---

## Rate Limiting (Without Authentication)

### How Rate Limits Work Without Auth

**Problem**: Without API keys, how does Raydium prevent abuse?

**Solution**: IP-based rate limiting (assumed, not documented)

**Typical Approach**:
1. Raydium servers track requests per IP address
2. If IP exceeds threshold, return 429 Too Many Requests
3. IP-based tracking is common for public APIs
4. No explicit documentation of limits

**Best Practices**:
1. Implement client-side caching
2. Avoid rapid polling (use gRPC for real-time)
3. Use batch endpoints (e.g., comma-separated IDs)
4. Respect 429 errors if received

**Third-Party RPC Providers**:
- Some providers offer authenticated RPC access
- Higher rate limits with API keys
- Examples: Chainstack, QuickNode, Triton One
- These are for Solana RPC, not Raydium-specific

---

## Implementation Notes for Connector

### What NOT to Implement

When building a Raydium connector (compared to KuCoin):

**Do NOT implement**:
- ❌ API key configuration
- ❌ API secret storage
- ❌ Passphrase encryption
- ❌ HMAC-SHA256 signature generation
- ❌ Timestamp header creation
- ❌ Authentication header builder
- ❌ Signature string formatting

**Why?** Because Raydium APIs require **zero authentication**.

### What TO Implement (If Building Trading Connector)

**For Read-Only Monitoring** (simplest):
```rust
// Just HTTP GET requests, no auth
pub async fn get_pool_list(&self) -> Result<Vec<Pool>> {
    let url = "https://api-v3.raydium.io/pools/info/list";
    let response = self.client.get(url).send().await?;
    // Parse JSON, no auth needed
}
```

**For Trading Execution** (complex, requires wallet integration):
```rust
use solana_sdk::{signature::Keypair, transaction::Transaction};

pub struct RaydiumTrader {
    // No API key/secret needed
    keypair: Keypair, // User's Solana wallet
    rpc_client: RpcClient, // Solana RPC connection
}

impl RaydiumTrader {
    pub async fn execute_swap(&self, input: &str, output: &str, amount: u64) -> Result<String> {
        // 1. Get quote (no auth)
        let quote = self.get_quote(input, output, amount).await?;

        // 2. Build transaction (no auth)
        let tx = self.build_transaction(quote).await?;

        // 3. Sign with wallet keypair (ed25519)
        let signed_tx = tx.sign(&[&self.keypair]);

        // 4. Submit to Solana blockchain
        let signature = self.rpc_client.send_transaction(&signed_tx)?;

        Ok(signature)
    }
}
```

**Key Dependencies**:
- `reqwest` for HTTP requests (no special auth headers)
- `solana-sdk` for wallet operations and transaction signing
- `ed25519-dalek` (included in solana-sdk) for signatures
- `borsh` or `bincode` for serialization

---

## Security Considerations

### Private Key Management

**Critical**: The private key IS the authentication.

**Best Practices**:
1. **Never hardcode** private keys in source code
2. **Use environment variables** or secure key management systems
3. **Hardware wallets** for high-value operations
4. **Key derivation** from seed phrases (BIP39)
5. **Separate keys** for testing (devnet) and production (mainnet)

**Example (Safe)**:
```rust
use solana_sdk::signature::Keypair;

// Load from environment or secure storage
let keypair = Keypair::from_bytes(&secret_key_bytes)?;

// Never log or expose
// Never include in error messages
// Never commit to git
```

### Transaction Security

**Validate Before Signing**:
```rust
// Always verify transaction details before signing
fn validate_swap_transaction(tx: &Transaction) -> Result<()> {
    // Check recipient address is Raydium program
    // Verify amounts are within expected range
    // Confirm slippage is acceptable
    // Ensure fee payer is correct
    Ok(())
}

// Only sign after validation
if validate_swap_transaction(&tx).is_ok() {
    let signed = tx.sign(&[&keypair]);
} else {
    return Err("Transaction validation failed");
}
```

### API Security (Public APIs)

**Even though no auth required**:
1. Use HTTPS (verify SSL certificates)
2. Validate JSON responses
3. Sanitize user inputs before building API URLs
4. Handle rate limiting gracefully
5. Don't trust API data blindly (verify on-chain if critical)

---

## Testing Without Production Funds

### Devnet Testing

**Devnet API**: `https://api-v3-devnet.raydium.io/`

**Devnet Wallet**:
```rust
use solana_sdk::signature::Keypair;

// Generate test keypair (never use in production)
let test_keypair = Keypair::new();

// Connect to devnet RPC
let rpc = RpcClient::new("https://api.devnet.solana.com");

// Request devnet SOL airdrop (test tokens)
rpc.request_airdrop(&test_keypair.pubkey(), 1_000_000_000)?;
```

**No Real Value**:
- Devnet tokens have no monetary value
- Safe for testing swap execution
- Reset wallet anytime without risk

---

## Summary

### Key Takeaways

1. **Raydium REST APIs require ZERO authentication**
   - No API keys
   - No signatures
   - No headers
   - Anyone can query

2. **Trading requires wallet signatures, not API auth**
   - User signs transactions with private key
   - Ed25519 algorithm (not HMAC)
   - Validation happens on-chain
   - No trusted intermediary

3. **DEX architecture is fundamentally different from CEX**
   - No account system
   - No custody of funds
   - No API-based trading execution
   - Smart contracts handle everything

4. **For a read-only connector**: No auth implementation needed
5. **For a trading connector**: Implement Solana wallet integration, not API authentication

---

## Sources

Research compiled from the following official sources:

- [Raydium API Documentation](https://docs.raydium.io/raydium/for-developers/api)
- [Raydium Trade API Documentation](https://docs.raydium.io/raydium/for-developers/trade-api)
- [Raydium SDK V2 GitHub](https://github.com/raydium-io/raydium-sdk-V2)
- [Solana Documentation - Transactions](https://docs.solana.com/developing/programming-model/transactions)
- [Solana Cookbook - Keypairs and Wallets](https://solanacookbook.com/references/keypairs-and-wallets.html)
- [Ed25519 Signature Scheme](https://ed25519.cr.yp.to/)

---

**Document Version**: 1.0
**Last Updated**: 2026-01-20
**Research Completed By**: Claude Code Research Agent
