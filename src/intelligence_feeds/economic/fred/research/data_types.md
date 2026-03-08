# FRED - Data Types Catalog

FRED provides 840,000+ economic time series from 118 sources covering U.S. and international economic data.

## Standard Market Data

- [ ] Current Price - Not applicable (economic data, not market prices)
- [ ] Bid/Ask Spread - Not applicable
- [ ] 24h Ticker Stats - Not applicable
- [ ] OHLC/Candlesticks - Not applicable (economic indicators use single values)
- [ ] Level 2 Orderbook - Not applicable
- [ ] Recent Trades - Not applicable
- [ ] Volume (24h, intraday) - Not applicable

**Note**: FRED is for economic/macro data, not market trading data.

## Historical Data

- [x] Historical prices - For financial market series (stock indices, commodity prices, exchange rates)
- [x] Minute bars - Not available (lowest frequency is daily)
- [x] Daily bars - Available for daily frequency series
- [ ] Tick data - Not available
- [x] Adjusted prices - Some series provide inflation-adjusted values

**Historical depth**: Varies by series:
- Some series from 1700s (very rare)
- Many series from early 1900s
- Most series from 1950s onward
- New series added continuously

## Derivatives Data (Crypto/Futures)

Not applicable - FRED does not provide derivatives market data.

## Options Data (if applicable)

Not applicable - FRED does not provide options data.

## Fundamental Data (Stocks)

Limited stock market data:
- [x] Stock Market Indices (S&P 500, Dow Jones, NASDAQ, Wilshire 5000)
- [ ] Company Profile - Not available
- [ ] Financial Statements - Not available
- [ ] Earnings (EPS, revenue, guidance) - Not available
- [x] Dividends - Dividend yield series available for indices
- [ ] Stock Splits - Not available
- [ ] Analyst Ratings - Not available
- [ ] Insider Trading - Not available
- [ ] Institutional Holdings - Not available
- [x] Financial Ratios - Some aggregate ratios (Shiller P/E for S&P 500)
- [ ] Valuation Metrics - Limited (primarily index-level)

## On-chain Data (Crypto)

Not applicable - FRED does not provide blockchain/crypto data.

## Macro/Economic Data (Economics)

**THIS IS FRED'S PRIMARY FOCUS** - 80+ major categories of economic data:

### National Accounts
- [x] GDP (Real, Nominal, by component, by industry)
  - Series: GDP, GDPC1, GDPCA, etc.
  - Quarterly and annual frequencies
  - Chain-weighted, inflation-adjusted versions
- [x] GNP (Gross National Product)
  - Series: GNP, GNPCA
- [x] Personal Income
  - Series: PI, DSPI (Disposable Personal Income)
- [x] Personal Consumption Expenditures (PCE)
  - Series: PCE, PCEC96
- [x] Investment (Business, Residential)
- [x] Government Spending
- [x] Net Exports
- [x] Inventory Changes

### Prices & Inflation
- [x] CPI (Consumer Price Index)
  - Series: CPIAUCSL (All Urban Consumers)
  - Multiple variants (all items, core, energy, food)
  - 320+ CPI series
- [x] PPI (Producer Price Index)
  - 10,000+ PPI series
  - By industry, commodity, stage of processing
