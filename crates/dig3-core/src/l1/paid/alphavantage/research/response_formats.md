# AlphaVantage - Response Formats

**All examples from official AlphaVantage API - exact structure, not invented.**

## General Response Structure

AlphaVantage responses typically have two main sections:
1. **Meta Data** - Information about the request and data
2. **Data Section** - Actual time series or result data

Field names use numbered prefixes (e.g., "1. open", "2. high").

---

## Stock Time Series

### TIME_SERIES_DAILY

**Request**:
```
GET https://www.alphavantage.co/query?function=TIME_SERIES_DAILY&symbol=IBM&apikey=demo
```

**Response**:
```json
{
  "Meta Data": {
    "1. Information": "Daily Prices (open, high, low, close) and Volumes",
    "2. Symbol": "IBM",
    "3. Last Refreshed": "2024-01-25",
    "4. Output Size": "Compact",
    "5. Time Zone": "US/Eastern"
  },
  "Time Series (Daily)": {
    "2024-01-25": {
      "1. open": "185.4900",
      "2. high": "187.7900",
      "3. low": "184.8000",
      "4. close": "186.0900",
      "5. volume": "5428898"
    },
    "2024-01-24": {
      "1. open": "186.5000",
      "2. high": "187.6500",
      "3. low": "185.7600",
      "4. close": "186.7300",
      "5. volume": "7206448"
    }
  }
}
```

**Fields**:
- `1. open` (string): Opening price
- `2. high` (string): High price
- `3. low` (string): Low price
- `4. close` (string): Closing price
- `5. volume` (string): Trading volume

**Note**: All numeric values returned as strings.

### TIME_SERIES_INTRADAY (Premium)

**Request**:
```
GET https://www.alphavantage.co/query?function=TIME_SERIES_INTRADAY&symbol=IBM&interval=5min&apikey=YOUR_KEY
```

**Response**:
```json
{
  "Meta Data": {
    "1. Information": "Intraday (5min) open, high, low, close prices and volume",
    "2. Symbol": "IBM",
    "3. Last Refreshed": "2024-01-25 16:00:00",
    "4. Interval": "5min",
    "5. Output Size": "Compact",
    "6. Time Zone": "US/Eastern"
  },
  "Time Series (5min)": {
    "2024-01-25 16:00:00": {
      "1. open": "186.2000",
      "2. high": "186.4500",
      "3. low": "186.1000",
      "4. close": "186.3200",
      "5. volume": "125834"
    },
    "2024-01-25 15:55:00": {
      "1. open": "186.1500",
      "2. high": "186.2800",
      "3. low": "186.1200",
      "4. close": "186.2100",
      "5. volume": "98456"
    }
  }
}
```

**Note**: Key name changes based on interval: "Time Series (1min)", "Time Series (5min)", etc.

### GLOBAL_QUOTE

**Request**:
```
GET https://www.alphavantage.co/query?function=GLOBAL_QUOTE&symbol=IBM&apikey=demo
```

**Response**:
```json
{
  "Global Quote": {
    "01. symbol": "IBM",
    "02. open": "186.3000",
    "03. high": "187.9000",
    "04. low": "185.5000",
    "05. price": "186.7500",
    "06. volume": "4563200",
    "07. latest trading day": "2024-01-25",
    "08. previous close": "185.9000",
    "09. change": "0.8500",
    "10. change percent": "0.4571%"
  }
}
```

**Fields**:
- `01. symbol`: Ticker symbol
- `02. open`: Opening price
- `03. high`: Day high
- `04. low`: Day low
- `05. price`: Current/close price
- `06. volume`: Volume
- `07. latest trading day`: Date
- `08. previous close`: Previous close
- `09. change`: Price change
- `10. change percent`: Percent change with % symbol

---

## Forex (FX)

### CURRENCY_EXCHANGE_RATE

**Request**:
```
GET https://www.alphavantage.co/query?function=CURRENCY_EXCHANGE_RATE&from_currency=EUR&to_currency=USD&apikey=YOUR_KEY
```

**Response**:
```json
{
  "Realtime Currency Exchange Rate": {
    "1. From_Currency Code": "EUR",
    "2. From_Currency Name": "Euro",
    "3. To_Currency Code": "USD",
    "4. To_Currency Name": "United States Dollar",
    "5. Exchange Rate": "1.08450000",
    "6. Last Refreshed": "2024-01-25 15:30:01",
    "7. Time Zone": "UTC",
    "8. Bid Price": "1.08440000",
    "9. Ask Price": "1.08460000"
  }
}
```

