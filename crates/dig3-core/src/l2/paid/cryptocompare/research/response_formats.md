# CryptoCompare - Response Formats

All responses are in JSON format. This document provides EXACT examples from official sources.

## Common Response Structure

### Success Response
```json
{
  "Response": "Success",
  "Message": "",
  "HasWarning": false,
  "Type": 100,
  "Data": { ... }
}
```

### Error Response
```json
{
  "Response": "Error",
  "Message": "Error description here",
  "HasWarning": false,
  "Type": 99,
  "Data": {}
}
```

## Price Endpoints

### GET /data/price
**Example:** `https://min-api.cryptocompare.com/data/price?fsym=BTC&tsyms=USD,EUR`

**Response:**
```json
{
  "USD": 45000.50,
  "EUR": 41000.25
}
```

**Simple object:** Keys are to-symbols, values are prices.

### GET /data/pricemulti
**Example:** `https://min-api.cryptocompare.com/data/pricemulti?fsyms=BTC,ETH&tsyms=USD,EUR`

**Response:**
```json
{
  "BTC": {
    "USD": 45000.50,
    "EUR": 41000.25
  },
  "ETH": {
    "USD": 2500.75,
    "EUR": 2280.50
  }
}
```

**Nested object:** First level keys are from-symbols, second level keys are to-symbols.

### GET /data/pricemultifull
**Example:** `https://min-api.cryptocompare.com/data/pricemultifull?fsyms=BTC&tsyms=USD`

**Response:**
```json
{
  "RAW": {
    "BTC": {
      "USD": {
        "TYPE": "5",
        "MARKET": "CCCAGG",
        "FROMSYMBOL": "BTC",
        "TOSYMBOL": "USD",
        "FLAGS": "4",
        "PRICE": 45000.50,
        "LASTUPDATE": 1706280000,
        "MEDIAN": 45000.25,
        "LASTVOLUME": 0.5,
        "LASTVOLUMETO": 22500.25,
        "LASTTRADEID": "1234567890",
        "VOLUMEDAY": 1250.75,
        "VOLUMEDAYTO": 56281250.00,
        "VOLUME24HOUR": 1500.50,
        "VOLUME24HOURTO": 67500000.00,
        "OPENDAY": 44500.00,
        "HIGHDAY": 45500.00,
        "LOWDAY": 44000.00,
        "OPEN24HOUR": 44200.00,
        "HIGH24HOUR": 45500.00,
        "LOW24HOUR": 43800.00,
        "LASTMARKET": "Binance",
        "VOLUMEHOUR": 125.75,
        "VOLUMEHOURTO": 5628125.00,
        "OPENHOUR": 44800.00,
        "HIGHHOUR": 45100.00,
        "LOWHOUR": 44700.00,
        "TOPTIERVOLUME24HOUR": 1200.00,
        "TOPTIERVOLUME24HOURTO": 54000000.00,
        "CHANGE24HOUR": 800.50,
        "CHANGEPCT24HOUR": 1.81,
        "CHANGEDAY": 500.50,
        "CHANGEPCTDAY": 1.12,
        "CHANGEHOUR": 200.50,
        "CHANGEPCTHOUR": 0.45,
        "CONVERSIONTYPE": "direct",
        "CONVERSIONSYMBOL": "",
        "SUPPLY": 19000000,
        "MKTCAP": 855000000000.00,
        "MKTCAPPENALTY": 0,
        "TOTALVOLUME24H": 1500.50,
        "TOTALVOLUME24HTO": 67500000.00,
        "TOTALTOPTIERVOLUME24H": 1200.00,
        "TOTALTOPTIERVOLUME24HTO": 54000000.00,
        "IMAGEURL": "/media/37746251/btc.png"
      }
    }
  },
  "DISPLAY": {
    "BTC": {
      "USD": {
        "FROMSYMBOL": "Ƀ",
        "TOSYMBOL": "$",
        "MARKET": "CryptoCompare Index",
        "PRICE": "$ 45,000.50",
        "LASTUPDATE": "Just now",
        "LASTVOLUME": "Ƀ 0.50",
        "LASTVOLUMETO": "$ 22,500.25",
        "LASTTRADEID": "1234567890",
        "VOLUMEDAY": "Ƀ 1,250.75",
        "VOLUMEDAYTO": "$ 56,281,250.00",
        "VOLUME24HOUR": "Ƀ 1,500.50",
        "VOLUME24HOURTO": "$ 67,500,000.00",
        "OPENDAY": "$ 44,500.00",
        "HIGHDAY": "$ 45,500.00",
        "LOWDAY": "$ 44,000.00",
        "OPEN24HOUR": "$ 44,200.00",
        "HIGH24HOUR": "$ 45,500.00",
        "LOW24HOUR": "$ 43,800.00",
        "LASTMARKET": "Binance",
        "VOLUMEHOUR": "Ƀ 125.75",
        "VOLUMEHOURTO": "$ 5,628,125.00",
        "OPENHOUR": "$ 44,800.00",
        "HIGHHOUR": "$ 45,100.00",
        "LOWHOUR": "$ 44,700.00",
        "TOPTIERVOLUME24HOUR": "Ƀ 1,200.00",
        "TOPTIERVOLUME24HOURTO": "$ 54,000,000.00",
        "CHANGE24HOUR": "$ 800.50",
        "CHANGEPCT24HOUR": "1.81",
        "CHANGEDAY": "$ 500.50",
        "CHANGEPCTDAY": "1.12",
        "CHANGEHOUR": "$ 200.50",
        "CHANGEPCTHOUR": "0.45",
        "CONVERSIONTYPE": "direct",
        "CONVERSIONSYMBOL": "",
        "SUPPLY": "Ƀ 19,000,000.0",
        "MKTCAP": "$ 855.00 B",
        "MKTCAPPENALTY": "0 %",
        "TOTALVOLUME24H": "Ƀ 1,500.50",
        "TOTALVOLUME24HTO": "$ 67.50 M",
        "TOTALTOPTIERVOLUME24H": "Ƀ 1,200.00",
        "TOTALTOPTIERVOLUME24HTO": "$ 54.00 M",
        "IMAGEURL": "/media/37746251/btc.png"
      }
    }
  }
}
```

