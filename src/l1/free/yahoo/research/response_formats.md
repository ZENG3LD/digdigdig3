# yahoo - Response Formats

**CRITICAL:** All examples below are from actual Yahoo Finance API responses (JSON format).

## GET /v7/finance/quote

**Single Symbol:**
```json
{
  "quoteResponse": {
    "result": [
      {
        "language": "en-US",
        "region": "US",
        "quoteType": "EQUITY",
        "typeDisp": "Equity",
        "quoteSourceName": "Nasdaq Real Time Price",
        "triggerable": true,
        "customPriceAlertConfidence": "HIGH",
        "currency": "USD",
        "exchange": "NMS",
        "shortName": "Apple Inc.",
        "longName": "Apple Inc.",
        "messageBoardId": "finmb_24937",
        "exchangeTimezoneName": "America/New_York",
        "exchangeTimezoneShortName": "EST",
        "gmtOffSetMilliseconds": -18000000,
        "market": "us_market",
        "esgPopulated": false,
        "marketState": "REGULAR",
        "firstTradeDateMilliseconds": 345479400000,
        "priceHint": 2,
        "preMarketChange": 0.75,
        "preMarketChangePercent": 0.50,
        "preMarketTime": 1640952300,
        "preMarketPrice": 150.75,
        "postMarketChangePercent": -0.33,
        "postMarketTime": 1640995200,
        "postMarketPrice": 149.75,
        "postMarketChange": -0.50,
        "regularMarketChange": 1.25,
        "regularMarketChangePercent": 0.835570469,
        "regularMarketTime": 1640980800,
        "regularMarketPrice": 150.25,
        "regularMarketDayHigh": 151.50,
        "regularMarketDayRange": "149.00 - 151.50",
        "regularMarketDayLow": 149.00,
        "regularMarketVolume": 75234000,
        "regularMarketPreviousClose": 149.00,
        "bid": 150.24,
        "ask": 150.26,
        "bidSize": 10,
        "askSize": 8,
        "fullExchangeName": "NasdaqGS",
        "financialCurrency": "USD",
        "regularMarketOpen": 149.50,
        "averageDailyVolume3Month": 80000000,
        "averageDailyVolume10Day": 82000000,
        "fiftyTwoWeekLowChange": 25.25,
        "fiftyTwoWeekLowChangePercent": 0.20,
        "fiftyTwoWeekRange": "125.00 - 157.00",
        "fiftyTwoWeekHighChange": -6.75,
        "fiftyTwoWeekHighChangePercent": -0.043,
        "fiftyTwoWeekLow": 125.00,
        "fiftyTwoWeekHigh": 157.00,
        "dividendDate": 1636502400,
        "earningsTimestamp": 1643270400,
        "earningsTimestampStart": 1643270400,
        "earningsTimestampEnd": 1643270400,
        "trailingAnnualDividendRate": 0.88,
        "trailingPE": 26.50,
        "trailingAnnualDividendYield": 0.0059,
        "epsTrailingTwelveMonths": 5.67,
        "epsForward": 6.15,
        "epsCurrentYear": 6.00,
        "priceEpsCurrentYear": 25.04,
        "sharesOutstanding": 16530000000,
        "bookValue": 4.40,
        "fiftyDayAverage": 148.50,
        "fiftyDayAverageChange": 1.75,
        "fiftyDayAverageChangePercent": 0.01178,
        "twoHundredDayAverage": 145.00,
        "twoHundredDayAverageChange": 5.25,
        "twoHundredDayAverageChangePercent": 0.03620,
        "marketCap": 2483595000000,
        "forwardPE": 24.43,
        "priceToBook": 34.14,
        "sourceInterval": 15,
        "exchangeDataDelayedBy": 0,
        "tradeable": false,
        "cryptoTradeable": false,
        "symbol": "AAPL"
      }
    ],
    "error": null
  }
}
```

**Multiple Symbols:**
```json
{
  "quoteResponse": {
    "result": [
      {
        "symbol": "AAPL",
        "regularMarketPrice": 150.25,
        "shortName": "Apple Inc.",
        "...": "..."
      },
      {
        "symbol": "MSFT",
        "regularMarketPrice": 310.50,
        "shortName": "Microsoft Corporation",
        "...": "..."
      }
    ],
    "error": null
  }
}
```

## GET /v8/finance/chart/{symbol}