**Fields**:
- `5. Exchange Rate`: Current exchange rate
- `8. Bid Price`: Bid price
- `9. Ask Price`: Ask price

### FX_DAILY

**Request**:
```
GET https://www.alphavantage.co/query?function=FX_DAILY&from_symbol=EUR&to_symbol=USD&apikey=YOUR_KEY
```

**Response**:
```json
{
  "Meta Data": {
    "1. Information": "Forex Daily Prices (open, high, low, close)",
    "2. From Symbol": "EUR",
    "3. To Symbol": "USD",
    "4. Output Size": "Compact",
    "5. Last Refreshed": "2024-01-25 23:59:59",
    "6. Time Zone": "GMT+8"
  },
  "Time Series FX (Daily)": {
    "2024-01-25": {
      "1. open": "1.08300000",
      "2. high": "1.08850000",
      "3. low": "1.08250000",
      "4. close": "1.08450000"
    },
    "2024-01-24": {
      "1. open": "1.08100000",
      "2. high": "1.08400000",
      "3. low": "1.08000000",
      "4. close": "1.08300000"
    }
  }
}
```

**Note**: FX data has OHLC but **no volume**.

### FX_INTRADAY (Premium)

**Request**:
```
GET https://www.alphavantage.co/query?function=FX_INTRADAY&from_symbol=EUR&to_symbol=USD&interval=5min&apikey=YOUR_KEY
```

**Response**:
```json
{
  "Meta Data": {
    "1. Information": "FX Intraday (5min) Time Series",
    "2. From Symbol": "EUR",
    "3. To Symbol": "USD",
    "4. Last Refreshed": "2024-01-25 15:30:00",
    "5. Interval": "5min",
    "6. Output Size": "Compact",
    "7. Time Zone": "GMT"
  },
  "Time Series FX (5min)": {
    "2024-01-25 15:30:00": {
      "1. open": "1.08420000",
      "2. high": "1.08460000",
      "3. low": "1.08400000",
      "4. close": "1.08450000"
    },
    "2024-01-25 15:25:00": {
      "1. open": "1.08390000",
      "2. high": "1.08430000",
      "3. low": "1.08380000",
      "4. close": "1.08420000"
    }
  }
}
```

---

## Cryptocurrency

### DIGITAL_CURRENCY_DAILY

**Request**:
```
GET https://www.alphavantage.co/query?function=DIGITAL_CURRENCY_DAILY&symbol=BTC&market=USD&apikey=YOUR_KEY
```

**Response**:
```json
{
  "Meta Data": {
    "1. Information": "Daily Prices and Volumes for Digital Currency",
    "2. Digital Currency Code": "BTC",
    "3. Digital Currency Name": "Bitcoin",
    "4. Market Code": "USD",
    "5. Market Name": "United States Dollar",
    "6. Last Refreshed": "2024-01-25 00:00:00",
    "7. Time Zone": "UTC"
  },
  "Time Series (Digital Currency Daily)": {
    "2024-01-25": {
      "1a. open (USD)": "42350.50000000",
      "1b. open (USD)": "42350.50000000",
      "2a. high (USD)": "43200.00000000",
      "2b. high (USD)": "43200.00000000",
      "3a. low (USD)": "42100.00000000",
      "3b. low (USD)": "42100.00000000",
      "4a. close (USD)": "42850.00000000",
      "4b. close (USD)": "42850.00000000",
      "5. volume": "28545.12000000",
      "6. market cap (USD)": "1220450000.00000000"
    },
    "2024-01-24": {
      "1a. open (USD)": "42100.00000000",
      "1b. open (USD)": "42100.00000000",
      "2a. high (USD)": "42500.00000000",
      "2b. high (USD)": "42500.00000000",
      "3a. low (USD)": "41800.00000000",
      "3b. low (USD)": "41800.00000000",
      "4a. close (USD)": "42350.50000000",
      "4b. close (USD)": "42350.50000000",
      "5. volume": "25600.00000000",
      "6. market cap (USD)": "1076800000.00000000"
    }
  }
}
```

