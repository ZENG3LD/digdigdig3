# HyperLiquid Authentication

HyperLiquid uses **EIP-712 typed data signing** for authentication, with wallet-based signatures instead of traditional API keys.

---

## Authentication Overview

### Core Concept
- **No API keys**: Uses Ethereum wallet private keys
- **EIP-712 signatures**: Structured data signing standard
- **Two signing schemes**: L1 actions vs User-signed actions
- **Agent wallets**: Delegated signing authorities

### Critical Recommendation
**From official documentation**: "It is recommended to use an existing SDK instead of manually generating signatures."

Manual signature implementation is extremely error-prone due to:
- Complex field ordering requirements
- Msgpack serialization nuances
- Numeric precision requirements
- Address formatting rules
- Phantom agent construction

---

## Two Signing Schemes

### 1. L1 Actions (Phantom Agent)

**Used for**: Trading operations
- Place order
- Cancel order
- Modify order
- Update leverage
- Update isolated margin
- USD class transfer (spot↔perp)

**Characteristics**:
- Uses msgpack serialization
- Phantom agent construction
- Field ordering is critical
- No trailing zeros in numeric values

**Signature Type**: `sign_l1_action`

---

### 2. User-Signed Actions (Direct EIP-712)

**Used for**: Administrative operations
- Withdraw to L1
- Internal USDC transfer (`usdSend`)
- Internal spot token transfer (`spotSend`)
- Approve agent wallet
- Agent registration

**Characteristics**:
- Direct EIP-712 typed data
- JSON structure
- Requires `signatureChainId` field
- Requires `hyperliquidChain` field

**Signature Type**: `sign_user_signed_action`

---

## EIP-712 Domain Parameters

Based on official Python SDK implementation:

```python
domain = {
    "name": "Exchange",
    "version": "1",
    "chainId": 421614,  # Arbitrum Sepolia testnet
    "verifyingContract": "0x0000000000000000000000000000000000000000"
}
```

**Mainnet**: Uses Arbitrum One chain ID (`42161`)
**Testnet**: Uses Arbitrum Sepolia chain ID (`421614`)

**Note**: The verifying contract is zero address for HyperLiquid L1 operations.

---

## Nonce Requirements

### Validation Rules
- Must be larger than smallest nonce currently stored (100 most recent)
- Must never have been used before by that signer
- Must fall within time window: **(T - 2 days, T + 1 day)**
  - T = unix millisecond timestamp on the transaction's block

### Recommended Practice
Use current timestamp in milliseconds:
```rust
let nonce = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_millis() as u64;
```

### Nonce Storage
- Stored per signer (not per account)
- User address if signed with user's private key
- Agent address if signed with API wallet

