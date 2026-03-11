# Vertex Protocol Authentication

Vertex Protocol uses EIP-712 typed structured data signing for authentication. All execute operations (place order, cancel, withdraw, etc.) require signed transactions using Ethereum wallet private keys.

## Overview

- **Standard**: EIP-712 (Ethereum Improvement Proposal 712)
- **Signature Type**: ECDSA with secp256k1 curve
- **Hash Function**: Keccak-256 (SHA3-256)
- **No API Keys**: Uses Ethereum wallet signatures instead

## EIP-712 Domain

### Domain Separator Fields

```json
{
  "name": "Vertex",
  "version": "0.0.1",
  "chainId": 42161,
  "verifyingContract": "0x..."
}
```

**Fields**:
- `name`: "Vertex" (constant)
- `version`: "0.0.1" (constant)
- `chainId`: Network chain ID
  - Arbitrum One: `42161`
  - Arbitrum Sepolia: `421613`
- `verifyingContract`: Address of Vertex endpoint contract

### Domain Type Hash

```solidity
EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)
```

## Subaccount (Sender) Format

Vertex uses a `bytes32` sender field that combines wallet address and subaccount identifier:

```
sender = address (20 bytes) + subaccount_name (12 bytes)
```

### Default Subaccount

```
Address: 0x7a5ec2748e9065794491a8d29dcf3f9edb8d7c43
Subaccount: "default" = 64656661756c740000000000 (hex encoded + zero padded)
Full sender: 0x7a5ec2748e9065794491a8d29dcf3f9edb8d7c43000000000000000000000000
```

### Custom Subaccount

```
Address: 0x7a5ec2748e9065794491a8d29dcf3f9edb8d7c43
Subaccount: "test0" = 74657374300000000000 (hex encoded + zero padded)
Full sender: 0x7a5ec2748e9065794491a8d29dcf3f9edb8d7c43746573743000000000000000
```

### Implementation (Rust)

```rust
fn convert_address_to_sender(address: &str, subaccount: &str) -> String {
    let addr_hex = address.strip_prefix("0x").unwrap_or(address);
    let subaccount_hex = hex::encode(subaccount);
    let padding = "0".repeat(24 - subaccount_hex.len());
    format!("0x{}{}{}", addr_hex, subaccount_hex, padding)
}
```

## EIP-712 Struct Definitions

### 1. Order (Place Order)

**Struct**:
```solidity
Order(bytes32 sender,int128 priceX18,int128 amount,uint64 expiration,uint64 nonce)
```

**Example Message**:
```json
{
  "sender": "0x7a5ec2748e9065794491a8d29dcf3f9edb8d7c43000000000000000000000000",
  "priceX18": "30000000000000000000000",
  "amount": "1000000000000000000",
  "expiration": 4611686018427387904,
  "nonce": 1234567890123
}
```

**Field Types**:
- `sender`: bytes(32)
- `priceX18`: int(128) - signed 128-bit integer
- `amount`: int(128) - positive for buy, negative for sell
- `expiration`: uint(64) - timestamp with TIF bits
- `nonce`: uint(64) - unique order ID

### 2. Cancellation (Cancel Orders)

**Struct**:
```solidity
Cancellation(bytes32 sender,uint32[] productIds,bytes32[] digests,uint64 nonce)
```

**Example Message**:
```json
{
  "sender": "0x7a5ec2748e9065794491a8d29dcf3f9edb8d7c43000000000000000000000000",
  "productIds": [2, 4],
  "digests": [
    "0x123abc...",
    "0x456def..."
  ],
  "nonce": 1234567890124
}
```

**Field Types**:
- `sender`: bytes(32)
- `productIds`: Array of uint(32)
- `digests`: Array of bytes(32) - order hashes
- `nonce`: uint(64)

### 3. CancellationProducts (Cancel Product Orders)

**Struct**:
```solidity
CancellationProducts(bytes32 sender,uint32[] productIds,uint64 nonce)
```

**Example Message**:
```json
{
  "sender": "0x7a5ec2748e9065794491a8d29dcf3f9edb8d7c43000000000000000000000000",
  "productIds": [2],
  "nonce": 1234567890125
}
```

**Field Types**:
- `sender`: bytes(32)
- `productIds`: Array of uint(32)
- `nonce`: uint(64)