**Two sections:**
- `RAW`: Numeric values (for calculations)
- `DISPLAY`: Formatted strings (for display)

## Historical Data Endpoints

### GET /data/histoday
**Example:** `https://min-api.cryptocompare.com/data/histoday?fsym=BTC&tsym=USD&limit=2`

**Response:**
```json
{
  "Response": "Success",
  "Message": "",
  "HasWarning": false,
  "Type": 100,
  "RateLimit": {},
  "Data": {
    "Aggregated": false,
    "TimeFrom": 1706140800,
    "TimeTo": 1706313600,
    "Data": [
      {
        "time": 1706140800,
        "high": 45200.00,
        "low": 44800.00,
        "open": 44900.00,
        "volumefrom": 1250.50,
        "volumeto": 56281250.00,
        "close": 45000.00,
        "conversionType": "direct",
        "conversionSymbol": ""
      },
      {
        "time": 1706227200,
        "high": 45500.00,
        "low": 44500.00,
        "open": 45000.00,
        "volumefrom": 1350.75,
        "volumeto": 60825000.00,
        "close": 45100.00,
        "conversionType": "direct",
        "conversionSymbol": ""
      },
      {
        "time": 1706313600,
        "high": 45800.00,
        "low": 45000.00,
        "open": 45100.00,
        "volumefrom": 1400.00,
        "volumeto": 63700000.00,
        "close": 45500.00,
        "conversionType": "direct",
        "conversionSymbol": ""
      }
    ]
  }
}
```