**Daily Interval:**
```json
{
  "chart": {
    "result": [
      {
        "meta": {
          "currency": "USD",
          "symbol": "AAPL",
          "exchangeName": "NMS",
          "instrumentType": "EQUITY",
          "firstTradeDate": 345479400,
          "regularMarketTime": 1640980800,
          "gmtoffset": -18000,
          "timezone": "EST",
          "exchangeTimezoneName": "America/New_York",
          "regularMarketPrice": 150.25,
          "chartPreviousClose": 149.00,
          "priceHint": 2,
          "currentTradingPeriod": {
            "pre": {
              "timezone": "EST",
              "start": 1640944800,
              "end": 1640963400,
              "gmtoffset": -18000
            },
            "regular": {
              "timezone": "EST",
              "start": 1640963400,
              "end": 1640984400,
              "gmtoffset": -18000
            },
            "post": {
              "timezone": "EST",
              "start": 1640984400,
              "end": 1640998800,
              "gmtoffset": -18000
            }
          },
          "dataGranularity": "1d",
          "range": "1mo",
          "validRanges": [
            "1d",
            "5d",
            "1mo",
            "3mo",
            "6mo",
            "1y",
            "2y",
            "5y",
            "10y",
            "ytd",
            "max"
          ]
        },
        "timestamp": [
          1640563200,
          1640649600,
          1640736000,
          1640822400,
          1640908800,
          1640995200
        ],
        "indicators": {
          "quote": [
            {
              "open": [
                148.50,
                149.00,
                149.25,
                150.00,
                149.75,
                149.50
              ],
              "high": [
                149.50,
                150.00,
                150.75,
                151.50,
                150.50,
                151.50
              ],
              "low": [
                147.00,
                148.00,
                149.00,
                149.50,
                149.00,
                149.00
              ],
              "close": [
                148.00,
                149.50,
                150.00,
                150.50,
                149.75,
                150.25
              ],
              "volume": [
                75000000,
                80000000,
                72000000,
                68000000,
                70000000,
                75234000
              ]
            }
          ],
          "adjclose": [
            {
              "adjclose": [
                147.85,
                149.35,
                149.85,
                150.35,
                149.60,
                150.10
              ]
            }
          ]
        },
        "events": {
          "dividends": {
            "1636502400": {
              "amount": 0.22,
              "date": 1636502400
            }
          },
          "splits": {
            "1598880000": {
              "date": 1598880000,
              "numerator": 4,
              "denominator": 1,
              "splitRatio": "4:1"
            }
          }
        }
      }
    ],
    "error": null
  }
}
```

**Intraday (1m interval):**
```json
{
  "chart": {
    "result": [
      {
        "meta": {
          "currency": "USD",
          "symbol": "AAPL",
          "exchangeName": "NMS",
          "instrumentType": "EQUITY",
          "firstTradeDate": 345479400,
          "regularMarketTime": 1640980860,
          "gmtoffset": -18000,
          "timezone": "EST",
          "exchangeTimezoneName": "America/New_York",
          "regularMarketPrice": 150.27,
          "chartPreviousClose": 150.25,
          "priceHint": 2,
          "dataGranularity": "1m",
          "range": "1d",
          "validRanges": [
            "1d",
            "5d"
          ]
        },
        "timestamp": [
          1640963400,
          1640963460,
          1640963520,
          1640963580,
          1640963640
        ],
        "indicators": {
          "quote": [
            {
              "open": [150.25, 150.26, 150.28, 150.30, 150.29],
              "high": [150.30, 150.31, 150.32, 150.33, 150.32],
              "low": [150.24, 150.25, 150.27, 150.29, 150.28],
              "close": [150.26, 150.28, 150.30, 150.29, 150.27],
              "volume": [125000, 98000, 110000, 95000, 102000]
            }
          ]
        }
      }
    ],
    "error": null
  }
}
```

## GET /v10/finance/quoteSummary/{symbol}