**Fields**:
- `1a. open (USD)` / `1b. open (USD)`: Open price (a and b same for USD)
- Similar pattern for high, low, close
- `5. volume`: Trading volume
- `6. market cap (USD)`: Market capitalization

**Note**: Crypto responses have duplicate `a` and `b` fields (for different market currencies).

### CRYPTO_RATING

**Request**:
```
GET https://www.alphavantage.co/query?function=CRYPTO_RATING&symbol=BTC&apikey=YOUR_KEY
```

**Response**:
```json
{
  "Crypto Rating (FCAS)": {
    "1. symbol": "BTC",
    "2. name": "Bitcoin",
    "3. fcas rating": "Superb",
    "4. fcas score": "912",
    "5. developer score": "942",
    "6. market maturity score": "850",
    "7. utility score": "945",
    "8. last refreshed": "2024-01-25 00:00:00",
    "9. timezone": "UTC"
  }
}
```

**FCAS Ratings**: Superb, Attractive, Basic, Caution

---

## Technical Indicators

### SMA (Simple Moving Average)

**Request**:
```
GET https://www.alphavantage.co/query?function=SMA&symbol=IBM&interval=daily&time_period=20&series_type=close&apikey=YOUR_KEY
```

**Response**:
```json
{
  "Meta Data": {
    "1: Symbol": "IBM",
    "2: Indicator": "Simple Moving Average (SMA)",
    "3: Last Refreshed": "2024-01-25",
    "4: Interval": "daily",
    "5: Time Period": 20,
    "6: Series Type": "close",
    "7: Time Zone": "US/Eastern"
  },
  "Technical Analysis: SMA": {
    "2024-01-25": {
      "SMA": "185.2450"
    },
    "2024-01-24": {
      "SMA": "185.1200"
    },
    "2024-01-23": {
      "SMA": "185.0100"
    }
  }
}
```

**Note**: Technical indicators return single value per timestamp.

### RSI (Relative Strength Index)

**Request**:
```
GET https://www.alphavantage.co/query?function=RSI&symbol=IBM&interval=daily&time_period=14&series_type=close&apikey=YOUR_KEY
```

**Response**:
```json
{
  "Meta Data": {
    "1: Symbol": "IBM",
    "2: Indicator": "Relative Strength Index (RSI)",
    "3: Last Refreshed": "2024-01-25",
    "4: Interval": "daily",
    "5: Time Period": 14,
    "6: Series Type": "close",
    "7: Time Zone": "US/Eastern"
  },
  "Technical Analysis: RSI": {
    "2024-01-25": {
      "RSI": "52.3456"
    },
    "2024-01-24": {
      "RSI": "51.8765"
    }
  }
}
```

### BBANDS (Bollinger Bands)

**Request**:
```
GET https://www.alphavantage.co/query?function=BBANDS&symbol=IBM&interval=daily&time_period=20&series_type=close&apikey=YOUR_KEY
```

**Response**:
```json
{
  "Meta Data": {
    "1: Symbol": "IBM",
    "2: Indicator": "Bollinger Bands (BBANDS)",
    "3: Last Refreshed": "2024-01-25",
    "4: Interval": "daily",
    "5: Time Period": 20,
    "6: Series Type": "close",
    "7: Time Zone": "US/Eastern"
  },
  "Technical Analysis: BBANDS": {
    "2024-01-25": {
      "Real Upper Band": "188.5678",
      "Real Middle Band": "186.0900",
      "Real Lower Band": "183.6122"
    },
    "2024-01-24": {
      "Real Upper Band": "188.4321",
      "Real Middle Band": "186.0000",
      "Real Lower Band": "183.5679"
    }
  }
}
```

**Note**: Bollinger Bands return three values: upper, middle, lower bands.

---

## Fundamental Data

### COMPANY_OVERVIEW

**Request**:
```
GET https://www.alphavantage.co/query?function=COMPANY_OVERVIEW&symbol=IBM&apikey=YOUR_KEY
```

