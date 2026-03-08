# FRED - Complete Endpoint Reference

Base URL: https://api.stlouisfed.org

All endpoints require `api_key` parameter and support `file_type` parameter (xml, json, csv, xlsx).

---

## Category: Categories (6 endpoints)

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /fred/category | Get a category | Yes | Yes | 120/min | Returns category details |
| GET | /fred/category/children | Get child categories | Yes | Yes | 120/min | Hierarchical category structure |
| GET | /fred/category/related | Get related categories | Yes | Yes | 120/min | Sibling categories |
| GET | /fred/category/series | Get series in category | Yes | Yes | 120/min | Supports pagination |
| GET | /fred/category/tags | Get tags for category | Yes | Yes | 120/min | FRED tag system |
| GET | /fred/category/related_tags | Get related tags | Yes | Yes | 120/min | Tag filtering |

### GET /fred/category

Get a category by ID.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| api_key | string | Yes | - | 32 character alpha-numeric API key |
| file_type | string | No | xml | xml, json, csv, xlsx |
| category_id | integer | No | 0 | Category ID (0 = root) |

### GET /fred/category/children

Get child categories for a parent category.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| api_key | string | Yes | - | API key |
| file_type | string | No | xml | Response format |
| category_id | integer | No | 0 | Parent category ID |
| realtime_start | date | No | today | YYYY-MM-DD |
| realtime_end | date | No | today | YYYY-MM-DD |

### GET /fred/category/related

Get related categories for a category.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| api_key | string | Yes | - | API key |
| file_type | string | No | xml | Response format |
| category_id | integer | Yes | - | Category ID |
| realtime_start | date | No | today | YYYY-MM-DD |
| realtime_end | date | No | today | YYYY-MM-DD |

### GET /fred/category/series

Get economic data series in a category.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| api_key | string | Yes | - | API key |
| file_type | string | No | xml | Response format |
| category_id | integer | Yes | - | Category ID |
| realtime_start | date | No | today | YYYY-MM-DD |
| realtime_end | date | No | today | YYYY-MM-DD |
| limit | integer | No | 1000 | Max results (1-1000) |
| offset | integer | No | 0 | Pagination offset |
| order_by | string | No | series_id | series_id, title, units, frequency, seasonal_adjustment, realtime_start, realtime_end, last_updated, observation_start, observation_end, popularity, group_popularity |
| sort_order | string | No | asc | asc, desc |
| filter_variable | string | No | - | frequency, units, seasonal_adjustment |
| filter_value | string | No | - | Value to filter by |
| tag_names | string | No | - | Semicolon-separated tag names |
| exclude_tag_names | string | No | - | Semicolon-separated tag names |

### GET /fred/category/tags

Get FRED tags for a category.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| api_key | string | Yes | - | API key |
| file_type | string | No | xml | Response format |
| category_id | integer | Yes | - | Category ID |
| realtime_start | date | No | today | YYYY-MM-DD |
| realtime_end | date | No | today | YYYY-MM-DD |
| tag_names | string | No | - | Filter by tag names |
| tag_group_id | string | No | - | freq, gen, geo, geot, rls, seas, src |
| search_text | string | No | - | Search words |
| limit | integer | No | 1000 | Max results (1-1000) |
| offset | integer | No | 0 | Pagination offset |
| order_by | string | No | series_count | series_count, popularity, created, name, group_id |
| sort_order | string | No | asc | asc, desc |

### GET /fred/category/related_tags

Get related FRED tags within a category.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| api_key | string | Yes | - | API key |
| file_type | string | No | xml | Response format |
| category_id | integer | Yes | - | Category ID |
| realtime_start | date | No | today | YYYY-MM-DD |
| realtime_end | date | No | today | YYYY-MM-DD |
| tag_names | string | Yes | - | Semicolon-separated tag names |
| exclude_tag_names | string | No | - | Semicolon-separated tag names |
| tag_group_id | string | No | - | freq, gen, geo, geot, rls, seas, src |
| search_text | string | No | - | Search words |
| limit | integer | No | 1000 | Max results (1-1000) |
| offset | integer | No | 0 | Pagination offset |
| order_by | string | No | series_count | series_count, popularity, created, name, group_id |
| sort_order | string | No | asc | asc, desc |

