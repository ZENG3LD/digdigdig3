# Lighter Exchange Authentication

## Overview

Lighter uses a cryptographic signature-based authentication system. Transactions are signed client-side using ECDSA with the API key's private key before submission to the API.

---

## Authentication Methods

### 1. Transaction Signing (Write Operations)

All write operations (create order, cancel order, transfers, withdrawals) require cryptographic signatures.

**Signing Process**:
1. Generate API key pair (public/private key)
2. Sign transaction data with API key private key
3. Submit signed transaction via `/sendTx` or `/sendTxBatch`

**Key Components**:
- **API Key Private Key**: Used to sign transactions (keep secure, never share)
- **API Key Public Key**: Registered on-chain, used to verify signatures
- **Nonce**: Incremental counter per API key to prevent replay attacks
- **Signature**: ECDSA signature of transaction data

---

### 2. Auth Tokens (Read Operations)

For read-only access to authenticated endpoints and WebSocket channels.

**Standard Auth Token Structure**:
```
{expiry_unix}:{account_index}:{api_key_index}:{random_hex}
```

**Fields**:
- `expiry_unix`: Unix timestamp when token expires
- `account_index`: Account identifier
- `api_key_index`: API key index (3-254)
- `random_hex`: Random hexadecimal string for uniqueness

**Expiry Limits**:
- **Maximum**: 8 hours
- **Minimum**: Not specified

**Generation Methods**:
- Python SDK: `create_auth_token_with_expiry()` function
- GO SDK: Dedicated token creation methods

**Example**:
```
1640999999:1:3:a1b2c3d4e5f6
```

---

### 3. Read-Only Auth Tokens

For data access without transaction capabilities (safer for sharing).

**Structure**:
```
ro:{account_index}:{single|all}:{expiry_unix}:{random_hex}
```

**Fields**:
- `ro`: Read-only prefix
- `account_index`: Account identifier
- `single|all`: Scope - single account or all sub-accounts
- `expiry_unix`: Unix timestamp when token expires
- `random_hex`: Random hexadecimal string

**Expiry Limits**:
- **Maximum**: 10 years
- **Minimum**: 1 day

**Generation Methods**:
- `createToken` endpoint
- Frontend application

**Example**:
```
ro:1:single:1735689599:7f8e9d0c1b2a
```

---

## API Keys

### Key Management

**Index Allocation**:
- `0`: Reserved for desktop application
- `1`: Reserved for mobile PWA
- `2`: Reserved for mobile app
- `3-254`: Available for API keys (252 total)
- `255`: Special value for "all keys" in queries

**Per Account**:
- Each account (master or sub-account) has separate API key indexes
- Each API key has its own public/private key pair
- Each API key maintains its own nonce counter

**Key Structure**:
- **Public Key**: Registered on-chain for signature verification
- **Private Key**: Stored securely client-side, used for signing

---

### Creating API Keys

API keys are created through:
1. Frontend application (app.lighter.xyz)
2. SDK methods (if available)
3. On-chain transactions (TxTypeL2ChangePubKey = 8)

**Security Best Practices**:
- Store private keys securely (environment variables, secure vaults)
- Never commit private keys to version control
- Use separate API keys for different applications/purposes
- Rotate keys periodically
- Use read-only tokens when write access isn't needed

---

## Nonce Management

### Purpose
Nonces prevent replay attacks by ensuring each transaction is unique and processed in order.

### Requirements
- Each API key has its own nonce counter
- Nonce must increment by 1 with each transaction
- Nonce starts at 0 (or 1, verify in SDK)
- Out-of-order nonces will be rejected

### Getting Next Nonce

**Endpoint**: `GET /api/v1/nextNonce`

**Parameters**:
- `account_index` (required, int)
- `api_key_index` (required, int)

**Response**:
```json
{
  "code": 200,
  "message": "string",
  "nonce": 42
}
```

**SDK Handling**:
- Python SDK: Automatically manages nonces
- Manual implementation: Must track and increment nonces

**Important Notes**:
- Query next nonce before each transaction
- For batch transactions, nonces must be sequential
- Failed transactions still consume nonces
- Nonce gaps will block subsequent transactions

---

## Transaction Signing

### Signature Algorithm

**Type**: ECDSA (Elliptic Curve Digital Signature Algorithm)

**Curve**: Likely secp256k1 (standard for Ethereum-compatible systems)

### Transaction Data to Sign

Each transaction type has specific fields that must be signed:

**Common Fields** (all transactions):
- `account_index`
- `api_key_index`
- `nonce`
- `expire_at` (optional expiry timestamp)

