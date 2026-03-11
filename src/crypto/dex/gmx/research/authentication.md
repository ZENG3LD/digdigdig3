# GMX Authentication & Wallet Signatures

GMX is a decentralized exchange operating entirely on-chain. Unlike centralized exchanges, GMX does not use API keys or traditional authentication. Instead, it relies on **blockchain wallet signatures** for all trading operations.

## Authentication Model

### No Traditional API Authentication

GMX's REST API endpoints are **public and unauthenticated**:
- Market data (prices, tickers, markets) - No authentication required
- OHLC data (candlesticks) - No authentication required
- Market statistics (APY, performance) - No authentication required

### Wallet-Based Authentication

All **write operations** (trading) require:
1. **Ethereum wallet** with private key
2. **Transaction signing** using the private key
3. **Gas fees** paid in native token (ETH on Arbitrum, AVAX on Avalanche)

## Wallet Integration

### Supported Networks

**Arbitrum (Chain ID: 42161)**
- Native token: ETH
- RPC: `https://arb1.arbitrum.io/rpc`

**Avalanche (Chain ID: 43114)**
- Native token: AVAX
- RPC: `https://api.avax.network/ext/bc/C/rpc`

**Botanix (Chain ID: TBD)**
- Check official documentation for current RPC endpoints

### Wallet Requirements

To interact with GMX smart contracts, you need:

1. **Ethereum-compatible wallet**
   - Private key or seed phrase
   - Supports EIP-1559 transactions
   - Supports contract interaction

2. **Native token for gas**
   - ETH (Arbitrum)
   - AVAX (Avalanche)

3. **Collateral tokens**
   - USDC, USDT, ETH, BTC, etc.
   - Must approve token spending before trading

## Transaction Signing Flow

### 1. Token Approval (ERC20)

Before trading, approve the ExchangeRouter to spend your tokens:

```rust
// Pseudo-code for token approval
let token_address = "0x..."; // USDC, ETH, etc.
let spender = "0x602b805EedddBbD9ddff44A7dcBD46cb07849685"; // ExchangeRouter
let amount = u256::MAX; // Unlimited approval

// Create approval transaction
let tx = erc20_contract.approve(spender, amount);

// Sign with private key
let signed_tx = wallet.sign_transaction(tx);

// Broadcast to network
let tx_hash = provider.send_transaction(signed_tx).await?;
```

**Contract ABI:**
```json
{
  "name": "approve",
  "type": "function",
  "inputs": [
    {"name": "spender", "type": "address"},
    {"name": "amount", "type": "uint256"}
  ],
  "outputs": [{"name": "", "type": "bool"}]
}
```

### 2. Transfer Collateral to OrderVault

Before creating an order, transfer collateral in the **same transaction**:

```rust
// Transfer tokens to OrderVault
let order_vault = "0x31eF83a530Fde1B38EE9A18093A333D8Bbbc40D5"; // Arbitrum
let collateral_amount = parse_units("100", 6); // 100 USDC

let transfer_tx = erc20_contract.transfer(order_vault, collateral_amount);
```

**Critical:** Transfer and order creation must be in ONE transaction (multicall).

### 3. Create Order via ExchangeRouter

Sign and submit order creation transaction:

```rust
// CreateOrder parameters
struct CreateOrderParams {
    addresses: CreateOrderParamsAddresses,
    numbers: CreateOrderParamsNumbers,
    orderType: OrderType,
    decreasePositionSwapType: DecreasePositionSwapType,
    isLong: bool,
    shouldUnwrapNativeToken: bool,
    referralCode: bytes32,
}

// Sign transaction
let create_order_tx = exchange_router.createOrder(params);
let signed_tx = wallet.sign_transaction(create_order_tx);
let tx_hash = provider.send_transaction(signed_tx).await?;
```

### 4. Monitor Transaction & Order Execution

After submission:

1. **Wait for transaction confirmation**
   - Monitor tx_hash on blockchain
   - Typical confirmation: 1-2 blocks (2-4 seconds on Arbitrum)

2. **Extract order key from receipt**
   - Parse transaction logs
   - Find OrderCreated event
   - Extract bytes32 order key

3. **Monitor order execution**
   - Keepers execute orders asynchronously
   - Subscribe to OrderExecuted events
   - Typical execution: 5-30 seconds after creation

## EIP-712 Typed Data Signing

GMX uses **EIP-712** for structured, human-readable signatures in certain operations.

