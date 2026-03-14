# Lighter Go SDK — Transaction Signing Deep Dive

Source: https://github.com/elliottech/lighter-go (Apache-2.0 license)
Fetched: 2026-03-15

---

## Repository Structure

```
lighter-go/
├── signer/
│   └── key_manager.go        # ECgFp5 key management + Schnorr signing
├── types/
│   ├── tx_request.go         # Request structs + ConstructXxx functions
│   └── txtypes/
│       ├── constants.go      # All protocol constants (chain IDs, limits, enums)
│       ├── interface.go      # TxInfo interface + OrderInfo struct
│       ├── create_order.go   # L2CreateOrderTxInfo — Hash() packs 16 field elements
│       ├── cancel_order.go   # L2CancelOrderTxInfo — Hash() packs 8 field elements
│       ├── change_pub_key.go # L2ChangePubKeyTxInfo — includes L1 sig body
│       ├── tx_attributes.go  # L2TxAttributes — integrator fees, AggregateTxHash
│       ├── utils.go          # Message templates, hex helpers
│       └── ... (18 tx type files total)
├── client/
│   ├── client.go             # Global client registry, GenerateAPIKey
│   ├── tx_client.go          # TxClient struct, NewTxClient, FullFillDefaultOps
│   ├── tx_get.go             # GetXxxTransaction methods on TxClient
│   ├── interface.go          # MinimalHTTPClient interface
│   └── http/
│       ├── client.go         # HTTP transport (30s timeout, TLS 1.2+)
│       ├── requests.go       # GetNextNonce, GetApiKey — GET requests
│       └── http_types.go     # ResultCode, NextNonce, ApiKey, AccountApiKeys
├── sharedlib/
│   └── main.go               # C-exported functions (CGo sharedlib)
├── examples/
│   └── example.cpp           # C++ usage example
└── go.mod                    # poseidon_crypto v0.0.15, go-ethereum v1.15.6
```

---

## 1. Key Management (`signer/key_manager.go`)

### Types

```go
type Signer interface {
    Sign(message []byte, hFunc hash.Hash) ([]byte, error)
}

type KeyManager interface {
    Signer
    PubKey() gFp5.Element          // 5 Goldilocks field elements = 40 bytes
    PubKeyBytes() [40]byte
    PrvKeyBytes() []byte
}

type keyManager struct {
    key curve.ECgFp5Scalar         // Private key = ECgFp5 scalar
}
```

### Constructor

```go
// Private key must be EXACTLY 40 bytes (little-endian)
func NewKeyManager(b []byte) (KeyManager, error) {
    if len(b) != 40 {
        return nil, fmt.Errorf("invalid private key length. expected: 40 got: %v", len(b))
    }
    return &keyManager{key: curve.ScalarElementFromLittleEndianBytes(b)}, nil
}
```

### Signing

```go
func (key *keyManager) Sign(hashedMessage []byte, hFunc hash.Hash) ([]byte, error) {
    // 1. Parse the 40-byte Poseidon2 hash as a GFp5 element
    hashedMessageAsQuinticExtension, err := gFp5.FromCanonicalLittleEndianBytes(hashedMessage)
    // 2. Perform Schnorr signature over ECgFp5 curve
    return schnorr.SchnorrSignHashedMessage(hashedMessageAsQuinticExtension, key.key).ToBytes(), nil
}

func (key *keyManager) PubKey() gFp5.Element {
    return schnorr.SchnorrPkFromSk(key.key)
}
```

**Key facts:**
- Private key: 40 bytes, ECgFp5Scalar, little-endian
- Public key: 40 bytes, GFp5 element (quintic extension of Goldilocks), little-endian
- Signature algorithm: Schnorr over ECgFp5 curve
- The message passed to `Sign()` is the **already-hashed** Poseidon2 digest (40 bytes)
- `hFunc hash.Hash` parameter is accepted but the actual hashing is done **before** calling Sign — the hash used in calling code is `p2.NewPoseidon2()` (Poseidon2 over Goldilocks)

---

## 2. Chain ID Constant

From the C++ example (`examples/example.cpp`):

