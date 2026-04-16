# dYdX v4 Authentication

## Overview

dYdX v4 uses a fundamentally different authentication model compared to traditional centralized exchanges and even dYdX v3:

- **No API Keys**: Unlike centralized exchanges, dYdX v4 does not use API key/secret pairs
- **No STARK Signatures**: Unlike dYdX v3, v4 does not use StarkEx-based STARK signatures
- **Cosmos Blockchain**: v4 is built on Cosmos SDK and uses standard blockchain wallet authentication

## Authentication Architecture

### Two-Tier API System

1. **Indexer API** (Read-Only)
   - No authentication required
   - Public REST and WebSocket endpoints
   - Used for querying market data, account info, positions, orders

2. **Node API** (Write Operations)
   - Authentication required via signed transactions
   - gRPC/Protobuf protocol
   - Used for placing orders, canceling orders, transfers, deposits, withdrawals

## Account Structure

### Main Account (Wallet)
- Associated with a public-private keypair
- Represents the trader's on-chain identity
- Holds tokens for gas fees and collateral
- Cannot directly execute trades (only subaccounts can trade)
- Multiple main accounts can derive from one mnemonic phrase

**Address Format**: Cosmos-style addresses (e.g., `dydx1...`)

### Subaccounts
- Each main account can have up to 128,001 subaccounts
- Identified by `(main_account_address, subaccount_number)` pairs
- Subaccounts 0-127: Parent subaccounts (cross-margin)
- Subaccounts 128-128,000: Child subaccounts (isolated positions)
- Automatically created upon first deposit
- Only the main account can execute transactions for its subaccounts
- No gas costs for trading activities (gas paid by main account)

**Mapping Formula**: `parent_subaccount_number = child_subaccount_number % 128`

Example:
- Parent subaccount 0 has child subaccounts 128, 256, 384...
- Parent subaccount 1 has child subaccounts 129, 257, 385...

## Wallet Authentication

### Mnemonic Phrase (Secret Phrase)
On dYdX Chain, a trader's unique "secret phrase" is a set of **24 words** (mnemonic) used to back up and access their account.

**Purpose**:
- Derives the dYdX Chain address
- Can be used to create addresses across various Cosmos blockchains
- Acts as the recovery seed for the wallet

**Security**:
- Reveals your private key - keep it secure
- Never share with anyone
- Store in a secure location (hardware wallet, encrypted storage)

**How to Access**:
1. Click on your dYdX address in the upper right corner (on web interface)
2. Click on "reveal secret phrase"
3. Your 24-word mnemonic will be displayed

### Private Keys
- Derived from the mnemonic phrase using BIP-39/BIP-44 standards
- Used to sign transactions
- Never exposed directly in normal operations
- ECDSA signatures on the Cosmos blockchain (not STARK signatures)

### Key Derivation Path
Standard Cosmos derivation path:
```
m/44'/118'/0'/0/0
```

Where:
- `44'` - BIP-44 purpose
- `118'` - Cosmos coin type
- `0'` - Account index
- `0` - Change (external)
- `0` - Address index

## Transaction Signing

### gRPC Transaction Flow

1. **Create Transaction Message**
   - Example: `MsgPlaceOrder`, `MsgCancelOrder`, `MsgTransfer`
   - Include all required parameters (subaccount, market, price, size, etc.)

2. **Build Transaction**
   - Set gas limit and fee
   - Set sequence number (nonce)
   - Set chain ID and account number

3. **Sign Transaction**
   - Use private key to sign the transaction
   - Creates a valid Cosmos signature

4. **Broadcast Transaction**
   - Send to a validator node via gRPC
   - Validators verify the signature
   - Transaction included in a block

### Replay Prevention
- **Short-term orders**: Use prunable block heights (goodTilBlock)
- **Stateful orders**: Use Cosmos sequence numbers
- Each transaction has a unique sequence number that prevents replay attacks

## Client Libraries