### What is EIP-712?

EIP-712 provides a standard for signing **typed structured data** instead of raw bytes. This allows wallets to display readable information to users before signing.

### EIP-712 Use Cases in GMX

While GMX primarily uses direct transaction signing for trading, EIP-712 may be used for:

1. **Off-chain order signing** (if supported)
2. **Permit functions** (gasless approvals via EIP-2612)
3. **Meta-transactions** (delegated execution)

### EIP-712 Message Structure Example

```json
{
  "types": {
    "EIP712Domain": [
      {"name": "name", "type": "string"},
      {"name": "version", "type": "string"},
      {"name": "chainId", "type": "uint256"},
      {"name": "verifyingContract", "type": "address"}
    ],
    "Order": [
      {"name": "account", "type": "address"},
      {"name": "market", "type": "address"},
      {"name": "sizeDelta", "type": "uint256"},
      {"name": "price", "type": "uint256"},
      {"name": "nonce", "type": "uint256"}
    ]
  },
  "primaryType": "Order",
  "domain": {
    "name": "GMX",
    "version": "1",
    "chainId": 42161,
    "verifyingContract": "0x602b805EedddBbD9ddff44A7dcBD46cb07849685"
  },
  "message": {
    "account": "0x...",
    "market": "0x...",
    "sizeDelta": "1000000000000000000",
    "price": "2500000000000000000000000000000000",
    "nonce": 1
  }
}
```

### Signing EIP-712 Messages

```rust
// Pseudo-code for EIP-712 signing
let domain = EIP712Domain {
    name: "GMX",
    version: "1",
    chain_id: 42161,
    verifying_contract: exchange_router_address,
};

let message = OrderMessage {
    account: user_address,
    market: market_address,
    size_delta: U256::from(1e18),
    price: U256::from(2500e30),
    nonce: 1,
};

// Sign using eth_signTypedData_v4
let signature = wallet.sign_typed_data(&domain, &message).await?;

// Signature format: (v, r, s) or 65-byte compact signature
```

## Oracle Price Signatures

GMX uses **signed oracle prices** to prevent front-running and ensure fair execution.

### How Oracle Signatures Work

1. **Oracle keepers** fetch prices from reference exchanges
2. **Sign price data** with keeper private keys
3. **Execution keepers** include signed prices when executing orders
4. **Smart contracts verify** signatures on-chain

### Signed Price Format

```json
{
  "tokenSymbol": "ETH",
  "minPrice": "2500000000000000000000000000000000",
  "maxPrice": "2501000000000000000000000000000000",
  "timestamp": 1674567890,
  "signature": "0x1234...abcd"
}
```

**Signature verification:**
- Uses `ecrecover` to extract signer address
- Compares against trusted oracle addresses
- Rejects if timestamp too old (stale prices)

### User Interaction with Oracle Prices

**Users do NOT sign oracle prices.** The flow is:

1. User creates order → No price signature needed
2. Keeper executes order → Includes oracle price signatures
3. Contract verifies → Oracle signatures validated
4. Order executes → Using verified oracle prices

## Gas Management

### Gas Price Strategies

GMX transactions require gas fees. Implement dynamic gas pricing:

```rust
// Get current gas price
let base_fee = provider.get_gas_price().await?;

// EIP-1559 transaction
let max_priority_fee = U256::from(1_000_000_000); // 1 gwei tip
let max_fee = base_fee + max_priority_fee;

let tx = transaction
    .max_fee_per_gas(max_fee)
    .max_priority_fee_per_gas(max_priority_fee);
```

### Execution Fees

GMX charges **execution fees** for keeper gas costs:

```rust
// Calculate execution fee for order
let execution_fee = calculate_execution_fee(
    gas_limits.execution_fee_base_amount,
    gas_price,
    gas_limits.per_order_keeper_gas,
);

// Include in transaction value
let tx_value = execution_fee; // Sent as ETH/AVAX value
```

**Typical execution fees:**
- Market orders: ~0.001-0.005 ETH
- Limit orders: ~0.002-0.01 ETH
- Complex orders: ~0.005-0.02 ETH

## Security Best Practices

### 1. Private Key Management

**NEVER expose private keys in code:**

```rust
// ❌ BAD - Hardcoded key
let private_key = "0x1234...";

// ✅ GOOD - Environment variable
let private_key = std::env::var("GMX_PRIVATE_KEY")?;

// ✅ BETTER - Hardware wallet or secure keystore
let wallet = Wallet::from_keystore("./keystore.json", password)?;
```