```cpp
CreateClient(nullptr, apiResp.privateKey, 304, apiKeyIndex, 100);
//                                         ^^^
//                                     chain_id = 304 (mainnet)
```

The chain ID is passed into every `Hash(lighterChainId uint32)` call. **Mainnet chain_id = 304**.

---

## 3. Transaction Type Constants (`types/txtypes/constants.go`)

```go
const (
    TxTypeL2CreateOrder       = 14
    TxTypeL2CancelOrder       = 15
    TxTypeL2CancelAllOrders   = 16
    TxTypeL2ModifyOrder       = 17
    TxTypeL2Transfer          = 12
    TxTypeL2Withdraw          = 13
    TxTypeL2ChangePubKey      = 8
    TxTypeL2CreateSubAccount  = 9
    TxTypeL2MintShares        = 18
    TxTypeL2BurnShares        = 19
    TxTypeL2UpdateLeverage    = 20
    TxTypeL2UpdateMargin      = 29
    TxTypeL2StakeAssets       = 35
    TxTypeL2UnstakeAssets     = 36
    TxTypeL2ApproveIntegrator = 45
    TxTypeL2CreateGroupedOrders = 28
)

// Order Types
const (
    LimitOrder           = 0
    MarketOrder          = 1
    StopLossOrder        = 2
    StopLossLimitOrder   = 3
    TakeProfitOrder      = 4
    TakeProfitLimitOrder = 5
    TWAPOrder            = 6
)

// Time-In-Force
const (
    ImmediateOrCancel = 0
    GoodTillTime      = 1
    PostOnly          = 2
)

// Bounds
const (
    MaxOrderNonce       int64  = (1 << 48) - 1
    MaxClientOrderIndex int64  = (1 << 48) - 1
    MaxOrderBaseAmount  int64  = (1 << 48) - 1
    MaxOrderPrice       uint32 = (1 << 32) - 1
    MaxTimestamp               = (1 << 48) - 1  // ExpiredAt limit
    OneUSDC                    = 1_000_000       // 6 decimal places
    FeeTick             int64  = 1_000_000       // 100% fee = 1_000_000
    MaxApiKeyIndex      uint8  = 254
    NilApiKeyIndex             = MaxApiKeyIndex + 1  // = 255
)
```

---

## 4. OrderInfo Struct (`types/txtypes/interface.go`)

```go
type OrderInfo struct {
    MarketIndex      int16
    ClientOrderIndex int64
    BaseAmount       int64
    Price            uint32
    IsAsk            uint8
    Type             uint8
    TimeInForce      uint8
    ReduceOnly       uint8
    TriggerPrice     uint32
    OrderExpiry      int64
}
```

---

## 5. Create Order — Hash Function (`types/txtypes/create_order.go`)

The `Hash()` packs **16 Goldilocks field elements** in this exact order:

```go
func (txInfo *L2CreateOrderTxInfo) Hash(lighterChainId uint32, extra ...g.Element) (msgHash []byte, err error) {
    elems := make([]g.Element, 0, 16)

    elems = append(elems, g.FromUint32(lighterChainId))          // [0] chain_id = 304
    elems = append(elems, g.FromUint32(TxTypeL2CreateOrder))     // [1] tx_type = 14
    elems = append(elems, g.FromInt64(txInfo.Nonce))             // [2] nonce
    elems = append(elems, g.FromInt64(txInfo.ExpiredAt))         // [3] expired_at (Unix ms)

    elems = append(elems, g.FromInt64(txInfo.AccountIndex))      // [4] account_index
    elems = append(elems, g.FromUint32(uint32(txInfo.ApiKeyIndex))) // [5] api_key_index
    elems = append(elems, g.FromUint32(uint32(txInfo.MarketIndex))) // [6] market_index
    elems = append(elems, g.FromInt64(txInfo.ClientOrderIndex))  // [7] client_order_index
    elems = append(elems, g.FromInt64(txInfo.BaseAmount))        // [8] base_amount
    elems = append(elems, g.FromUint32(txInfo.Price))            // [9] price
    elems = append(elems, g.FromUint32(uint32(txInfo.IsAsk)))    // [10] is_ask (0 or 1)
    elems = append(elems, g.FromUint32(uint32(txInfo.Type)))     // [11] order_type
    elems = append(elems, g.FromUint32(uint32(txInfo.TimeInForce))) // [12] time_in_force
    elems = append(elems, g.FromUint32(uint32(txInfo.ReduceOnly))) // [13] reduce_only
    elems = append(elems, g.FromUint32(txInfo.TriggerPrice))    // [14] trigger_price
    elems = append(elems, g.FromInt64(txInfo.OrderExpiry))       // [15] order_expiry

    txHash := p2.HashToQuinticExtension(elems)
    return txInfo.L2TxAttributes.AggregrateTxHash(txHash)
}
```

