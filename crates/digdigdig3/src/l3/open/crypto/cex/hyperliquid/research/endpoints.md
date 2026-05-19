# HyperLiquid API Endpoints

## Base URLs

- **Mainnet REST**: `https://api.hyperliquid.xyz`
- **Testnet REST**: `https://api.hyperliquid.xyz`
- **Mainnet WebSocket**: `wss://api.hyperliquid.xyz/ws`
- **Testnet WebSocket**: `wss://api.hyperliquid-testnet.xyz/ws`
- **EVM JSON-RPC**: `https://rpc.hyperliquid.xyz/evm`

## Endpoint Structure

HyperLiquid uses a unified POST endpoint structure:
- **Info Endpoint**: `POST /info` - Read-only market and user data
- **Exchange Endpoint**: `POST /exchange` - Authenticated trading operations

---

# MarketData Trait Endpoints

## Get Server Time

**Method**: POST
**Endpoint**: `/info`
**Request**:
```json
{
  "type": "metaAndAssetCtxs"
}
```
**Response**: Contains timestamp in asset context data
**Notes**: Extract current time from response metadata

---

## Get Exchange Info / Symbols

**Method**: POST
**Endpoint**: `/info`

### Perpetuals Metadata
**Request**:
```json
{
  "type": "meta",
  "dex": ""
}
```
**Response**:
```json
{
  "universe": [
    {
      "name": "BTC",
      "szDecimals": 5,
      "maxLeverage": 50,
      "onlyIsolated": false
    }
  ]
}
```

### Spot Metadata
**Request**:
```json
{
  "type": "spotMeta"
}
```
**Response**:
```json
{
  "universe": [
    {
      "tokens": [0, 1],
      "name": "PURR/USDC",
      "index": 0,
      "isCanonical": true
    }
  ],
  "tokens": [
    {
      "name": "USDC",
      "szDecimals": 8,
      "weiDecimals": 6,
      "index": 0,
      "tokenId": "0x...",
      "isCanonical": true
    }
  ]
}
```

---

## Get Ticker / 24hr Stats

**Method**: POST
**Endpoint**: `/info`
**Request**:
```json
{
  "type": "metaAndAssetCtxs"
}
```
**Response** (per asset):
```json
{
  "ctx": {
    "dayNtlVlm": "123456789.0",
    "funding": "0.00001234",
    "openInterest": "1234567.89",
    "prevDayPx": "50000.0",
    "markPx": "50123.45",
    "midPx": "50123.5",
    "impactPxs": ["50120.0", "50127.0"],
    "premium": "0.5",
    "oraclePx": "50122.95"
  }
}
```
**Notes**: Calculate 24h change from `prevDayPx` and current `markPx`

---

## Get Order Book (L2)

**Method**: POST
**Endpoint**: `/info`
**Request**:
```json
{
  "type": "l2Book",
  "coin": "BTC",
  "nSigFigs": null,
  "mantissa": null
}
```
**Optional Parameters**:
- `nSigFigs`: 2-5 for aggregation
- `mantissa`: 1, 2, or 5 for rounding

**Response**:
```json
{
  "coin": "BTC",
  "time": 1704067200000,
  "levels": [
    [
      {"px": "50123.5", "sz": "1.234", "n": 3}
    ],
    [
      {"px": "50122.0", "sz": "0.567", "n": 1}
    ]
  ]
}
```
**Notes**: Up to 20 levels per side, `levels[0]` = bids, `levels[1]` = asks

---

## Get Recent Trades

**Method**: POST
**Endpoint**: `/info`
**Request**:
```json
{
  "type": "recentTrades",
  "coin": "BTC"
}
```
**Response**:
```json
[
  {
    "coin": "BTC",
    "side": "B",
    "px": "50123.45",
    "sz": "0.5",
    "hash": "0x...",
    "time": 1704067200000,
    "tid": 123456789,
    "fee": "0.25"
  }
]
```

---

## Get Klines / Candles

**Method**: POST
**Endpoint**: `/info`
**Request**:
```json
{
  "type": "candleSnapshot",
  "req": {
    "coin": "BTC",
    "interval": "15m",
    "startTime": 1704000000000,
    "endTime": 1704067200000
  }
}
```
**Supported Intervals**: `"1m"`, `"3m"`, `"5m"`, `"15m"`, `"30m"`, `"1h"`, `"2h"`, `"4h"`, `"8h"`, `"12h"`, `"1d"`, `"3d"`, `"1w"`, `"1M"`