**Fields:**
- `time`: Unix timestamp (seconds) - candle start time
- `open`: Open price
- `high`: High price
- `low`: Low price
- `close`: Close price
- `volumefrom`: Volume in base currency (BTC)
- `volumeto`: Volume in quote currency (USD)
- `conversionType`: "direct" or "force_direct" or "inverse"
- `conversionSymbol`: Symbol used for conversion (if any)

### GET /data/histohour
**Same format as histoday**, but hourly intervals.

### GET /data/histominute
**Same format as histoday**, but minute intervals.

### GET /data/pricehistorical
**Example:** `https://min-api.cryptocompare.com/data/pricehistorical?fsym=BTC&tsyms=USD,EUR&ts=1452680400`

**Response:**
```json
{
  "BTC": {
    "USD": 450.25,
    "EUR": 410.50
  }
}
```

**Same format as `/data/price`** but for historical timestamp.

## Exchange & Top Lists

### GET /data/top/exchanges
**Example:** `https://min-api.cryptocompare.com/data/top/exchanges?fsym=BTC&tsym=USD&limit=3`

**Response:**
```json
{
  "Response": "Success",
  "Message": "",
  "HasWarning": false,
  "Type": 100,
  "RateLimit": {},
  "Data": [
    {
      "exchange": "Binance",
      "fromSymbol": "BTC",
      "toSymbol": "USD",
      "volume24h": 125000.50,
      "volume24hTo": 5625000000.00
    },
    {
      "exchange": "Coinbase",
      "fromSymbol": "BTC",
      "toSymbol": "USD",
      "volume24h": 85000.25,
      "volume24hTo": 3825000000.00
    },
    {
      "exchange": "Kraken",
      "fromSymbol": "BTC",
      "toSymbol": "USD",
      "volume24h": 45000.00,
      "volume24hTo": 2025000000.00
    }
  ]
}
```

### GET /data/top/pairs
**Example:** `https://min-api.cryptocompare.com/data/top/pairs?fsym=BTC&limit=3`

**Response:**
```json
{
  "Response": "Success",
  "Message": "",
  "Type": 100,
  "Data": [
    {
      "exchange": "CCCAGG",
      "fromSymbol": "BTC",
      "toSymbol": "USD",
      "volume24h": 1500000.00,
      "volume24hTo": 67500000000.00
    },
    {
      "exchange": "CCCAGG",
      "fromSymbol": "BTC",
      "toSymbol": "USDT",
      "volume24h": 1200000.00,
      "volume24hTo": 54000000000.00
    },
    {
      "exchange": "CCCAGG",
      "fromSymbol": "BTC",
      "toSymbol": "EUR",
      "volume24h": 250000.00,
      "volume24hTo": 11250000000.00
    }
  ]
}
```

### GET /data/top/volumes
**Example:** `https://min-api.cryptocompare.com/data/top/volumes?tsym=USD&limit=3`

**Response:**
```json
{
  "Response": "Success",
  "Message": "",
  "Type": 100,
  "Data": [
    {
      "SYMBOL": "BTC",
      "SUPPLY": 19000000,
      "FULLNAME": "Bitcoin",
      "NAME": "BTC",
      "ID": "1182",
      "VOLUME24HOURTO": 67500000000.00
    },
    {
      "SYMBOL": "ETH",
      "SUPPLY": 120000000,
      "FULLNAME": "Ethereum",
      "NAME": "ETH",
      "ID": "7605",
      "VOLUME24HOURTO": 28000000000.00
    },
    {
      "SYMBOL": "USDT",
      "SUPPLY": 95000000000,
      "FULLNAME": "Tether",
      "NAME": "USDT",
      "ID": "5031",
      "VOLUME24HOURTO": 95000000000.00
    }
  ]
}
```

## Metadata Endpoints

### GET /data/all/coinlist
**Example:** `https://min-api.cryptocompare.com/data/all/coinlist`

