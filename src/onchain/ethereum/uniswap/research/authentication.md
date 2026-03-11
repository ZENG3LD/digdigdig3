# Uniswap Authentication Methods

## Overview

Uniswap uses different authentication methods depending on the API type:

1. **REST API (Trading API)**: API key authentication
2. **GraphQL (The Graph)**: API key in URL path
3. **On-Chain Contracts**: Wallet signature (transaction signing)
4. **WebSocket**: Ethereum node authentication

---

## 1. Trading API Authentication

### API Key Header Method

**Required Header:**
```http
x-api-key: <YOUR_API_KEY>
```

### Example Request

```bash
curl -X POST 'https://trade-api.gateway.uniswap.org/v1/quote' \
  -H 'Content-Type: application/json' \
  -H 'x-api-key: YOUR_API_KEY_HERE' \
  -d '{
    "type": "EXACT_INPUT",
    "amount": "1000000000000000000",
    "tokenInChainId": 1,
    "tokenOutChainId": 1,
    "tokenIn": "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
    "tokenOut": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
    "swapper": "0x..."
  }'
```

### Obtaining API Keys

**Source:** [Uniswap Developer Portal](https://api-docs.uniswap.org/)

**Process:**
1. Visit the Uniswap Developer Portal
2. Create an account or sign in
3. Generate a new API key
4. Copy the key for use in requests

**Key Properties:**
- Keys are tied to billing accounts
- Multiple keys can be created per account
- Keys should be kept secret (server-side only)
- Beta environment keys available separately

---

## 2. The Graph Subgraph Authentication

### API Key in URL Path

**Endpoint Format:**
```
https://gateway.thegraph.com/api/<YOUR_API_KEY>/subgraphs/id/<SUBGRAPH_ID>
```

### Example Request

```bash
curl -X POST \
  'https://gateway.thegraph.com/api/YOUR_API_KEY_HERE/subgraphs/id/5zvR82QoaXYFyDEKLZ9t6v9adgnptxYpKpSbxtgVENFV' \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "{ pools(first: 5) { id token0 { symbol } token1 { symbol } } }"
  }'
```

### Obtaining The Graph API Keys

**Source:** [The Graph Studio](https://thegraph.com/studio/apikeys/)

**Process:**
1. Visit The Graph Studio
2. Connect your wallet
3. Create a new API key
4. Set optional spending limits (monthly budget in USD)
5. Copy the key and insert into endpoint URLs

**Key Features:**
- Billing per calendar month
- Optional spending limits
- Multiple keys per account
- Usage tracking in Studio dashboard

---

## 3. Smart Contract Authentication (On-Chain)

### Wallet Signature Required

Smart contract interactions require transaction signing with a private key.

### Connection Methods

#### A. Web3 Provider (ethers.js)

```javascript
import { ethers } from 'ethers';

// Connect to Ethereum node
const provider = new ethers.JsonRpcProvider('https://mainnet.infura.io/v3/YOUR_INFURA_KEY');

// Create wallet instance
const wallet = new ethers.Wallet('PRIVATE_KEY', provider);

// Interact with contract
const routerAddress = '0xE592427A0AEce92De3Edee1F18E0157C05861564';
const router = new ethers.Contract(routerAddress, ABI, wallet);

// Execute swap (automatically signs transaction)
const tx = await router.exactInputSingle({
  tokenIn: '0x...',
  tokenOut: '0x...',
  fee: 3000,
  recipient: wallet.address,
  deadline: Math.floor(Date.now() / 1000) + 60 * 20,
  amountIn: ethers.parseEther('1.0'),
  amountOutMinimum: 0,
  sqrtPriceLimitX96: 0
});

await tx.wait();
```

#### B. Rust (alloy/ethers-rs)

```rust
use alloy::primitives::{Address, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::signers::local::PrivateKeySigner;

// Create provider with signer
let signer = PrivateKeySigner::from_bytes(&private_key)?;
let provider = ProviderBuilder::new()
    .with_recommended_fillers()
    .signer(signer)
    .on_http(rpc_url);

// Build and send transaction
let tx = contract.exactInputSingle(params).send().await?;
let receipt = tx.get_receipt().await?;
```

### Transaction Signing Process

1. **Build transaction data** (method + parameters encoded)
2. **Estimate gas** (optional but recommended)
3. **Sign transaction** with private key
4. **Broadcast** to network via RPC
5. **Wait for confirmation**

### Required Parameters

```rust
struct TransactionRequest {
    from: Address,
    to: Address,
    data: Vec<u8>,         // Encoded function call
    value: U256,           // ETH amount (0 for token swaps)
    gas_limit: U256,
    max_fee_per_gas: U256,
    max_priority_fee_per_gas: U256,
    nonce: u64,
    chain_id: u64,
}
```

---

## 4. Permit2 Authentication (ERC-2612)

### Overview

Permit2 allows token approvals via off-chain signatures instead of on-chain transactions.

### Signature Structure

**EIP-712 Typed Data:**
```json
{
  "domain": {
    "name": "Permit2",
    "chainId": 1,
    "verifyingContract": "0x000000000022D473030F116dDEE9F6B43aC78BA3"
  },
  "types": {
    "PermitSingle": [
      { "name": "details", "type": "PermitDetails" },
      { "name": "spender", "type": "address" },
      { "name": "sigDeadline", "type": "uint256" }
    ],
    "PermitDetails": [
      { "name": "token", "type": "address" },
      { "name": "amount", "type": "uint160" },
      { "name": "expiration", "type": "uint48" },
      { "name": "nonce", "type": "uint48" }
    ]
  },
  "message": {
    "details": {
      "token": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
      "amount": "1461501637330902918203684832716283019655932542975",
      "expiration": 1735689600,
      "nonce": 0
    },
    "spender": "0x...",
    "sigDeadline": 1735689600
  }
}
```

### Request Permit2 Signature

When calling `/quote` or `/swap`, include:
```json
{
  "generatePermitAsTransaction": false
}
```

The response will include `permitData` for wallet signing.

### Signing Process

```javascript
// Using ethers.js v6
const signature = await wallet.signTypedData(
  permitData.domain,
  permitData.types,
  permitData.values
);
```

```rust
// Using alloy
let signature = wallet.sign_typed_data(&permit_data).await?;
```

### Benefits

- **Gas savings**: No separate approval transaction
- **Better UX**: Single-step swaps
- **Security**: Time-limited approvals (30 days default)

---

## 5. WebSocket Authentication (For Event Monitoring)

### Ethereum Node Connection

WebSocket connections to Ethereum nodes require node-specific authentication.

#### A. Infura

```bash
wss://mainnet.infura.io/ws/v3/YOUR_INFURA_PROJECT_ID
```

**Authentication:** Project ID in URL

#### B. Alchemy

```bash
wss://eth-mainnet.g.alchemy.com/v2/YOUR_ALCHEMY_API_KEY
```

**Authentication:** API key in URL

#### C. Chainstack

```bash
wss://nd-123-456-789.p2pify.com/YOUR_CHAINSTACK_API_KEY
```

**Authentication:** API key in URL

### Example WebSocket Subscription

```javascript
import { WebSocket } from 'ws';

const ws = new WebSocket('wss://mainnet.infura.io/ws/v3/YOUR_PROJECT_ID');

ws.on('open', () => {
  // Subscribe to new block headers
  ws.send(JSON.stringify({
    jsonrpc: '2.0',
    id: 1,
    method: 'eth_subscribe',
    params: ['newHeads']
  }));
});

ws.on('message', (data) => {
  const response = JSON.parse(data);
  console.log('New block:', response);
});
```

### No Additional Auth Headers

WebSocket connections to Ethereum nodes don't use HTTP headers. All authentication is in the URL.

---

## 6. No Authentication Methods

### Public Read-Only Operations

Some operations don't require authentication:

1. **Public RPC calls** (via public nodes)
   - View functions (no state changes)
   - Reading blockchain data
   - May have rate limits

2. **Uniswap Info** (analytics website)
   - https://info.uniswap.org
   - Public pool/token data
   - No API key needed (uses subgraph)

### Example Public RPC Call

```bash
curl -X POST https://cloudflare-eth.com \
  -H 'Content-Type: application/json' \
  -d '{
    "jsonrpc": "2.0",
    "method": "eth_call",
    "params": [{
      "to": "0x...",
      "data": "0x..."
    }, "latest"],
    "id": 1
  }'
```

**Limitations:**
- Strict rate limits
- No guaranteed uptime
- Slower response times
- May block certain methods

---

## 7. Security Best Practices

### API Key Management

1. **Never commit keys to version control**
   ```bash
   # Add to .gitignore
   .env
   config.toml
   secrets.json
   ```

2. **Use environment variables**
   ```rust
   let api_key = std::env::var("UNISWAP_API_KEY")?;
   ```

3. **Rotate keys regularly**
   - Create new key
   - Update application
   - Delete old key

4. **Separate keys by environment**
   - Development key
   - Staging key
   - Production key
   - Testing/Beta key

### Private Key Security

1. **Never expose private keys**
   - Keep server-side only
   - Use hardware wallets for manual operations
   - Consider key management services (AWS KMS, HashiCorp Vault)

2. **Use separate wallets**
   - Trading wallet (hot)
   - Storage wallet (cold)
   - Testing wallet (testnet only)

3. **Monitor wallet activity**
   - Set up alerts for large transactions
   - Regular balance checks
   - Transaction history audits

### Request Security

1. **Use HTTPS only**
   ```rust
   // Good
   "https://trade-api.gateway.uniswap.org"

   // Bad - will be rejected
   "http://trade-api.gateway.uniswap.org"
   ```

2. **Validate SSL certificates**
   ```rust
   let client = reqwest::Client::builder()
       .danger_accept_invalid_certs(false)  // Always verify certs
       .build()?;
   ```

3. **Implement retry logic with backoff**
   ```rust
   let mut attempts = 0;
   loop {
       match make_request().await {
           Ok(response) => return Ok(response),
           Err(_) if attempts < 3 => {
               attempts += 1;
               tokio::time::sleep(Duration::from_secs(2u64.pow(attempts))).await;
           }
           Err(e) => return Err(e),
       }
   }
   ```

---

## 8. Error Handling

### Authentication Errors

**401 Unauthorized**
```json
{
  "error": "Invalid API key",
  "message": "The provided API key is not valid or has been revoked"
}
```

**Causes:**
- Missing `x-api-key` header
- Invalid API key
- Expired or revoked key
- Wrong API key for environment (prod vs beta)

**Solutions:**
- Check header is present
- Verify key from dashboard
- Generate new key if expired

---

**403 Forbidden**
```json
{
  "error": "Access denied",
  "message": "This API key does not have access to the requested resource"
}
```

**Causes:**
- API key lacks permissions
- IP restrictions
- Account suspended

---

**429 Too Many Requests**
```json
{
  "error": "Rate limit exceeded",
  "message": "You have exceeded the rate limit for this API key"
}
```

**Solutions:**
- Implement request queuing
- Add delays between requests
- Contact support for higher limits

---

## Summary

| Auth Method | API Type | Location | Format |
|-------------|----------|----------|--------|
| API Key (Header) | Trading API | `x-api-key` header | String |
| API Key (URL) | The Graph | URL path segment | String |
| Private Key | Smart Contracts | Transaction signing | 32 bytes hex |
| Permit2 Signature | Token approvals | EIP-712 typed data | Signature bytes |
| Node API Key | WebSocket | WSS URL | String |

**Key Takeaways:**
1. Trading API: Use `x-api-key` header
2. Subgraph: Embed key in URL
3. On-chain: Sign transactions with wallet private key
4. Keep all keys and private keys secure
5. Use separate keys for different environments