**Response**:
```json
[
  {
    "t": 1704067200000,
    "T": 1704067259999,
    "s": "BTC",
    "i": "15m",
    "o": "50100.0",
    "c": "50200.0",
    "h": "50250.0",
    "l": "50050.0",
    "v": "123.456",
    "n": 1234
  }
]
```
**Limit**: Maximum 5000 most recent candles

---

## Get All Mids (All Symbols Price)

**Method**: POST
**Endpoint**: `/info`
**Request**:
```json
{
  "type": "allMids",
  "dex": ""
}
```
**Response**:
```json
{
  "BTC": "50123.45",
  "ETH": "2500.67",
  "SOL": "100.23"
}
```

---

## Get Funding Rate

**Method**: POST
**Endpoint**: `/info`
**Request**:
```json
{
  "type": "metaAndAssetCtxs"
}
```
**Response** (extract from asset context):
```json
{
  "ctx": {
    "funding": "0.00001234"
  }
}
```

### Historical Funding Rates
**Request**:
```json
{
  "type": "fundingHistory",
  "coin": "BTC",
  "startTime": 1704000000000,
  "endTime": 1704067200000
}
```
**Response**:
```json
[
  {
    "coin": "BTC",
    "fundingRate": "0.00001234",
    "premium": "0.5",
    "time": 1704067200000
  }
]
```

### Predicted Funding
**Request**:
```json
{
  "type": "predictedFundings"
}
```
**Notes**: Only supported for the main perpetual DEX

---

# Trading Trait Endpoints

All trading endpoints use: `POST /exchange`

## Place Order

**Request**:
```json
{
  "action": {
    "type": "order",
    "orders": [
      {
        "a": 0,
        "b": true,
        "p": "50000.0",
        "s": "0.1",
        "r": false,
        "t": {
          "limit": {
            "tif": "Gtc"
          }
        },
        "c": "0x1234567890abcdef1234567890abcdef"
      }
    ],
    "grouping": "na"
  },
  "nonce": 1704067200000,
  "signature": {
    "r": "0x...",
    "s": "0x...",
    "v": 27
  },
  "vaultAddress": null
}
```

**Order Fields**:
- `a` (asset): Asset index (0 for BTC on mainnet) or `10000 + spot_index` for spot
- `b` (isBuy): `true` for buy, `false` for sell
- `p` (price): Limit price as string
- `s` (size): Order size as string
- `r` (reduceOnly): `true` to only reduce position
- `t` (type): Order type object
- `c` (cloid): Optional 128-bit hex client order ID

**Order Types**:

### Limit Order (GTC)
```json
{
  "limit": {
    "tif": "Gtc"
  }
}
```

### Limit Order (IOC)
```json
{
  "limit": {
    "tif": "Ioc"
  }
}
```

### Limit Order (Post-Only/ALO)
```json
{
  "limit": {
    "tif": "Alo"
  }
}
```

### Trigger Order (Stop Market)
```json
{
  "trigger": {
    "triggerPx": "49000.0",
    "isMarket": true,
    "tpsl": "sl"
  }
}
```

### Trigger Order (Stop Limit)
```json
{
  "trigger": {
    "triggerPx": "49000.0",
    "isMarket": false,
    "tpsl": "sl"
  }
}
```

### Take Profit Orders
```json
{
  "trigger": {
    "triggerPx": "51000.0",
    "isMarket": true,
    "tpsl": "tp"
  }
}
```

**Response**:
```json
{
  "status": "ok",
  "response": {
    "type": "order",
    "data": {
      "statuses": [
        {
          "resting": {
            "oid": 123456789,
            "cloid": "0x1234567890abcdef1234567890abcdef"
          }
        }
      ]
    }
  }
}
```

**Error Response**:
```json
{
  "status": "ok",
  "response": {
    "type": "order",
    "data": {
      "statuses": [
        {
          "error": "Insufficient margin to place order."
        }
      ]
    }
  }
}
```

---

## Cancel Order

**By Order ID**:
```json
{
  "action": {
    "type": "cancel",
    "cancels": [
      {
        "a": 0,
        "o": 123456789
      }
    ]
  },
  "nonce": 1704067200000,
  "signature": {...}
}
```