- [x] PCE Price Index (Fed's preferred inflation measure)
  - Series: PCEPI, PCEPILFE (core)
- [x] GDP Deflator
- [x] Import/Export Price Indices
- [x] Core inflation measures (excluding food & energy)

### Employment & Labor Market
- [x] Unemployment Rate
  - Series: UNRATE (headline)
  - U1-U6 alternative measures
  - By state, metro area, demographic
- [x] Nonfarm Payrolls (NFP)
  - Series: PAYEMS
  - 440+ Current Employment Statistics series
  - By industry, sector, occupation
- [x] Labor Force Participation Rate
  - Series: CIVPART
- [x] Initial Jobless Claims
  - Series: ICSA (weekly)
- [x] Continuing Claims
- [x] JOLTS (Job Openings and Labor Turnover)
  - Job openings, hires, separations, quits
- [x] Average Hourly Earnings
  - Series: CES0500000003
- [x] Hours Worked
- [x] Employment Cost Index
- [x] Productivity measures

### Interest Rates
- [x] Federal Funds Rate
  - Series: DFF (effective), FEDFUNDS
  - Target rate, upper/lower bounds
- [x] Treasury Yields
  - 1-month to 30-year maturities
  - Series: DGS1MO, DGS3MO, DGS1, DGS2, DGS5, DGS10, DGS30
- [x] LIBOR rates (historical - discontinued 2023)
- [x] SOFR (Secured Overnight Financing Rate)
- [x] Prime Rate
  - Series: DPRIME
- [x] Mortgage Rates
  - 15-year, 30-year fixed
  - Series: MORTGAGE15US, MORTGAGE30US
- [x] Corporate Bond Yields (AAA, BAA, High Yield)
  - Series: AAA, BAA
- [x] Municipal Bond Yields
- [x] Commercial Paper rates
- [x] Certificate of Deposit rates

### Money Supply & Banking
- [x] M1 Money Supply
  - Series: M1SL
- [x] M2 Money Supply
  - Series: M2SL
- [x] Money Velocity (M1V, M2V)
- [x] Monetary Base
- [x] Bank Credit
- [x] Commercial Bank Assets
- [x] Reserves (Required, Excess)
- [x] Bank Lending (Commercial, Industrial, Real Estate)
- [x] Delinquency Rates
- [x] Charge-off Rates

### Trade & Balance of Payments
- [x] Trade Balance
  - Goods, Services, Total
  - Series: BOPGSTB
- [x] Exports (by country, category)
- [x] Imports (by country, category)
- [x] Current Account Balance
- [x] Capital Account
- [x] Foreign Direct Investment

### Manufacturing & Production
- [x] Industrial Production Index
  - Series: INDPRO
  - By sector (manufacturing, mining, utilities)
- [x] Capacity Utilization
  - Series: TCU
- [x] Manufacturing Orders
- [x] Durable Goods Orders
  - Series: DGORDER
- [x] Factory Orders
- [x] Inventories
- [x] ISM Manufacturing PMI
  - Series: NAPM (now ISM)
- [x] ISM Services PMI

### Consumer Metrics
- [x] Retail Sales
  - Series: RSXFS
  - By category (auto, food, online, etc.)
- [x] Consumer Confidence
  - Series: UMCSENT (University of Michigan)
  - Conference Board Consumer Confidence
- [x] Consumer Credit
  - Total, revolving, non-revolving
- [x] Personal Saving Rate
  - Series: PSAVERT
- [x] Vehicle Sales
  - Series: TOTALSA

### Housing & Construction
- [x] Housing Starts
  - Series: HOUST
  - Single-family, multi-family
- [x] Building Permits
  - Series: PERMIT
- [x] New Home Sales
  - Series: HSN1F
- [x] Existing Home Sales
- [x] Home Prices
  - Case-Shiller Index: CSUSHPISA
  - FHFA House Price Index
  - Zillow indices
  - 390+ house price series
- [x] Homeownership Rate
- [x] Housing Inventory
- [x] Mortgage Applications
- [x] Construction Spending

### Regional Data (460,000+ series)
- [x] State-level economic indicators
  - GDP by state
  - Employment by state
  - Income by state
  - Population by state
- [x] Metropolitan Statistical Area (MSA) data
  - Employment
  - Unemployment
  - House prices
  - Income
- [x] County-level data (limited)

### International Data
- [x] Foreign Exchange Rates
  - 170+ exchange rate series
  - Major currencies vs USD
  - Trade-weighted dollar indices
  - Historical rates
- [x] International GDP
- [x] International Inflation
- [x] International Trade
- [x] Foreign central bank rates
- [x] OECD data

### Financial Markets
- [x] Stock Market Indices
  - S&P 500: SP500
  - Dow Jones: DJIA
  - NASDAQ Composite: NASDAQCOM
  - Wilshire 5000: WILL5000IND
  - Russell 2000
  - VIX (volatility index): VIXCLS
- [x] Bond Market Indices
- [x] Commodity Prices
  - Oil (WTI, Brent): DCOILWTICO, DCOILBRENTEU
  - Gold: GOLDAMGBD228NLBM
  - Silver, Copper, etc.
  - Agricultural commodities
- [x] Credit Spreads
  - TED Spread: TEDRATE
  - Corporate spreads
- [x] Financial Stress Indices

### Government Finance
- [x] Federal Debt
  - Total Public Debt: GFDEBTN
  - Debt held by public
  - Debt as % of GDP: GFDEGDQ188S
- [x] Federal Surplus/Deficit
- [x] Tax Receipts
- [x] Government Spending
  - By function (defense, education, etc.)
- [x] State & Local Government Finance

### Demographics & Population
- [x] Population (Total, by age, by state)
  - Series: POP
- [x] Birth Rates
- [x] Death Rates
- [x] Migration
- [x] Labor Force Demographics

### Energy
- [x] Oil Prices (WTI, Brent)
- [x] Natural Gas Prices
- [x] Gasoline Prices
- [x] Electricity Prices
- [x] Coal Prices
- [x] Energy Consumption
- [x] Energy Production

### Other Categories
- [x] Academic Data (research studies)
- [x] Business Surveys
- [x] Climate/Weather data (limited)
- [x] Financial Accounts of the US (Z.1)
- [x] Flow of Funds
- [x] Input-Output Tables
- [x] Household Surveys
- [x] Poverty Statistics
- [x] Income Distribution (GINI coefficient)

## Metadata & Reference

- [x] Series Lists - 840,000+ series available via /fred/series/search
- [x] Category Information - 80+ categories via /fred/categories
- [x] Release Information - Economic data releases via /fred/releases
- [x] Source Information - 118 sources via /fred/sources
- [x] Tag System - Extensive tagging (frequency, geography, type, etc.)
- [x] Vintage/Revision History (ALFRED) - Historical data revisions
- [x] Documentation per series (title, units, frequency, seasonal adjustment)

## News & Sentiment (if applicable)

- [ ] News Articles - Not available
- [ ] Press Releases - Release announcements via API
- [ ] Social Sentiment - Not available
- [ ] Analyst Reports - Not available

## Unique/Custom Data

**What makes FRED special:**

1. **Archival Data (ALFRED)**: Historical revisions of economic data
   - Track how GDP estimates changed over time
   - Research data revision patterns
   - Critical for economic research

2. **Real-time vs Vintage Data**:
   - Access data as it was known at any point in history
   - Essential for backtesting economic models

3. **Comprehensive Coverage**:
   - 840,000+ series (largest free economic database)
   - 118 different data sources
   - International + U.S. + Regional

4. **Long Historical Depth**:
   - Many series from 1940s-1950s
   - Some from 1800s or earlier
   - Unmatched for historical economic analysis

5. **Standardized API**:
   - Unified access to diverse sources
   - Consistent data format
   - Free and unrestricted (non-commercial use)

6. **Data Transformations**:
   - Built-in transformations (% change, log, etc.)
   - Frequency aggregation (daily → monthly → quarterly)
   - Seasonal adjustment options

7. **Geographic Granularity**:
   - National level
   - State level
   - MSA (metro area) level
   - County level (limited)

8. **Cross-sectional and Time-series**:
   - Panel data for states/regions
   - Long time series for national indicators

## Popular Series Examples

| Series ID | Description | Frequency | Start Date |
|-----------|-------------|-----------|------------|
| GDP | Gross Domestic Product | Quarterly | 1947 |
| GDPC1 | Real GDP | Quarterly | 1947 |
| UNRATE | Unemployment Rate | Monthly | 1948 |
| CPIAUCSL | Consumer Price Index | Monthly | 1947 |
| FEDFUNDS | Federal Funds Rate | Monthly | 1954 |
| DFF | Federal Funds Effective Rate | Daily | 1954 |
| DGS10 | 10-Year Treasury Yield | Daily | 1962 |
| SP500 | S&P 500 Index | Daily | 1927 |
| DCOILWTICO | WTI Crude Oil Price | Daily | 1986 |
| PAYEMS | Nonfarm Payrolls | Monthly | 1939 |
| HOUST | Housing Starts | Monthly | 1959 |
| RSXFS | Retail Sales | Monthly | 1992 |
| M2SL | M2 Money Supply | Monthly | 1959 |
| VIXCLS | VIX Volatility Index | Daily | 1990 |
| DEXUSEU | EUR/USD Exchange Rate | Daily | 1999 |

## Data Frequency Distribution

- Daily: Financial markets, some rates
- Weekly: Initial claims, some surveys
- Monthly: Most economic indicators (employment, inflation, retail)
- Quarterly: GDP, many national accounts
- Annual: Demographics, some international data
- Irregular: Some surveys, special reports

## Data Update Patterns

- Real-time: Stock indices, exchange rates (updated shortly after market close)
- Scheduled releases: Most economic data (employment first Friday, CPI mid-month, etc.)
- Revisions: Many series revised weeks/months after initial release
- Vintage data: ALFRED preserves all historical versions