**Response:**
```json
{
  "Response": "Success",
  "Message": "",
  "HasWarning": false,
  "Type": 100,
  "RateLimit": {},
  "Data": {
    "BTC": {
      "Id": "1182",
      "Url": "/coins/btc/overview",
      "ImageUrl": "/media/37746251/btc.png",
      "ContentCreatedOn": 1367174841,
      "Name": "BTC",
      "Symbol": "BTC",
      "CoinName": "Bitcoin",
      "FullName": "Bitcoin (BTC)",
      "Algorithm": "SHA256",
      "ProofType": "PoW",
      "FullyPremined": "0",
      "TotalCoinSupply": "21000000",
      "BuiltOn": "",
      "SmartContractAddress": "",
      "DecimalPlaces": 8,
      "PreMinedValue": "N/A",
      "TotalCoinsFreeFloat": "N/A",
      "SortOrder": "1",
      "Sponsored": false,
      "Taxonomy": {
        "Access": "Public",
        "FCA": "Unregulated",
        "FINMA": "Payment token",
        "Industry": "Financial",
        "CollateralizedAsset": "",
        "CollateralizedAssetType": "",
        "CollateralType": ""
      },
      "Rating": {
        "Weiss": {
          "Rating": "B",
          "TechnologyAdoptionRating": "B",
          "MarketPerformanceRating": "C"
        }
      },
      "IsTrading": true,
      "TotalCoinsMined": 19000000,
      "BlockNumber": 780000,
      "NetHashesPerSecond": 350000000000000000000,
      "BlockReward": 6.25,
      "BlockTime": 600
    },
    "ETH": {
      "Id": "7605",
      "Url": "/coins/eth/overview",
      "ImageUrl": "/media/37746238/eth.png",
      "ContentCreatedOn": 1438388070,
      "Name": "ETH",
      "Symbol": "ETH",
      "CoinName": "Ethereum",
      "FullName": "Ethereum (ETH)",
      "Algorithm": "Ethash",
      "ProofType": "PoS",
      "FullyPremined": "0",
      "TotalCoinSupply": "0",
      "BuiltOn": "",
      "SmartContractAddress": "",
      "DecimalPlaces": 18,
      "PreMinedValue": "N/A",
      "TotalCoinsFreeFloat": "N/A",
      "SortOrder": "2",
      "Sponsored": false,
      "Taxonomy": {
        "Access": "Public",
        "FCA": "Unregulated",
        "FINMA": "Asset token",
        "Industry": "Smart Contract Platform",
        "CollateralizedAsset": "",
        "CollateralizedAssetType": "",
        "CollateralType": ""
      },
      "Rating": {
        "Weiss": {
          "Rating": "A",
          "TechnologyAdoptionRating": "A",
          "MarketPerformanceRating": "B"
        }
      },
      "IsTrading": true,
      "TotalCoinsMined": 120000000,
      "BlockNumber": 19000000,
      "NetHashesPerSecond": 0,
      "BlockReward": 2,
      "BlockTime": 13
    }
  },
  "BaseImageUrl": "https://www.cryptocompare.com",
  "BaseLinkUrl": "https://www.cryptocompare.com"
}
```

**Large object:** Keys are symbols, values are coin metadata objects.

### GET /data/all/exchanges
**Example:** `https://min-api.cryptocompare.com/data/all/exchanges`

**Response:**
```json
{
  "Binance": {
    "BTC": ["USD", "USDT", "EUR", "ETH"],
    "ETH": ["USD", "USDT", "BTC"],
    "BNB": ["USD", "USDT", "BTC"]
  },
  "Coinbase": {
    "BTC": ["USD", "EUR", "GBP"],
    "ETH": ["USD", "EUR", "BTC"],
    "LTC": ["USD", "BTC"]
  },
  "Kraken": {
    "BTC": ["USD", "EUR", "ETH"],
    "ETH": ["USD", "EUR", "BTC"],
    "XRP": ["USD", "BTC"]
  }
}
```

**Nested object:**
- First level: Exchange names
- Second level: Base symbols
- Third level: Array of quote symbols

## News Endpoints

### GET /data/v2/news/
**Example:** `https://min-api.cryptocompare.com/data/v2/news/?lang=EN&api_key=YOUR_KEY`