**By Client Order ID**:
```json
{
  "action": {
    "type": "cancelByCloid",
    "cancels": [
      {
        "asset": 0,
        "cloid": "0x1234567890abcdef1234567890abcdef"
      }
    ]
  },
  "nonce": 1704067200000,
  "signature": {...}
}
```

**Response**:
```json
{
  "status": "ok",
  "response": {
    "type": "cancel",
    "data": {
      "statuses": [
        "success"
      ]
    }
  }
}
```

---

## Modify Order

**Request**:
```json
{
  "action": {
    "type": "modify",
    "oid": 123456789,
    "order": {
      "a": 0,
      "b": true,
      "p": "50100.0",
      "s": "0.15",
      "r": false,
      "t": {"limit": {"tif": "Gtc"}},
      "c": null
    }
  },
  "nonce": 1704067200000,
  "signature": {...}
}
```
**Notes**: Can use `oid` (order ID) or client order ID

---

## Cancel All Orders

**Request**:
```json
{
  "action": {
    "type": "cancel",
    "cancels": [
      {"a": 0, "o": 123456789},
      {"a": 0, "o": 123456790},
      {"a": 1, "o": 987654321}
    ]
  },
  "nonce": 1704067200000,
  "signature": {...}
}
```
**Notes**: Batch cancel by providing array of all order IDs

---

# Account Trait Endpoints

## Get Account Balance / Portfolio

**Method**: POST
**Endpoint**: `/info`

### Perpetuals Account
**Request**:
```json
{
  "type": "clearinghouseState",
  "user": "0x1234567890abcdef1234567890abcdef12345678",
  "dex": ""
}
```
**Response**:
```json
{
  "assetPositions": [
    {
      "position": {
        "coin": "BTC",
        "szi": "1.5",
        "leverage": {
          "type": "cross",
          "value": 5
        },
        "entryPx": "49500.0",
        "positionValue": "74250.0",
        "unrealizedPnl": "750.0",
        "returnOnEquity": "0.015",
        "liquidationPx": "40000.0"
      },
      "type": "oneWay"
    }
  ],
  "crossMarginSummary": {
    "accountValue": "100000.0",
    "totalNtlPos": "74250.0",
    "totalRawUsd": "25750.0",
    "totalMarginUsed": "14850.0",
    "withdrawable": "10900.0"
  },
  "marginSummary": {
    "accountValue": "100000.0",
    "totalNtlPos": "74250.0",
    "totalRawUsd": "25750.0"
  },
  "time": 1704067200000
}
```

### Spot Account
**Request**:
```json
{
  "type": "spotClearinghouseState",
  "user": "0x1234567890abcdef1234567890abcdef12345678"
}
```
**Response**:
```json
{
  "balances": [
    {
      "coin": "USDC",
      "hold": "1000.0",
      "total": "10000.0",
      "entryNtl": "10000.0",
      "token": 0
    }
  ]
}
```

### Portfolio Summary
**Request**:
```json
{
  "type": "portfolio",
  "user": "0x1234567890abcdef1234567890abcdef12345678"
}
```
**Response**: Comprehensive portfolio with PnL breakdown by timeframe

---

## Get Open Orders

**Method**: POST
**Endpoint**: `/info`
**Request**:
```json
{
  "type": "openOrders",
  "user": "0x1234567890abcdef1234567890abcdef12345678",
  "dex": ""
}
```
**Response**:
```json
[
  {
    "coin": "BTC",
    "limitPx": "50000.0",
    "oid": 123456789,
    "side": "B",
    "sz": "0.1",
    "timestamp": 1704067200000,
    "origSz": "0.1",
    "cloid": "0x1234567890abcdef1234567890abcdef"
  }
]
```

---

## Get Order Status

**Method**: POST
**Endpoint**: `/info`
**Request**:
```json
{
  "type": "orderStatus",
  "user": "0x1234567890abcdef1234567890abcdef12345678",
  "oid": 123456789
}
```
**Can also use cloid**: `"oid": {"cloid": "0x..."}`

