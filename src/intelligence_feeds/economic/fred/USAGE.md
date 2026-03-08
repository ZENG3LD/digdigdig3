# FRED Connector - Comprehensive API Guide

This document provides a complete reference for all FRED (Federal Reserve Economic Data) API endpoints available in the connector.

## Table of Contents

- [Quick Start](#quick-start)
- [Series Endpoints](#series-endpoints-10-methods)
- [Category Endpoints](#category-endpoints-6-methods)
- [Release Endpoints](#release-endpoints-8-methods)
- [Source Endpoints](#source-endpoints-3-methods)
- [Tag Endpoints](#tag-endpoints-3-methods)
- [Popular Series IDs](#popular-series-ids)
- [Advanced Usage](#advanced-usage)

---

## Quick Start

```rust
use digdigdig3::data_feeds::fred::FredConnector;

// Create connector (reads FRED_API_KEY from environment)
let fred = FredConnector::from_env();

// Get latest unemployment rate
let observations = fred.get_series_observations("UNRATE", None, None, Some(1)).await?;
println!("Unemployment: {:?}", observations[0].value);

// Search for GDP series
let series = fred.search_series("GDP", Some(10)).await?;
for s in series {
    println!("Found: {}", s);
}
```

---

## Series Endpoints (10 methods)

Series endpoints are the core of FRED - they provide access to the actual economic time series data.

### 1. `get_series_observations()` - **MOST IMPORTANT**

Get time series data points (observations) for an economic series.

```rust
pub async fn get_series_observations(
    &self,
    series_id: &str,              // Required: FRED series ID
    observation_start: Option<&str>, // Optional: Start date (YYYY-MM-DD)
    observation_end: Option<&str>,   // Optional: End date (YYYY-MM-DD)
    limit: Option<u32>,              // Optional: Max results (1-100000)
) -> ExchangeResult<Vec<Observation>>
```

**Example:**
```rust
// Get last 10 unemployment rate observations
let obs = fred.get_series_observations("UNRATE", None, None, Some(10)).await?;
for o in obs {
    println!("{}: {}", o.date, o.value.unwrap_or(0.0));
}

// Get GDP for specific date range
let obs = fred.get_series_observations(
    "GDP",
    Some("2020-01-01"),
    Some("2023-12-31"),
    None
).await?;
```

### 2. `get_series_metadata()`

Get metadata about an economic series (title, frequency, units, etc.).

```rust
pub async fn get_series_metadata(
    &self,
    series_id: &str,
) -> ExchangeResult<SeriesMetadata>
```

**Example:**
```rust
let metadata = fred.get_series_metadata("GDP").await?;
println!("Title: {}", metadata.title);
println!("Frequency: {}", metadata.frequency);
println!("Units: {}", metadata.units);
println!("Last updated: {}", metadata.last_updated);
```

### 3. `search_series()`

Search for economic data series by keywords.

```rust
pub async fn search_series(
    &self,
    search_text: &str,
    limit: Option<u32>,
) -> ExchangeResult<Vec<String>>
```

**Example:**
```rust
// Find all GDP-related series
let results = fred.search_series("GDP", Some(20)).await?;
for series_id in results {
    let meta = fred.get_series_metadata(&series_id).await?;
    println!("{}: {}", series_id, meta.title);
}
```

### 4. `get_series_categories()`

Get categories that a series belongs to.

```rust
pub async fn get_series_categories(
    &self,
    series_id: &str,
    realtime_start: Option<&str>,
    realtime_end: Option<&str>,
) -> ExchangeResult<Vec<Category>>
```

**Example:**
```rust
let categories = fred.get_series_categories("UNRATE", None, None).await?;
for cat in categories {
    println!("Category: {} (ID: {})", cat.name, cat.id);
}
```

### 5. `get_series_release()`

Get the release information for a series.

```rust
pub async fn get_series_release(
    &self,
    series_id: &str,
    realtime_start: Option<&str>,
    realtime_end: Option<&str>,
) -> ExchangeResult<Vec<Release>>
```

**Example:**
```rust
let releases = fred.get_series_release("GDP", None, None).await?;
for rel in releases {
    println!("Release: {} (ID: {})", rel.name, rel.id);
}
```

### 6. `get_series_tags()`

Get FRED tags for a series.

```rust
pub async fn get_series_tags(
    &self,
    series_id: &str,
    realtime_start: Option<&str>,
    realtime_end: Option<&str>,
    order_by: Option<&str>,
    sort_order: Option<&str>,
) -> ExchangeResult<Vec<Tag>>
```

**Example:**
```rust
let tags = fred.get_series_tags("UNRATE", None, None, None, None).await?;
for tag in tags {
    println!("Tag: {} (group: {})", tag.name, tag.group_id);
}
```

### 7. `get_series_search_tags()`

Get tags for series that match a search query.

```rust
pub async fn get_series_search_tags(
    &self,
    series_search_text: &str,
    realtime_start: Option<&str>,
    realtime_end: Option<&str>,
    tag_names: Option<&str>,
    tag_group_id: Option<&str>,
    tag_search_text: Option<&str>,
    limit: Option<u32>,
    offset: Option<u32>,
    order_by: Option<&str>,
    sort_order: Option<&str>,
) -> ExchangeResult<Vec<Tag>>
```

**Example:**
```rust
let tags = fred.get_series_search_tags(
    "unemployment",
    None, None, None, None, None,
    Some(10), None, None, None
).await?;
```

### 8. `get_series_search_related_tags()`

Get related tags for a series search.

```rust
pub async fn get_series_search_related_tags(
    &self,
    series_search_text: &str,
    tag_names: &str,
    // ... many optional parameters
) -> ExchangeResult<Vec<Tag>>
```

### 9. `get_series_updates()`

Get recently updated series.

```rust
pub async fn get_series_updates(
    &self,
    realtime_start: Option<&str>,
    realtime_end: Option<&str>,
    limit: Option<u32>,
    offset: Option<u32>,
    filter_value: Option<&str>, // "all", "macro", "regional"
    start_time: Option<&str>,   // YYYY-MM-DD HH:MM:SS
    end_time: Option<&str>,
) -> ExchangeResult<Vec<SeriesUpdate>>
```

**Example:**
```rust
// Get 10 most recently updated series
let updates = fred.get_series_updates(
    None, None,
    Some(10), None,
    Some("all"), None, None
).await?;

for update in updates {
    println!("{}: {} (updated: {})",
        update.series_id, update.title, update.last_updated);
}
```

### 10. `get_series_vintage_dates()`

Get vintage dates for a series (ALFRED - revision history).

```rust
pub async fn get_series_vintage_dates(
    &self,
    series_id: &str,
    realtime_start: Option<&str>,
    realtime_end: Option<&str>,
    limit: Option<u32>,
    offset: Option<u32>,
    sort_order: Option<&str>,
) -> ExchangeResult<Vec<VintageDate>>
```

**Example:**
```rust
// Get revision history for GDP
let vintages = fred.get_series_vintage_dates("GDP", None, None, Some(10), None, None).await?;
for v in vintages {
    println!("Revision date: {}", v.date);
}
```

---

## Category Endpoints (6 methods)

Categories organize economic data into a hierarchical structure.

### 1. `get_categories()`

Get category information. Pass `Some(0)` for root categories.

```rust
pub async fn get_categories(
    &self,
    category_id: Option<i64>,
) -> ExchangeResult<Vec<Category>>
```

**Example:**
```rust
// Get root categories
let root = fred.get_categories(Some(0)).await?;
for cat in root {
    println!("{} (ID: {})", cat.name, cat.id);
}
```

### 2. `get_category_children()`

Get child categories for a parent category.

```rust
pub async fn get_category_children(
    &self,
    category_id: i64,
) -> ExchangeResult<Vec<Category>>
```

**Example:**
```rust
// Get children of "Money, Banking, & Finance" (32991)
let children = fred.get_category_children(32991).await?;
```

### 3. `get_category_related()`

Get related (sibling) categories.

```rust
pub async fn get_category_related(
    &self,
    category_id: i64,
    realtime_start: Option<&str>,
    realtime_end: Option<&str>,
) -> ExchangeResult<Vec<Category>>
```

### 4. `get_category_series()`

Get all series in a category.

```rust
pub async fn get_category_series(
    &self,
    category_id: i64,
    limit: Option<u32>,
) -> ExchangeResult<Vec<String>>
```

**Example:**
```rust
let series = fred.get_category_series(32991, Some(10)).await?;
for s in series {
    println!("Series: {}", s);
}
```

### 5. `get_category_tags()`

Get FRED tags for a category.

```rust
pub async fn get_category_tags(
    &self,
    category_id: i64,
    tag_names: Option<&str>,
    tag_group_id: Option<&str>,
    search_text: Option<&str>,
    limit: Option<u32>,
    offset: Option<u32>,
    order_by: Option<&str>,
    sort_order: Option<&str>,
) -> ExchangeResult<Vec<Tag>>
```

### 6. `get_category_related_tags()`

Get related tags within a category.

```rust
pub async fn get_category_related_tags(
    &self,
    category_id: i64,
    tag_names: &str,
    exclude_tag_names: Option<&str>,
    // ... more optional parameters
) -> ExchangeResult<Vec<Tag>>
```

---

## Release Endpoints (8 methods)

Releases represent scheduled publications of economic data.

### 1. `get_releases()`

Get all releases of economic data.

```rust
pub async fn get_releases(
    &self,
    limit: Option<u32>,
) -> ExchangeResult<Vec<Release>>
```

**Example:**
```rust
let releases = fred.get_releases(Some(20)).await?;
for rel in releases {
    println!("{} (ID: {})", rel.name, rel.id);
}
```

### 2. `get_release()`

Get a specific release by ID.

```rust
pub async fn get_release(
    &self,
    release_id: i64,
) -> ExchangeResult<Vec<Release>>
```

**Example:**
```rust
// Get GDP release (ID: 53)
let release = fred.get_release(53).await?;
```

### 3. `get_releases_dates()`

Get release dates for ALL releases.

```rust
pub async fn get_releases_dates(
    &self,
    realtime_start: Option<&str>,
    realtime_end: Option<&str>,
    limit: Option<u32>,
    offset: Option<u32>,
    order_by: Option<&str>,
    sort_order: Option<&str>,
    include_release_dates_with_no_data: Option<bool>,
) -> ExchangeResult<Vec<ReleaseDate>>
```

### 4. `get_release_dates()`

Get release dates for a SPECIFIC release.

```rust
pub async fn get_release_dates(
    &self,
    release_id: i64,
    realtime_start: Option<&str>,
    realtime_end: Option<&str>,
    limit: Option<u32>,
    offset: Option<u32>,
    sort_order: Option<&str>,
    include_release_dates_with_no_data: Option<bool>,
) -> ExchangeResult<Vec<ReleaseDate>>
```

**Example:**
```rust
// Get next GDP release dates
let dates = fred.get_release_dates(53, None, None, Some(5), None, None, None).await?;
for date in dates {
    println!("GDP release: {}", date.date);
}
```

### 5. `get_release_series()`

Get all series in a release.

```rust
pub async fn get_release_series(
    &self,
    release_id: i64,
    // ... many optional parameters for filtering/pagination
) -> ExchangeResult<Vec<String>>
```

**Example:**
```rust
let series = fred.get_release_series(
    53, None, None, Some(10), None, None, None, None, None, None, None
).await?;
```

### 6. `get_release_sources()`

Get data sources for a release.

```rust
pub async fn get_release_sources(
    &self,
    release_id: i64,
    realtime_start: Option<&str>,
    realtime_end: Option<&str>,
) -> ExchangeResult<Vec<Source>>
```

### 7. `get_release_tags()`

Get FRED tags for a release.

```rust
pub async fn get_release_tags(
    &self,
    release_id: i64,
    // ... optional filtering parameters
) -> ExchangeResult<Vec<Tag>>
```

### 8. `get_release_related_tags()`

Get related tags for a release.

```rust
pub async fn get_release_related_tags(
    &self,
    release_id: i64,
    tag_names: &str,
    // ... more optional parameters
) -> ExchangeResult<Vec<Tag>>
```

---

## Source Endpoints (3 methods)

Sources represent the institutions that provide economic data.

### 1. `get_sources()`

Get all data sources.

```rust
pub async fn get_sources(
    &self,
    limit: Option<u32>,
) -> ExchangeResult<Vec<Source>>
```

**Example:**
```rust
let sources = fred.get_sources(Some(20)).await?;
for src in sources {
    println!("{} (ID: {})", src.name, src.id);
    if let Some(link) = src.link {
        println!("  Link: {}", link);
    }
}
```

### 2. `get_source()`

Get a specific source by ID.

```rust
pub async fn get_source(
    &self,
    source_id: i64,
    realtime_start: Option<&str>,
    realtime_end: Option<&str>,
) -> ExchangeResult<Vec<Source>>
```

**Example:**
```rust
// Get Board of Governors (source ID: 1)
let source = fred.get_source(1, None, None).await?;
```

### 3. `get_source_releases()`

Get all releases from a source.

```rust
pub async fn get_source_releases(
    &self,
    source_id: i64,
    realtime_start: Option<&str>,
    realtime_end: Option<&str>,
    limit: Option<u32>,
    offset: Option<u32>,
    order_by: Option<&str>,
    sort_order: Option<&str>,
) -> ExchangeResult<Vec<Release>>
```

**Example:**
```rust
let releases = fred.get_source_releases(
    1, None, None, Some(10), None, None, None
).await?;
```

---

## Tag Endpoints (3 methods)

Tags are attributes assigned to series for classification and filtering.

### 1. `get_tags()`

Get all FRED tags.

```rust
pub async fn get_tags(
    &self,
    limit: Option<u32>,
) -> ExchangeResult<Vec<Tag>>
```

**Example:**
```rust
let tags = fred.get_tags(Some(50)).await?;
for tag in tags {
    println!("{} (group: {}, series: {})",
        tag.name, tag.group_id, tag.series_count.unwrap_or(0));
}
```

### 2. `get_related_tags()`

Get related tags for one or more tags.

```rust
pub async fn get_related_tags(
    &self,
    tag_names: &str,              // Semicolon-separated
    realtime_start: Option<&str>,
    realtime_end: Option<&str>,
    exclude_tag_names: Option<&str>,
    tag_group_id: Option<&str>,
    search_text: Option<&str>,
    limit: Option<u32>,
    offset: Option<u32>,
    order_by: Option<&str>,
    sort_order: Option<&str>,
) -> ExchangeResult<Vec<Tag>>
```

**Example:**
```rust
let tags = fred.get_related_tags(
    "monetary aggregates",
    None, None, None, None, None, Some(10), None, None, None
).await?;
```

### 3. `get_tags_series()`

Get series that match specific tags.

```rust
pub async fn get_tags_series(
    &self,
    tag_names: &str,              // Semicolon-separated
    exclude_tag_names: Option<&str>,
    realtime_start: Option<&str>,
    realtime_end: Option<&str>,
    limit: Option<u32>,
    offset: Option<u32>,
    order_by: Option<&str>,
    sort_order: Option<&str>,
) -> ExchangeResult<Vec<String>>
```

**Example:**
```rust
let series = fred.get_tags_series(
    "gdp;quarterly",
    None, None, None,
    Some(10), None, None, None
).await?;
```

---

## Popular Series IDs

Here are the most commonly used FRED series:

### Interest Rates
- `DFF` - Federal Funds Effective Rate (daily)
- `DGS10` - 10-Year Treasury Constant Maturity Rate (daily)
- `DGS2` - 2-Year Treasury Constant Maturity Rate (daily)
- `DGS30` - 30-Year Treasury Constant Maturity Rate (daily)
- `MORTGAGE30US` - 30-Year Fixed Rate Mortgage Average (weekly)

### Inflation
- `CPIAUCSL` - Consumer Price Index for All Urban Consumers (monthly)
- `PCEPI` - Personal Consumption Expenditures Price Index (monthly)
- `CPILFESL` - CPI Less Food & Energy (monthly)

### Employment
- `UNRATE` - Unemployment Rate (monthly)
- `PAYEMS` - All Employees: Total Nonfarm Payrolls (monthly)
- `CIVPART` - Labor Force Participation Rate (monthly)
- `UNEMPLOY` - Unemployed (monthly)

### GDP & Production
- `GDP` - Gross Domestic Product (quarterly)
- `GDPC1` - Real Gross Domestic Product (quarterly)
- `INDPRO` - Industrial Production Index (monthly)

### Money Supply
- `M1SL` - M1 Money Stock (monthly)
- `M2SL` - M2 Money Stock (monthly)
- `WM2NS` - M2 Money Stock (weekly)

### Markets
- `SP500` - S&P 500 Index (daily)
- `VIXCLS` - CBOE Volatility Index: VIX (daily)
- `DEXUSEU` - US / Euro Foreign Exchange Rate (daily)

### Commodities
- `DCOILWTICO` - Crude Oil Prices: West Texas Intermediate (daily)
- `GOLDAMGBD228NLBM` - Gold Fixing Price (daily)

---

## Advanced Usage

### Working with Date Ranges

```rust
// Get CPI data for 2020-2023
let observations = fred.get_series_observations(
    "CPIAUCSL",
    Some("2020-01-01"),
    Some("2023-12-31"),
    None
).await?;

// Calculate inflation rate
for i in 1..observations.len() {
    let current = observations[i].value.unwrap_or(0.0);
    let previous = observations[i-1].value.unwrap_or(0.0);
    let rate = ((current - previous) / previous) * 100.0;
    println!("{}: {:.2}%", observations[i].date, rate);
}
```

### Pagination

```rust
// Get all unemployment data in chunks
let mut offset = 0;
let limit = 100;
loop {
    let mut params = HashMap::new();
    params.insert("series_id".to_string(), "UNRATE".to_string());
    params.insert("limit".to_string(), limit.to_string());
    params.insert("offset".to_string(), offset.to_string());

    let chunk = fred.get_series_observations("UNRATE", None, None, Some(limit)).await?;
    if chunk.is_empty() {
        break;
    }

    // Process chunk
    for obs in chunk {
        println!("{}: {}", obs.date, obs.value.unwrap_or(0.0));
    }

    offset += limit;
}
```

### Tag Filtering

```rust
// Find all quarterly GDP series
let series = fred.get_tags_series(
    "gdp;quarterly;usa",
    None,
    None,
    None,
    Some(20),
    None,
    Some("popularity"),
    Some("desc")
).await?;

for s in series {
    let meta = fred.get_series_metadata(&s).await?;
    println!("{}: {}", s, meta.title);
}
```

### Monitoring Updates

```rust
// Check what economic data was updated today
let updates = fred.get_series_updates(
    None,
    None,
    Some(50),
    None,
    Some("macro"),
    None,
    None
).await?;

for update in updates {
    println!("{} was updated at {}",
        update.series_id, update.last_updated);
}
```

---

## Rate Limits

- **Free Tier**: 120 requests per minute
- **Cost**: Completely free for non-commercial use
- **No WebSocket**: FRED provides REST API only

## Error Handling

```rust
match fred.get_series_observations("INVALID", None, None, None).await {
    Ok(obs) => println!("Got {} observations", obs.len()),
    Err(ExchangeError::Api { code, message }) => {
        println!("API Error {}: {}", code, message);
    }
    Err(ExchangeError::Parse(msg)) => {
        println!("Parse error: {}", msg);
    }
    Err(e) => println!("Error: {:?}", e),
}
```

---

## MarketData Trait Compatibility

FRED also implements the `MarketData` trait for compatibility:

```rust
use digdigdig3::core::traits::MarketData;

// Get latest value as "price"
let price = fred.get_price(Symbol::new("UNRATE", ""), AccountType::Spot).await?;

// Get observations as "klines"
let klines = fred.get_klines(Symbol::new("GDP", ""), "quarterly", Some(10), AccountType::Spot).await?;
```

Note: `get_ticker()` and `get_orderbook()` return `UnsupportedOperation` as FRED is economic data, not trading data.

---

## Resources

- [FRED Website](https://fred.stlouisfed.org/)
- [FRED API Documentation](https://fred.stlouisfed.org/docs/api/fred/)
- [Get API Key](https://fred.stlouisfed.org/docs/api/api_key.html)
- [Series Search](https://fred.stlouisfed.org/)