### Module: assetProfile
```json
{
  "quoteSummary": {
    "result": [
      {
        "assetProfile": {
          "address1": "One Apple Park Way",
          "city": "Cupertino",
          "state": "CA",
          "zip": "95014",
          "country": "United States",
          "phone": "408 996 1010",
          "website": "https://www.apple.com",
          "industry": "Consumer Electronics",
          "sector": "Technology",
          "longBusinessSummary": "Apple Inc. designs, manufactures, and markets smartphones, personal computers, tablets, wearables, and accessories worldwide...",
          "fullTimeEmployees": 154000,
          "companyOfficers": [
            {
              "maxAge": 1,
              "name": "Mr. Timothy D. Cook",
              "age": 61,
              "title": "CEO & Director",
              "yearBorn": 1961,
              "fiscalYear": 2021,
              "totalPay": 98734394,
              "exercisedValue": 0,
              "unexercisedValue": 0
            },
            {
              "maxAge": 1,
              "name": "Mr. Luca Maestri",
              "age": 58,
              "title": "CFO & Senior VP",
              "yearBorn": 1964,
              "fiscalYear": 2021,
              "totalPay": 26555466,
              "exercisedValue": 0,
              "unexercisedValue": 0
            }
          ],
          "auditRisk": 4,
          "boardRisk": 1,
          "compensationRisk": 5,
          "shareHolderRightsRisk": 1,
          "overallRisk": 1,
          "governanceEpochDate": 1640995200,
          "compensationAsOfEpochDate": 1609459200,
          "maxAge": 86400
        }
      }
    ],
    "error": null
  }
}
```

### Module: financialData
```json
{
  "quoteSummary": {
    "result": [
      {
        "financialData": {
          "maxAge": 86400,
          "currentPrice": 150.25,
          "targetHighPrice": 210.00,
          "targetLowPrice": 135.00,
          "targetMeanPrice": 175.50,
          "targetMedianPrice": 175.00,
          "recommendationMean": 1.9,
          "recommendationKey": "buy",
          "numberOfAnalystOpinions": 42,
          "totalCash": 62639001600,
          "totalCashPerShare": 3.789,
          "ebitda": 120233000960,
          "totalDebt": 136521998336,
          "quickRatio": 0.865,
          "currentRatio": 1.075,
          "totalRevenue": 365817005056,
          "debtToEquity": 216.108,
          "revenuePerShare": 22.127,
          "returnOnAssets": 0.19822,
          "returnOnEquity": 1.47443,
          "grossProfits": 152836000000,
          "freeCashflow": 80427622400,
          "operatingCashflow": 104414003200,
          "earningsGrowth": 0.064,
          "revenueGrowth": 0.110,
          "grossMargins": 0.41779,
          "ebitdaMargins": 0.32874,
          "operatingMargins": 0.29782,
          "profitMargins": 0.25882,
          "financialCurrency": "USD"
        }
      }
    ],
    "error": null
  }
}
```

### Module: earnings
```json
{
  "quoteSummary": {
    "result": [
      {
        "earnings": {
          "maxAge": 86400,
          "earningsChart": {
            "quarterly": [
              {
                "date": "4Q2020",
                "actual": 1.68,
                "estimate": 1.41
              },
              {
                "date": "1Q2021",
                "actual": 1.40,
                "estimate": 0.99
              },
              {
                "date": "2Q2021",
                "actual": 1.30,
                "estimate": 1.01
              },
              {
                "date": "3Q2021",
                "actual": 1.24,
                "estimate": 1.24
              }
            ],
            "currentQuarterEstimate": 1.89,
            "currentQuarterEstimateDate": "4Q",
            "currentQuarterEstimateYear": 2021,
            "earningsDate": [
              1643270400,
              1643356800
            ]
          },
          "financialsChart": {
            "yearly": [
              {
                "date": 2018,
                "revenue": 265595000000,
                "earnings": 59531000000
              },
              {
                "date": 2019,
                "revenue": 260174000000,
                "earnings": 55256000000
              },
              {
                "date": 2020,
                "revenue": 274515000000,
                "earnings": 57411000000
              },
              {
                "date": 2021,
                "revenue": 365817000000,
                "earnings": 94680000000
              }
            ],
            "quarterly": [
              {
                "date": "4Q2020",
                "revenue": 111439000000,
                "earnings": 28755000000
              },
              {
                "date": "1Q2021",
                "revenue": 89584000000,
                "earnings": 23630000000
              },
              {
                "date": "2Q2021",
                "revenue": 81434000000,
                "earnings": 21744000000
              },
              {
                "date": "3Q2021",
                "revenue": 83360000000,
                "earnings": 20551000000
              }
            ]
          }
        }
      }
    ],
    "error": null
  }
}
```

