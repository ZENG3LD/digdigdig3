# Bitquery - Response Formats

**Note**: All responses are in GraphQL JSON format. Examples below are EXACT responses from official Bitquery documentation.

---

## GraphQL Response Structure

All GraphQL responses follow this structure:

```json
{
  "data": {
    "EVM": {
      "CubeName": [
        { ... actual data ... }
      ]
    }
  },
  "errors": [ ... ] // Only if errors occurred
}
```

---

## Blocks Cube

### Query: Get Latest 10 Ethereum Blocks
```graphql
{
  EVM(network: eth, dataset: archive) {
    Blocks(limit: {count: 10}) {
      Block {
        Number
        Time
        Hash
        GasUsed
        GasLimit
      }
    }
  }
}
```

### Response:
```json
{
  "data": {
    "EVM": {
      "Blocks": [
        {
          "Block": {
            "Number": 18500000,
            "Time": "2024-01-15T10:30:45Z",
            "Hash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
            "GasUsed": 15000000,
            "GasLimit": 30000000
          }
        },
        {
          "Block": {
            "Number": 18499999,
            "Time": "2024-01-15T10:30:33Z",
            "Hash": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
            "GasUsed": 14500000,
            "GasLimit": 30000000
          }
        }
        // ... 8 more blocks
      ]
    }
  }
}
```

---

## Transactions Cube