**Response**:
```json
{
  "order": {
    "coin": "BTC",
    "side": "B",
    "limitPx": "50000.0",
    "sz": "0.1",
    "oid": 123456789,
    "timestamp": 1704067200000,
    "origSz": "0.1",
    "cloid": null
  },
  "status": "open",
  "statusTimestamp": 1704067200000
}
```
**Status Values**: `"open"`, `"filled"`, `"canceled"`, `"triggered"`, `"rejected"`, `"marginCanceled"`

---

## Get Trade History / User Fills

**Method**: POST
**Endpoint**: `/info`

### Recent Fills (Max 2000)
**Request**:
```json
{
  "type": "userFills",
  "user": "0x1234567890abcdef1234567890abcdef12345678",
  "aggregateByTime": false
}
```

### Fills By Time Range
**Request**:
```json
{
  "type": "userFillsByTime",
  "user": "0x1234567890abcdef1234567890abcdef12345678",
  "startTime": 1704000000000,
  "endTime": 1704067200000,
  "aggregateByTime": false
}
```
**Limit**: Maximum 2000 fills per response, only 10000 most recent available

**Response**:
```json
[
  {
    "coin": "BTC",
    "px": "50100.0",
    "sz": "0.1",
    "side": "B",
    "time": 1704067200000,
    "startPosition": "0.0",
    "dir": "Open Long",
    "closedPnl": "0.0",
    "hash": "0x...",
    "oid": 123456789,
    "crossed": true,
    "fee": "2.505",
    "feeToken": "USDC",
    "tid": 987654321,
    "builderFee": "0.0",
    "cloid": null
  }
]
```

---

## Get Historical Orders

**Method**: POST
**Endpoint**: `/info`
**Request**:
```json
{
  "type": "historicalOrders",
  "user": "0x1234567890abcdef1234567890abcdef12345678"
}
```
**Limit**: Maximum 2000 most recent historical orders

**Response**: Array of order objects with status information

---

## Get User Fees

**Method**: POST
**Endpoint**: `/info`
**Request**:
```json
{
  "type": "userFees",
  "user": "0x1234567890abcdef1234567890abcdef12345678"
}
```
**Response**:
```json
{
  "dailyUserVlm": [
    {
      "time": 1704067200000,
      "vlm": "123456.78"
    }
  ],
  "feeSchedule": {
    "tiers": [
      {
        "vlm": "0",
        "maker": "0.00020",
        "taker": "0.00035"
      }
    ]
  },
  "activeDiscounts": [],
  "staking": null
}
```

---

## Get Rate Limit Status

**Method**: POST
**Endpoint**: `/info`
**Request**:
```json
{
  "type": "userRateLimit",
  "user": "0x1234567890abcdef1234567890abcdef12345678"
}
```
**Response**:
```json
{
  "cumVlm": "1234567.89",
  "nRequestsUsed": 150,
  "nRequestsCap": 1234567,
  "nRequestsSurplus": 10000
}
```

---

# Positions Trait Endpoints

## Get Positions

**Method**: POST
**Endpoint**: `/info`
**Request**:
```json
{
  "type": "clearinghouseState",
  "user": "0x1234567890abcdef1234567890abcdef12345678",
  "dex": ""
}
```
**Response** (extract from clearinghouseState):
```json
{
  "assetPositions": [
    {
      "position": {
        "coin": "BTC",
        "szi": "1.5",
        "leverage": {
          "type": "cross",
          "value": 5
        },
        "entryPx": "49500.0",
        "positionValue": "74250.0",
        "unrealizedPnl": "750.0",
        "returnOnEquity": "0.015",
        "liquidationPx": "40000.0",
        "marginUsed": "14850.0",
        "maxTradeSzs": ["5.0", "5.0"],
        "cumFunding": {
          "allTime": "12.34",
          "sinceChange": "5.67",
          "sinceOpen": "8.90"
        }
      },
      "type": "oneWay"
    }
  ]
}
```
**Notes**: `szi` is signed size (positive = long, negative = short)

---

## Set Leverage

**Method**: POST
**Endpoint**: `/exchange`
**Request**:
```json
{
  "action": {
    "type": "updateLeverage",
    "asset": 0,
    "isCross": true,
    "leverage": 10
  },
  "nonce": 1704067200000,
  "signature": {...}
}
```
**Notes**: `isCross` = true for cross margin, false for isolated

---

## Update Isolated Margin