**Note:** The result is then optionally aggregated with the `L2TxAttributes` hash (integrator fees). If no attributes are set (`IsEmpty()` = true), the raw Poseidon2 hash bytes are returned directly.

```go
type L2CreateOrderTxInfo struct {
    AccountIndex int64
    ApiKeyIndex  uint8
    *OrderInfo
    ExpiredAt  int64
    Nonce      int64
    Sig        []byte
    SignedHash string `json:"-"`
    L2TxAttributes
}
```

---

## 6. Cancel Order — Hash Function (`types/txtypes/cancel_order.go`)

**8 Goldilocks field elements:**

```go
func (txInfo *L2CancelOrderTxInfo) Hash(lighterChainId uint32, extra ...g.Element) (msgHash []byte, err error) {
    elems := make([]g.Element, 0, 8)

    elems = append(elems, g.FromUint32(lighterChainId))           // [0] chain_id = 304
    elems = append(elems, g.FromUint32(TxTypeL2CancelOrder))      // [1] tx_type = 15
    elems = append(elems, g.FromInt64(txInfo.Nonce))              // [2] nonce
    elems = append(elems, g.FromInt64(txInfo.ExpiredAt))          // [3] expired_at (Unix ms)

    elems = append(elems, g.FromInt64(txInfo.AccountIndex))       // [4] account_index
    elems = append(elems, g.FromUint32(uint32(txInfo.ApiKeyIndex))) // [5] api_key_index
    elems = append(elems, g.FromUint32(uint32(txInfo.MarketIndex))) // [6] market_index
    elems = append(elems, g.FromInt64(txInfo.Index))              // [7] order_index (client or server)

    return p2.HashToQuinticExtension(elems).ToLittleEndianBytes(), nil
}
```

```go
type L2CancelOrderTxInfo struct {
    AccountIndex int64
    ApiKeyIndex  uint8
    MarketIndex  int16
    Index        int64  // Client Order Index OR Order Index (both accepted)
    ExpiredAt    int64
    Nonce        int64
    Sig          []byte
    SignedHash   string `json:"-"`
}
```

**Note:** Cancel does NOT use `L2TxAttributes` — the hash is returned directly without aggregation.

---

## 7. Auth Token Generation (`types/tx_request.go`)

The `ConstructAuthToken` function (from summarized source — exact body):

```
Message format: "{deadline_unix_seconds}:{account_index}:{api_key_index}"
  → convert to Goldilocks field element
  → Poseidon2 hash → quintic extension (40 bytes)
  → Schnorr sign
  → hex-encode signature
  → return as string auth token
```

Called from `TxClient.GetAuthToken()`:

```go
func (c *TxClient) GetAuthToken(deadline time.Time) (string, error) {
    return types.ConstructAuthToken(c.keyManager, deadline, &types.TransactOpts{
        ApiKeyIndex:      &c.apiKeyIndex,
        FromAccountIndex: &c.accountIndex,
    })
}
```

And from the C sharedlib:

```go
// deadline = Unix timestamp (seconds). 0 = now + 7 hours.
func CreateAuthToken(cDeadline C.longlong, cApiKeyIndex C.int, cAccountIndex C.longlong) C.StrOrErr {
    deadline := int64(cDeadline)
    if deadline == 0 {
        deadline = time.Now().Add(time.Hour * 7).Unix()
    }
    authToken, err := c.GetAuthToken(time.Unix(deadline, 0))
    // returns hex string
}
```

**Auth token validity: max 8 hours.** The SDK default is 7 hours.