**Response:**
```json
{
  "Type": 100,
  "Message": "Success",
  "Promoted": [],
  "Data": [
    {
      "id": "123456",
      "guid": "https://newssite.com/article/123456",
      "published_on": 1706280000,
      "imageurl": "https://newssite.com/images/article.jpg",
      "title": "Bitcoin Reaches New All-Time High",
      "url": "https://newssite.com/article/bitcoin-ath",
      "source": "CoinDesk",
      "body": "Bitcoin has surged to a new all-time high today, reaching $45,000...",
      "tags": "BTC|Bitcoin|Price",
      "categories": "BTC|Trading",
      "upvotes": "0",
      "downvotes": "0",
      "lang": "EN",
      "source_info": {
        "name": "CoinDesk",
        "lang": "EN",
        "img": "https://images.cryptocompare.com/news/default/coindesk.png"
      }
    },
    {
      "id": "123457",
      "guid": "https://newssite2.com/article/789",
      "published_on": 1706279500,
      "imageurl": "https://newssite2.com/images/eth.jpg",
      "title": "Ethereum Upgrade Successfully Completed",
      "url": "https://newssite2.com/article/eth-upgrade",
      "source": "CoinTelegraph",
      "body": "The Ethereum network has successfully completed its latest upgrade...",
      "tags": "ETH|Ethereum|Technology",
      "categories": "ETH|Blockchain",
      "upvotes": "5",
      "downvotes": "0",
      "lang": "EN",
      "source_info": {
        "name": "CoinTelegraph",
        "lang": "EN",
        "img": "https://images.cryptocompare.com/news/default/cointelegraph.png"
      }
    }
  ],
  "RateLimit": {
    "calls_made": {
      "second": 1,
      "minute": 5,
      "hour": 50
    },
    "calls_left": {
      "second": 49,
      "minute": 995,
      "hour": 149950
    }
  },
  "HasWarning": false
}
```

## Social Stats Endpoints

### GET /data/social/coin/latest
**Example:** `https://min-api.cryptocompare.com/data/social/coin/latest?coinId=1182&api_key=YOUR_KEY`

**Response:**
```json
{
  "Response": "Success",
  "Message": "",
  "Data": {
    "General": {
      "Points": 850000,
      "Name": "BTC",
      "CoinName": "Bitcoin",
      "Type": "Webpagecoinp"
    },
    "CryptoCompare": {
      "Points": 125000,
      "Followers": 85000,
      "Posts": 12500,
      "Comments": 45000,
      "PageViewsSplit": {
        "Overview": 0.85,
        "Markets": 0.05,
        "Analysis": 0.05,
        "Charts": 0.03,
        "Trades": 0.01,
        "Forum": 0.01
      }
    },
    "Twitter": {
      "following": 500,
      "account_creation": 1298937600,
      "name": "Bitcoin",
      "lists": 12500,
      "statuses": 85000,
      "favourites": 250,
      "followers": 5500000,
      "link": "https://twitter.com/bitcoin",
      "Points": 450000
    },
    "Reddit": {
      "posts_per_hour": 25.5,
      "comments_per_hour": 125.0,
      "posts_per_day": 612.0,
      "comments_per_day": 3000.0,
      "name": "Bitcoin",
      "link": "https://www.reddit.com/r/Bitcoin/",
      "active_users": 25000,
      "community_creation": 1284042391,
      "subscribers": 4500000,
      "Points": 250000
    },
    "Facebook": {
      "likes": 1500000,
      "link": "https://www.facebook.com/bitcoins",
      "is_closed": false,
      "talking_about": 50000,
      "name": "Bitcoin",
      "Points": 15000
    },
    "CodeRepository": {
      "List": [
        {
          "created_at": 1292771646,
          "open_total_issues": 500,
          "parent": {
            "Name": "",
            "url": "",
            "InternalId": ""
          },
          "size": 150000,
          "closed_total_issues": 8500,
          "stars": 70000,
          "language": "C++",
          "forks": 35000,
          "url": "https://github.com/bitcoin/bitcoin",
          "closed_issues": 50,
          "closed_pull_issues": 8450,
          "fork": false,
          "last_update": 1706280000,
          "last_push": 1706279500,
          "source": {
            "Name": "",
            "url": "",
            "InternalId": ""
          },
          "open_pull_issues": 250,
          "open_issues": 250,
          "subscribers": 5000,
          "contributors": 850
        }
      ],
      "Points": 10000
    }
  }
}
```