**Response** (truncated for brevity):
```json
{
  "Symbol": "IBM",
  "AssetType": "Common Stock",
  "Name": "International Business Machines Corporation",
  "Description": "International Business Machines Corporation...",
  "CIK": "51143",
  "Exchange": "NYSE",
  "Currency": "USD",
  "Country": "USA",
  "Sector": "TECHNOLOGY",
  "Industry": "COMPUTER & OFFICE EQUIPMENT",
  "Address": "1 NEW ORCHARD ROAD, ARMONK, NY, US",
  "FiscalYearEnd": "December",
  "LatestQuarter": "2023-09-30",
  "MarketCapitalization": "170538000000",
  "EBITDA": "12345000000",
  "PERatio": "22.45",
  "PEGRatio": "1.35",
  "BookValue": "23.45",
  "DividendPerShare": "6.60",
  "DividendYield": "0.0352",
  "EPS": "8.29",
  "RevenuePerShareTTM": "63.45",
  "ProfitMargin": "0.1305",
  "OperatingMarginTTM": "0.1456",
  "ReturnOnAssetsTTM": "0.0385",
  "ReturnOnEquityTTM": "0.2567",
  "RevenueTTM": "61860000000",
  "GrossProfitTTM": "32156000000",
  "DilutedEPSTTM": "8.29",
  "QuarterlyEarningsGrowthYOY": "0.023",
  "QuarterlyRevenueGrowthYOY": "0.015",
  "AnalystTargetPrice": "165.00",
  "TrailingPE": "22.45",
  "ForwardPE": "18.50",
  "PriceToSalesRatioTTM": "2.75",
  "PriceToBookRatio": "7.95",
  "EVToRevenue": "2.95",
  "EVToEBITDA": "14.35",
  "Beta": "0.875",
  "52WeekHigh": "190.00",
  "52WeekLow": "115.00",
  "50DayMovingAverage": "150.25",
  "200DayMovingAverage": "140.35",
  "SharesOutstanding": "915000000",
  "DividendDate": "2024-03-09",
  "ExDividendDate": "2024-02-08"
}
```

**Note**: Single object, not time series. No "Meta Data" section.

### EARNINGS

**Request**:
```
GET https://www.alphavantage.co/query?function=EARNINGS&symbol=IBM&apikey=YOUR_KEY
```

**Response**:
```json
{
  "symbol": "IBM",
  "annualEarnings": [
    {
      "fiscalDateEnding": "2023-12-31",
      "reportedEPS": "8.29"
    },
    {
      "fiscalDateEnding": "2022-12-31",
      "reportedEPS": "7.16"
    }
  ],
  "quarterlyEarnings": [
    {
      "fiscalDateEnding": "2023-09-30",
      "reportedDate": "2023-10-25",
      "reportedEPS": "2.20",
      "estimatedEPS": "2.15",
      "surprise": "0.05",
      "surprisePercentage": "2.3256"
    },
    {
      "fiscalDateEnding": "2023-06-30",
      "reportedDate": "2023-07-19",
      "reportedEPS": "2.18",
      "estimatedEPS": "2.12",
      "surprise": "0.06",
      "surprisePercentage": "2.8302"
    }
  ]
}
```

**Note**: Array format for historical data.

### INCOME_STATEMENT

**Request**:
```
GET https://www.alphavantage.co/query?function=INCOME_STATEMENT&symbol=IBM&apikey=YOUR_KEY
```

**Response**:
```json
{
  "symbol": "IBM",
  "annualReports": [
    {
      "fiscalDateEnding": "2023-12-31",
      "reportedCurrency": "USD",
      "grossProfit": "32156000000",
      "totalRevenue": "61860000000",
      "costOfRevenue": "29704000000",
      "costofGoodsAndServicesSold": "29704000000",
      "operatingIncome": "9012000000",
      "sellingGeneralAndAdministrative": "15234000000",
      "researchAndDevelopment": "7910000000",
      "operatingExpenses": "23144000000",
      "investmentIncomeNet": "156000000",
      "netInterestIncome": "-1234000000",
      "interestIncome": "234000000",
      "interestExpense": "1468000000",
      "nonInterestIncome": "62094000000",
      "otherNonOperatingIncome": "123000000",
      "depreciation": "3456000000",
      "depreciationAndAmortization": "4567000000",
      "incomeBeforeTax": "7901000000",
      "incomeTaxExpense": "987000000",
      "interestAndDebtExpense": "1468000000",
      "netIncomeFromContinuingOperations": "6914000000",
      "comprehensiveIncomeNetOfTax": "7234000000",
      "ebit": "9369000000",
      "ebitda": "13936000000",
      "netIncome": "7582000000"
    }
  ],
  "quarterlyReports": [
    {
      "fiscalDateEnding": "2023-09-30",
      "reportedCurrency": "USD",
      "grossProfit": "8123000000",
      "totalRevenue": "15536000000",
      "costOfRevenue": "7413000000",
      "operatingIncome": "2345000000",
      "netIncome": "2012000000",
      "ebitda": "3456000000"
    }
  ]
}
```