---

## 8. TxClient — Full Struct and Constructor (`client/tx_client.go`)

```go
type TxClient struct {
    apiClient    MinimalHTTPClient
    chainId      uint32
    keyManager   signer.KeyManager
    accountIndex int64
    apiKeyIndex  uint8
}

// Private key: hex-encoded 40 bytes. "0x" prefix is optional and stripped.
func NewTxClient(apiClient MinimalHTTPClient, apiKeyPrivateKey string, accountIndex int64, apiKeyIndex uint8, chainId uint32) (*TxClient, error) {
    if apiKeyPrivateKey[:2] == "0x" {
        apiKeyPrivateKey = apiKeyPrivateKey[2:]
    }
    b, err := hex.DecodeString(apiKeyPrivateKey)  // → 40 bytes
    keyManager, err := signer.NewKeyManager(b)
    return &TxClient{
        apiClient:    apiClient,
        apiKeyIndex:  apiKeyIndex,
        accountIndex: accountIndex,
        chainId:      chainId,
        keyManager:   keyManager,
    }, nil
}
```

### FullFillDefaultOps (auto-fills nonce via HTTP)

```go
var DefaultExpireTime = time.Minute*10 - time.Second  // ~9m59s

func (c *TxClient) FullFillDefaultOps(ops *types.TransactOpts) (*types.TransactOpts, error) {
    if ops == nil { ops = new(types.TransactOpts) }
    if ops.ExpiredAt == 0 {
        ops.ExpiredAt = time.Now().Add(DefaultExpireTime).UnixMilli()  // milliseconds!
    }
    if ops.FromAccountIndex == nil { ops.FromAccountIndex = &c.accountIndex }
    if ops.ApiKeyIndex == nil { ops.ApiKeyIndex = &c.apiKeyIndex }
    if ops.Nonce == nil || *ops.Nonce == -1 {
        // HTTP GET /api/v1/nextNonce?account_index=X&api_key_index=Y
        nonce, err := c.apiClient.GetNextNonce(*ops.FromAccountIndex, *ops.ApiKeyIndex)
        ops.Nonce = &nonce
    }
    return ops, nil
}
```

**Important:** `ExpiredAt` is stored in **Unix milliseconds**, not seconds.

---

## 9. TransactOpts — Request Options

```go
type TransactOpts struct {
    FromAccountIndex *int64
    ApiKeyIndex      *uint8
    ExpiredAt        int64   // Unix MILLISECONDS
    Nonce            *int64  // -1 = auto-fetch via HTTP
    DryRun           bool
}
```

---

## 10. SignCreateOrder — C Sharedlib Entry Point (`sharedlib/main.go`)

Full function signature and body:

```go
//export SignCreateOrder
func SignCreateOrder(
    cMarketIndex          C.int,
    cClientOrderIndex     C.longlong,
    cBaseAmount           C.longlong,
    cPrice                C.int,
    cIsAsk                C.int,
    cOrderType            C.int,
    cTimeInForce          C.int,
    cReduceOnly           C.int,
    cTriggerPrice         C.int,
    cOrderExpiry          C.longlong,   // -1 = now + 28 days
    cIntegratorAccountIndex C.longlong,
    cIntegratorTakerFee   C.int,
    cIntegratorMakerFee   C.int,
    cNonce                C.longlong,
    cApiKeyIndex          C.int,
    cAccountIndex         C.longlong,
) C.SignedTxResponse {
    // ...
    if orderExpiry == -1 {
        orderExpiry = time.Now().Add(time.Hour * 24 * 28).UnixMilli() // 28 days
    }
    tx := &types.CreateOrderTxReq{
        MarketIndex:             int16(cMarketIndex),
        ClientOrderIndex:        int64(cClientOrderIndex),
        BaseAmount:              int64(cBaseAmount),
        Price:                   uint32(cPrice),
        IsAsk:                   uint8(cIsAsk),
        Type:                    uint8(cOrderType),
        TimeInForce:             uint8(cTimeInForce),
        ReduceOnly:              uint8(cReduceOnly),
        TriggerPrice:            uint32(cTriggerPrice),
        OrderExpiry:             int64(cOrderExpiry),
        IntegratorAccountIndex:  int(cIntegratorAccountIndex),
        IntegratorTakerFee:      int(cIntegratorTakerFee),
        IntegratorMakerFee:      int(cIntegratorMakerFee),
    }
    ops := &types.TransactOpts{Nonce: &nonce}
    txInfo, err := c.GetCreateOrderTransaction(tx, ops)
    // returns C.SignedTxResponse{txType, txInfo (JSON string), txHash, messageToSign}
}
```