### 4. WithdrawCollateral

**Struct**:
```solidity
WithdrawCollateral(bytes32 sender,uint32 productId,uint128 amount,uint64 nonce)
```

**Example Message**:
```json
{
  "sender": "0x7a5ec2748e9065794491a8d29dcf3f9edb8d7c43000000000000000000000000",
  "productId": 0,
  "amount": "10000000000000000000",
  "nonce": 1234567890126
}
```

**Field Types**:
- `sender`: bytes(32)
- `productId`: uint(32)
- `amount`: uint(128)
- `nonce`: uint(64)

## Signature Generation

### Process Flow

1. **Create Domain Separator**
   ```
   domainSeparator = keccak256(
     abi.encode(
       keccak256("EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"),
       keccak256("Vertex"),
       keccak256("0.0.1"),
       chainId,
       verifyingContract
     )
   )
   ```

2. **Hash Struct**
   ```
   structHash = keccak256(
     abi.encode(
       TYPE_HASH,
       ...message_fields
     )
   )
   ```

3. **Create Digest**
   ```
   digest = keccak256(
     abi.encodePacked(
       "\x19\x01",
       domainSeparator,
       structHash
     )
   )
   ```

4. **Sign Digest**
   ```
   signature = sign(digest, privateKey)
   ```

5. **Format Signature**
   ```
   signatureHex = "0x" + r (32 bytes) + s (32 bytes) + v (1 byte)
   ```

### Signature Components

- **r**: First 32 bytes of ECDSA signature
- **s**: Second 32 bytes of ECDSA signature
- **v**: Recovery ID (27 or 28) = `signature[64] + 27`

**Total Length**: 65 bytes (130 hex characters + "0x")

### Implementation (Rust)

```rust
use sha3::{Keccak256, Digest};
use secp256k1::{SecretKey, Message, Secp256k1};

fn sign_payload(
    message: &[u8],
    domain: &EIP712Domain,
    private_key: &str,
) -> Result<String, Error> {
    // 1. Create domain separator
    let domain_separator = create_domain_separator(domain);

    // 2. Hash struct
    let struct_hash = keccak256(message);

    // 3. Create digest
    let mut digest_data = Vec::new();
    digest_data.extend_from_slice(b"\x19\x01");
    digest_data.extend_from_slice(&domain_separator);
    digest_data.extend_from_slice(&struct_hash);
    let digest = keccak256(&digest_data);

    // 4. Sign
    let secp = Secp256k1::new();
    let secret_key = SecretKey::from_slice(
        &hex::decode(private_key.strip_prefix("0x").unwrap_or(private_key))?
    )?;
    let message = Message::from_slice(&digest)?;
    let signature = secp.sign_ecdsa_recoverable(&message, &secret_key);

    // 5. Format signature
    let (recovery_id, sig_bytes) = signature.serialize_compact();
    let v = recovery_id.to_i32() as u8 + 27;

    Ok(format!("0x{}{:02x}", hex::encode(sig_bytes), v))
}

fn keccak256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(data);
    hasher.finalize().into()
}
```

## Nonce Generation

The nonce must be unique for each operation. Common pattern:

```rust
fn generate_nonce() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let random = rand::random::<u32>() as u64;

    timestamp_ms * 1000 + random % 1000
}
```

## Expiration Generation

The expiration field combines timestamp and time-in-force flags:

```rust
fn generate_expiration(seconds_valid: u64, time_in_force: TimeInForce) -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let expiration = now + seconds_valid;

    match time_in_force {
        TimeInForce::GTC => expiration,
        TimeInForce::IOC => expiration | (1 << 62),
        TimeInForce::FOK => expiration | (1 << 63),
        TimeInForce::PostOnly => expiration | (1 << 62) | (1 << 63),
    }
}
```

**Time-in-Force Bits**:
- Bit 62: IOC flag
- Bit 63: FOK flag
- Bits 0-61: Unix timestamp

**Values**:
- GTC: `timestamp`
- IOC: `timestamp | 0x4000000000000000`
- FOK: `timestamp | 0x8000000000000000`
- POST_ONLY: `timestamp | 0xC000000000000000`

## Order Digest Calculation

Order digest is the hash used to identify orders:

```rust
fn generate_digest(order_message: &[u8]) -> String {
    let hash = keccak256(order_message);
    format!("0x{}", hex::encode(hash))
}
```

