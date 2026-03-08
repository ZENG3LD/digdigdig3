# Raydium DEX API Endpoints Research

**Research Date**: 2026-01-20

This document contains comprehensive research on Raydium's API endpoints. Raydium is a Solana-based automated market maker (AMM) and DEX that combines on-chain smart contracts with off-chain APIs for data access.

---

## Table of Contents

- [Important Note: DEX Architecture](#important-note-dex-architecture)
- [Base URLs](#base-urls)
- [API V3 Endpoints](#api-v3-endpoints)
- [Trade API Endpoints](#trade-api-endpoints)
- [Legacy V2 Endpoints](#legacy-v2-endpoints)
- [On-Chain Program IDs](#on-chain-program-ids)
- [SDK Methods vs API Endpoints](#sdk-methods-vs-api-endpoints)

---

## Important Note: DEX Architecture

**Critical Distinction: Raydium is a DEX (Decentralized Exchange)**

Unlike centralized exchanges (CEX) like KuCoin or Binance, Raydium operates as a decentralized exchange on Solana. This means:

1. **No Trading Accounts**: Users interact directly with smart contracts using their Solana wallets
2. **No Balance Endpoints**: Balances are queried from the Solana blockchain, not Raydium's API
3. **No Order Management**: Swaps are atomic on-chain transactions, not order book operations
4. **API Purpose**: APIs provide **read-only data** (pool info, prices, routing) for monitoring
5. **Execution Method**: Trading happens via:
   - Direct on-chain transaction signing (using SDK)
   - Trade API that serializes transactions for user signature
   - Smart contract calls to Raydium program IDs

**What Raydium APIs Do:**
- Provide pool data (liquidity, TVL, APY)
- Return token lists and metadata
- Calculate optimal swap routes
- Serialize swap transactions (Trade API)
- Monitor farm/yield opportunities

**What Raydium APIs Don't Do:**
- Execute trades (user must sign transactions)
- Manage user accounts or balances
- Place/cancel orders (AMM model, not order book)
- Require authentication (public read-only APIs)

---

## Base URLs

### API V3 (Primary Data API)

| Environment | Base URL | Documentation |
|------------|----------|---------------|
| **Mainnet** | `https://api-v3.raydium.io/` | `https://api-v3.raydium.io/docs/` |
| **Devnet** | `https://api-v3-devnet.raydium.io/` | `https://api-v3-devnet.raydium.io/docs/` |

**Purpose**: Access to pool data, token info, farm data, and IDO information. Read-only monitoring and data retrieval.

**Note**: APIs are for data access and monitoring — **not real-time tracking**. For real-time pool creation events, use gRPC subscriptions via Solana Geyser plugin.

### Trade API (Transaction Serialization)

| Purpose | Base URL |
|---------|----------|
| **Swap Routing** | `https://transaction-v1.raydium.io/` |
| **Priority Fees** | `https://api-v3.raydium.io/` |

**Purpose**: Calculate swap quotes and serialize transactions for user signature. Does NOT execute trades.

---

## API V3 Endpoints

### Main - General Platform Info

#### 1. Get API Version

- **Endpoint**: `GET /main/version`
- **Full URL**: `https://api-v3.raydium.io/main/version`
- **Auth Required**: No (Public)
- **Description**: Fetches current version of Raydium UI V3
- **Response**: Version information

#### 2. Get RPC Endpoints

- **Endpoint**: `GET /main/rpcs`
- **Full URL**: `https://api-v3.raydium.io/main/rpcs`
- **Auth Required**: No (Public)
- **Description**: Fetches list of recommended RPC endpoints for Raydium UI
- **Response**: Array of Solana RPC URLs

#### 3. Get Priority Fees

- **Endpoint**: `GET /main/auto-fee`
- **Full URL**: `https://api-v3.raydium.io/main/auto-fee`
- **Auth Required**: No (Public)
- **Description**: Retrieves auto fee information for transaction priority
- **Response Format**:
  ```json
  {
    "id": "string",
    "success": true,
    "data": {
      "default": {
        "vh": 1000000,
        "h": 500000,
        "m": 100000
      }
    }
  }
  ```
- **Fields**:
  - `vh`: Very high priority (microLamports)
  - `h`: High priority (microLamports)
  - `m`: Medium priority (microLamports)

---

### Mint - Token Information

#### 4. Get Token List

- **Endpoint**: `GET /mint/list`
- **Full URL**: `https://api-v3.raydium.io/mint/list`
- **Auth Required**: No (Public)
- **Description**: Retrieves Raydium's default token list (mainnet only)
- **Response**: Array of token metadata including mint addresses, decimals, symbols
- **SDK Method**: `raydium.api.getTokenList()`

#### 5. Get Token Info by Mints

- **Endpoint**: `GET /mint/ids`
- **Full URL**: `https://api-v3.raydium.io/mint/ids`
- **Auth Required**: No (Public)
- **Query Parameters**:
  - `mints` (required, string): Comma-separated token mint addresses
- **Description**: Gets detailed info for specific tokens recognized by Raydium
- **Example**: `/mint/ids?mints=So11111111111111111111111111111111111111112,EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v`
- **SDK Method**: `raydium.api.getTokenInfo([mint1, mint2])`

#### 6. Get Token Prices

- **Endpoint**: `GET /mint/price`
- **Full URL**: `https://api-v3.raydium.io/mint/price`
- **Auth Required**: No (Public)
- **Query Parameters**:
  - `mints` (optional, string): Comma-separated mint addresses
- **Description**: Fetches current prices for specified tokens
- **Response**: Price data indexed by mint address

---

### Pools - Liquidity Pool Data

#### 7. Get Pool List

- **Endpoint**: `GET /pools/info/list`
- **Full URL**: `https://api-v3.raydium.io/pools/info/list`
- **Auth Required**: No (Public)
- **Query Parameters**:
  - `type` (optional): Pool type filter (All, Standard, Concentrated, etc.)
  - `sort` (optional): Sort field (liquidity, volume24h, apr24h, etc.)
  - `order` (optional): Sort order (asc, desc)
  - `page` (optional, int): Page number
  - `pageSize` (optional, int): Results per page
- **Description**: Fetches pool list with optional filtering and pagination (mainnet only)
- **Response Format**: Paginated pool data with TVL, volume, APR
- **SDK Method**: `raydium.api.getPoolList({})`

#### 8. Get Pool by ID

- **Endpoint**: `GET /pools/info/ids`
- **Full URL**: `https://api-v3.raydium.io/pools/info/ids`
- **Auth Required**: No (Public)
- **Query Parameters**:
  - `ids` (required, string): Comma-separated pool IDs
- **Description**: Retrieves specific pool information by IDs
- **Example**: `/pools/info/ids?ids=AVs9TA4nWDzfPJE9gGVNJMVhcQy3V9PGazuz33BfG2RA`
- **SDK Method**: `raydium.api.fetchPoolById({ ids: "poolId" })`
- **Response Fields** (key fields):
  - `id`: Pool ID
  - `mintA`, `mintB`: Token mint addresses
  - `mintAmountA`, `mintAmountB`: Reserve amounts
  - `price`: Price ratio
  - `tvl`: Total value locked
  - `volume24h`: 24-hour volume
  - `apr24h`: 24-hour APR

#### 9. Get Pools by Mint Pair

- **Endpoint**: `GET /pools/info/mint`
- **Full URL**: `https://api-v3.raydium.io/pools/info/mint`
- **Auth Required**: No (Public)
- **Query Parameters**:
  - `mint1` (required, string): First token mint address
  - `mint2` (optional, string): Second token mint address
  - `type` (optional): Pool type filter
  - `sort` (optional): Sort field
  - `order` (optional): Sort order
  - `page` (optional, int): Page number
- **Description**: Finds pools containing specified token pair
- **Example**: `/pools/info/mint?mint1=So11111111111111111111111111111111111111112&mint2=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v`
- **SDK Method**: `raydium.api.fetchPoolByMints({ mint1, mint2 })`

#### 10. Get Pool Positions

- **Endpoint**: `GET /pools/position/list`
- **Full URL**: `https://api-v3.raydium.io/pools/position/list`
- **Auth Required**: No (Public)
- **Query Parameters**:
  - `poolId` (optional, string): Filter by pool ID
  - `owner` (optional, string): Filter by owner address
- **Description**: Fetches liquidity positions for pools

---

### Farms - Yield Farming Data

#### 11. Get Farm List

- **Endpoint**: `GET /farms/info/list`
- **Full URL**: `https://api-v3.raydium.io/farms/info/list`
- **Auth Required**: No (Public)
- **Description**: Retrieves list of active farms with APY and TVL data
- **Response**: Array of farm pools with staking info

#### 12. Get Farm Info by ID

- **Endpoint**: `GET /farms/info/ids`
- **Full URL**: `https://api-v3.raydium.io/farms/info/ids`
- **Auth Required**: No (Public)
- **Query Parameters**:
  - `ids` (required, string): Comma-separated farm IDs
- **Description**: Gets detailed farm data for specific IDs (mainnet only)
- **Example**: `/farms/info/ids?ids=4EwbZo8BZXP5313z5A2H11MRBP15M5n6YxfmkjXESKAW`
- **SDK Method**: `raydium.api.fetchFarmInfoById({ ids: "farmId" })`

---

### IDO - Initial DEX Offering

#### 13. Get IDO Pool Keys

- **Endpoint**: `GET /ido/pool-keys`
- **Full URL**: `https://api-v3.raydium.io/ido/pool-keys`
- **Auth Required**: No (Public)
- **Description**: Retrieves Initial DEX Offering pool keys
- **Response**: IDO pool configuration data

---

## Trade API Endpoints

### Quote and Swap Serialization

#### 14. Get Swap Quote (Base-In)

- **Endpoint**: `GET /compute/swap-base-in`
- **Full URL**: `https://transaction-v1.raydium.io/compute/swap-base-in`
- **Auth Required**: No (Public)
- **Description**: Returns a quote for swapping tokens (specify input amount)
- **Query Parameters**:
  - `inputMint` (required, string): Source token address
  - `outputMint` (required, string): Destination token address
  - `amount` (required, string): Quantity in base units (lamports)
  - `slippageBps` (required, int): Tolerance in basis points (50 = 0.5%)
  - `txVersion` (required, enum): Transaction version ("V0" or "LEGACY")
- **Example**: `/compute/swap-base-in?inputMint=So11...&outputMint=EPjF...&amount=1000000000&slippageBps=50&txVersion=V0`
- **Response**: Quote object with pricing and route information

#### 15. Get Swap Quote (Base-Out)

- **Endpoint**: `GET /compute/swap-base-out`
- **Full URL**: `https://transaction-v1.raydium.io/compute/swap-base-out`
- **Auth Required**: No (Public)
- **Description**: Returns quote specifying exact output amount (slippage on input)
- **Query Parameters**: Same as base-in, but `amount` specifies desired output
- **Use Case**: When you want to receive an exact amount of output token

#### 16. Serialize Swap Transaction (Base-In)

- **Endpoint**: `POST /transaction/swap-base-in`
- **Full URL**: `https://transaction-v1.raydium.io/transaction/swap-base-in`
- **Auth Required**: No (Public)
- **HTTP Method**: POST
- **Content-Type**: `application/json`
- **Description**: Constructs executable transaction from quote for user to sign
- **Request Body Parameters**:
  - `swapResponse` (required, object): Quote output from compute endpoint
  - `wallet` (required, string): User's Solana public key
  - `txVersion` (required, enum): "V0" or "LEGACY"
  - `wrapSol` (optional, boolean): Wrap SOL to wSOL if needed
  - `unwrapSol` (optional, boolean): Unwrap wSOL to SOL after swap
  - `inputAccount` (optional, string): Source token account (omit for SOL)
  - `outputAccount` (optional, string): Destination token account (omit for SOL)
  - `computeUnitPriceMicroLamports` (optional, int): Priority fee setting
- **Response**: Serialized transaction for user signature
- **Important**: Does NOT execute the swap. User must sign and submit transaction to blockchain.

#### 17. Serialize Swap Transaction (Base-Out)

- **Endpoint**: `POST /transaction/swap-base-out`
- **Full URL**: `https://transaction-v1.raydium.io/transaction/swap-base-out`
- **Auth Required**: No (Public)
- **HTTP Method**: POST
- **Description**: Same as base-in but for exact output swaps
- **Request Body**: Same parameters as base-in

---

## Legacy V2 Endpoints

These are older endpoints that may still be functional but are superseded by V3 API.

### Legacy Data Endpoints

#### 18. Liquidity Pool Data (V2)

- **Endpoint**: N/A (Static JSON file)
- **Full URL**: `https://api.raydium.io/v2/sdk/liquidity/mainnet.json`
- **Auth Required**: No (Public)
- **Description**: Static JSON file with all liquidity pool pubkeys and configuration
- **Format**: Array of pool objects with full on-chain addresses

#### 19. Token List - Solana Format (V2)

- **Endpoint**: N/A (Static JSON file)
- **Full URL**: `https://api.raydium.io/v2/sdk/token/solana.mainnet.json`
- **Auth Required**: No (Public)
- **Description**: Token list in Solana token-list standard format

#### 20. Token List - Raydium Format (V2)

- **Endpoint**: N/A (Static JSON file)
- **Full URL**: `https://api.raydium.io/v2/sdk/token/raydium.mainnet.json`
- **Auth Required**: No (Public)
- **Description**: Token list in Raydium's custom format

#### 21. Token Prices (V2)

- **Endpoint**: `GET /coin/price`
- **Full URL**: `https://api.raydium.io/coin/price`
- **Auth Required**: No (Public)
- **Description**: Legacy endpoint for token prices
- **Status**: Likely superseded by V3 `/mint/price`

#### 22. Trading Pairs Info (V2)

- **Endpoint**: `GET /pairs`
- **Full URL**: `https://api.raydium.io/pairs`
- **Auth Required**: No (Public)
- **Description**: Legacy endpoint for trading pair information and fees
- **Status**: Likely superseded by V3 pool endpoints

---

## On-Chain Program IDs

These are Solana program addresses for direct on-chain interaction:

### Raydium Programs

| Program | Address |
|---------|---------|
| **Raydium Liquidity Pool V4** | `675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8` |
| **Raydium Authority V4** | `5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1` |
| **Raydium Launchpad** | `LanMV9sAd7wArD4vJFi2qDdfnVhFxYSUg6eADduJ3uj` |

**Usage**: These program IDs are used when:
- Building transactions with SDK
- Monitoring program accounts via Geyser gRPC
- Querying on-chain state directly
- Creating/managing liquidity pools

**Note**: Direct program interaction requires Solana RPC and SDK. The REST APIs abstract these interactions.

---

## SDK Methods vs API Endpoints

The Raydium SDK V2 wraps API calls with convenient TypeScript methods:

| SDK Method | Corresponding Endpoint | Purpose |
|------------|------------------------|---------|
| `raydium.api.getTokenList()` | `GET /mint/list` | Fetch token list |
| `raydium.api.getTokenInfo([mints])` | `GET /mint/ids?mints=...` | Get token metadata |
| `raydium.api.getPoolList({})` | `GET /pools/info/list` | List all pools |
| `raydium.api.fetchPoolById({ ids })` | `GET /pools/info/ids?ids=...` | Get specific pools |
| `raydium.api.fetchPoolByMints({ mint1, mint2 })` | `GET /pools/info/mint?mint1=...&mint2=...` | Find pools by token pair |
| `raydium.api.fetchFarmInfoById({ ids })` | `GET /farms/info/ids?ids=...` | Get farm data |

**SDK Advantages**:
- Built-in TypeScript types
- Response normalization
- Automatic pagination handling
- Connection management
- On-chain transaction building
- Integrated swap execution

---

## Real-Time Data: gRPC vs REST API

### REST API Limitations

The official documentation states:
> "APIs are for data access and monitoring — not real-time tracking."

REST APIs have:
- Polling-based updates (not push-based)
- Caching delays (data may be minutes old)
- Rate limits on request frequency
- Not suitable for real-time pool monitoring

### gRPC Alternative (Recommended for Real-Time)

For real-time pool creation and updates, use **Solana Geyser gRPC**:

**Providers**:
- **Shyft**: gRPC network for Raydium transactions
- **Chainstack**: Geyser-based subscribers (hundreds of milliseconds faster)
- **QuickNode**: Yellowstone Geyser gRPC marketplace add-on
- **Triton One**: Program-specific real-time streams
- **bloXroute**: WebSocket subscriptions for pool streams

**Advantages**:
- Sub-second latency
- Push-based updates (no polling)
- Direct on-chain event monitoring
- Filter by program ID
- Transaction streaming

**Example Use Case**: Monitor new Raydium pool creation by subscribing to `675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8` program account updates.

**Reference**: See Raydium SDK V2 demo repository for gRPC examples.

---

## Rate Limits

### Official Information

The documentation mentions:
> "Services within Raydium API have a quota so that services are not compromised."

However, **specific rate limit numbers are not publicly documented**:
- No explicit requests/second limit stated
- No documented rate limit headers
- No public quota information per endpoint

### Best Practices

1. **Implement caching** to reduce API call frequency
2. **Use batch endpoints** when fetching multiple resources (e.g., comma-separated IDs)
3. **Avoid rapid polling** - APIs are cached and not real-time
4. **Use gRPC/WebSocket** for real-time data needs instead of polling REST
5. **Respect 429 errors** if returned (though not documented)

### Provider Alternatives

Third-party RPC providers like Chainstack offer:
- No rate limits on request rate (with subscription)
- Dedicated endpoints
- Higher throughput

---

## Endpoint Summary Table

| Category | Count | Auth | Real-Time | Purpose |
|----------|-------|------|-----------|---------|
| **Main** | 3 | No | No | Platform info, RPC list, fees |
| **Mint** | 3 | No | No | Token list, metadata, prices |
| **Pools** | 4 | No | No | Pool data, liquidity, positions |
| **Farms** | 2 | No | No | Yield farming info |
| **IDO** | 1 | No | No | IDO pool keys |
| **Trade API** | 4 | No | No | Quote & transaction serialization |
| **Legacy V2** | 5 | No | No | Deprecated/superseded endpoints |

**Total REST Endpoints**: 22 documented endpoints

---

## Important Architectural Notes

### 1. No Authentication Required

All Raydium APIs are **public and read-only**:
- No API keys needed
- No authentication headers
- No user accounts
- Anyone can query data

**Why?** Because Raydium is a DEX. Trading happens on-chain via wallet signatures, not through authenticated API calls.

### 2. No Trading Execution via API

The Trade API only:
- **Calculates** swap quotes
- **Serializes** transactions
- **Returns** unsigned transactions

**It does NOT**:
- Execute swaps
- Sign transactions
- Require user credentials

**User must**:
- Sign transaction with their Solana wallet
- Submit signed transaction to Solana blockchain
- Pay gas fees in SOL

### 3. Data is Cached

API responses are cached for performance:
- Pool data may be several minutes old
- Not suitable for arbitrage or HFT
- Use gRPC for sub-second updates

### 4. Mainnet Only for Most Endpoints

Many endpoints only support mainnet:
- Pool list (mainnet only)
- Farm data (mainnet only)
- Token info (mainnet only)
- Devnet API exists but has limited functionality

---

## Migration Notes: CEX to DEX Connector

If adapting a CEX connector pattern (like KuCoin) to Raydium, note these key differences:

| Feature | CEX (KuCoin) | DEX (Raydium) |
|---------|--------------|---------------|
| **Account Balance** | `GET /api/v1/accounts` | Query Solana blockchain via RPC |
| **Place Order** | `POST /api/v1/orders` | Build tx with SDK + user signs + submit to blockchain |
| **Cancel Order** | `DELETE /api/v1/orders/{id}` | N/A (AMM swaps are atomic) |
| **Order History** | `GET /api/v1/orders` | Query on-chain transaction history |
| **Authentication** | HMAC-SHA256 with API keys | Wallet signature (ed25519) |
| **Market Data** | REST API (ticker, orderbook, klines) | REST API (pools, prices) + gRPC |
| **Real-Time Updates** | WebSocket (ticker, trades, balance) | gRPC (pool events, transactions) |
| **Execution Model** | Centralized order book | AMM liquidity pools |

**Key Insight**: A Raydium connector is primarily a **data monitoring tool**, not a trading execution system. Execution requires Solana wallet integration and transaction signing.

---

## Sources

Research compiled from the following official sources:

- [Raydium API Documentation](https://docs.raydium.io/raydium/for-developers/api)
- [Raydium Trade API Documentation](https://docs.raydium.io/raydium/for-developers/trade-api)
- [Raydium Developers Page](https://docs.raydium.io/raydium/protocol/developers)
- [Raydium API V3 Swagger](https://api-v3.raydium.io/docs/)
- [Raydium SDK V2 GitHub](https://github.com/raydium-io/raydium-sdk-V2)
- [Raydium SDK V2 Demo GitHub](https://github.com/raydium-io/raydium-sdk-V2-demo)
- [Raydium AMM GitHub](https://github.com/raydium-io/raydium-amm)
- [Solana Raydium API - Bitquery](https://docs.bitquery.io/docs/blockchain/Solana/Solana-Raydium-DEX-API/)
- [Real-time Solana analytics - Chainstack](https://chainstack.com/solana-geyser-raydium-bonk/)
- [Monitor Raydium Liquidity Pool - Helius](https://www.helius.dev/blog/how-to-monitor-a-raydium-liquidity-pool)

---

**Document Version**: 1.0
**Last Updated**: 2026-01-20
**Research Completed By**: Claude Code Research Agent