### SignedTxResponse (C struct)

```c
typedef struct {
    uint8_t  txType;
    char*    txInfo;      // JSON-serialized L2CreateOrderTxInfo
    char*    txHash;      // hex-encoded signed hash (= tx hash on Lighter)
    char*    messageToSign; // L1 ETH message (only for ChangePubKey/Transfer/ApproveIntegrator)
    char*    err;
} SignedTxResponse;
```

---

## 11. SignCancelOrder — C Sharedlib Entry Point

```go
//export SignCancelOrder
func SignCancelOrder(
    cMarketIndex  C.int,
    cOrderIndex   C.longlong,   // Client Order Index OR server Order Index
    cNonce        C.longlong,
    cApiKeyIndex  C.int,
    cAccountIndex C.longlong,
) C.SignedTxResponse {
    tx := &types.CancelOrderTxReq{
        MarketIndex: int16(cMarketIndex),
        Index:       int64(cOrderIndex),
    }
    ops := &types.TransactOpts{Nonce: &nonce}
    txInfo, err := c.GetCancelOrderTransaction(tx, ops)
    // returns SignedTxResponse
}
```

---

## 12. L2TxAttributes — Integrator Fee Aggregation (`types/txtypes/tx_attributes.go`)

When integrator fees are set, the tx hash is further hashed with the attribute hash:

```go
type L2TxAttributes map[uint8]int  // attribute_type → value

const (
    AttributeTypeIntegratorAccountIndex = 1  // 6 bytes
    AttributeTypeIntegratorTakerFee     = 2  // 4 bytes
    AttributeTypeIntegratorMakerFee     = 3  // 4 bytes
)

// If attributes empty → return txHash.ToLittleEndianBytes() unchanged
// If attributes set:
//   attrHash = Poseidon2_plonky2( [type1, val1, type2, val2, type3, val3, 0, 0] )  // 8 GF elems
//   combined = Poseidon2_plonky2( txHash[5] ++ attrHash[5] )                       // 10 GF elems
//   return combined.ToLittleEndianBytes()
func (attr L2TxAttributes) AggregateTxHash(txHash gFp5.Element) ([]byte, error)
```

**Note:** The tx hash uses `poseidon2_goldilocks` but the attribute aggregation uses `poseidon2_goldilocks_plonky2` — a different variant.

---

## 13. Nonce Fetch — HTTP API

```
GET /api/v1/nextNonce?account_index={account_index}&api_key_index={api_key_index}

Response:
{
  "code": 200,
  "nonce": 722
}
```

---

## 14. Full Signing Pipeline (end-to-end)

### For CreateOrder:

```
1. Collect fields:
   chain_id=304, tx_type=14, nonce, expired_at_ms,
   account_index, api_key_index, market_index,
   client_order_index, base_amount, price, is_ask,
   order_type, time_in_force, reduce_only, trigger_price, order_expiry

2. Pack as 16 Goldilocks field elements (g.FromUint32 / g.FromInt64)

3. txHash = p2.HashToQuinticExtension(elems)  → gFp5.Element (5 × u64 = 40 bytes)

4. If integrator attrs non-empty:
   attrHash = p2.HashToQuinticExtension(attr_elems)  [plonky2 variant]
   finalHash = p2.HashToQuinticExtension(txHash_elems ++ attrHash_elems)  [plonky2 variant]
   msgHash = finalHash.ToLittleEndianBytes()  (40 bytes)
   Else:
   msgHash = txHash.ToLittleEndianBytes()  (40 bytes)

5. sig = schnorr.SchnorrSignHashedMessage(
       gFp5.FromCanonicalLittleEndianBytes(msgHash),
       privateKey_as_ECgFp5Scalar
   )
   → 80 bytes Schnorr signature

6. txInfo.Sig = sig
   txInfo.SignedHash = hex.Encode(msgHash)

7. Serialize txInfo to JSON → send to Lighter API
```