---

## Category: Releases (8 endpoints)

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /fred/releases | Get all releases | Yes | Yes | 120/min | Full release list |
| GET | /fred/releases/dates | Get release dates | Yes | Yes | 120/min | All releases |
| GET | /fred/release | Get a release | Yes | Yes | 120/min | By release_id |
| GET | /fred/release/dates | Get dates for release | Yes | Yes | 120/min | Specific release |
| GET | /fred/release/series | Get series in release | Yes | Yes | 120/min | Supports pagination |
| GET | /fred/release/sources | Get sources for release | Yes | Yes | 120/min | Data provenance |
| GET | /fred/release/tags | Get tags for release | Yes | Yes | 120/min | Tag filtering |
| GET | /fred/release/related_tags | Get related tags | Yes | Yes | 120/min | Tag relationships |

### GET /fred/releases

Get all releases of economic data.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| api_key | string | Yes | - | API key |
| file_type | string | No | xml | Response format |
| realtime_start | date | No | today | YYYY-MM-DD |
| realtime_end | date | No | today | YYYY-MM-DD |
| limit | integer | No | 1000 | Max results (1-1000) |
| offset | integer | No | 0 | Pagination offset |
| order_by | string | No | release_id | release_id, name, press_release, realtime_start, realtime_end |
| sort_order | string | No | asc | asc, desc |

### GET /fred/releases/dates

Get release dates for all releases of economic data.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| api_key | string | Yes | - | API key |
| file_type | string | No | xml | Response format |
| realtime_start | date | No | today | YYYY-MM-DD |
| realtime_end | date | No | today | YYYY-MM-DD |
| limit | integer | No | 1000 | Max results (1-1000) |
| offset | integer | No | 0 | Pagination offset |
| order_by | string | No | release_date | release_date, release_id, release_name |
| sort_order | string | No | asc | asc, desc |
| include_release_dates_with_no_data | boolean | No | false | Include dates with no data |

### GET /fred/release

Get a release of economic data.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| api_key | string | Yes | - | API key |
| file_type | string | No | xml | Response format |
| release_id | integer | Yes | - | Release ID |
| realtime_start | date | No | today | YYYY-MM-DD |
| realtime_end | date | No | today | YYYY-MM-DD |

### GET /fred/release/dates

Get release dates for a release of economic data.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| api_key | string | Yes | - | API key |
| file_type | string | No | xml | Response format |
| release_id | integer | Yes | - | Release ID |
| realtime_start | date | No | today | YYYY-MM-DD |
| realtime_end | date | No | today | YYYY-MM-DD |
| limit | integer | No | 10000 | Max results (1-10000) |
| offset | integer | No | 0 | Pagination offset |
| sort_order | string | No | asc | asc, desc |
| include_release_dates_with_no_data | boolean | No | false | Include dates with no data |

### GET /fred/release/series

Get economic data series in a release.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| api_key | string | Yes | - | API key |
| file_type | string | No | xml | Response format |
| release_id | integer | Yes | - | Release ID |
| realtime_start | date | No | today | YYYY-MM-DD |
| realtime_end | date | No | today | YYYY-MM-DD |
| limit | integer | No | 1000 | Max results (1-1000) |
| offset | integer | No | 0 | Pagination offset |
| order_by | string | No | series_id | series_id, title, units, frequency, seasonal_adjustment, realtime_start, realtime_end, last_updated, observation_start, observation_end, popularity, group_popularity |
| sort_order | string | No | asc | asc, desc |
| filter_variable | string | No | - | frequency, units, seasonal_adjustment |
| filter_value | string | No | - | Value to filter by |
| tag_names | string | No | - | Semicolon-separated tag names |
| exclude_tag_names | string | No | - | Semicolon-separated tag names |

