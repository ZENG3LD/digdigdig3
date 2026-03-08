# FRED - Response Formats

All examples are from official FRED API documentation. FRED supports 4 file types: XML (default), JSON, CSV, XLSX.

---

## GET /fred/category

Get a category.

### JSON Response
```json
{
  "categories": [
    {
      "id": 125,
      "name": "Trade Balance",
      "parent_id": 13
    }
  ]
}
```

### XML Response
```xml
<?xml version="1.0" encoding="utf-8" ?>
<categories>
  <category id="125" name="Trade Balance" parent_id="13"/>
</categories>
```

---

## GET /fred/category/children

Get child categories for a category.

### JSON Response
```json
{
  "categories": [
    {
      "id": 193,
      "name": "U.S. Trade & International Transactions",
      "parent_id": 32992
    },
    {
      "id": 33705,
      "name": "Foreign Trade Zones",
      "parent_id": 32992
    }
  ]
}
```

---

## GET /fred/category/series

Get series in a category.

### JSON Response
```json
{
  "realtime_start": "2024-01-15",
  "realtime_end": "2024-01-15",
  "order_by": "series_id",
  "sort_order": "asc",
  "count": 42,
  "offset": 0,
  "limit": 1000,
  "seriess": [
    {
      "id": "AITGCBN",
      "realtime_start": "2024-01-15",
      "realtime_end": "2024-01-15",
      "title": "U.S. Imports of Services - Travel and Transportation",
      "observation_start": "1999-01-01",
      "observation_end": "2023-06-01",
      "frequency": "Quarterly",
      "frequency_short": "Q",
      "units": "Billions of Dollars",
      "units_short": "Bil. of $",
      "seasonal_adjustment": "Seasonally Adjusted",
      "seasonal_adjustment_short": "SA",
      "last_updated": "2023-09-28 07:46:02-05",
      "popularity": 0,
      "notes": "..."
    }
  ]
}
```

---

## GET /fred/releases

Get all releases of economic data.

### JSON Response
```json
{
  "realtime_start": "2024-01-15",
  "realtime_end": "2024-01-15",
  "order_by": "release_id",
  "sort_order": "asc",
  "count": 300,
  "offset": 0,
  "limit": 1000,
  "releases": [
    {
      "id": 9,
      "realtime_start": "2024-01-15",
      "realtime_end": "2024-01-15",
      "name": "Advance Monthly Sales for Retail and Food Services",
      "press_release": true,
      "link": "http://www.census.gov/retail/"
    },
    {
      "id": 10,
      "realtime_start": "2024-01-15",
      "realtime_end": "2024-01-15",
      "name": "Consumer Price Index",
      "press_release": true,
      "link": "http://www.bls.gov/cpi/"
    }
  ]
}
```

---

## GET /fred/releases/dates

Get release dates for all releases.

### JSON Response
```json
{
  "realtime_start": "2024-01-15",
  "realtime_end": "2024-01-15",
  "order_by": "release_date",
  "sort_order": "asc",
  "count": 12000,
  "offset": 0,
  "limit": 1000,
  "release_dates": [
    {
      "release_id": 9,
      "release_name": "Advance Monthly Sales for Retail and Food Services",
      "date": "2023-01-18"
    },
    {
      "release_id": 10,
      "release_name": "Consumer Price Index",
      "date": "2023-01-12"
    }
  ]
}
```

---

## GET /fred/release

Get a release of economic data.

### JSON Response
```json
{
  "releases": [
    {
      "id": 53,
      "realtime_start": "2024-01-15",
      "realtime_end": "2024-01-15",
      "name": "Gross Domestic Product",
      "press_release": true,
      "link": "http://www.bea.gov/national/index.htm"
    }
  ]
}
```

---

## GET /fred/series

Get an economic data series.