### For CancelOrder:

```
1. Pack 8 GF elements: chain_id, tx_type=15, nonce, expired_at_ms,
                       account_index, api_key_index, market_index, order_index

2. msgHash = p2.HashToQuinticExtension(elems).ToLittleEndianBytes()  (40 bytes)
   [NO attribute aggregation for cancel]

3. sig = schnorr.SchnorrSignHashedMessage(gFp5(msgHash), privKey)
```

---

## 15. GenerateAPIKey — Key Derivation

```go
func GenerateAPIKey() (privateKeyStr, publicKeyStr string, err error) {
    key := curve.SampleScalar()                          // random ECgFp5Scalar
    publicKey := schnorr.SchnorrPkFromSk(key)            // gFp5.Element
    privateKeyStr = hexutil.Encode(key.ToLittleEndianBytes())    // "0x" + 80 hex chars
    publicKeyStr = hexutil.Encode(publicKey.ToLittleEndianBytes()) // "0x" + 80 hex chars
    return
}
```

Both private and public keys are 40 bytes = 80 hex chars, little-endian.

---

## 16. C++ Example — Complete Usage Pattern

```cpp
// Step 1: Generate API key
ApiKeyResponse apiResp = GenerateAPIKey(nullptr);
// apiResp.privateKey = "0x..."  (80 hex chars)
// apiResp.publicKey  = "0x..."  (80 hex chars)

// Step 2: Create client (chain_id=304 for mainnet)
CreateClient(/*url=*/nullptr, apiResp.privateKey, /*chain_id=*/304,
             /*api_key_index=*/0, /*account_index=*/100);

// Step 3: Generate auth token (deadline = now + 7 hours, in milliseconds)
StrOrErr tokenResp = CreateAuthToken(now_ms() + 7 * 60 * 60 * 1000, 0, 100);
// tokenResp.str = auth token string

// Step 4: Sign orders with explicit nonce management
long long nonce = 1;

// Create limit post-only order: sell 10000 base @ price 400000, market 0 (ETH perps?)
// client_order_index=i, is_ask=true, order_type=0 (Limit), time_in_force=2 (PostOnly)
// trigger_price=0, expiry=now+1hr, integrator_params=0,0,0
auto create = SignCreateOrder(
    /*market*/0, /*client_order_idx*/i, /*base_amount*/10000, /*price*/400000,
    /*is_ask*/true, /*order_type*/0, /*time_in_force*/2, /*reduce_only*/0,
    /*trigger_price*/0, /*expiry_ms*/now_ms() + 60*60*1000,
    /*integrator_account*/0, /*taker_fee*/0, /*maker_fee*/0,
    /*nonce*/nonce, /*api_key_idx*/0, /*account_idx*/100
);
nonce += 1;

// Cancel by client order index i on market 0
auto cancel = SignCancelOrder(/*market*/0, /*order_idx*/i, /*nonce*/nonce, 0, 100);
nonce += 1;
```

---

## 17. Module Dependencies

```
github.com/elliottech/poseidon_crypto v0.0.15
github.com/ethereum/go-ethereum v1.15.6
```

### poseidon_crypto sub-packages used:

| Import path | Purpose |
|---|---|
| `poseidon_crypto/curve/ecgfp5` | ECgFp5 curve — scalar sampling, conversion |
| `poseidon_crypto/field/goldilocks` | GF(2^64 - 2^32 + 1) field elements, FromUint32/FromInt64 |
| `poseidon_crypto/field/goldilocks_quintic_extension` | GFp5 = Goldilocks^5, ToLittleEndianBytes |
| `poseidon_crypto/hash/poseidon2_goldilocks` | Poseidon2 over Goldilocks, HashToQuinticExtension |
| `poseidon_crypto/hash/poseidon2_goldilocks_plonky2` | Plonky2 variant — used for tx attribute aggregation |
| `poseidon_crypto/signature/schnorr` | SchnorrSignHashedMessage, SchnorrPkFromSk, Validate |