### GET /fred/release/sources

Get sources for a release of economic data.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| api_key | string | Yes | - | API key |
| file_type | string | No | xml | Response format |
| release_id | integer | Yes | - | Release ID |
| realtime_start | date | No | today | YYYY-MM-DD |
| realtime_end | date | No | today | YYYY-MM-DD |

### GET /fred/release/tags

Get FRED tags for a release.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| api_key | string | Yes | - | API key |
| file_type | string | No | xml | Response format |
| release_id | integer | Yes | - | Release ID |
| realtime_start | date | No | today | YYYY-MM-DD |
| realtime_end | date | No | today | YYYY-MM-DD |
| tag_names | string | No | - | Filter by tag names |
| tag_group_id | string | No | - | freq, gen, geo, geot, rls, seas, src |
| search_text | string | No | - | Search words |
| limit | integer | No | 1000 | Max results (1-1000) |
| offset | integer | No | 0 | Pagination offset |
| order_by | string | No | series_count | series_count, popularity, created, name, group_id |
| sort_order | string | No | asc | asc, desc |

### GET /fred/release/related_tags

Get related FRED tags for a release.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| api_key | string | Yes | - | API key |
| file_type | string | No | xml | Response format |
| release_id | integer | Yes | - | Release ID |
| realtime_start | date | No | today | YYYY-MM-DD |
| realtime_end | date | No | today | YYYY-MM-DD |
| tag_names | string | Yes | - | Semicolon-separated tag names |
| exclude_tag_names | string | No | - | Semicolon-separated tag names |
| tag_group_id | string | No | - | freq, gen, geo, geot, rls, seas, src |
| search_text | string | No | - | Search words |
| limit | integer | No | 1000 | Max results (1-1000) |
| offset | integer | No | 0 | Pagination offset |
| order_by | string | No | series_count | series_count, popularity, created, name, group_id |
| sort_order | string | No | asc | asc, desc |

---

## Category: Series (10 endpoints)

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /fred/series | Get a series | Yes | Yes | 120/min | Series metadata |
| GET | /fred/series/categories | Get series categories | Yes | Yes | 120/min | Category membership |
| GET | /fred/series/observations | Get series data | Yes | Yes | 120/min | **CORE ENDPOINT** |
| GET | /fred/series/release | Get series release | Yes | Yes | 120/min | Release info |
| GET | /fred/series/search | Search series | Yes | Yes | 120/min | Full-text search |
| GET | /fred/series/search/tags | Search tags | Yes | Yes | 120/min | Tag search |
| GET | /fred/series/search/related_tags | Search related tags | Yes | Yes | 120/min | Tag relationships |
| GET | /fred/series/tags | Get series tags | Yes | Yes | 120/min | Tag attributes |
| GET | /fred/series/updates | Get series updates | Yes | Yes | 120/min | Recently updated |
| GET | /fred/series/vintagedates | Get vintage dates | Yes | Yes | 120/min | ALFRED revisions |

### GET /fred/series

Get an economic data series.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| api_key | string | Yes | - | API key |
| file_type | string | No | xml | Response format |
| series_id | string | Yes | - | Series ID (e.g., "GNPCA") |
| realtime_start | date | No | today | YYYY-MM-DD |
| realtime_end | date | No | today | YYYY-MM-DD |

### GET /fred/series/categories

Get categories for an economic data series.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| api_key | string | Yes | - | API key |
| file_type | string | No | xml | Response format |
| series_id | string | Yes | - | Series ID |
| realtime_start | date | No | today | YYYY-MM-DD |
| realtime_end | date | No | today | YYYY-MM-DD |

### GET /fred/series/observations