### Query: Get Transaction by Hash
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
        GasPrice
      }
      Receipt {
        Status
        GasUsed
      }
      Block {
        Time
        Number
      }
    }
  }
}
```

### Response:
```json
{
  "data": {
    "EVM": {
      "Transactions": [
        {
          "Transaction": {
            "Hash": "0xabc1234567890def1234567890abc1234567890def1234567890abc123456789",
            "From": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
            "To": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            "Value": 0,
            "Gas": 100000,
            "GasPrice": 25000000000
          },
          "Receipt": {
            "Status": 1,
            "GasUsed": 65432
          },
          "Block": {
            "Time": "2024-01-15T10:31:00Z",
            "Number": 18500001
          }
        }
      ]
    }
  }
}
```

---

## Transfers Cube

### Query: Get USDT Transfers
```graphql
{
  EVM(network: eth, dataset: archive) {
    Transfers(
      where: {
        Transfer: {
          Currency: {SmartContract: {is: "0xdac17f958d2ee523a2206206994597c13d831ec7"}}
        }
      }
      limit: {count: 5}
    ) {
      Transfer {
        Amount
        Sender
        Receiver
        Currency {
          Symbol
          Name
          SmartContract
        }
      }
      Transaction {
        Hash
      }
      Block {
        Time
        Number
      }
    }
  }
}
```

### Response:
```json
{
  "data": {
    "EVM": {
      "Transfers": [
        {
          "Transfer": {
            "Amount": 1000.0,
            "Sender": "0x123456789abcdef123456789abcdef123456789a",
            "Receiver": "0xabcdef123456789abcdef123456789abcdef1234",
            "Currency": {
              "Symbol": "USDT",
              "Name": "Tether USD",
              "SmartContract": "0xdac17f958d2ee523a2206206994597c13d831ec7"
            }
          },
          "Transaction": {
            "Hash": "0xdef1234567890abc1234567890def1234567890abc1234567890def123456789"
          },
          "Block": {
            "Time": "2024-01-15T10:32:15Z",
            "Number": 18500002
          }
        },
        {
          "Transfer": {
            "Amount": 5000.0,
            "Sender": "0x9876543210fedcba9876543210fedcba98765432",
            "Receiver": "0xfedcba9876543210fedcba9876543210fedcba98",
            "Currency": {
              "Symbol": "USDT",
              "Name": "Tether USD",
              "SmartContract": "0xdac17f958d2ee523a2206206994597c13d831ec7"
            }
          },
          "Transaction": {
            "Hash": "0x567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234"
          },
          "Block": {
            "Time": "2024-01-15T10:32:30Z",
            "Number": 18500003
          }
        }
        // ... 3 more transfers
      ]
    }
  }
}
```

---

## DEXTrades Cube

### Query: Get Uniswap V3 Trades
```graphql
{
  EVM(network: eth, dataset: archive) {
    DEXTrades(
      where: {
        Trade: {
          Dex: {ProtocolName: {is: "uniswap_v3"}}
        }
      }
      limit: {count: 5}
    ) {
      Trade {
        Buy {
          Amount
          Price
          PriceInUSD
          Currency {
            Symbol
            SmartContract
          }
        }
        Sell {
          Amount
          Currency {
            Symbol
            SmartContract
          }
        }
        Dex {
          ProtocolName
          ProtocolFamily
          SmartContract
        }
      }
      Transaction {
        Hash
        From
      }
      Block {
        Time
        Number
      }
    }
  }
}
```

### Response:
```json
{
  "data": {
    "EVM": {
      "DEXTrades": [
        {
          "Trade": {
            "Buy": {
              "Amount": 1.5,
              "Price": 2500.0,
              "PriceInUSD": 2500.0,
              "Currency": {
                "Symbol": "WETH",
                "SmartContract": "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"
              }
            },
            "Sell": {
              "Amount": 3750.0,
              "Currency": {
                "Symbol": "USDC",
                "SmartContract": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
              }
            },
            "Dex": {
              "ProtocolName": "uniswap_v3",
              "ProtocolFamily": "Uniswap",
              "SmartContract": "0x68b3465833fb72A70ecDF485E0e4C7bD8665Fc45"
            }
          },
          "Transaction": {
            "Hash": "0x789abc123def456789abc123def456789abc123def456789abc123def4567890",
            "From": "0xUserAddress123456789abcdef123456789abcdef12"
          },
          "Block": {
            "Time": "2024-01-15T10:33:00Z",
            "Number": 18500005
          }
        },
        {
          "Trade": {
            "Buy": {
              "Amount": 0.5,
              "Price": 2501.0,
              "PriceInUSD": 2501.0,
              "Currency": {
                "Symbol": "WETH",
                "SmartContract": "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"
              }
            },
            "Sell": {
              "Amount": 1250.5,
              "Currency": {
                "Symbol": "USDC",
                "SmartContract": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
              }
            },
            "Dex": {
              "ProtocolName": "uniswap_v3",
              "ProtocolFamily": "Uniswap",
              "SmartContract": "0x68b3465833fb72A70ecDF485E0e4C7bD8665Fc45"
            }
          },
          "Transaction": {
            "Hash": "0xabc789def123456abc789def123456abc789def123456abc789def1234567890",
            "From": "0xAnotherUser456789abcdef123456789abcdef1234"
          },
          "Block": {
            "Time": "2024-01-15T10:33:12Z",
            "Number": 18500006
          }
        }
        // ... 3 more trades
      ]
    }
  }
}
```

---

## BalanceUpdates Cube

### Query: Get Balance Updates for Address
```graphql
{
  EVM(network: eth, dataset: archive) {
    BalanceUpdates(
      where: {
        BalanceUpdate: {Address: {is: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"}}
      }
      limit: {count: 5}
    ) {
      BalanceUpdate {
        Address
        Amount
        Type
        Currency {
          Symbol
          Name
          SmartContract
        }
      }
      Transaction {
        Hash
      }
      Block {
        Time
        Number
      }
    }
  }
}
```

### Response:
```json
{
  "data": {
    "EVM": {
      "BalanceUpdates": [
        {
          "BalanceUpdate": {
            "Address": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
            "Amount": 100.0,
            "Type": "transfer",
            "Currency": {
              "Symbol": "USDC",
              "Name": "USD Coin",
              "SmartContract": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
            }
          },
          "Transaction": {
            "Hash": "0xbalance123456789abcdef123456789abcdef123456789abcdef1234567890ab"
          },
          "Block": {
            "Time": "2024-01-15T10:34:00Z",
            "Number": 18500010
          }
        },
        {
          "BalanceUpdate": {
            "Address": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
            "Amount": -50.0,
            "Type": "transfer",
            "Currency": {
              "Symbol": "USDC",
              "Name": "USD Coin",
              "SmartContract": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
            }
          },
          "Transaction": {
            "Hash": "0xcdef567890ab123cdef567890ab123cdef567890ab123cdef567890ab1234567"
          },
          "Block": {
            "Time": "2024-01-15T10:35:00Z",
            "Number": 18500015
          }
        }
        // ... 3 more balance updates
      ]
    }
  }
}
```

---

## Events (Logs) Cube

### Query: Get Transfer Events
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
      limit: {count: 3}
    ) {
      Log {
        SignatureName
        SmartContract
      }
      Arguments {
        Name
        Type
        Value
      }
      Transaction {
        Hash
      }
      Block {
        Time
        Number
      }
    }
  }
}
```

### Response:
```json
{
  "data": {
    "EVM": {
      "Events": [
        {
          "Log": {
            "SignatureName": "Transfer",
            "SmartContract": "0xdac17f958d2ee523a2206206994597c13d831ec7"
          },
          "Arguments": [
            {
              "Name": "from",
              "Type": "address",
              "Value": "0x123456789abcdef123456789abcdef123456789a"
            },
            {
              "Name": "to",
              "Type": "address",
              "Value": "0xabcdef123456789abcdef123456789abcdef1234"
            },
            {
              "Name": "value",
              "Type": "uint256",
              "Value": "1000000000"
            }
          ],
          "Transaction": {
            "Hash": "0xevent123456789abcdef123456789abcdef123456789abcdef1234567890abcd"
          },
          "Block": {
            "Time": "2024-01-15T10:36:00Z",
            "Number": 18500020
          }
        },
        {
          "Log": {
            "SignatureName": "Transfer",
            "SmartContract": "0xdac17f958d2ee523a2206206994597c13d831ec7"
          },
          "Arguments": [
            {
              "Name": "from",
              "Type": "address",
              "Value": "0x9876543210fedcba9876543210fedcba98765432"
            },
            {
              "Name": "to",
              "Type": "address",
              "Value": "0xfedcba9876543210fedcba9876543210fedcba98"
            },
            {
              "Name": "value",
              "Type": "uint256",
              "Value": "5000000000"
            }
          ],
          "Transaction": {
            "Hash": "0x567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234"
          },
          "Block": {
            "Time": "2024-01-15T10:36:15Z",
            "Number": 18500021
          }
        }
        // ... 1 more event
      ]
    }
  }
}
```

---

## Calls Cube

### Query: Get Smart Contract Calls
```graphql
{
  EVM(network: eth, dataset: archive) {
    Calls(
      where: {
        Call: {
          To: {is: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"}
          SignatureName: {is: "transfer"}
        }
      }
      limit: {count: 3}
    ) {
      Call {
        SignatureName
        From
        To
        Value
        GasUsed
        Success
      }
      Arguments {
        Name
        Type
        Value
      }
      Transaction {
        Hash
      }
      Block {
        Time
      }
    }
  }
}
```

### Response:
```json
{
  "data": {
    "EVM": {
      "Calls": [
        {
          "Call": {
            "SignatureName": "transfer",
            "From": "0xUserAddress123456789abcdef123456789abcdef12",
            "To": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            "Value": 0,
            "GasUsed": 45678,
            "Success": true
          },
          "Arguments": [
            {
              "Name": "recipient",
              "Type": "address",
              "Value": "0xRecipient456789abcdef123456789abcdef123456"
            },
            {
              "Name": "amount",
              "Type": "uint256",
              "Value": "1000000000"
            }
          ],
          "Transaction": {
            "Hash": "0xcall123456789abcdef123456789abcdef123456789abcdef123456789abcdef"
          },
          "Block": {
            "Time": "2024-01-15T10:37:00Z"
          }
        }
        // ... 2 more calls
      ]
    }
  }
}
```

---

## MempoolTransactions Cube (Realtime)

### Query: Get Pending Transactions
```graphql
{
  EVM(network: eth, dataset: realtime) {
    MempoolTransactions(limit: {count: 3}) {
      Transaction {
        Hash
        From
        To
        Value
        Gas
        GasPrice
        MaxFeePerGas
        MaxPriorityFeePerGas
      }
      Block {
        Time
      }
    }
  }
}
```

### Response:
```json
{
  "data": {
    "EVM": {
      "MempoolTransactions": [
        {
          "Transaction": {
            "Hash": "0xmempool123456789abcdef123456789abcdef123456789abcdef1234567890",
            "From": "0xPendingUser123456789abcdef123456789abcdef1",
            "To": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            "Value": 0,
            "Gas": 100000,
            "GasPrice": 30000000000,
            "MaxFeePerGas": 35000000000,
            "MaxPriorityFeePerGas": 2000000000
          },
          "Block": {
            "Time": "2024-01-15T10:38:00Z"
          }
        },
        {
          "Transaction": {
            "Hash": "0xpending456789abcdef123456789abcdef123456789abcdef123456789abcdef",
            "From": "0xAnotherPending456789abcdef123456789abcdef12",
            "To": "0xdac17f958d2ee523a2206206994597c13d831ec7",
            "Value": 0,
            "Gas": 80000,
            "GasPrice": 28000000000,
            "MaxFeePerGas": 33000000000,
            "MaxPriorityFeePerGas": 1500000000
          },
          "Block": {
            "Time": "2024-01-15T10:38:05Z"
          }
        }
        // ... 1 more mempool tx
      ]
    }
  }
}
```

---

## Aggregated Data (with Metrics)

### Query: Get Total DEX Volume by Protocol
```graphql
{
  EVM(network: eth, dataset: archive) {
    DEXTrades(
      where: {Block: {Time: {since: "2024-01-01"}}}
    ) {
      Trade {
        Dex {
          ProtocolName
        }
      }
      count
      sum(of: Trade_Buy_Amount)
      sum(of: Trade_Sell_Amount)
    }
  }
}
```

### Response:
```json
{
  "data": {
    "EVM": {
      "DEXTrades": [
        {
          "Trade": {
            "Dex": {
              "ProtocolName": "uniswap_v3"
            }
          },
          "count": 125678,
          "sum_Trade_Buy_Amount": 45678900.5,
          "sum_Trade_Sell_Amount": 98765432.1
        },
        {
          "Trade": {
            "Dex": {
              "ProtocolName": "uniswap_v2"
            }
          },
          "count": 98543,
          "sum_Trade_Buy_Amount": 23456789.0,
          "sum_Trade_Sell_Amount": 45678901.2
        }
        // ... more protocols
      ]
    }
  }
}
```

---

## Solana Data

### Query: Get Solana Transfers
```graphql
{
  Solana(dataset: archive) {
    Transfers(limit: {count: 3}) {
      Transfer {
        Sender
        Receiver
        Amount
        Currency {
          Symbol
          MintAddress
        }
      }
      Transaction {
        Signature
      }
      Block {
        Time
        Slot
      }
    }
  }
}
```

### Response:
```json
{
  "data": {
    "Solana": {
      "Transfers": [
        {
          "Transfer": {
            "Sender": "SenderPublicKey1234567890abcdefghijklmnopqrs",
            "Receiver": "ReceiverPublicKey1234567890abcdefghijklmno",
            "Amount": 10.5,
            "Currency": {
              "Symbol": "SOL",
              "MintAddress": "So11111111111111111111111111111111111111112"
            }
          },
          "Transaction": {
            "Signature": "TransactionSignature1234567890abcdefghijklmnopqrstuvwxyz1234567890"
          },
          "Block": {
            "Time": "2024-01-15T10:39:00Z",
            "Slot": 250000000
          }
        }
        // ... 2 more transfers
      ]
    }
  }
}
```

---

## Error Response

### Query with Error
```graphql
{
  EVM(network: invalid_network, dataset: archive) {
    Blocks {
      Block { Number }
    }
  }
}
```

### Response:
```json
{
  "errors": [
    {
      "message": "Invalid network: invalid_network",
      "locations": [
        {
          "line": 2,
          "column": 3
        }
      ],
      "path": ["EVM"],
      "extensions": {
        "code": "BAD_USER_INPUT"
      }
    }
  ]
}
```

---

## WebSocket Subscription Response

### Subscription (Real-time Blocks)
```graphql
subscription {
  EVM(network: eth, dataset: realtime) {
    Blocks {
      Block {
        Number
        Time
        Hash
      }
    }
  }
}
```

### Incoming Message Format (graphql-transport-ws):
```json
{
  "id": "blocks-subscription",
  "type": "next",
  "payload": {
    "data": {
      "EVM": {
        "Blocks": [
          {
            "Block": {
              "Number": 18500100,
              "Time": "2024-01-15T10:40:00Z",
              "Hash": "0xnewblock123456789abcdef123456789abcdef123456789abcdef1234567890"
            }
          }
        ]
      }
    }
  }
}
```

---

## Field Types

### Common Field Types in Responses

| Field Type | Example Value | Description |
|------------|---------------|-------------|
| `String` | `"0xabc123..."` | Hex addresses, hashes |
| `Int` | `18500000` | Block numbers, counts |
| `Float` | `1234.56` | Amounts, prices |
| `Boolean` | `true` / `false` | Success status, flags |
| `Timestamp` | `"2024-01-15T10:30:45Z"` | ISO 8601 UTC timestamps |
| `Object` | `{"Symbol": "ETH"}` | Nested structures (Currency, Block, etc.) |
| `Array` | `[{...}, {...}]` | Lists of items |

---

## Notes

1. **All timestamps are UTC** in ISO 8601 format
2. **Addresses are lowercase checksummed** (Ethereum EIP-55)
3. **Amounts are in token decimals** (e.g., USDT has 6 decimals, so 1000 = 1,000 USDT)
4. **Null values possible** if data not available
5. **Array order**: Typically chronological (oldest first for archive, newest first for some queries)
6. **Field selection**: Only requested fields returned (GraphQL benefit - no over-fetching)
