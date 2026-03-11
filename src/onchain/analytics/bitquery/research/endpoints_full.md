# Bitquery - Complete Endpoint Reference

**IMPORTANT**: Bitquery is a GraphQL API, not REST. It doesn't have traditional "endpoints" but rather **GraphQL Cubes** (data tables) and **Fields**.

All queries go to a single GraphQL endpoint, but access different data through:
- **Cubes**: Multi-dimensional data tables (e.g., DEXTrades, Blocks, Transfers)
- **Dimensions**: Fields for grouping/filtering
- **Metrics**: Aggregation functions (count, sum, max, min, etc.)

## GraphQL Endpoint

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| POST | https://streaming.bitquery.io/graphql | V2 GraphQL API | Yes* | Yes | 10 req/min (free) | *Free tier with limits |
| POST | https://graphql.bitquery.io | V1 GraphQL API (legacy) | Yes* | Yes | 10 req/min (free) | Legacy, use V2 |
| WebSocket | wss://streaming.bitquery.io/graphql | GraphQL Subscriptions | Yes* | Yes | 2 streams (free) | Real-time data |

---

## Category: Blockchain Data Cubes (V2 API)

### EVM Blockchains (Ethereum, BSC, Polygon, Arbitrum, Base, etc.)

#### Blocks Cube
**Description**: Block-level data (block number, timestamp, miner, gas, etc.)

**Available Fields**:
- `Block.Number` - Block number
- `Block.Time` - Block timestamp
- `Block.Hash` - Block hash
- `Block.GasLimit` - Gas limit
- `Block.GasUsed` - Gas used
- `Block.BaseFee` - Base fee (EIP-1559)
- `Block.Coinbase` - Miner/validator address
- `Block.Difficulty` - Mining difficulty
- `Block.Size` - Block size in bytes
- `Block.TxCount` - Transaction count

**Metrics**: `count`, `sum`, `avg`, `min`, `max`

**Example Query**:
```graphql
{
  EVM(network: eth, dataset: archive) {
    Blocks(limit: {count: 10}) {
      Block {
        Number
        Time
        GasUsed
        TxCount
      }
    }
  }
}
```

---

#### Transactions Cube
**Description**: Transaction data (hash, from, to, value, gas, status)

**Available Fields**:
- `Transaction.Hash` - Transaction hash
- `Transaction.From` - Sender address
- `Transaction.To` - Recipient address
- `Transaction.Value` - ETH value
- `Transaction.Gas` - Gas limit
- `Transaction.GasPrice` - Gas price
- `Transaction.GasUsed` - Actual gas used
- `Transaction.Nonce` - Transaction nonce
- `Transaction.Index` - Transaction index in block
- `Transaction.Type` - Transaction type (0=legacy, 2=EIP-1559)
- `Transaction.Cost` - Total cost (gas * gasPrice)
- `Receipt.Status` - Success/failure (1/0)
- `Receipt.GasUsed` - Gas consumed
- `Receipt.EffectiveGasPrice` - Effective gas price

**Filters**: `{Transaction: {Hash: {is: "0x..."}}}`

**Example Query**:
```graphql
{
  EVM(network: eth, dataset: archive) {
    Transactions(
      where: {Transaction: {Hash: {is: "0xabc123..."}}}
    ) {
      Transaction {
        Hash
        From
        To
        Value
        Gas
      }
      Receipt {
        Status
        GasUsed
      }
    }
  }
}
```

---

#### Transfers Cube
**Description**: Token transfers (ERC-20, ERC-721, ERC-1155, native transfers)

**Available Fields**:
- `Transfer.Amount` - Amount transferred
- `Transfer.Sender` - Sender address
- `Transfer.Receiver` - Receiver address
- `Transfer.Currency.Symbol` - Token symbol
- `Transfer.Currency.Name` - Token name
- `Transfer.Currency.SmartContract` - Token contract address
- `Transfer.Currency.Fungible` - Is fungible (true/false)
- `Transfer.Currency.Decimals` - Token decimals
- `Transfer.Type` - Transfer type (transfer, mint, burn)
- `Transfer.Id` - Token ID (for NFTs)
- `Block.Time` - Timestamp
- `Transaction.Hash` - Transaction hash

**Filters**:
- By sender: `{Transfer: {Sender: {is: "0x..."}}}`
- By receiver: `{Transfer: {Receiver: {is: "0x..."}}}`
- By currency: `{Transfer: {Currency: {SmartContract: {is: "0x..."}}}}`

