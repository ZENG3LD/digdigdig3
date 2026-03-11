# Raydium WebSocket and Real-Time Data

**Research Date**: 2026-01-20

Comprehensive documentation of real-time data access for Raydium DEX. **Important**: Raydium does not provide a traditional WebSocket API like centralized exchanges.

---

## Table of Contents

1. [Critical Understanding](#critical-understanding)
2. [Why No WebSocket API](#why-no-websocket-api)
3. [gRPC Alternative (Recommended)](#grpc-alternative-recommended)
4. [Solana Account Subscriptions](#solana-account-subscriptions)
5. [Comparison with CEX WebSocket](#comparison-with-cex-websocket)
6. [Implementation Approaches](#implementation-approaches)

---

## Critical Understanding

### Raydium Does NOT Have a WebSocket API

**Unlike Centralized Exchanges** (KuCoin, Binance, Coinbase):
- ❌ No `wss://ws.raydium.io/` endpoint
- ❌ No ticker subscriptions like `/market/ticker:SOL-USDC`
- ❌ No orderbook update streams
- ❌ No trade execution feeds
- ❌ No balance update channels

**Why?** Because Raydium is a **decentralized exchange** on Solana:
- All state is stored on-chain
- Updates happen via blockchain transactions
- Real-time data comes from monitoring the Solana blockchain, not Raydium servers

**Official Raydium Statement**:
> "APIs are for data access and monitoring — not real-time tracking. For real-time pool creation, refer to gRPC example in SDK demo."

---

## Why No WebSocket API

### CEX Architecture (Traditional WebSocket)

**Centralized Exchange Flow**:
```
User → WebSocket → Exchange Server → Order Book (in-memory) → Stream Updates
```

**Characteristics**:
- Exchange maintains order book state in memory
- Exchange pushes updates to WebSocket clients
- Centralized infrastructure
- Sub-millisecond latency (server-side)

### DEX Architecture (Blockchain-Based)

**Decentralized Exchange Flow**:
```
User → Blockchain Transaction → Solana Validators → On-Chain State → gRPC Stream
```

**Characteristics**:
- State is on Solana blockchain (decentralized)
- Updates via transaction execution
- No centralized update server
- Monitor blockchain for state changes
- 400ms-1s block time (Solana consensus)

**Key Insight**: Real-time data for DEX means monitoring blockchain events, not connecting to exchange WebSocket.

---

## gRPC Alternative (Recommended)

### Solana Geyser Plugin

**What is Geyser?**
- Plugin for Solana validators
- Streams account updates in real-time
- Push-based (not polling)
- Sub-second latency
- gRPC protocol

**Official Raydium Recommendation**:
- Use gRPC for real-time pool creation monitoring
- Example in Raydium SDK V2 demo repository
- Faster than RPC polling by hundreds of milliseconds

### gRPC vs REST Polling

| Method | Latency | Bandwidth | Reliability | Complexity |
|--------|---------|-----------|-------------|------------|
| **gRPC (Geyser)** | <1s | Low (push) | High | Medium |
| **REST Polling** | 1-60s | High (poll) | Medium | Low |
| **WebSocket (N/A)** | - | - | - | Not available |

**Winner**: gRPC is the recommended approach for real-time Raydium data.

---

## gRPC Providers

### 1. Chainstack

**Features**:
- Geyser-based subscribers
- Detects events earlier than other methods (hundreds of ms faster)
- Solana Geyser endpoint speaks gRPC
- Works across languages: Rust, TypeScript, Python, Go

**Documentation**: [Chainstack Geyser Guide](https://chainstack.com/solana-geyser-raydium-bonk/)

**Use Case**: Real-time token analytics for Raydium and Bonk.fun tokens

### 2. Shyft

**Features**:
- Stream real-time Raydium liquidity pool v4 transactions
- Geyser-fed gRPC account/program subscriptions
- Easy-to-use API

**Documentation**: [Shyft gRPC Network Guide](https://blogs.shyft.to/how-to-stream-and-parse-raydium-transactions-with-shyfts-grpc-network-b16d5b3af249)

**Use Case**: Stream and parse Raydium transactions

### 3. QuickNode

**Features**:
- Yellowstone Geyser gRPC Marketplace Add-on
- Monitor Solana programs with Yellowstone Geyser
- Rust support

**Documentation**: [QuickNode Yellowstone Guide](https://www.quicknode.com/guides/solana-development/tooling/geyser/yellowstone-rust)

**Use Case**: Monitor Solana programs with Yellowstone Geyser (Rust)

### 4. Triton One

**Features**:
- Raydium data via program-specific real-time streams
- Geyser-fed gRPC account/program subscriptions

**Use Case**: Program-specific real-time streams

### 5. bloXroute

**Features**:
- WebSocket subscriptions for Raydium pool streams
- GetNewRaydiumPoolsStream endpoint
- Stream New Raydium Pools

**Documentation**:
- [GetNewRaydiumPoolsStream](https://docs.bloxroute.com/solana/trader-api/api-endpoints/raydium/getnewraydiumpoolsstream)
- [Stream New Raydium Pools](https://docs.bloxroute.com/solana/trader-api-v2/api-endpoints/raydium/stream-new-raydium-pools)

**Use Case**: WebSocket-based pool creation monitoring

### 6. Helius

**Features**:
- geyser-enhanced transactionSubscribe method
- Beta feature for monitoring Raydium pools

**Documentation**: [Helius Raydium Pool Monitoring](https://www.helius.dev/blog/how-to-monitor-a-raydium-liquidity-pool)

**Use Case**: Monitor Raydium liquidity pools

---

## Solana Account Subscriptions

### Native Solana RPC WebSocket

**Solana provides WebSocket subscriptions** (not Raydium-specific):

**Connection URL**:
```
wss://api.mainnet-beta.solana.com
```

**Subscription Methods**:
1. `accountSubscribe` - Monitor specific account changes
2. `programSubscribe` - Monitor all accounts owned by program
3. `logsSubscribe` - Monitor transaction logs
4. `signatureSubscribe` - Monitor transaction confirmations
5. `slotSubscribe` - Monitor slot changes

### Example: Subscribe to Raydium Pool Account

**JavaScript Example**:
```javascript
const { Connection } = require('@solana/web3.js');

const connection = new Connection(
  'wss://api.mainnet-beta.solana.com',
  'confirmed'
);

const poolAddress = 'AVs9TA4nWDzfPJE9gGVNJMVhcQy3V9PGazuz33BfG2RA';

// Subscribe to pool account updates
const subscriptionId = connection.onAccountChange(
  poolAddress,
  (accountInfo, context) => {
    console.log('Pool account updated:', accountInfo);
    // Parse account data to extract pool state
  },
  'confirmed'
);

// Unsubscribe when done
// connection.removeAccountChangeListener(subscriptionId);
```

**Rust Example**:
```rust
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcProgramAccountsConfig;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

let rpc = RpcClient::new("https://api.mainnet-beta.solana.com");

let pool_address = Pubkey::from_str("AVs9TA4nWDzfPJE9gGVNJMVhcQy3V9PGazuz33BfG2RA")?;

// Poll account data (no native WebSocket in solana-client)
loop {
    let account = rpc.get_account(&pool_address)?;
    // Parse account.data to extract pool state
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
}
```

### Example: Subscribe to Raydium Program

**Subscribe to ALL Raydium pools**:
```javascript
const RAYDIUM_PROGRAM_ID = '675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8';

connection.onProgramAccountChange(
  RAYDIUM_PROGRAM_ID,
  (keyedAccountInfo) => {
    const address = keyedAccountInfo.accountId;
    const account = keyedAccountInfo.accountInfo;
    console.log(`Raydium account ${address} updated:`, account);
  },
  'confirmed'
);
```

**Use Case**: Detect new pool creation by monitoring program accounts.

### Limitations of Native RPC WebSocket

**Challenges**:
1. **Rate Limits**: Public RPC endpoints have connection limits
2. **Reliability**: Public endpoints can be unstable
3. **Filtering**: Limited filtering capabilities
4. **Parsing**: Must parse raw account data yourself
5. **Latency**: Higher latency than Geyser

**Recommendation**: Use Geyser providers instead of native RPC WebSocket.

---

## Comparison with CEX WebSocket

### KuCoin WebSocket (CEX Example)

**Connection**:
```javascript
const ws = new WebSocket('wss://ws-api-spot.kucoin.com/?token=...');

// Subscribe to ticker
ws.send(JSON.stringify({
  id: Date.now(),
  type: 'subscribe',
  topic: '/market/ticker:BTC-USDT',
  response: true
}));

// Receive updates
ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  if (data.type === 'message' && data.topic === '/market/ticker:BTC-USDT') {
    console.log('Ticker update:', data.data);
  }
};
```

**Features**:
- Human-readable JSON messages
- Topic-based subscriptions
- Ping/pong keepalive
- Reconnection with token refresh
- Channel-specific data formats

---

### Raydium gRPC (DEX Equivalent)

**Connection** (using Shyft as example):
```rust
use shyft_grpc_client::GeyserClient;

let client = GeyserClient::connect("grpc://shyft-grpc.com:443").await?;

// Subscribe to Raydium program accounts
let mut stream = client.subscribe_program_accounts(
    "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"
).await?;

// Receive updates
while let Some(update) = stream.next().await {
    let account = update.account;
    let address = update.address;
    println!("Account {} updated", address);
    // Parse account.data to extract pool state
}
```

**Features**:
- Binary protobuf messages (not JSON)
- Account/program-based subscriptions
- Automatic reconnection (gRPC)
- Raw account data (requires parsing)
- Lower-level than CEX WebSocket

---

### Feature Comparison

| Feature | CEX WebSocket (KuCoin) | DEX gRPC (Raydium) |
|---------|------------------------|---------------------|
| **Connection URL** | `wss://ws-api-spot.kucoin.com` | Provider-specific gRPC endpoint |
| **Protocol** | WebSocket (JSON) | gRPC (Protobuf) |
| **Auth** | Token from REST API | Provider API key (if required) |
| **Subscription** | Topic strings (e.g., `/market/ticker:BTC-USDT`) | Account/program pubkeys |
| **Message Format** | Parsed JSON objects | Raw account data bytes |
| **Data Parsing** | Exchange provides parsed fields | Must parse Raydium program data |
| **Channels** | Ticker, orderbook, trades, orders, balance | Account updates, program updates, transactions |
| **Reconnection** | Token refresh (24h expiry) | gRPC automatic |
| **Latency** | <100ms (exchange internal) | <1s (blockchain confirmation) |
| **Real-Time** | True real-time (order book updates) | Near real-time (block time ~400ms) |

---

## Implementation Approaches

### Approach 1: REST API Polling (Simplest)

**Pros**:
- Easy to implement
- No complex dependencies
- Works with standard HTTP client

**Cons**:
- High latency (polling interval)
- Wastes bandwidth
- May hit rate limits
- Not true real-time

**Implementation**:
```rust
pub struct RaydiumPoller {
    client: reqwest::Client,
    base_url: String,
    poll_interval: Duration,
}

impl RaydiumPoller {
    pub async fn poll_pool(&self, pool_id: &str) -> Result<Pool> {
        loop {
            let pool = self.fetch_pool(pool_id).await?;
            // Process pool data
            tokio::time::sleep(self.poll_interval).await;
        }
    }
}
```

**Use Case**: Low-frequency monitoring (once per minute)

---

### Approach 2: Solana RPC Account Subscribe (Medium)

**Pros**:
- Push-based (no polling)
- Lower latency than REST
- Native Solana support

**Cons**:
- Must parse raw account data
- Public RPC has rate limits
- Connection stability issues
- Higher latency than Geyser

**Implementation**:
```rust
use solana_client::nonblocking::pubsub_client::PubsubClient;
use solana_sdk::pubkey::Pubkey;

pub async fn subscribe_to_pool(pool_id: &str) -> Result<()> {
    let pubsub = PubsubClient::new("wss://api.mainnet-beta.solana.com").await?;

    let pool_pubkey = Pubkey::from_str(pool_id)?;

    let (mut stream, _unsub) = pubsub.account_subscribe(
        &pool_pubkey,
        Some(solana_client::rpc_config::RpcAccountInfoConfig {
            encoding: Some(solana_account_decoder::UiAccountEncoding::Base64),
            commitment: Some(solana_sdk::commitment_config::CommitmentConfig::confirmed()),
            ..Default::default()
        })
    ).await?;

    while let Some(response) = stream.next().await {
        match response.value.data {
            solana_account_decoder::UiAccountData::Binary(data, _) => {
                // Decode Base64 and parse Raydium pool data
                let account_data = base64::decode(data)?;
                let pool = parse_raydium_pool(&account_data)?;
                println!("Pool updated: {:?}", pool);
            }
            _ => {}
        }
    }

    Ok(())
}
```

**Use Case**: Medium-frequency monitoring with moderate latency tolerance

---

### Approach 3: Geyser gRPC (Recommended)

**Pros**:
- Lowest latency (<1s)
- Push-based updates
- Reliable providers
- Production-ready
- Scales well

**Cons**:
- Requires provider subscription (may have cost)
- More complex setup
- Must parse account data
- Provider-specific APIs

**Implementation** (Conceptual - depends on provider):
```rust
use geyser_client::GeyserClient; // Hypothetical

pub async fn subscribe_raydium_pools() -> Result<()> {
    let client = GeyserClient::connect("grpc://provider.com:443").await?;

    let mut stream = client.subscribe_program(
        "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8", // Raydium program
        Some(FilterConfig {
            account_type: AccountType::Pool,
        })
    ).await?;

    while let Some(update) = stream.next().await {
        let pool = parse_raydium_pool(&update.account_data)?;
        println!("Pool {} updated: TVL={}", update.address, pool.tvl);
    }

    Ok(())
}
```

**Use Case**: High-frequency trading bots, market makers, real-time analytics

---

### Approach 4: Hybrid (Recommended for Most)

**Combination**:
1. Use REST API for initial data fetch
2. Use gRPC for real-time updates
3. Cache data locally
4. Fall back to REST polling if gRPC unavailable

**Implementation**:
```rust
pub struct RaydiumConnector {
    rest_client: RestClient,
    grpc_client: Option<GeyserClient>,
    cache: Arc<RwLock<HashMap<String, Pool>>>,
}

impl RaydiumConnector {
    pub async fn start(&mut self) -> Result<()> {
        // Initial fetch via REST
        let pools = self.rest_client.get_pool_list().await?;
        for pool in pools {
            self.cache.write().await.insert(pool.id.clone(), pool);
        }

        // Subscribe to updates via gRPC
        if let Some(client) = &mut self.grpc_client {
            tokio::spawn(self.subscribe_updates(client.clone()));
        } else {
            // Fallback: Poll REST API
            tokio::spawn(self.poll_updates());
        }

        Ok(())
    }

    async fn subscribe_updates(&self, client: GeyserClient) {
        let mut stream = client.subscribe_program(RAYDIUM_PROGRAM_ID).await.unwrap();
        while let Some(update) = stream.next().await {
            let pool = parse_raydium_pool(&update.account_data).unwrap();
            self.cache.write().await.insert(pool.id.clone(), pool);
        }
    }

    async fn poll_updates(&self) {
        loop {
            let pools = self.rest_client.get_pool_list().await.unwrap();
            for pool in pools {
                self.cache.write().await.insert(pool.id.clone(), pool);
            }
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    }
}
```

**Use Case**: Production connectors with reliability and performance balance

---

## Parsing Raydium Account Data

### Challenge

Raydium pool state is stored as **raw bytes** on-chain. You must:
1. Understand Raydium program's account layout
2. Deserialize bytes into structs
3. Extract relevant fields (reserves, fees, etc.)

### Account Layout (Raydium V4)

**Pool Account Structure** (simplified):
```rust
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct AmmInfo {
    pub status: u64,           // 0 = uninitialized, 1 = initialized, etc.
    pub nonce: u64,            // Nonce for PDA derivation
    pub order_num: u64,        // Total number of orders
    pub depth: u64,            // Depth of order book
    pub coin_decimals: u64,    // Base token decimals
    pub pc_decimals: u64,      // Quote token decimals
    pub state: u64,            // Pool state
    pub reset_flag: u64,       // Reset flag
    pub min_size: u64,         // Minimum order size
    pub vol_max_cut_ratio: u64, // Volume max cut ratio
    pub amount_wave: u64,      // Amount wave
    pub coin_lot_size: u64,    // Base lot size
    pub pc_lot_size: u64,      // Quote lot size
    pub min_price_multiplier: u64, // Min price multiplier
    pub max_price_multiplier: u64, // Max price multiplier
    pub sys_decimal_value: u64,    // System decimal value
    // ... more fields
    pub coin_vault_balance: u64,   // Base token reserve
    pub pc_vault_balance: u64,     // Quote token reserve
    // ... more fields
}

impl AmmInfo {
    pub fn parse(data: &[u8]) -> Result<Self> {
        AmmInfo::try_from_slice(data)
            .map_err(|e| Error::ParseError(e.to_string()))
    }
}
```

**Note**: Exact layout depends on Raydium program version. Refer to:
- [Raydium AMM GitHub](https://github.com/raydium-io/raydium-amm)
- Raydium SDK source code

### Parsing Example

```rust
async fn on_account_update(account_data: Vec<u8>) -> Result<()> {
    // Parse account data
    let amm_info = AmmInfo::parse(&account_data)?;

    // Extract reserves
    let reserve_a = amm_info.coin_vault_balance;
    let reserve_b = amm_info.pc_vault_balance;

    // Calculate price
    let price = (reserve_b as f64 / 10_f64.powi(amm_info.pc_decimals as i32))
        / (reserve_a as f64 / 10_f64.powi(amm_info.coin_decimals as i32));

    println!("Pool price: {}", price);

    Ok(())
}
```

---

## Raydium SDK V2 Example

**Official Example** (from SDK demo):

**GitHub**: [Raydium SDK V2 Demo](https://github.com/raydium-io/raydium-sdk-V2-demo)

**gRPC Pool Monitoring Example**:
```typescript
import { Connection } from '@solana/web3.js';
import { Raydium } from '@raydium-io/raydium-sdk-v2';

const connection = new Connection('https://api.mainnet-beta.solana.com');
const raydium = await Raydium.load({ connection });

// Subscribe to pool account
const POOL_ID = 'AVs9TA4nWDzfPJE9gGVNJMVhcQy3V9PGazuz33BfG2RA';

connection.onAccountChange(POOL_ID, async (accountInfo) => {
  // Parse pool state from account data
  const poolState = await raydium.liquidity.getPoolInfo(POOL_ID);
  console.log('Pool updated:', poolState);
});
```

**Recommendation**: Study the official examples for best practices.

---

## Summary

### Key Takeaways

1. **No Traditional WebSocket API**: Raydium doesn't provide WebSocket like CEX
2. **Use gRPC Instead**: Geyser gRPC is the recommended real-time solution
3. **Blockchain Monitoring**: Real-time = monitoring Solana blockchain events
4. **Multiple Providers**: Chainstack, Shyft, QuickNode, Triton, bloXroute, Helius
5. **Account Parsing Required**: Must parse raw Raydium program account data
6. **Hybrid Approach**: Combine REST (initial), gRPC (updates), cache (local)

### Implementation Recommendations

| Use Case | Recommended Approach |
|----------|---------------------|
| **Read-only monitoring** | REST API polling (1-5 min intervals) |
| **Low-latency monitoring** | gRPC (Geyser provider) |
| **High-frequency trading** | gRPC + local caching + on-chain transactions |
| **Simple dashboard** | REST API polling |
| **Production bot** | Hybrid (REST + gRPC + cache) |

### Not Recommended

- ❌ Native Solana RPC WebSocket (use Geyser instead)
- ❌ High-frequency REST polling (use gRPC)
- ❌ Expecting CEX-like WebSocket (doesn't exist)

---

## Sources

Research compiled from the following sources:

- [Raydium API Documentation](https://docs.raydium.io/raydium/for-developers/api)
- [Raydium SDK V2 GitHub](https://github.com/raydium-io/raydium-sdk-V2)
- [Raydium SDK V2 Demo](https://github.com/raydium-io/raydium-sdk-V2-demo)
- [Chainstack Geyser Guide](https://chainstack.com/solana-geyser-raydium-bonk/)
- [Shyft gRPC Network](https://blogs.shyft.to/how-to-stream-and-parse-raydium-transactions-with-shyfts-grpc-network-b16d5b3af249)
- [QuickNode Yellowstone Geyser](https://www.quicknode.com/guides/solana-development/tooling/geyser/yellowstone-rust)
- [Helius Raydium Monitoring](https://www.helius.dev/blog/how-to-monitor-a-raydium-liquidity-pool)
- [bloXroute Raydium Streams](https://docs.bloxroute.com/solana/trader-api/api-endpoints/raydium/getnewraydiumpoolsstream)
- [Solana WebSocket Documentation](https://docs.solana.com/developing/clients/jsonrpc-api#subscription-websocket)

---

**Document Version**: 1.0
**Last Updated**: 2026-01-20
**Research Completed By**: Claude Code Research Agent