## Blockchain Data Endpoints

### GET /data/blockchain/histo/day
**Example:** `https://min-api.cryptocompare.com/data/blockchain/histo/day?fsym=BTC&limit=2`

**Response:**
```json
{
  "Response": "Success",
  "Message": "",
  "HasWarning": false,
  "Type": 100,
  "Data": {
    "Data": [
      {
        "time": 1706140800,
        "transaction_count": 250000,
        "symbol": "BTC",
        "block_height": 779500,
        "hashrate": 350000000000000000000,
        "difficulty": 70000000000000,
        "block_time": 605,
        "block_size": 1500000,
        "current_supply": 19000000
      },
      {
        "time": 1706227200,
        "transaction_count": 255000,
        "symbol": "BTC",
        "block_height": 779650,
        "hashrate": 355000000000000000000,
        "difficulty": 71000000000000,
        "block_time": 598,
        "block_size": 1520000,
        "current_supply": 19000100
      }
    ]
  }
}
```

## Rate Limit Endpoints

### GET /stats/rate/limit
**Example:** `https://min-api.cryptocompare.com/stats/rate/limit?api_key=YOUR_KEY`

**Response:**
```json
{
  "Response": "Success",
  "Message": "",
  "Data": {
    "calls_made": {
      "second": 12,
      "minute": 345,
      "hour": 8921
    },
    "calls_left": {
      "second": 38,
      "minute": 655,
      "hour": 141079
    }
  }
}
```

## WebSocket Message Formats

See `websocket_full.md` for complete WebSocket message formats.

### Summary:
- **Channel 0 (TRADE):** Individual trade messages
- **Channel 2 (CURRENT):** Exchange-specific ticker
- **Channel 5 (CURRENTAGG):** Aggregate ticker
- **Channel 16 (ORDERBOOK):** Orderbook snapshot/delta
- **Channel 17 (OHLC):** Candlestick updates
- **Channel 24 (VOLUME):** Volume updates

## Error Response Examples

### Rate Limit Error
```json
{
  "Response": "Error",
  "Message": "You are over your rate limit please upgrade your account!",
  "HasWarning": false,
  "Type": 99,
  "RateLimit": {
    "calls_made": {
      "second": 51,
      "minute": 1005,
      "hour": 150100
    },
    "calls_left": {
      "second": 0,
      "minute": 0,
      "hour": 0
    }
  },
  "Data": {}
}
```

### Invalid Parameter Error
```json
{
  "Response": "Error",
  "Message": "fsym param is invalid.",
  "HasWarning": false,
  "Type": 2,
  "Data": {}
}
```

### General Error
```json
{
  "Response": "Error",
  "Message": "There is no data for the symbol BTC on the exchange InvalidExchange.",
  "HasWarning": false,
  "Type": 1,
  "Data": {}
}
```

## Notes

- All timestamps are Unix seconds (not milliseconds)
- Prices are floating-point numbers
- Volume fields: `volumefrom` (base currency), `volumeto` (quote currency)
- `conversionType`: "direct" (pair exists), "force_direct" (forced), "inverse" (inverted pair)
- Response always includes `Response`, `Message`, `Type` fields
- `Type: 100` = Success, `Type: 1` = Error, `Type: 2` = Invalid param, `Type: 99` = Rate limit
- `RAW` vs `DISPLAY`: Use `RAW` for calculations, `DISPLAY` for showing to users
- Image URLs are relative: prepend `https://www.cryptocompare.com`