## WebSocket Authentication

For authenticated WebSocket subscriptions (OrderUpdate, Fill, PositionChange):

### Authentication Message

```json
{
  "method": "authenticate",
  "id": 0,
  "tx": {
    "sender": "0x7a5ec2748e9065794491a8d29dcf3f9edb8d7c43000000000000000000000000",
    "expiration": 1234567890000
  },
  "signature": "0x..."
}
```

**Fields**:
- `method`: "authenticate"
- `id`: Request ID (integer)
- `tx.sender`: bytes32 subaccount identifier
- `tx.expiration`: Expiration timestamp in milliseconds since Unix epoch
- `signature`: EIP-712 signature of the tx object

### Connection Limits

- **Max Connections per Wallet**: 5 WebSocket connections
- **Heartbeat Requirement**: Send ping frames every 30 seconds
- **Disconnection**: Connections exceeding limits are automatically dropped

## REST Authentication

REST requests do NOT use HTTP headers for authentication. Instead:

1. Build the execute payload (e.g., place_order)
2. Sign the transaction data using EIP-712
3. Include signature in the JSON payload
4. POST to `/execute` endpoint

**No Authorization Header Required**

## Referral Headers

Optional broker identification:

```http
POST /execute
Content-Type: application/json
Referer: vertex-hummingbot-1.0
```

## Security Considerations

### Private Key Storage

- Store private keys securely (environment variables, key management systems)
- Never log or expose private keys
- Use hardware wallets for production trading

### Signature Verification

Vertex verifies signatures by:

1. Extracting sender address from signature recovery
2. Comparing with sender field in the message
3. Checking signature validity against EIP-712 standard

### Nonce Uniqueness

- Use timestamp + random component to ensure uniqueness
- Track used nonces to avoid replay attacks
- Server rejects duplicate nonces

### Expiration Validation

- Server checks if current time < expiration timestamp
- Expired signatures are rejected
- Set reasonable expiration windows (e.g., 5 minutes)

## Example: Full Order Signing Flow

```rust
use vertex_v5::{VertexAuth, Order};

async fn place_order_example() -> Result<(), Error> {
    // 1. Setup
    let private_key = "0x..."; // Your wallet private key
    let address = "0x7a5ec2748e9065794491a8d29dcf3f9edb8d7c43";
    let subaccount = "default";
    let chain_id = 42161; // Arbitrum One
    let verifying_contract = "0x..."; // Vertex endpoint address

    // 2. Create sender
    let sender = convert_address_to_sender(address, subaccount);

    // 3. Build order
    let order = Order {
        sender: sender.clone(),
        price_x18: "30000000000000000000000".to_string(), // $30,000
        amount: "1000000000000000000".to_string(), // 1.0 BTC
        expiration: generate_expiration(300, TimeInForce::GTC), // 5 minutes
        nonce: generate_nonce(),
    };

    // 4. Create EIP-712 domain
    let domain = EIP712Domain {
        name: "Vertex".to_string(),
        version: "0.0.1".to_string(),
        chain_id,
        verifying_contract: verifying_contract.to_string(),
    };

    // 5. Encode struct
    let order_bytes = encode_order(&order);

    // 6. Sign
    let signature = sign_payload(&order_bytes, &domain, private_key)?;

    // 7. Build payload
    let payload = json!({
        "place_order": {
            "product_id": 2,
            "order": {
                "sender": order.sender,
                "priceX18": order.price_x18,
                "amount": order.amount,
                "expiration": order.expiration,
                "nonce": order.nonce,
            },
            "signature": signature,
        }
    });

    // 8. Send request
    let response = reqwest::Client::new()
        .post("https://gateway.prod.vertexprotocol.com/v1/execute")
        .json(&payload)
        .send()
        .await?;

    println!("Response: {:?}", response.json::<Value>().await?);

    Ok(())
}
```

## Resources

- [EIP-712 Specification](https://eips.ethereum.org/EIPS/eip-712)
- [Vertex Protocol Signing Examples](https://docs.vertexprotocol.com/developer-resources/api/gateway/signing/examples)
- [Ethereum Signature Standards](https://www.cyfrin.io/blog/understanding-ethereum-signature-standards-eip-191-eip-712)