**Example Query**:
```graphql
{
  EVM(network: eth, dataset: archive) {
    Transfers(
      where: {Transfer: {Currency: {SmartContract: {is: "0xdac17f958d2ee523a2206206994597c13d831ec7"}}}}
      limit: {count: 100}
    ) {
      Transfer {
        Amount
        Sender
        Receiver
        Currency {
          Symbol
          Name
        }
      }
      Block {
        Time
      }
    }
  }
}
```

---

#### DEXTrades Cube
**Description**: Decentralized exchange trades (Uniswap, PancakeSwap, SushiSwap, etc.)

**Available Fields**:
- **Buy side** (what pool receives):
  - `Trade.Buy.Amount` - Buy amount
  - `Trade.Buy.Buyer` - Pool address
  - `Trade.Buy.Currency.Symbol` - Bought token
  - `Trade.Buy.Currency.SmartContract` - Token address
  - `Trade.Buy.Price` - Price in quote currency
  - `Trade.Buy.PriceInUSD` - Price in USD
- **Sell side** (what pool gives out):
  - `Trade.Sell.Amount` - Sell amount
  - `Trade.Sell.Seller` - User address
  - `Trade.Sell.Currency.Symbol` - Sold token
  - `Trade.Sell.Currency.SmartContract` - Token address
  - `Trade.Sell.Price` - Price
- **DEX info**:
  - `Trade.Dex.ProtocolName` - Protocol (Uniswap V2, V3, etc.)
  - `Trade.Dex.ProtocolFamily` - Protocol family
  - `Trade.Dex.SmartContract` - DEX contract
  - `Trade.Dex.Pair.SmartContract` - Pair address
- **Transaction**:
  - `Transaction.Hash`
  - `Transaction.From` - Transaction initiator
  - `Block.Time`
  - `Trade.Index` - Trade index (for multi-hop)

**Metrics**: `count`, `sum(Trade.Buy.Amount)`, `avg(Trade.Buy.Price)`

**Example Query** (Uniswap V2 WETH/USDT trades):
```graphql
{
  EVM(network: eth, dataset: archive) {
    DEXTrades(
      where: {
        Trade: {
          Dex: {ProtocolName: {is: "uniswap_v2"}}
          Buy: {Currency: {SmartContract: {is: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"}}}
          Sell: {Currency: {SmartContract: {is: "0xdAC17F958D2ee523a2206206994597C13D831ec7"}}}
        }
      }
      limit: {count: 100}
    ) {
      Trade {
        Buy {
          Amount
          Price
          Currency {
            Symbol
          }
        }
        Sell {
          Amount
          Currency {
            Symbol
          }
        }
        Dex {
          ProtocolName
        }
      }
      Block {
        Time
      }
    }
  }
}
```

---

#### BalanceUpdates Cube
**Description**: Historical and real-time balance changes for addresses

**Available Fields**:
- `BalanceUpdate.Address` - Address
- `BalanceUpdate.Amount` - Balance change amount
- `BalanceUpdate.Type` - Type (transfer, mint, burn, reward, fee)
- `BalanceUpdate.Currency.Symbol`
- `BalanceUpdate.Currency.SmartContract`
- `Block.Time`
- `Transaction.Hash`

**Use Cases**:
- Track wallet balances over time
- NFT ownership tracking
- Token holder snapshots
- Staking rewards
- Miner/validator rewards

**Example Query**:
```graphql
{
  EVM(network: eth, dataset: archive) {
    BalanceUpdates(
      where: {BalanceUpdate: {Address: {is: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"}}}
      limit: {count: 100}
    ) {
      BalanceUpdate {
        Address
        Amount
        Type
        Currency {
          Symbol
        }
      }
      Block {
        Time
      }
    }
  }
}
```

---

#### Events (Logs) Cube
**Description**: Smart contract event logs

**Available Fields**:
- `Log.Signature` - Event signature hash
- `Log.SignatureName` - Decoded event name
- `Log.SmartContract` - Contract address
- `Arguments` - Array of event arguments
  - `Arguments.Name` - Argument name
  - `Arguments.Type` - Argument type
  - `Arguments.Value` - Argument value
- `Transaction.Hash`
- `Block.Time`

**Example Query** (Transfer events):
```graphql
{
  EVM(network: eth, dataset: archive) {
    Events(
      where: {
        Log: {
          Signature: {Name: {is: "Transfer"}}
          SmartContract: {is: "0xdac17f958d2ee523a2206206994597c13d831ec7"}
        }
      }
      limit: {count: 100}
    ) {
      Log {
        SignatureName
        SmartContract
      }
      Arguments {
        Name
        Value
      }
      Block {
        Time
      }
    }
  }
}
```