**MOST IMPORTANT ENDPOINT** - Get observations/data values for an economic data series.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| api_key | string | Yes | - | 32 character alpha-numeric API key |
| file_type | string | No | xml | xml, json, csv, xlsx |
| series_id | string | Yes | - | Series ID (e.g., "GNPCA", "UNRATE") |
| realtime_start | date | No | today | Real-time period start (YYYY-MM-DD) |
| realtime_end | date | No | today | Real-time period end (YYYY-MM-DD) |
| observation_start | date | No | 1776-07-04 | Observation period start (YYYY-MM-DD) |
| observation_end | date | No | 9999-12-31 | Observation period end (YYYY-MM-DD) |
| limit | integer | No | 100000 | Max results (1-100000) |
| offset | integer | No | 0 | Pagination offset |
| sort_order | string | No | asc | asc, desc |
| units | string | No | lin | lin, chg, ch1, pch, pc1, pca, cch, cca, log |
| frequency | string | No | - | d, w, bw, m, q, sa, a, wef, weth, wew, wetu, wem, wesu, wesa, bwew, bwem |
| aggregation_method | string | No | avg | avg, sum, eop |
| output_type | integer | No | 1 | 1, 2, 3, 4 |
| vintage_dates | string | No | - | Comma-separated dates (YYYY-MM-DD) |

**Units transformations:**
- lin = Levels (no transformation)
- chg = Change
- ch1 = Change from Year Ago
- pch = Percent Change
- pc1 = Percent Change from Year Ago
- pca = Compounded Annual Rate of Change
- cch = Continuously Compounded Rate of Change
- cca = Continuously Compounded Annual Rate of Change
- log = Natural Log

**Frequencies:**
- d = Daily
- w = Weekly
- bw = Biweekly
- m = Monthly
- q = Quarterly
- sa = Semiannual
- a = Annual
- wef = Weekly, Ending Friday
- weth = Weekly, Ending Thursday
- wew = Weekly, Ending Wednesday
- wetu = Weekly, Ending Tuesday
- wem = Weekly, Ending Monday
- wesu = Weekly, Ending Sunday
- wesa = Weekly, Ending Saturday
- bwew = Biweekly, Ending Wednesday
- bwem = Biweekly, Ending Monday

**Output types:**
- 1 = Observations by Real-Time Period
- 2 = Observations by Vintage Date, All Observations
- 3 = Observations by Vintage Date, New and Revised Observations Only
- 4 = Observations, Initial Release Only

### GET /fred/series/release

Get the release for an economic data series.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| api_key | string | Yes | - | API key |
| file_type | string | No | xml | Response format |
| series_id | string | Yes | - | Series ID |
| realtime_start | date | No | today | YYYY-MM-DD |
| realtime_end | date | No | today | YYYY-MM-DD |

### GET /fred/series/search

Search for economic data series matching keywords.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| api_key | string | Yes | - | API key |
| file_type | string | No | xml | Response format |
| search_text | string | Yes | - | Keywords to search |
| search_type | string | No | full_text | full_text, series_id |
| realtime_start | date | No | today | YYYY-MM-DD |
| realtime_end | date | No | today | YYYY-MM-DD |
| limit | integer | No | 1000 | Max results (1-1000) |
| offset | integer | No | 0 | Pagination offset |
| order_by | string | No | search_rank | search_rank, series_id, title, units, frequency, seasonal_adjustment, realtime_start, realtime_end, last_updated, observation_start, observation_end, popularity, group_popularity |
| sort_order | string | No | asc | asc, desc |
| filter_variable | string | No | - | frequency, units, seasonal_adjustment |
| filter_value | string | No | - | Value to filter by |
| tag_names | string | No | - | Semicolon-separated tag names |
| exclude_tag_names | string | No | - | Semicolon-separated tag names |

### GET /fred/series/search/tags