### Important
HyperLiquid stores the **100 highest nonces** per address, allowing flexible transaction ordering (unlike Ethereum's sequential nonces).

---

## API Wallets (Agent Wallets)

### Overview
Agent wallets are authorization mechanisms where a master account approves them to sign transactions on behalf of the master account or sub-accounts.

### Key Features
- Separate wallet per trading process (recommended)
- Can operate on master account or sub-accounts
- Independent nonce management
- Requires approval by master account

### Critical Pitfall
**From documentation**: "To query the account data associated with a master or sub-account, you must pass in the actual address of that account. A common pitfall is to use the agent wallet which leads to an empty result."

**Correct**:
```json
{
  "type": "clearinghouseState",
  "user": "0x<MASTER_ACCOUNT_ADDRESS>",
  "dex": ""
}
```

**Wrong**:
```json
{
  "type": "clearinghouseState",
  "user": "0x<AGENT_WALLET_ADDRESS>",  // Returns empty!
  "dex": ""
}
```

### Agent Wallet Pruning

Agent wallets may be pruned when:
1. Wallet is deregistered (new unnamed agent or name collision)
2. Wallet expires
3. Registering account loses all funds

**Critical Warning**: "It is **strongly** suggested to not reuse agent addresses. Once an agent is deregistered, its used nonce state may be pruned."

### Best Practice
Generate separate API wallet per trading process to prevent nonce collisions.

---

## Request Signature Structure

### Standard Request Format
```json
{
  "action": {
    "type": "order",
    // ... action-specific fields
  },
  "nonce": 1704067200000,
  "signature": {
    "r": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
    "s": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
    "v": 27
  },
  "vaultAddress": null  // Optional: for sub-account/vault operations
}
```

### Signature Components
- **r**: First 32 bytes of signature (hex)
- **s**: Last 32 bytes of signature (hex)
- **v**: Recovery ID (27 or 28)

---

## User-Signed Action Details

### Required Fields
```json
{
  "action": {
    "type": "usdSend" | "spotSend" | "withdraw3",
    "hyperliquidChain": "Mainnet" | "Testnet",
    "signatureChainId": "0xa4b1",  // Hex format
    "time": 1704067200000,         // Current timestamp
    // ... type-specific fields
  },
  "nonce": 1704067200000,
  "signature": {...}
}
```

### Chain IDs
- **Arbitrum One (Mainnet)**: `0xa4b1` (decimal: 42161)
- **Arbitrum Sepolia (Testnet)**: `0x66eee` (decimal: 421614)

### Example: Withdraw Action
```json
{
  "action": {
    "type": "withdraw3",
    "hyperliquidChain": "Mainnet",
    "signatureChainId": "0xa4b1",
    "destination": "0xabcdef1234567890abcdef1234567890abcdef12",
    "amount": "1000.0",
    "time": 1704067200000
  },
  "nonce": 1704067200000,
  "signature": {
    "r": "0x...",
    "s": "0x...",
    "v": 27
  }
}
```

---

## L1 Action Signing (Phantom Agent)

### Concept
L1 actions use a "phantom agent" construction:
1. Create connection ID from nonce
2. Construct phantom agent message
3. Sign with EIP-712
4. Include signature in request

### Phantom Agent Construction
The phantom agent allows batch requests while maintaining signature security.

**Important**: Msgpack field ordering must be exact. Fields must appear in specific order.

### Common Errors with L1 Actions
1. **Trailing zeros**: Numeric values must not have trailing zeros
2. **Field order**: Msgpack fields must be in exact order
3. **Address format**: Must be lowercase before signing
4. **Price precision**: Must match tick size exactly

---

## Signature Validation Issues

### Unhelpful Error Messages
Incorrect signatures result in:
- `"User or API Wallet 0x0123... does not exist"`
- Deposit requirement messages
- Wrong recovered signer address

**Why**: The signature recovers to a different address, which doesn't exist on HyperLiquid.

### Debugging Strategy
From official documentation:
1. Use existing SDK as reference
2. Add logging to identify divergence points
3. Compare serialized payloads byte-by-byte
4. Do NOT attempt manual construction without deep EIP-712 knowledge

---

## Address Formatting Rules

### Critical Requirement
**Addresses must be lowercase before signing**

**Correct**:
```rust
let address = "0xabcdef1234567890abcdef1234567890abcdef12"; // lowercase
```

**Wrong**:
```rust
let address = "0xAbCdEf1234567890AbCdEf1234567890AbCdEf12"; // mixed case
```

### Impact
Inconsistent casing breaks signature recovery, causing authentication failures.

---

## Signature Examples (Conceptual)

### Signing Flow (Rust)
```rust
use ethers::signers::{LocalWallet, Signer};
use ethers::types::transaction::eip712::Eip712;

// 1. Create wallet from private key
let wallet = "0x<private_key>".parse::<LocalWallet>()?;

// 2. Construct EIP-712 typed data
let typed_data = Eip712 {
    domain: domain_separator(),
    types: action_types(),
    primary_type: "Action".to_string(),
    message: action_message(),
};

// 3. Sign typed data
let signature = wallet.sign_typed_data(&typed_data).await?;

// 4. Extract r, s, v
let r = hex::encode(&signature.r);
let s = hex::encode(&signature.s);
let v = signature.v;
```

**Note**: This is conceptual. Actual implementation requires exact type structures from SDK.

---

## Multi-Process Scenarios

### Problem
Multiple processes using same account can cause nonce collisions.

### Solution 1: Separate API Wallets
```
Process A → API Wallet A → Master Account
Process B → API Wallet B → Master Account
```
Each API wallet maintains independent nonce sequence.

### Solution 2: Centralized Nonce Management
- Single atomic counter service
- All processes request nonces from central service
- Adds latency and complexity

**Recommendation**: Use Solution 1 (separate API wallets)

---

## Batching and Nonces

### Batched Requests
Single batch = one nonce, multiple operations:
```json
{
  "action": {
    "type": "order",
    "orders": [
      {...},  // Order 1
      {...},  // Order 2
      {...}   // Order 3
    ]
  },
  "nonce": 1704067200000,  // One nonce for entire batch
  "signature": {...}
}
```

### Rate Limit Impact
- **IP limit**: Batch counts as 1 + floor(batch_length / 40)
- **Address limit**: Each order/cancel counts separately

---

## Implementation Recommendations

### For Rust Implementation

1. **Use ethers-rs for signing**:
   ```toml
   [dependencies]
   ethers = "2.0"
   ```

2. **Reference official Python SDK**:
   - https://github.com/hyperliquid-dex/hyperliquid-python-sdk
   - Study `sign_l1_action` and `sign_user_signed_action` methods
   - Compare type definitions

3. **Start with user-signed actions**:
   - Simpler than L1 actions
   - Standard EIP-712 (no phantom agent)
   - Test with `usdSend` between your accounts

4. **Add comprehensive logging**:
   - Log serialized data before signing
   - Log recovered signer address
   - Compare with expected address

5. **Implement nonce management**:
   ```rust
   use std::sync::atomic::{AtomicU64, Ordering};

   static NONCE_COUNTER: AtomicU64 = AtomicU64::new(0);

   fn get_next_nonce() -> u64 {
       let now = current_timestamp_ms();
       NONCE_COUNTER.fetch_max(now, Ordering::SeqCst);
       NONCE_COUNTER.fetch_add(1, Ordering::SeqCst)
   }
   ```

---

## Security Considerations

### Private Key Storage
- Never commit private keys to source control
- Use environment variables or secure key management
- Consider hardware wallet for production

### Agent Wallet Lifecycle
- Generate new agent per deployment
- Never reuse deregistered agents
- Monitor agent expiration

### Signature Verification
- Always verify recovered address matches expected
- Log signature failures for debugging
- Implement retry with new nonce on failure

---

## Testing Strategy

### 1. Test on Testnet First
- URL: `https://api.hyperliquid-testnet.xyz`
- Get testnet funds from faucet
- Test all signature types

### 2. Start Simple
```
✓ User-signed actions (usdSend)
✓ Simple limit orders (L1 action)
✓ Batch orders
✓ Complex orders (triggers, TP/SL)
```

### 3. Verify Each Component
- [ ] Nonce generation within valid range
- [ ] Address lowercase conversion
- [ ] EIP-712 domain parameters
- [ ] Type structure matches SDK
- [ ] Signature r, s, v extraction
- [ ] Correct action type for signing scheme

---

## Common Signature Errors

| Error Message | Likely Cause | Solution |
|---------------|--------------|----------|
| "User 0x... does not exist" | Wrong signature, recovered wrong address | Check address lowercase, verify EIP-712 types |
| "Please deposit" | Signature recovered to unfunded address | Verify signature, check wallet has funds |
| "Invalid nonce" | Nonce outside valid range | Use current timestamp ± 1 day |
| "Order already exists" | Nonce reused | Implement atomic nonce counter |
| Empty response | Using agent wallet for queries | Use master/subaccount address for queries |

---

## Reference Implementation

### Python SDK Reference
- **Repository**: https://github.com/hyperliquid-dex/hyperliquid-python-sdk
- **Key Files**:
  - `hyperliquid/utils/signing.py` - Signature construction
  - `hyperliquid/utils/types.py` - Type definitions
  - `hyperliquid/api/exchange.py` - Exchange endpoint usage

### TypeScript Reference
- **Repository**: https://github.com/nomeida/hyperliquid
- Alternative implementation for cross-reference

### Rust Crates Needed
```toml
[dependencies]
ethers = "2.0"              # EIP-712 signing
hex = "0.4"                 # Hex encoding/decoding
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"          # JSON serialization
rmp-serde = "1.1"           # Msgpack for L1 actions
sha2 = "0.10"               # Hashing
```

---

## Summary

### Key Takeaways
1. **Use SDK as reference**: Don't implement from scratch
2. **Two signing schemes**: L1 actions vs user-signed actions
3. **Nonce management**: Millisecond timestamp, 100-nonce window
4. **Address format**: Always lowercase before signing
5. **Agent wallets**: Separate per process, don't reuse
6. **Testing**: Start with testnet and simple operations

### Implementation Order
1. Set up EIP-712 domain and basic signing
2. Implement user-signed actions (simpler)
3. Test with internal transfers
4. Implement L1 action signing (phantom agent)
5. Test with simple orders
6. Add batch order support
7. Implement agent wallet management

### Critical Success Factors
- Reference official SDK implementation
- Test incrementally on testnet
- Log extensively for debugging
- Verify recovered signer address
- Never reuse agent wallets