---

#### Calls Cube
**Description**: Smart contract function calls

**Available Fields**:
- `Call.Signature` - Function signature
- `Call.SignatureName` - Decoded function name
- `Call.From` - Caller address
- `Call.To` - Contract address
- `Call.Value` - ETH value sent
- `Call.GasUsed` - Gas consumed
- `Call.Success` - Call success status
- `Arguments` - Function arguments
- `Transaction.Hash`

**Example Query**:
```graphql
{
  EVM(network: eth, dataset: archive) {
    Calls(
      where: {
        Call: {
          To: {is: "0xdac17f958d2ee523a2206206994597c13d831ec7"}
          SignatureName: {is: "transfer"}
        }
      }
      limit: {count: 100}
    ) {
      Call {
        SignatureName
        From
        To
        Success
      }
      Arguments {
        Name
        Value
      }
    }
  }
}
```

---

#### MempoolTransactions Cube
**Description**: Real-time mempool (pending transactions)

**Available Fields**:
- `Transaction.Hash`
- `Transaction.From`
- `Transaction.To`
- `Transaction.Value`
- `Transaction.Gas`
- `Transaction.GasPrice`
- `Transaction.MaxFeePerGas` (EIP-1559)
- `Transaction.MaxPriorityFeePerGas`
- `Block.Time` - When seen in mempool

**Dataset**: `realtime` only

**Example Query**:
```graphql
{
  EVM(network: eth, dataset: realtime) {
    MempoolTransactions(limit: {count: 50}) {
      Transaction {
        Hash
        From
        To
        Value
        GasPrice
      }
    }
  }
}
```

---

### Solana

#### Solana Blocks Cube
**Available Fields**:
- `Block.Slot`
- `Block.Hash`
- `Block.Time`
- `Block.Height`
- `Block.ParentSlot`

#### Solana Transactions Cube
**Available Fields**:
- `Transaction.Signature`
- `Transaction.Signer`
- `Transaction.FeePayer`
- `Transaction.Fee`
- `Transaction.Success`
- `Instruction.Program.Name`
- `Instruction.Program.Address`

#### Solana Transfers Cube
**Available Fields**:
- `Transfer.Sender`
- `Transfer.Receiver`
- `Transfer.Amount`
- `Transfer.Currency.Symbol`
- `Transfer.Currency.MintAddress`

#### Solana DEXTrades Cube
**Available Fields**:
- `Trade.Buy.Account` - Buyer
- `Trade.Buy.Amount`
- `Trade.Buy.Currency.MintAddress`
- `Trade.Sell.Account` - Seller
- `Trade.Sell.Amount`
- `Trade.Sell.Currency.MintAddress`
- `Trade.Dex.ProtocolName` (Raydium, Orca, etc.)
- `Trade.Market.MarketAddress` - Pool address

---

### Bitcoin (UTXO-based chains)

#### Bitcoin Blocks Cube
**Available Fields**:
- `Block.Number`
- `Block.Hash`
- `Block.Time`
- `Block.Coinbase` - Miner address
- `Block.Difficulty`

#### Bitcoin Transactions Cube
**Available Fields**:
- `Transaction.Hash`
- `Transaction.Fee`
- `Transaction.InputCount`
- `Transaction.OutputCount`

#### Bitcoin Inputs Cube
**Available Fields**:
- `Input.Value`
- `Input.Address` - Spending address
- `Input.OutputTransaction` - Previous tx
- `Input.ScriptType`

#### Bitcoin Outputs Cube
**Available Fields**:
- `Output.Value`
- `Output.Address` - Receiving address
- `Output.Index`
- `Output.ScriptType`

---

## Category: Metadata & Reference Data

### Symbol/Instrument Lists
**Query**: Use `Transfers` or `DEXTrades` with grouping by `Currency.Symbol`

**Example** (All tokens traded on Uniswap):
```graphql
{
  EVM(network: eth, dataset: archive) {
    DEXTrades(
      where: {Trade: {Dex: {ProtocolName: {is: "uniswap_v2"}}}}
    ) {
      Trade {
        Buy {
          Currency {
            Symbol
            Name
            SmartContract
          }
        }
      }
    }
  }
}
```

---

## Category: Account/Usage Management