Get FRED tags for a series search.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| api_key | string | Yes | - | API key |
| file_type | string | No | xml | Response format |
| series_search_text | string | Yes | - | Keywords to search |
| realtime_start | date | No | today | YYYY-MM-DD |
| realtime_end | date | No | today | YYYY-MM-DD |
| tag_names | string | No | - | Filter by tag names |
| tag_group_id | string | No | - | freq, gen, geo, geot, rls, seas, src |
| tag_search_text | string | No | - | Search tag names |
| limit | integer | No | 1000 | Max results (1-1000) |
| offset | integer | No | 0 | Pagination offset |
| order_by | string | No | series_count | series_count, popularity, created, name, group_id |
| sort_order | string | No | asc | asc, desc |

### GET /fred/series/search/related_tags

Get related FRED tags for a series search.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| api_key | string | Yes | - | API key |
| file_type | string | No | xml | Response format |
| series_search_text | string | Yes | - | Keywords to search |
| realtime_start | date | No | today | YYYY-MM-DD |
| realtime_end | date | No | today | YYYY-MM-DD |
| tag_names | string | Yes | - | Semicolon-separated tag names |
| exclude_tag_names | string | No | - | Semicolon-separated tag names |
| tag_group_id | string | No | - | freq, gen, geo, geot, rls, seas, src |
| tag_search_text | string | No | - | Search tag names |
| limit | integer | No | 1000 | Max results (1-1000) |
| offset | integer | No | 0 | Pagination offset |
| order_by | string | No | series_count | series_count, popularity, created, name, group_id |
| sort_order | string | No | asc | asc, desc |

### GET /fred/series/tags

Get FRED tags for a series.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| api_key | string | Yes | - | API key |
| file_type | string | No | xml | Response format |
| series_id | string | Yes | - | Series ID |
| realtime_start | date | No | today | YYYY-MM-DD |
| realtime_end | date | No | today | YYYY-MM-DD |
| order_by | string | No | series_count | series_count, popularity, created, name, group_id |
| sort_order | string | No | asc | asc, desc |

### GET /fred/series/updates

Get economic data series sorted by when observations were updated.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| api_key | string | Yes | - | API key |
| file_type | string | No | xml | Response format |
| realtime_start | date | No | today | YYYY-MM-DD |
| realtime_end | date | No | today | YYYY-MM-DD |
| limit | integer | No | 1000 | Max results (1-1000) |
| offset | integer | No | 0 | Pagination offset |
| filter_value | string | No | all | all, macro, regional |
| start_time | datetime | No | - | YYYY-MM-DD HH:MM:SS |
| end_time | datetime | No | - | YYYY-MM-DD HH:MM:SS |

### GET /fred/series/vintagedates

Get dates when a series' data values were revised (ALFRED - Archival FRED).

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| api_key | string | Yes | - | API key |
| file_type | string | No | xml | Response format |
| series_id | string | Yes | - | Series ID |
| realtime_start | date | No | 1776-07-04 | YYYY-MM-DD |
| realtime_end | date | No | 9999-12-31 | YYYY-MM-DD |
| limit | integer | No | 10000 | Max results (1-10000) |
| offset | integer | No | 0 | Pagination offset |
| sort_order | string | No | asc | asc, desc |

---

## Category: Sources (3 endpoints)

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /fred/sources | Get all sources | Yes | Yes | 120/min | All data providers |
| GET | /fred/source | Get a source | Yes | Yes | 120/min | By source_id |
| GET | /fred/source/releases | Get releases for source | Yes | Yes | 120/min | Source's releases |

### GET /fred/sources

Get all sources of economic data.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| api_key | string | Yes | - | API key |
| file_type | string | No | xml | Response format |
| realtime_start | date | No | today | YYYY-MM-DD |
| realtime_end | date | No | today | YYYY-MM-DD |
| limit | integer | No | 1000 | Max results (1-1000) |
| offset | integer | No | 0 | Pagination offset |
| order_by | string | No | source_id | source_id, name, realtime_start, realtime_end |
| sort_order | string | No | asc | asc, desc |

### GET /fred/source

Get a source of economic data.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| api_key | string | Yes | - | API key |
| file_type | string | No | xml | Response format |
| source_id | integer | Yes | - | Source ID |
| realtime_start | date | No | today | YYYY-MM-DD |
| realtime_end | date | No | today | YYYY-MM-DD |