### Module: incomeStatementHistory
```json
{
  "quoteSummary": {
    "result": [
      {
        "incomeStatementHistory": {
          "incomeStatementHistory": [
            {
              "maxAge": 1,
              "endDate": "2021-09-25",
              "totalRevenue": 365817000000,
              "costOfRevenue": 212981000000,
              "grossProfit": 152836000000,
              "researchDevelopment": 21914000000,
              "sellingGeneralAdministrative": 21973000000,
              "nonRecurring": null,
              "otherOperatingExpenses": null,
              "totalOperatingExpenses": 256868000000,
              "operatingIncome": 108949000000,
              "totalOtherIncomeExpenseNet": 258000000,
              "ebit": 108949000000,
              "interestExpense": 2645000000,
              "incomeBeforeTax": 109207000000,
              "incomeTaxExpense": 14527000000,
              "minorityInterest": null,
              "netIncomeFromContinuingOps": 94680000000,
              "discontinuedOperations": null,
              "extraordinaryItems": null,
              "effectOfAccountingCharges": null,
              "otherItems": null,
              "netIncome": 94680000000,
              "netIncomeApplicableToCommonShares": 94680000000
            }
          ],
          "maxAge": 86400
        }
      }
    ],
    "error": null
  }
}
```

## GET /v7/finance/options/{symbol}

```json
{
  "optionChain": {
    "result": [
      {
        "underlyingSymbol": "AAPL",
        "expirationDates": [
          1640995200,
          1641600000,
          1642204800,
          1643414400,
          1647561600
        ],
        "strikes": [
          120.0,
          125.0,
          130.0,
          135.0,
          140.0,
          145.0,
          150.0,
          155.0,
          160.0,
          165.0,
          170.0,
          175.0,
          180.0
        ],
        "hasMiniOptions": false,
        "quote": {
          "language": "en-US",
          "region": "US",
          "quoteType": "EQUITY",
          "currency": "USD",
          "regularMarketPrice": 150.25,
          "...": "..."
        },
        "options": [
          {
            "expirationDate": 1640995200,
            "hasMiniOptions": false,
            "calls": [
              {
                "contractSymbol": "AAPL211231C00150000",
                "strike": 150.0,
                "currency": "USD",
                "lastPrice": 3.45,
                "change": 0.25,
                "percentChange": 7.81,
                "volume": 12500,
                "openInterest": 45000,
                "bid": 3.40,
                "ask": 3.50,
                "contractSize": "REGULAR",
                "expiration": 1640995200,
                "lastTradeDate": 1640980800,
                "impliedVolatility": 0.28125,
                "inTheMoney": true,
                "delta": 0.5234,
                "gamma": 0.0234,
                "theta": -0.0123,
                "vega": 0.0456,
                "rho": 0.0089
              }
            ],
            "puts": [
              {
                "contractSymbol": "AAPL211231P00150000",
                "strike": 150.0,
                "currency": "USD",
                "lastPrice": 2.75,
                "change": -0.15,
                "percentChange": -5.17,
                "volume": 8500,
                "openInterest": 32000,
                "bid": 2.70,
                "ask": 2.80,
                "contractSize": "REGULAR",
                "expiration": 1640995200,
                "lastTradeDate": 1640980800,
                "impliedVolatility": 0.26875,
                "inTheMoney": false,
                "delta": -0.4766,
                "gamma": 0.0234,
                "theta": -0.0098,
                "vega": 0.0456,
                "rho": -0.0081
              }
            ]
          }
        ]
      }
    ],
    "error": null
  }
}
```

## GET /v7/finance/download/{symbol} (CSV)

**Response is CSV, not JSON:**
```csv
Date,Open,High,Low,Close,Adj Close,Volume
2021-01-04,133.520004,133.610001,126.760002,129.410004,128.350876,143301900
2021-01-05,128.889999,131.740005,128.429993,131.009995,129.939697,97664900
2021-01-06,127.720001,131.050003,126.379997,126.599998,125.561348,155088000
2021-01-07,128.360001,131.630005,127.860001,130.919998,129.850449,109578200
2021-01-08,132.429993,132.630005,130.229996,132.050003,130.970947,105158200
2021-01-11,129.190002,130.169998,128.500000,128.979996,127.925728,100620900
```

## POST /v1/finance/screener