### JSON Response
```json
{
  "seriess": [
    {
      "id": "GNPCA",
      "realtime_start": "2024-01-15",
      "realtime_end": "2024-01-15",
      "title": "Real Gross National Product",
      "observation_start": "1929-01-01",
      "observation_end": "2022-01-01",
      "frequency": "Annual",
      "frequency_short": "A",
      "units": "Billions of Chained 2012 Dollars",
      "units_short": "Bil. of Chn. 2012 $",
      "seasonal_adjustment": "Not Seasonally Adjusted",
      "seasonal_adjustment_short": "NSA",
      "last_updated": "2023-09-28 07:46:03-05",
      "popularity": 39,
      "group_popularity": 39,
      "notes": "BEA Account Code: A001RX\n\nReal Gross National Product is the inflation adjusted value..."
    }
  ]
}
```

### XML Response
```xml
<?xml version="1.0" encoding="utf-8" ?>
<seriess realtime_start="2024-01-15" realtime_end="2024-01-15">
  <series id="GNPCA" realtime_start="2024-01-15" realtime_end="2024-01-15"
          title="Real Gross National Product"
          observation_start="1929-01-01" observation_end="2022-01-01"
          frequency="Annual" frequency_short="A"
          units="Billions of Chained 2012 Dollars" units_short="Bil. of Chn. 2012 $"
          seasonal_adjustment="Not Seasonally Adjusted" seasonal_adjustment_short="NSA"
          last_updated="2023-09-28 07:46:03-05" popularity="39" group_popularity="39"
          notes="BEA Account Code: A001RX&#xd;&#xd;Real Gross National Product..."/>
</seriess>
```

---

## GET /fred/series/observations

**MOST IMPORTANT ENDPOINT** - Get observations (data values) for a series.

### JSON Response
```json
{
  "realtime_start": "2024-01-15",
  "realtime_end": "2024-01-15",
  "observation_start": "1776-07-04",
  "observation_end": "9999-12-31",
  "units": "lin",
  "output_type": 1,
  "file_type": "json",
  "order_by": "observation_date",
  "sort_order": "asc",
  "count": 94,
  "offset": 0,
  "limit": 100000,
  "observations": [
    {
      "realtime_start": "2024-01-15",
      "realtime_end": "2024-01-15",
      "date": "1929-01-01",
      "value": "1120.7"
    },
    {
      "realtime_start": "2024-01-15",
      "realtime_end": "2024-01-15",
      "date": "1930-01-01",
      "value": "1025.0"
    },
    {
      "realtime_start": "2024-01-15",
      "realtime_end": "2024-01-15",
      "date": "1931-01-01",
      "value": "958.8"
    }
  ]
}
```

### XML Response
```xml
<?xml version="1.0" encoding="utf-8" ?>
<observations realtime_start="2024-01-15" realtime_end="2024-01-15"
               observation_start="1776-07-04" observation_end="9999-12-31"
               units="lin" output_type="1" file_type="xml"
               order_by="observation_date" sort_order="asc"
               count="94" offset="0" limit="100000">
  <observation realtime_start="2024-01-15" realtime_end="2024-01-15"
               date="1929-01-01" value="1120.7"/>
  <observation realtime_start="2024-01-15" realtime_end="2024-01-15"
               date="1930-01-01" value="1025.0"/>
  <observation realtime_start="2024-01-15" realtime_end="2024-01-15"
               date="1931-01-01" value="958.8"/>
</observations>
```

### CSV Response
```csv
realtime_start,realtime_end,date,value
2024-01-15,2024-01-15,1929-01-01,1120.7
2024-01-15,2024-01-15,1930-01-01,1025.0
2024-01-15,2024-01-15,1931-01-01,958.8
```