### TypeScript (Recommended)
```typescript
import { CompositeClient, Network } from '@dydxprotocol/v4-client-js';

// Connect to mainnet
const network = Network.mainnet();
const client = await CompositeClient.connect(network);

// Create wallet from mnemonic
const mnemonic = "your 24 word mnemonic phrase here...";
const wallet = await LocalWallet.fromMnemonic(mnemonic, BECH32_PREFIX);

// Place order (automatically signs transaction)
const order = await client.placeOrder(
  subaccount,
  market,
  type,
  side,
  price,
  size,
  clientId,
  timeInForce,
  goodTilBlock,
  execution,
  postOnly,
  reduceOnly
);
```

**Features**:
- Composite Client combines Node and Indexer clients
- Automatic transaction signing
- Helper methods for order placement, cancellation
- Market data queries

### Python
```python
from v4_client_py import Client, Wallet

# Create client
client = Client(network='mainnet')

# Create wallet from mnemonic
mnemonic = "your 24 word mnemonic phrase here..."
wallet = Wallet.from_mnemonic(mnemonic)

# Place order
order_response = client.place_order(
    subaccount=Subaccount(wallet, 0),
    market="BTC-USD",
    type=OrderType.LIMIT,
    side=OrderSide.BUY,
    price=50000.0,
    size=0.1,
    client_id=12345,
    time_in_force=OrderTimeInForce.GTT,
    good_til_block=0,
    good_til_time_in_seconds=3600,
    post_only=False,
    reduce_only=False
)
```

**Features**:
- Separate Node and Indexer clients (no Composite client)
- Manual transaction signing
- Protobuf message handling

### Rust
```rust
// Rust client library exists but is less mature
// Must use explicit Node and Indexer clients
// No Composite client available

// Example structure (hypothetical):
use dydx_v4::{NodeClient, IndexerClient};

let mnemonic = "your 24 word mnemonic phrase here...";
let wallet = Wallet::from_mnemonic(mnemonic)?;

let node_client = NodeClient::new(node_endpoint)?;
let indexer_client = IndexerClient::new(indexer_endpoint)?;

// Sign and send transaction
let tx = node_client.place_order(/* params */)?;
let signed_tx = wallet.sign_transaction(tx)?;
let response = node_client.broadcast_transaction(signed_tx)?;
```

**Notes**:
- Rust support is less mature than TypeScript/Python
- May need to implement gRPC clients manually
- Focus on Protobuf message handling and signing

## Authentication Flow Summary

### For Read Operations (Indexer API)
```
1. No authentication required
2. Make HTTP GET request to Indexer
3. Receive JSON response
```

### For Write Operations (Node API)
```
1. Create transaction message (e.g., MsgPlaceOrder)
2. Get account info (sequence number, account number)
3. Build transaction with gas and fees
4. Sign transaction with private key
5. Broadcast signed transaction to validator node
6. Wait for confirmation (transaction included in block)
7. Query Indexer for order status
```

## Security Considerations

### Best Practices
1. **Never share your mnemonic phrase**
   - Treat it like a password - it controls all your funds

2. **Store mnemonic securely**
   - Hardware wallets (Ledger, Trezor)
   - Encrypted password managers
   - Paper backup in secure location

3. **Use subaccounts for risk isolation**
   - Separate trading subaccounts from main holdings
   - Use child subaccounts for isolated positions

4. **Monitor gas costs**
   - Main account needs DYDX tokens for gas fees
   - Trading on subaccounts is gas-free

5. **Verify transaction signatures**
   - Double-check transaction details before signing
   - Use hardware wallets for additional security

6. **Use testnet for development**
   - Test on testnet before deploying to mainnet
   - Testnet faucet: `https://faucet.v4testnet.dydx.exchange`

### Common Pitfalls
1. **Insufficient gas balance**
   - Main account needs DYDX tokens for gas
   - Check balance before submitting transactions

2. **Incorrect sequence numbers**
   - Out-of-order transactions will fail
   - Query account info before each transaction