### 2. Token Approval Limits

**Avoid unlimited approvals in production:**

```rust
// ❌ RISKY - Unlimited approval
erc20.approve(spender, U256::MAX)?;

// ✅ SAFER - Limited approval
let approval_amount = required_amount * 110 / 100; // 10% buffer
erc20.approve(spender, approval_amount)?;
```

### 3. Transaction Verification

**Always verify transaction parameters before signing:**

```rust
// Verify order parameters
assert!(order.size_delta > 0, "Invalid size");
assert!(order.acceptable_price > 0, "Invalid price");
assert!(order.execution_fee >= min_execution_fee, "Insufficient fee");

// Sign only after validation
let signed_tx = wallet.sign_transaction(tx)?;
```

### 4. Nonce Management

**Handle nonces correctly to prevent stuck transactions:**

```rust
// Get current nonce
let nonce = provider.get_transaction_count(address, Some(BlockNumber::Pending)).await?;

// Use incremental nonces for multiple transactions
let tx1 = tx.nonce(nonce);
let tx2 = tx.nonce(nonce + 1);
let tx3 = tx.nonce(nonce + 2);
```

### 5. Slippage Protection

**Always set acceptable price limits:**

```rust
// For long position
let current_price = get_current_price("ETH")?;
let max_acceptable_price = current_price * 101 / 100; // 1% slippage

// For short position
let min_acceptable_price = current_price * 99 / 100; // 1% slippage

order.acceptable_price = acceptable_price;
```

## Implementation Checklist for V5 Connector

### Authentication Module (`auth.rs`)

- [ ] Implement wallet initialization from private key
- [ ] Support environment variable key loading
- [ ] Implement transaction signing function
- [ ] Add EIP-712 typed data signing (if needed)
- [ ] Handle nonce management
- [ ] Implement gas price estimation
- [ ] Add execution fee calculation

### No API Key Required

- [ ] Remove all API key authentication logic
- [ ] No signature generation for REST requests
- [ ] Public endpoints use no auth headers

### Transaction Helpers

- [ ] Token approval function
- [ ] Multicall builder (transfer + createOrder)
- [ ] Transaction monitoring/confirmation waiter
- [ ] Event log parser for order keys
- [ ] Gas limit estimator

### Error Handling

- [ ] Parse contract revert reasons
- [ ] Handle insufficient gas errors
- [ ] Detect nonce conflicts
- [ ] Retry logic for failed transactions

## Example: Complete Order Creation Flow

```rust
use ethers::prelude::*;

// 1. Initialize wallet
let wallet = "0x...".parse::<LocalWallet>()?
    .with_chain_id(42161u64);

// 2. Connect to provider
let provider = Provider::<Http>::try_from("https://arb1.arbitrum.io/rpc")?;
let client = SignerMiddleware::new(provider, wallet);

// 3. Approve token (if not already approved)
let usdc = IERC20::new(usdc_address, Arc::new(client.clone()));
let approval = usdc.approve(exchange_router, U256::MAX);
let tx = approval.send().await?.await?;

// 4. Prepare order parameters
let params = CreateOrderParams {
    // ... order details
};

// 5. Calculate execution fee
let execution_fee = U256::from(5_000_000_000_000_000u64); // 0.005 ETH

// 6. Create multicall transaction
let mut tx = exchange_router.createOrder(params);
tx = tx.value(execution_fee);

// 7. Sign and send
let pending_tx = tx.send().await?;
let receipt = pending_tx.await?;

// 8. Extract order key from logs
let order_key = parse_order_key_from_receipt(&receipt)?;

println!("Order created: {}", order_key);
```

## Sources

- [EIP-712 Explained](https://medium.com/@andrey_obruchkov/eip-712-explained-secure-off-chain-signatures-for-real-world-ethereum-apps-d2823c45227d)
- [GMX Synthetics Contracts](https://github.com/gmx-io/gmx-synthetics)
- [GMX SDK Documentation](https://docs.gmx.io/docs/sdk/)
- [Web3 Ethereum DeFi - GMX API](https://web3-ethereum-defi.readthedocs.io/api/gmx/_autosummary_gmx/eth_defi.gmx.api.html)
- [Polymarket Authentication Reference](https://docs.polymarket.com/developers/CLOB/authentication)