**Method**: POST
**Endpoint**: `/exchange`
**Request**:
```json
{
  "action": {
    "type": "updateIsolatedMargin",
    "asset": 0,
    "isBuy": true,
    "ntli": 1000000
  },
  "nonce": 1704067200000,
  "signature": {...}
}
```
**Notes**: `ntli` is amount in wei (1000000 = $1)

---

# Additional Account Operations

## Transfer USDC (Internal)

**Method**: POST
**Endpoint**: `/exchange`
**Request**:
```json
{
  "action": {
    "type": "usdSend",
    "hyperliquidChain": "Mainnet",
    "signatureChainId": "0xa4b1",
    "destination": "0xabcdef1234567890abcdef1234567890abcdef12",
    "amount": "100.0",
    "time": 1704067200000
  },
  "nonce": 1704067200000,
  "signature": {...}
}
```

---

## Transfer Spot Token (Internal)

**Method**: POST
**Endpoint**: `/exchange`
**Request**:
```json
{
  "action": {
    "type": "spotSend",
    "hyperliquidChain": "Mainnet",
    "signatureChainId": "0xa4b1",
    "destination": "0xabcdef1234567890abcdef1234567890abcdef12",
    "token": "PURR:0",
    "amount": "10.0",
    "time": 1704067200000
  },
  "nonce": 1704067200000,
  "signature": {...}
}
```
**Token Format**: `"tokenName:tokenId"`

---

## Withdraw to L1

**Method**: POST
**Endpoint**: `/exchange`
**Request**:
```json
{
  "action": {
    "type": "withdraw3",
    "hyperliquidChain": "Mainnet",
    "signatureChainId": "0xa4b1",
    "amount": "1000.0",
    "destination": "0xabcdef1234567890abcdef1234567890abcdef12",
    "time": 1704067200000
  },
  "nonce": 1704067200000,
  "signature": {...}
}
```
**Notes**: $1 fee, ~5 minutes to finalize

---

## Transfer Between Spot and Perp

**Method**: POST
**Endpoint**: `/exchange`
**Request**:
```json
{
  "action": {
    "type": "usdClassTransfer",
    "amount": "500.0",
    "toPerp": true,
    "nonce": 1704067200000
  },
  "nonce": 1704067200000,
  "signature": {...}
}
```
**Notes**: `toPerp: true` = spot→perp, `toPerp: false` = perp→spot

---

# Special Features

## Subaccounts and Vaults

**All operations support vaultAddress field**:
```json
{
  "action": {...},
  "nonce": 1704067200000,
  "signature": {...},
  "vaultAddress": "0xabcdef1234567890abcdef1234567890abcdef12"
}
```
**Notes**: Master account signs, vault/subaccount address in `vaultAddress` field

## Get Subaccounts

**Request**:
```json
{
  "type": "subAccounts",
  "user": "0x1234567890abcdef1234567890abcdef12345678"
}
```

## Get User Role

**Request**:
```json
{
  "type": "userRole",
  "user": "0x1234567890abcdef1234567890abcdef12345678"
}
```
**Responses**: `"User"`, `"Agent"`, `"Vault"`, `"Subaccount"`, `"Missing"`

---

# Builder Fees

## Get Max Builder Fee Approval

**Request**:
```json
{
  "type": "maxBuilderFee",
  "user": "0x1234567890abcdef1234567890abcdef12345678",
  "builder": "0xabcdef1234567890abcdef1234567890abcdef12"
}
```
**Response**: Returns max fee in tenths of basis point (1 = 0.001%)

---

# Common Parameters

## Symbol Formats

### Perpetuals
- **Name**: Direct coin name (e.g., `"BTC"`, `"ETH"`)
- **Asset ID**: Integer index from meta response (BTC = 0)
- **Builder DEX**: Format `"dex:COIN"` (e.g., `"xyz:XYZ100"`)

### Spot
- **PURR/USDC**: Use name `"PURR/USDC"` or `"@0"`
- **Other Pairs**: Use `"@{index}"` format (e.g., `"@1"`, `"@107"`)
- **Asset ID**: `10000 + spot_index`

## Address Format
- 42-character hexadecimal (e.g., `"0x1234567890abcdef1234567890abcdef12345678"`)
- Must be lowercase before signing

## Time Format
- Unix timestamp in milliseconds
- Example: `1704067200000`

## DEX Parameter
- Empty string `""` for main DEX
- DEX name for builder-deployed markets