3. **Short-term order expiry**
   - goodTilBlock must be within current block + 20 blocks
   - Orders expire if not matched within ~30 seconds

4. **Network selection**
   - Ensure correct network (mainnet vs testnet)
   - Different chain IDs and endpoints

## API Key Management (None Required)

Unlike traditional exchanges:
- **No API key generation** through web interface
- **No IP whitelisting** for API access
- **No permission scopes** (trading, withdrawal, etc.)
- **No rate limits per API key** (rate limits are per account/IP at the indexer level)

Authentication is purely wallet-based:
- Your private key IS your "API key"
- Signing transactions IS your authentication

## Comparison with Other Exchanges

### Traditional CEX (Binance, Bybit, etc.)
- API Key + Secret for HMAC signatures
- Permission-based access (read, trade, withdraw)
- Rate limits per API key
- Centralized authentication server

### dYdX v3 (StarkEx)
- API Key for indexer access
- STARK keys for order signing
- Ethereum wallet for deposits/withdrawals
- Layer 2 StarkEx signatures

### dYdX v4 (Cosmos)
- No API keys
- Cosmos wallet signatures
- Blockchain-native authentication
- Gas fees in DYDX tokens
- Fully decentralized (no centralized auth server)

## Permissioned Keys (Advanced)

**Note**: The documentation mentions "Permissioned Keys" as a related topic, but details are not provided in the current API documentation. This may be a future feature for:
- Delegated trading permissions
- Sub-key management
- Session keys
- Trading bots with limited permissions

Check the official dYdX documentation for updates on this feature.

## Testing Authentication

### Testnet
- **Indexer**: `https://indexer.v4testnet.dydx.exchange/v4`
- **Node gRPC**: `oegs-testnet.dydx.exchange:443`
- **Faucet**: `https://faucet.v4testnet.dydx.exchange`

### Steps to Test
1. Create a testnet wallet (generate mnemonic)
2. Request testnet DYDX from faucet
3. Connect to testnet node
4. Place a test order
5. Verify order appears in Indexer API

### Example (TypeScript)
```typescript
import { CompositeClient, Network } from '@dydxprotocol/v4-client-js';

const network = Network.testnet();
const client = await CompositeClient.connect(network);

// Test read (no auth)
const markets = await client.indexerClient.markets.getPerpetualMarkets();
console.log('Markets:', markets);

// Test write (auth required)
const mnemonic = "test mnemonic from testnet...";
const wallet = await LocalWallet.fromMnemonic(mnemonic, BECH32_PREFIX);

const order = await client.placeOrder(/* params */);
console.log('Order placed:', order);
```

## Resources

- **Official Client Libraries**:
  - TypeScript: `@dydxprotocol/v4-client-js`
  - Python: `dydx-v4-client`
  - Rust: (community libraries, less mature)

- **Documentation**:
  - Main Docs: https://docs.dydx.xyz
  - API Docs: https://docs.dydx.exchange
  - GitHub: https://github.com/dydxprotocol/v4-chain

- **Cosmos SDK Resources**:
  - Cosmos SDK: https://docs.cosmos.network
  - CosmJS: https://github.com/cosmos/cosmjs (TypeScript Cosmos library)
  - Cosmos Rust: https://github.com/cosmos/cosmos-rust

## Summary

dYdX v4 authentication is **fundamentally different** from traditional exchanges:

1. **No API keys** - Use blockchain wallets instead
2. **Mnemonic-based** - 24-word phrase derives all keys
3. **Signed transactions** - Every write operation is a blockchain transaction
4. **No auth for reads** - Indexer API is fully public
5. **Gas fees required** - Main account needs DYDX tokens
6. **Subaccount isolation** - Each subaccount is separately managed
7. **Cosmos-native** - Standard Cosmos SDK transaction signing

For Rust implementation:
- Focus on Cosmos wallet management (mnemonic → private key → signing)
- Implement gRPC clients for Node API
- Use REST client for Indexer API
- Handle Protobuf message encoding/decoding
- Manage sequence numbers and gas fees