**Note**: Extensive financial details, annual and quarterly.

---

## Economic Indicators

### REAL_GDP

**Request**:
```
GET https://www.alphavantage.co/query?function=REAL_GDP&interval=quarterly&apikey=YOUR_KEY
```

**Response**:
```json
{
  "name": "Real Gross Domestic Product",
  "interval": "quarterly",
  "unit": "billions of dollars",
  "data": [
    {
      "date": "2023-09-30",
      "value": "22017.832"
    },
    {
      "date": "2023-06-30",
      "value": "21862.555"
    },
    {
      "date": "2023-03-31",
      "value": "21704.710"
    }
  ]
}
```

**Note**: Economic indicators use `data` array with `date` and `value` fields.

### TREASURY_YIELD

**Request**:
```
GET https://www.alphavantage.co/query?function=TREASURY_YIELD&interval=monthly&maturity=10year&apikey=YOUR_KEY
```

**Response**:
```json
{
  "name": "10-Year Treasury Constant Maturity Rate",
  "interval": "monthly",
  "unit": "percent",
  "data": [
    {
      "date": "2024-01-01",
      "value": "4.25"
    },
    {
      "date": "2023-12-01",
      "value": "4.18"
    },
    {
      "date": "2023-11-01",
      "value": "4.52"
    }
  ]
}
```

### CPI (Consumer Price Index)

**Request**:
```
GET https://www.alphavantage.co/query?function=CPI&interval=monthly&apikey=YOUR_KEY
```

**Response**:
```json
{
  "name": "Consumer Price Index",
  "interval": "monthly",
  "unit": "index 1982-1984=100",
  "data": [
    {
      "date": "2023-12-01",
      "value": "306.746"
    },
    {
      "date": "2023-11-01",
      "value": "307.051"
    }
  ]
}
```

---

## News & Sentiment

### NEWS_SENTIMENT

**Request**:
```
GET https://www.alphavantage.co/query?function=NEWS_SENTIMENT&tickers=IBM&apikey=YOUR_KEY
```

**Response**:
```json
{
  "items": "50",
  "sentiment_score_definition": "x <= -0.35: Bearish; -0.35 < x <= -0.15: Somewhat-Bearish; -0.15 < x < 0.15: Neutral; 0.15 <= x < 0.35: Somewhat_Bullish; x >= 0.35: Bullish",
  "relevance_score_definition": "0 < x <= 1, with a higher score indicating higher relevance.",
  "feed": [
    {
      "title": "IBM Reports Strong Q4 Earnings",
      "url": "https://example.com/news/ibm-earnings",
      "time_published": "20240125T153000",
      "authors": ["John Doe"],
      "summary": "IBM reported quarterly earnings that beat analyst expectations...",
      "banner_image": "https://example.com/image.jpg",
      "source": "CNBC",
      "category_within_source": "Business",
      "source_domain": "cnbc.com",
      "topics": [
        {
          "topic": "Earnings",
          "relevance_score": "1.0"
        },
        {
          "topic": "Technology",
          "relevance_score": "0.928"
        }
      ],
      "overall_sentiment_score": 0.425,
      "overall_sentiment_label": "Bullish",
      "ticker_sentiment": [
        {
          "ticker": "IBM",
          "relevance_score": "0.985",
          "ticker_sentiment_score": "0.452",
          "ticker_sentiment_label": "Bullish"
        }
      ]
    }
  ]
}
```

**Fields**:
- `overall_sentiment_score`: -1 (bearish) to 1 (bullish)
- `overall_sentiment_label`: Bearish, Somewhat-Bearish, Neutral, Somewhat-Bullish, Bullish
- `ticker_sentiment`: Per-ticker sentiment and relevance
- `relevance_score`: 0-1, relevance to ticker

---

## Commodities

### WTI (Crude Oil)

**Request**:
```
GET https://www.alphavantage.co/query?function=WTI&interval=daily&apikey=YOUR_KEY
```