---

## 18. Signature Format

```
Signature length: 80 bytes (const SignatureLength = 80)
Public key length: 40 bytes (const PubKeyLength = gFp5.Bytes)
Hash length: 40 bytes (const HashLength = gQuint.Bytes)
L1 signature length: 65 bytes (Ethereum ECDSA)
```

---

## 19. Key Validation — Check()

```go
func (c *TxClient) Check() error {
    // GET /api/v1/apikeys?account_index=X&api_key_index=Y
    publicKey, err := c.HTTP().GetApiKey(c.accountIndex, c.apiKeyIndex)
    pubKeyBytes := c.GetKeyManager().PubKeyBytes()
    pubKeyStr := hexutil.Encode(pubKeyBytes[:])
    pubKeyStr = strings.Replace(pubKeyStr, "0x", "", 1)
    if publicKey != pubKeyStr {
        return fmt.Errorf("private key does not match the one on Lighter...")
    }
    return nil
}
```

---

## 20. HTTP Transport Config

```go
// Base URL: configured per deployment (mainnet: api.lighter.xyz or similar)
// GET /api/v1/nextNonce?account_index=X&api_key_index=Y → {"code":200,"nonce":N}
// GET /api/v1/apikeys?account_index=X&api_key_index=Y   → {"code":200,"api_keys":[...]}
// No SendTransaction shown in HTTP layer — likely done via external REST call
// Transport: MaxConnsPerHost=1000, 30s timeout, TLS 1.2+, no InsecureSkipVerify
```

---

## Summary: What Rust Implementation Needs

To replicate Lighter transaction signing in Rust:

1. **Private key**: 40-byte ECgFp5 scalar, little-endian hex encoding
2. **Poseidon2 hash**: Hash `Vec<GoldilocksField>` → `GFp5Element` (40 bytes LE)
   - Use `poseidon2_goldilocks` variant for transaction hashes
   - Use `poseidon2_goldilocks_plonky2` variant for attribute aggregation
3. **Schnorr sign**: `SchnorrSignHashedMessage(gfp5_hash, private_key)` → 80-byte signature
4. **For create order**: Pack 16 field elements in exact order above
5. **For cancel order**: Pack 8 field elements in exact order above
6. **For auth token**: Hash `"{deadline}:{account_index}:{api_key_index}"` as field element → Poseidon2 → Schnorr sign
7. **chain_id = 304** for mainnet
8. **ExpiredAt**: Unix **milliseconds** (not seconds)
9. **Nonce**: Fetch from `/api/v1/nextNonce` or supply manually (must be strictly increasing)

---

## Sources

- [elliottech/lighter-go — GitHub](https://github.com/elliottech/lighter-go)
- [signer/key_manager.go](https://github.com/elliottech/lighter-go/blob/main/signer/key_manager.go)
- [types/txtypes/create_order.go](https://github.com/elliottech/lighter-go/blob/main/types/txtypes/create_order.go)
- [types/txtypes/cancel_order.go](https://github.com/elliottech/lighter-go/blob/main/types/txtypes/cancel_order.go)
- [types/txtypes/constants.go](https://github.com/elliottech/lighter-go/blob/main/types/txtypes/constants.go)
- [types/txtypes/tx_attributes.go](https://github.com/elliottech/lighter-go/blob/main/types/txtypes/tx_attributes.go)
- [types/txtypes/utils.go](https://github.com/elliottech/lighter-go/blob/main/types/txtypes/utils.go)
- [client/tx_client.go](https://github.com/elliottech/lighter-go/blob/main/client/tx_client.go)
- [client/tx_get.go](https://github.com/elliottech/lighter-go/blob/main/client/tx_get.go)
- [client/client.go](https://github.com/elliottech/lighter-go/blob/main/client/client.go)
- [sharedlib/main.go](https://github.com/elliottech/lighter-go/blob/main/sharedlib/main.go)
- [examples/example.cpp](https://github.com/elliottech/lighter-go/blob/main/examples/example.cpp)
- [go.mod](https://github.com/elliottech/lighter-go/blob/main/go.mod)