### With Transformations (units=pch - Percent Change)
```json
{
  "realtime_start": "2024-01-15",
  "realtime_end": "2024-01-15",
  "observation_start": "2020-01-01",
  "observation_end": "2023-12-31",
  "units": "pch",
  "output_type": 1,
  "file_type": "json",
  "order_by": "observation_date",
  "sort_order": "asc",
  "count": 47,
  "offset": 0,
  "limit": 100000,
  "observations": [
    {
      "realtime_start": "2024-01-15",
      "realtime_end": "2024-01-15",
      "date": "2020-01-01",
      "value": "."
    },
    {
      "realtime_start": "2024-01-15",
      "realtime_end": "2024-01-15",
      "date": "2020-04-01",
      "value": "-8.9"
    },
    {
      "realtime_start": "2024-01-15",
      "realtime_end": "2024-01-15",
      "date": "2020-07-01",
      "value": "7.5"
    }
  ]
}
```

**Note**: Value "." indicates missing/not available data.

### With Frequency Aggregation (frequency=a - Annual)
```json
{
  "realtime_start": "2024-01-15",
  "realtime_end": "2024-01-15",
  "observation_start": "2020-01-01",
  "observation_end": "2023-12-31",
  "units": "lin",
  "output_type": 1,
  "file_type": "json",
  "order_by": "observation_date",
  "sort_order": "asc",
  "count": 4,
  "offset": 0,
  "limit": 100000,
  "observations": [
    {
      "realtime_start": "2024-01-15",
      "realtime_end": "2024-01-15",
      "date": "2020-01-01",
      "value": "19477.444"
    },
    {
      "realtime_start": "2024-01-15",
      "realtime_end": "2024-01-15",
      "date": "2021-01-01",
      "value": "20893.746"
    }
  ]
}
```

---

## GET /fred/series/search

Search for economic data series matching keywords.

### JSON Response
```json
{
  "realtime_start": "2024-01-15",
  "realtime_end": "2024-01-15",
  "order_by": "search_rank",
  "sort_order": "desc",
  "count": 8247,
  "offset": 0,
  "limit": 1000,
  "seriess": [
    {
      "id": "GNPCA",
      "realtime_start": "2024-01-15",
      "realtime_end": "2024-01-15",
      "title": "Real Gross National Product",
      "observation_start": "1929-01-01",
      "observation_end": "2022-01-01",
      "frequency": "Annual",
      "frequency_short": "A",
      "units": "Billions of Chained 2012 Dollars",
      "units_short": "Bil. of Chn. 2012 $",
      "seasonal_adjustment": "Not Seasonally Adjusted",
      "seasonal_adjustment_short": "NSA",
      "last_updated": "2023-09-28 07:46:03-05",
      "popularity": 39,
      "group_popularity": 39
    }
  ]
}
```

---

## GET /fred/series/updates

Get economic data series sorted by when observations were updated.

### JSON Response
```json
{
  "realtime_start": "2024-01-15",
  "realtime_end": "2024-01-15",
  "filter_value": "all",
  "count": 476000,
  "offset": 0,
  "limit": 1000,
  "seriess": [
    {
      "id": "GDP",
      "realtime_start": "2024-01-15",
      "realtime_end": "2024-01-15",
      "title": "Gross Domestic Product",
      "observation_start": "1947-01-01",
      "observation_end": "2023-07-01",
      "frequency": "Quarterly",
      "frequency_short": "Q",
      "units": "Billions of Dollars",
      "units_short": "Bil. of $",
      "seasonal_adjustment": "Seasonally Adjusted Annual Rate",
      "seasonal_adjustment_short": "SAAR",
      "last_updated": "2023-10-26 07:46:03-05",
      "popularity": 89,
      "group_popularity": 90
    }
  ]
}
```

---

## GET /fred/series/vintagedates

Get dates when a series' data values were revised (ALFRED).

### JSON Response
```json
{
  "realtime_start": "1776-07-04",
  "realtime_end": "9999-12-31",
  "order_by": "vintage_date",
  "sort_order": "asc",
  "count": 157,
  "offset": 0,
  "limit": 10000,
  "vintage_dates": [
    "1996-11-27",
    "1997-01-29",
    "1997-02-28",
    "1997-03-28",
    "1997-04-30",
    "1997-05-30"
  ]
}
```