**Order-Specific Fields** (CreateOrder):
- `market_id`
- `base_amount`
- `price`
- `side` (buy/sell)
- `order_type` (limit/market)
- `client_order_index`

**Cancel Order Fields**:
- `order_index` (must match `client_order_index` of order to cancel)

**Modify Order Fields**:
- `order_index`
- `new_base_amount` (optional)
- `new_price` (optional)

### Signing Process (Conceptual)

1. Construct transaction object with required fields
2. Serialize transaction data (specific format defined by Lighter)
3. Hash serialized data (likely Keccak256)
4. Sign hash with API key private key (ECDSA)
5. Encode signature (likely hex or base64)
6. Attach signature to transaction
7. Submit via `/sendTx` or `/sendTxBatch`

**Note**: Use official Lighter SDKs for proper signing implementation. The exact serialization format and hashing method are critical for valid signatures.

---

## SDK Usage Examples

### Python SDK

**Setup**:
```python
from lighter import ApiClient, SignerClient, TransactionApi
import os

# Initialize clients
api_client = ApiClient(base_url="https://mainnet.zklighter.elliot.ai")
signer_client = SignerClient(
    api_key_private_key=os.environ["API_KEY_PRIVATE_KEY"],
    api_key_index=3,
    account_index=1
)
transaction_api = TransactionApi(api_client)
```

**Create and Submit Order**:
```python
# Sign create order transaction
signed_tx = signer_client.sign_create_order(
    market_id=0,
    base_amount="1000000",  # As integer
    price="30246600",       # As integer
    side="buy",
    order_type="limit",
    client_order_index=12345,
    nonce=await transaction_api.next_nonce(
        account_index=1,
        api_key_index=3
    )
)

# Submit transaction
response = await transaction_api.send_tx(
    tx_type=14,  # L2CreateOrder
    tx_info=signed_tx
)
```

**Create Auth Token**:
```python
import time

# Create auth token with 1 hour expiry
auth_token = signer_client.create_auth_token_with_expiry(
    expiry=int(time.time()) + 3600  # 1 hour from now
)

# Use for authenticated WebSocket subscriptions
ws_client.subscribe(
    channel="account_market/0/1",
    auth=auth_token
)
```

**Cancel Order**:
```python
# Sign cancel order transaction
signed_cancel = signer_client.sign_cancel_order(
    order_index=12345,  # client_order_index of order to cancel
    nonce=await transaction_api.next_nonce(
        account_index=1,
        api_key_index=3
    )
)

# Submit cancel transaction
response = await transaction_api.send_tx(
    tx_type=15,  # L2CancelOrder
    tx_info=signed_cancel
)
```

---

### GO SDK

**Setup**:
```go
import (
    lighter "github.com/elliottech/lighter-go"
    "os"
)

// Initialize signer
signer := lighter.NewSigner(
    os.Getenv("API_KEY_PRIVATE_KEY"),
    3,  // api_key_index
    1,  // account_index
)
```

**Sign Transaction**:
```go
// Create order transaction
tx := signer.SignCreateOrder(
    marketId: 0,
    baseAmount: "1000000",
    price: "30246600",
    side: "buy",
    orderType: "limit",
    clientOrderIndex: 12345,
    nonce: nextNonce,
)

// Submit transaction
response := client.SendTx(14, tx)  // 14 = L2CreateOrder
```

---

## Authentication Flow Diagrams

### Write Operation Flow (Create Order)

```
Client                          API Server               Blockchain
  |                                 |                         |
  | 1. Get Next Nonce               |                         |
  |-------------------------------->|                         |
  |                                 |                         |
  | 2. Return Nonce=42              |                         |
  |<--------------------------------|                         |
  |                                 |                         |
  | 3. Sign Transaction             |                         |
  |    (with nonce=42)              |                         |
  |                                 |                         |
  | 4. POST /sendTx                 |                         |
  |    (signed transaction)         |                         |
  |-------------------------------->|                         |
  |                                 |                         |
  |                                 | 5. Verify Signature     |
  |                                 |    & Nonce              |
  |                                 |                         |
  |                                 | 6. Queue Transaction    |
  |                                 |------------------------>|
  |                                 |                         |
  | 7. Return tx_hash               |                         |
  |<--------------------------------|                         |
  |                                 |                         |
  |                                 | 8. Process Transaction  |
  |                                 |<------------------------|
```

### Read Operation Flow (with Auth Token)