**Response**:
```json
{
  "name": "Crude Oil Prices: West Texas Intermediate (WTI)",
  "interval": "daily",
  "unit": "USD per barrel",
  "data": [
    {
      "date": "2024-01-25",
      "value": "78.45"
    },
    {
      "date": "2024-01-24",
      "value": "77.89"
    }
  ]
}
```

---

## Symbol Search & Metadata

### SYMBOL_SEARCH

**Request**:
```
GET https://www.alphavantage.co/query?function=SYMBOL_SEARCH&keywords=microsoft&apikey=YOUR_KEY
```

**Response**:
```json
{
  "bestMatches": [
    {
      "1. symbol": "MSFT",
      "2. name": "Microsoft Corporation",
      "3. type": "Equity",
      "4. region": "United States",
      "5. marketOpen": "09:30",
      "6. marketClose": "16:00",
      "7. timezone": "UTC-04",
      "8. currency": "USD",
      "9. matchScore": "1.0000"
    },
    {
      "1. symbol": "MSF.DEX",
      "2. name": "Microsoft Corporation",
      "3. type": "Equity",
      "4. region": "XETRA",
      "5. marketOpen": "08:00",
      "6. marketClose": "20:00",
      "7. timezone": "UTC+01",
      "8. currency": "EUR",
      "9. matchScore": "0.8571"
    }
  ]
}
```

**Match Score**: 0-1, higher = better match

### MARKET_STATUS

**Request**:
```
GET https://www.alphavantage.co/query?function=MARKET_STATUS&apikey=YOUR_KEY
```

**Response**:
```json
{
  "endpoint": "Global Market Open & Close Status",
  "markets": [
    {
      "market_type": "Equity",
      "region": "United States",
      "primary_exchanges": "NASDAQ, NYSE, AMEX",
      "local_open": "09:30",
      "local_close": "16:00",
      "current_status": "open",
      "notes": ""
    },
    {
      "market_type": "Equity",
      "region": "United Kingdom",
      "primary_exchanges": "LSE",
      "local_open": "08:00",
      "local_close": "16:30",
      "current_status": "closed",
      "notes": ""
    },
    {
      "market_type": "Forex",
      "region": "Global",
      "primary_exchanges": "FX",
      "current_status": "open",
      "notes": "Forex market is open 24/5"
    },
    {
      "market_type": "Crypto",
      "region": "Global",
      "primary_exchanges": "Crypto",
      "current_status": "open",
      "notes": "Crypto market is open 24/7"
    }
  ]
}
```

---

## Error Responses

### Invalid API Key

```json
{
  "Error Message": "Invalid API call. Please retry or visit the documentation (https://www.alphavantage.co/documentation/) for API_KEY"
}
```

### Rate Limit Exceeded

```json
{
  "Note": "Thank you for using Alpha Vantage! Our standard API call frequency is 5 calls per minute. Please visit https://www.alphavantage.co/premium/ if you would like to target a higher API call frequency."
}
```

### Daily Limit Reached

```json
{
  "Note": "Thank you for using Alpha Vantage! You have reached the daily limit of 25 API requests. Please try again tomorrow or visit https://www.alphavantage.co/premium/ to upgrade."
}
```

### Invalid Parameter

```json
{
  "Error Message": "Invalid API call. Please retry or visit the documentation for SYMBOL"
}
```

### Premium Feature (Free Tier)

```json
{
  "Note": "This API endpoint is not available on your current plan. Please visit https://www.alphavantage.co/premium/ to upgrade."
}
```

---

## Data Type Notes

1. **All numeric values are strings** in responses
2. **Date formats**:
   - Daily: "YYYY-MM-DD"
   - Intraday: "YYYY-MM-DD HH:MM:SS"
   - Economic: "YYYY-MM-DD"
3. **Field naming**: Numbered prefixes (1., 2., 3., etc.)
4. **Timezones**: Usually "US/Eastern" for stocks, "UTC" for crypto/forex
5. **Volume**: Trading volume (stocks, crypto) - NOT in forex
6. **Percentages**: Sometimes include "%" symbol (e.g., GLOBAL_QUOTE change percent)

---

## CSV Format

For `datatype=csv`, responses are CSV strings instead of JSON:

**Example**:
```csv
timestamp,open,high,low,close,volume
2024-01-25,186.30,187.90,185.50,186.75,4563200
2024-01-24,185.90,186.50,185.20,186.30,5234100
```

**Note**: CSV format varies by endpoint. Check documentation for specific CSV structure.