---

## GET /fred/sources

Get all sources of economic data.

### JSON Response
```json
{
  "realtime_start": "2024-01-15",
  "realtime_end": "2024-01-15",
  "order_by": "source_id",
  "sort_order": "asc",
  "count": 118,
  "offset": 0,
  "limit": 1000,
  "sources": [
    {
      "id": 1,
      "realtime_start": "2024-01-15",
      "realtime_end": "2024-01-15",
      "name": "Board of Governors of the Federal Reserve System (US)",
      "link": "http://www.federalreserve.gov/"
    },
    {
      "id": 3,
      "realtime_start": "2024-01-15",
      "realtime_end": "2024-01-15",
      "name": "Federal Reserve Bank of Philadelphia",
      "link": "http://www.philadelphiafed.org/"
    }
  ]
}
```

---

## GET /fred/tags

Get FRED tags.

### JSON Response
```json
{
  "realtime_start": "2024-01-15",
  "realtime_end": "2024-01-15",
  "order_by": "series_count",
  "sort_order": "desc",
  "count": 4800,
  "offset": 0,
  "limit": 1000,
  "tags": [
    {
      "name": "nation",
      "group_id": "geot",
      "notes": "",
      "created": "2012-02-27 10:18:19-06",
      "popularity": 100,
      "series_count": 105200
    },
    {
      "name": "nsa",
      "group_id": "seas",
      "notes": "Not seasonally adjusted",
      "created": "2012-02-27 10:18:19-06",
      "popularity": 96,
      "series_count": 100343
    }
  ]
}
```

---

## Error Responses

### 400 Bad Request - Missing API Key

**JSON:**
```json
{
  "error_code": 400,
  "error_message": "Bad Request. Variable api_key is not set."
}
```

**XML:**
```xml
<?xml version="1.0" encoding="utf-8" ?>
<error code="400" message="Bad Request. Variable api_key is not set."/>
```

### 400 Bad Request - Invalid API Key

**JSON:**
```json
{
  "error_code": 400,
  "error_message": "Bad Request. The value for variable api_key is not registered."
}
```

### 404 Not Found - Invalid Series

**JSON:**
```json
{
  "error_code": 404,
  "error_message": "Not Found. Series does not exist."
}
```

### 429 Too Many Requests - Rate Limit

**JSON:**
```json
{
  "error_code": 429,
  "error_message": "Too Many Requests. Rate limit exceeded."
}
```

---

## Response Format Notes

### Common Fields Across All Responses

- `realtime_start`: Real-time period start (for ALFRED vintage data)
- `realtime_end`: Real-time period end
- `count`: Total number of results available
- `offset`: Current pagination offset
- `limit`: Maximum results per request
- `order_by`: Field used for sorting
- `sort_order`: "asc" or "desc"

### Date Formats

- **Dates**: YYYY-MM-DD (e.g., "2024-01-15")
- **Timestamps**: YYYY-MM-DD HH:MM:SS-TZ (e.g., "2023-10-26 07:46:03-05")

### Missing Values

- **"."** (dot/period) indicates missing or not available data
- Some series have sparse data with many missing observations

### File Type Comparison

| Format | Use Case | Pros | Cons |
|--------|----------|------|------|
| JSON | APIs, web apps | Easy to parse, compact | Larger than CSV |
| XML | Legacy systems | Well-structured | Verbose, harder to parse |
| CSV | Data analysis, Excel | Simple, lightweight | No metadata |
| XLSX | Excel, manual review | Native Excel format | Binary, harder to parse |

### Content-Type Headers

- **JSON**: `application/json; charset=utf-8`
- **XML**: `text/xml; charset=utf-8`
- **CSV**: `text/csv; charset=utf-8`
- **XLSX**: `application/vnd.openxmlformats-officedocument.spreadsheetml.sheet`

### Character Encoding

All responses use UTF-8 encoding.