```
Client                          API Server
  |                                 |
  | 1. Create Auth Token            |
  |    (with expiry)                |
  |                                 |
  | 2. GET /account                 |
  |    Header: Authorization: {token}
  |-------------------------------->|
  |                                 |
  |                                 | 3. Validate Token
  |                                 |    - Check expiry
  |                                 |    - Verify signature
  |                                 |    - Check permissions
  |                                 |
  | 4. Return Account Data          |
  |<--------------------------------|
```

---

## Security Considerations

### Private Key Security

1. **Never Expose**:
   - Don't hardcode in source code
   - Don't commit to version control
   - Don't log or display in plain text
   - Don't transmit unencrypted

2. **Storage**:
   - Use environment variables
   - Use secure key management systems (AWS KMS, HashiCorp Vault)
   - Encrypt at rest
   - Use hardware security modules (HSM) for production

3. **Access Control**:
   - Limit who can access private keys
   - Use separate keys for different environments (dev/staging/prod)
   - Implement key rotation policies

### Signature Verification

**Server-Side Checks**:
1. Signature matches public key
2. Public key belongs to claimed account
3. Nonce is next expected value
4. Transaction hasn't expired
5. Account has sufficient balance/permissions

### Auth Token Security

1. **Expiry**:
   - Use shortest practical expiry time
   - Standard tokens: max 8 hours
   - Read-only tokens: minimum 1 day

2. **Scope**:
   - Use read-only tokens when possible
   - Limit scope to specific accounts when possible
   - Don't share tokens across applications

3. **Transmission**:
   - Always use HTTPS/WSS
   - Include in headers, not URL parameters
   - Rotate tokens regularly

### Rate Limiting Impact

Failed authentication attempts count toward rate limits. Implement:
- Exponential backoff on failures
- Token caching to reduce re-authentication
- Proper error handling to avoid retry loops

---

## Common Authentication Errors

### Invalid Signature

**Error**: `400 Bad Request - Invalid signature`

**Causes**:
- Incorrect signing algorithm
- Wrong private key
- Corrupted signature data
- Incorrect transaction serialization

**Solutions**:
- Verify using correct API key private key
- Check SDK version compatibility
- Ensure proper data serialization

### Nonce Mismatch

**Error**: `400 Bad Request - Invalid nonce`

**Causes**:
- Nonce too low (already used)
- Nonce too high (gap in sequence)
- Concurrent transactions with same nonce
- Stale nonce from cache

**Solutions**:
- Query `/nextNonce` before each transaction
- Implement nonce management queue for concurrent operations
- Reset nonce tracking after failures

### Expired Token

**Error**: `401 Unauthorized - Token expired`

**Causes**:
- Token past expiry time
- System clock skew
- Token created with past timestamp

**Solutions**:
- Create new token with future expiry
- Synchronize system clock with NTP
- Implement token refresh before expiry

### Invalid API Key

**Error**: `401 Unauthorized - Invalid API key`

**Causes**:
- API key not registered on-chain
- Wrong api_key_index
- API key revoked or disabled

**Solutions**:
- Verify API key index is correct (3-254)
- Ensure API key was properly created on-chain
- Check account status and permissions

---

## Implementation Checklist

### For V5 Connector Implementation

- [ ] Implement transaction signing in `auth.rs`
  - [ ] ECDSA signing function
  - [ ] Transaction serialization
  - [ ] Signature encoding

- [ ] Implement nonce management
  - [ ] Query next nonce
  - [ ] Track nonce per API key
  - [ ] Handle nonce failures

- [ ] Implement auth token generation
  - [ ] Standard auth tokens
  - [ ] Read-only auth tokens
  - [ ] Token expiry management

- [ ] Secure credential storage
  - [ ] Load from environment variables
  - [ ] Support multiple API keys
  - [ ] Never log private keys

- [ ] Error handling
  - [ ] Parse authentication errors
  - [ ] Implement retry logic with backoff
  - [ ] Nonce recovery mechanisms

- [ ] Testing
  - [ ] Test signature generation
  - [ ] Test nonce sequencing
  - [ ] Test token expiry handling
  - [ ] Test error scenarios

---

## Reference Links

- [Lighter API Documentation](https://apidocs.lighter.xyz)
- [Lighter Python SDK](https://github.com/elliottech/lighter-python)
- [Lighter GO SDK](https://github.com/elliottech/lighter-go)
- [Get Started for Programmers](https://apidocs.lighter.xyz/docs/get-started-for-programmers-1)
- [API Keys Documentation](https://apidocs.lighter.xyz/docs/api-keys)