**Request Body:**
```json
{
  "size": 25,
  "offset": 0,
  "sortField": "intradaymarketcap",
  "sortType": "DESC",
  "quoteType": "EQUITY",
  "query": {
    "operator": "AND",
    "operands": [
      {
        "operator": "GT",
        "operands": ["intradaymarketcap", 2000000000]
      },
      {
        "operator": "LT",
        "operands": ["intradaymarketcap", 100000000000]
      }
    ]
  },
  "userId": "",
  "userIdType": "guid"
}
```

**Response:**
```json
{
  "finance": {
    "result": [
      {
        "id": "8040e2e4-e5f4-4984-bb93-1e25bb3d1688",
        "title": "",
        "description": "",
        "canonicalName": "",
        "criteriaMeta": {
          "size": 25,
          "offset": 0,
          "sortField": "intradaymarketcap",
          "sortType": "DESC",
          "quoteType": "EQUITY",
          "topOperator": "AND",
          "criteria": [
            {
              "field": "intradaymarketcap",
              "operators": ["GT"],
              "values": [2000000000],
              "labelsSelected": [2000000000]
            },
            {
              "field": "intradaymarketcap",
              "operators": ["LT"],
              "values": [100000000000],
              "labelsSelected": [100000000000]
            }
          ]
        },
        "rawCriteria": "{\"operator\":\"AND\",\"operands\":[{\"operator\":\"GT\",\"operands\":[\"intradaymarketcap\",2000000000]},{\"operator\":\"LT\",\"operands\":[\"intradaymarketcap\",100000000000]}]}",
        "start": 0,
        "count": 25,
        "total": 1247,
        "quotes": [
          {
            "symbol": "AAPL",
            "shortName": "Apple Inc.",
            "longName": "Apple Inc.",
            "sector": "Technology",
            "industry": "Consumer Electronics",
            "marketCap": 2483595000000,
            "regularMarketPrice": 150.25,
            "regularMarketChange": 1.25,
            "regularMarketChangePercent": 0.835570469,
            "regularMarketVolume": 75234000,
            "...": "..."
          },
          {
            "symbol": "MSFT",
            "...": "..."
          }
        ]
      }
    ],
    "error": null
  }
}
```

## GET /v1/finance/search

```json
{
  "explains": [],
  "count": 10,
  "quotes": [
    {
      "exchange": "NMS",
      "shortname": "Apple Inc.",
      "quoteType": "EQUITY",
      "symbol": "AAPL",
      "index": "quotes",
      "score": 500141.0,
      "typeDisp": "Equity",
      "longname": "Apple Inc.",
      "exchDisp": "NASDAQ",
      "sector": "Technology",
      "industry": "Consumer Electronics",
      "dispSecIndFlag": true,
      "isYahooFinance": true
    },
    {
      "exchange": "NMS",
      "shortname": "Apple Hospitality REIT Inc.",
      "quoteType": "EQUITY",
      "symbol": "APLE",
      "index": "quotes",
      "score": 14223.0,
      "typeDisp": "Equity",
      "longname": "Apple Hospitality REIT, Inc.",
      "exchDisp": "NASDAQ",
      "isYahooFinance": true
    }
  ],
  "news": [
    {
      "uuid": "abc12345-def6-7890-ghij-klmnopqrstuv",
      "title": "Apple Reports Record Q4 Earnings",
      "publisher": "Bloomberg",
      "link": "https://finance.yahoo.com/news/...",
      "providerPublishTime": 1640980800,
      "type": "STORY",
      "thumbnail": {
        "resolutions": [
          {
            "url": "https://s.yimg.com/...",
            "width": 140,
            "height": 140,
            "tag": "140x140"
          }
        ]
      },
      "relatedTickers": ["AAPL"]
    }
  ],
  "nav": [],
  "lists": [],
  "researchReports": [],
  "screenerFieldResults": [],
  "totalTime": 147,
  "timeTakenForQuotes": 400,
  "timeTakenForNews": 412,
  "timeTakenForAlgowatchlist": 400,
  "timeTakenForPredefinedScreener": 400,
  "timeTakenForCrunchbase": 400,
  "timeTakenForNav": 400,
  "timeTakenForResearchReports": 0,
  "timeTakenForScreenerField": 0,
  "timeTakenForCulturalAssets": 0
}
```

## GET /v1/finance/trending/{region}