### GET /fred/source/releases

Get releases for a source.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| api_key | string | Yes | - | API key |
| file_type | string | No | xml | Response format |
| source_id | integer | Yes | - | Source ID |
| realtime_start | date | No | today | YYYY-MM-DD |
| realtime_end | date | No | today | YYYY-MM-DD |
| limit | integer | No | 1000 | Max results (1-1000) |
| offset | integer | No | 0 | Pagination offset |
| order_by | string | No | release_id | release_id, name, press_release, realtime_start, realtime_end |
| sort_order | string | No | asc | asc, desc |

---

## Category: Tags (3 endpoints)

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /fred/tags | Get all tags | Yes | Yes | 120/min | FRED tag system |
| GET | /fred/related_tags | Get related tags | Yes | Yes | 120/min | Tag relationships |
| GET | /fred/tags/series | Get series by tags | Yes | Yes | 120/min | Tag-based filtering |

### GET /fred/tags

Get FRED tags (attributes assigned to series).

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| api_key | string | Yes | - | API key |
| file_type | string | No | xml | Response format |
| realtime_start | date | No | today | YYYY-MM-DD |
| realtime_end | date | No | today | YYYY-MM-DD |
| tag_names | string | No | - | Filter by tag names |
| tag_group_id | string | No | - | freq, gen, geo, geot, rls, seas, src |
| search_text | string | No | - | Search tag names |
| limit | integer | No | 1000 | Max results (1-1000) |
| offset | integer | No | 0 | Pagination offset |
| order_by | string | No | series_count | series_count, popularity, created, name, group_id |
| sort_order | string | No | asc | asc, desc |

**Tag Groups:**
- freq = Frequency
- gen = General or Concept
- geo = Geography
- geot = Geography Type
- rls = Release
- seas = Seasonal Adjustment
- src = Source

### GET /fred/related_tags

Get related FRED tags for one or more FRED tags.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| api_key | string | Yes | - | API key |
| file_type | string | No | xml | Response format |
| realtime_start | date | No | today | YYYY-MM-DD |
| realtime_end | date | No | today | YYYY-MM-DD |
| tag_names | string | Yes | - | Semicolon-separated tag names |
| exclude_tag_names | string | No | - | Semicolon-separated tag names |
| tag_group_id | string | No | - | freq, gen, geo, geot, rls, seas, src |
| search_text | string | No | - | Search tag names |
| limit | integer | No | 1000 | Max results (1-1000) |
| offset | integer | No | 0 | Pagination offset |
| order_by | string | No | series_count | series_count, popularity, created, name, group_id |
| sort_order | string | No | asc | asc, desc |

### GET /fred/tags/series

Get series matching tags.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| api_key | string | Yes | - | API key |
| file_type | string | No | xml | Response format |
| tag_names | string | Yes | - | Semicolon-separated tag names |
| exclude_tag_names | string | No | - | Semicolon-separated tag names |
| realtime_start | date | No | today | YYYY-MM-DD |
| realtime_end | date | No | today | YYYY-MM-DD |
| limit | integer | No | 1000 | Max results (1-1000) |
| offset | integer | No | 0 | Pagination offset |
| order_by | string | No | series_id | series_id, title, units, frequency, seasonal_adjustment, realtime_start, realtime_end, last_updated, observation_start, observation_end, popularity, group_popularity |
| sort_order | string | No | asc | asc, desc |

---

## Summary

**Total Endpoints: 30**

- Categories: 6 endpoints
- Releases: 8 endpoints
- Series: 10 endpoints (including /series/observations - the core data endpoint)
- Sources: 3 endpoints
- Tags: 3 endpoints

**All endpoints:**
- Require API key
- Support XML (default) and JSON output
- Use 120 requests/minute rate limit
- Are completely free for non-commercial use
- Support realtime period filtering (for ALFRED vintage data)