### Check Points Usage
**Not available via GraphQL** - Use account dashboard:
- Dashboard URL: https://account.bitquery.io
- Real-time point tracking during queries
- Monthly usage statistics

---

## GraphQL Query Parameters (All Cubes)

### Common Parameters

| Parameter | Type | Description | Example |
|-----------|------|-------------|---------|
| `limit` | `{count: Int, offset: Int}` | Pagination | `limit: {count: 100, offset: 200}` |
| `where` | Filter object | Filtering | `where: {Block: {Number: {gt: 1000000}}}` |
| `orderBy` | `{ascending/descending: Field}` | Sorting | `orderBy: {descending: Block_Time}` |

### Filter Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `is` | Exact match | `{Block: {Number: {is: 12345}}}` |
| `not` | Not equal | `{Transaction: {From: {not: "0x..."}}}` |
| `in` | In array | `{Currency: {Symbol: {in: ["USDT", "USDC"]}}}` |
| `notIn` | Not in array | `{Symbol: {notIn: ["WETH"]}}` |
| `gt` | Greater than | `{Block: {Time: {gt: "2024-01-01"}}}` |
| `lt` | Less than | `{Amount: {lt: 1000000}}}` |
| `ge` | Greater/equal | `{Block: {Number: {ge: 1000000}}}` |
| `le` | Less/equal | `{Block: {Number: {le: 2000000}}}` |
| `since` | Time range start | `{Block: {Time: {since: "2024-01-01"}}}` |
| `till` | Time range end | `{Block: {Time: {till: "2024-12-31"}}}` |

---

## Metrics/Aggregations

Available for most numeric fields:

| Metric | Description | Example |
|--------|-------------|---------|
| `count` | Count records | `count` |
| `sum` | Sum values | `sum(of: Trade_Buy_Amount)` |
| `average` | Average | `average(of: Trade_Buy_Price)` |
| `maximum` | Max value | `maximum(of: Block_GasUsed)` |
| `minimum` | Min value | `minimum(of: Block_GasUsed)` |
| `median` | Median value | `median(of: Trade_Buy_Amount)` |
| `any` | Any value | `any(of: Transaction_Hash)` |
| `uniq` | Unique count | `uniq(of: Transaction_From)` |

**Example with metrics**:
```graphql
{
  EVM(network: eth, dataset: archive) {
    DEXTrades(
      where: {Block: {Time: {since: "2024-01-01"}}}
    ) {
      count
      sum(of: Trade_Buy_Amount)
      average(of: Trade_Buy_PriceInUSD)
    }
  }
}
```

---

## Dataset Selection

### Archive Dataset
- Historical data
- Full blockchain history from genesis
- Slower queries (but still fast)
- Most queries use this

### Realtime Dataset
- Live blockchain data
- Mempool transactions
- Sub-second latency
- Flat cost: 5 points per cube
- Use for subscriptions

### Combined Dataset
- Both archive + realtime
- Not recommended for complex queries
- Higher resource usage

**Example**:
```graphql
{
  EVM(network: eth, dataset: realtime) {
    # ... realtime query
  }
}
```

---

## Supported Networks (network parameter)

### EVM Chains
- `eth` - Ethereum
- `bsc` - Binance Smart Chain
- `polygon` - Polygon
- `arbitrum` - Arbitrum One
- `base` - Base
- `optimism` - Optimism
- `avalanche` - Avalanche C-Chain
- `fantom` - Fantom
- `cronos` - Cronos
- `celo` - Celo
- `moonbeam` - Moonbeam
- `klaytn` - Klaytn

### Non-EVM Chains
- `solana` - Solana
- `bitcoin` - Bitcoin
- `litecoin` - Litecoin
- `bitcoincash` - Bitcoin Cash
- `bitcoinsv` - Bitcoin SV
- `cardano` - Cardano
- `ripple` - Ripple (XRP)
- `stellar` - Stellar
- `algorand` - Algorand
- `cosmos` - Cosmos
- `tron` - Tron
- `eos` - EOS
- `flow` - Flow
- `hedera` - Hedera
- `filecoin` - Filecoin

---

## Notes

1. **Not REST**: Bitquery is GraphQL-only. All requests are POST to single endpoint.
2. **Introspection**: Use GraphQL introspection to discover all fields dynamically.
3. **IDE recommended**: Use https://ide.bitquery.io for schema exploration.
4. **Points cost varies**: Complex queries cost more points.
5. **Real-time vs Archive**: Use `dataset: realtime` for live data, `archive` for historical.
6. **Multi-chain**: Single query can aggregate across multiple networks.