```json
{
  "finance": {
    "result": [
      {
        "count": 20,
        "quotes": [
          {
            "symbol": "TSLA",
            "regularMarketPrice": 1045.50,
            "regularMarketChange": 35.25,
            "regularMarketChangePercent": 3.49,
            "regularMarketVolume": 25000000,
            "...": "..."
          },
          {
            "symbol": "GME",
            "...": "..."
          }
        ],
        "jobTimestamp": 1640995200000,
        "startInterval": 202112312300
      }
    ],
    "error": null
  }
}
```

## WebSocket: PricingData (Protobuf decoded to JSON)

**Note:** Actual WebSocket messages are Protobuf binary. This is the decoded representation.

```json
{
  "id": "AAPL",
  "price": 150.27,
  "time": 1640995234567,
  "currency": "USD",
  "exchange": "NMS",
  "quoteType": 1,
  "marketHours": 1,
  "changePercent": 0.85,
  "dayHigh": 151.50,
  "dayLow": 149.00,
  "dayOpen": 149.50,
  "previousClose": 149.00,
  "bid": 150.26,
  "ask": 150.28,
  "bidSize": 12,
  "askSize": 8,
  "volume": 75345000,
  "change": 1.27,
  "shortName": "Apple Inc.",
  "exchangeName": "NasdaqGS",
  "sourceInterval": 15,
  "exchangeDataDelayed": 0,
  "tradeable": "true",
  "changePercentRealTime": 0.85,
  "changeRealTime": 1.27,
  "priceRealTime": 150.27,
  "exchangeTimezone": -18000000,
  "exchangeTimezoneName": "America/New_York",
  "gmtOffset": -18000,
  "marketState": "REGULAR"
}
```

## Error Responses

### HTTP 404 - Symbol Not Found
```json
{
  "chart": {
    "result": null,
    "error": {
      "code": "Not Found",
      "description": "No data found for this date range, symbol may be delisted"
    }
  }
}
```

### HTTP 401 - Invalid Cookie/Crumb
```json
{
  "finance": {
    "error": {
      "code": "Unauthorized",
      "description": "Invalid Cookie"
    }
  }
}
```

### HTTP 429 - Rate Limit (Plain Text)
```
Too Many Requests
```
(Not JSON!)

### General API Error
```json
{
  "quoteSummary": {
    "result": null,
    "error": {
      "code": "Bad Request",
      "description": "Invalid module requested"
    }
  }
}
```

## Key Field Explanations

### Price Fields
- `regularMarketPrice`: Current market price during regular trading hours
- `preMarketPrice`: Price during pre-market hours (4:00-9:30 AM ET)
- `postMarketPrice`: Price during after-hours trading (4:00-8:00 PM ET)
- `bid`: Current best bid price
- `ask`: Current best ask price

### Time Fields
- All timestamps are **Unix epoch** (seconds since Jan 1, 1970)
- `regularMarketTime`: Last update during regular hours
- `preMarketTime`: Last update during pre-market
- `postMarketTime`: Last update during after-hours
- `earningsTimestamp`: Next earnings release date

### Volume Fields
- `regularMarketVolume`: Volume during regular trading hours
- `averageDailyVolume3Month`: 3-month average daily volume
- `averageDailyVolume10Day`: 10-day average daily volume

### Price Change Fields
- `regularMarketChange`: Absolute price change from previous close
- `regularMarketChangePercent`: Percentage change from previous close
- Multiply by 100 to get percentage (e.g., 0.835 = 0.835%)

### Market Cap
- `marketCap`: Total market capitalization (price × shares outstanding)
- Value in currency (usually USD)

### Valuation Ratios
- `trailingPE`: Price-to-Earnings ratio (trailing 12 months)
- `forwardPE`: Forward P/E (based on estimated earnings)
- `priceToBook`: Price-to-Book ratio
- `epsTrailingTwelveMonths`: Earnings per share (TTM)

### Quote Types
- `EQUITY`: Stock
- `ETF`: Exchange-Traded Fund
- `MUTUALFUND`: Mutual Fund
- `INDEX`: Index
- `CURRENCY`: Forex pair
- `CRYPTOCURRENCY`: Cryptocurrency
- `FUTURE`: Futures contract
- `OPTION`: Options contract

### Market State
- `PRE`: Pre-market trading (before 9:30 AM ET)
- `REGULAR`: Regular trading hours (9:30 AM - 4:00 PM ET)
- `POST`: After-hours trading (after 4:00 PM ET)
- `CLOSED`: Market closed
- `PREPRE`: Early pre-market (some exchanges)
- `POSTPOST`: Late after-hours (some exchanges)
