# GDACS API Endpoints

## Base URLs

### JSON API
```
https://www.gdacs.org/gdacsapi/api/
```

### RSS/XML Feeds
```
https://www.gdacs.org/xml/
```

### KML Data
```
http://www.gdacs.org/kml.aspx
```

## Primary JSON Endpoints

### 1. Get Event List (Search)

**Endpoint**: `/events/geteventlist/SEARCH`

**Method**: GET

**Full URL**:
```
https://www.gdacs.org/gdacsapi/api/events/geteventlist/SEARCH
```

**Query Parameters**:

| Parameter | Type | Required | Description | Example |
|-----------|------|----------|-------------|---------|
| `eventlist` | string | No | Filter by disaster type | `EQ`, `TC`, `FL`, `VO`, `WF`, `DR` (comma-separated) |
| `fromdate` | string | No | Start date (ISO 8601) | `2023-01-01` |
| `todate` | string | No | End date (ISO 8601) | `2023-12-31` |
| `alertlevel` | string | No | Filter by alert level | `red`, `orange`, `green` (semicolon-separated: `red;orange`) |
| `pagenumber` | integer | No | Page number for pagination | `1`, `2`, `3` (default: 1) |
| `pagesize` | integer | No | Records per page | `10`, `50`, `100` (default: 100, max: 100) |

**Example Requests**:

```bash
# All recent earthquakes with Orange or Red alerts
https://www.gdacs.org/gdacsapi/api/events/geteventlist/SEARCH?eventlist=EQ&alertlevel=orange;red

# Floods between specific dates
https://www.gdacs.org/gdacsapi/api/events/geteventlist/SEARCH?eventlist=FL&fromdate=2023-04-11&todate=2023-10-11

# All disasters with Red alerts, paginated
https://www.gdacs.org/gdacsapi/api/events/geteventlist/SEARCH?alertlevel=red&pagesize=50&pagenumber=1

# Multiple disaster types
https://www.gdacs.org/gdacsapi/api/events/geteventlist/SEARCH?eventlist=EQ,TC,FL&alertlevel=orange;red
```

**Response Format**: GeoJSON FeatureCollection (see `response_formats.md`)

**Pagination**:
- Maximum 100 records per request
- Records ordered by `todate` (descending, most recent first)
- Use `pagenumber` to fetch subsequent pages
- Empty `features` array indicates no more data

### 2. Get Event List (Map View)

**Endpoint**: `/events/geteventlist/MAP`

**Method**: GET

**Full URL**:
```
https://www.gdacs.org/gdacsapi/api/events/geteventlist/MAP
```

**Query Parameters**: Same as SEARCH endpoint

**Difference**: May return different data optimized for map display (bounding boxes, simplified geometries)

### 3. Events for Mobile App

**Endpoint**: `/events/geteventlist/events4app`

**Method**: GET

**Full URL**:
```
https://www.gdacs.org/gdacsapi/api/events/geteventlist/events4app
```

**Query Parameters**: Similar to SEARCH (not fully documented)

**Purpose**: Optimized response for mobile applications (possibly reduced payload)

## Event Detail Endpoints

### Individual Event Details

**Pattern**: `/events/getdetails/{eventtype}/{eventid}`

**Example**:
```
https://www.gdacs.org/gdacsapi/api/events/getdetails/EQ/1234567
```

**Response**: Detailed event data including full description, impact calculations, affected countries

### Event Geometry

**URL Pattern**: Provided in each event's `url.geometry` field

**Example**:
```
https://www.gdacs.org/gdacsapi/api/polygons/geteventdata?eventtype=TC&eventid=1001256&episodeid=21
```

**Response**: Polygon/multipolygon geometry for affected area, forecast tracks (for TC)

### Event Report

**URL Pattern**: Provided in each event's `url.report` field

**Example**:
```
https://www.gdacs.org/report.aspx?eventtype=TC&eventid=1001256&episodeid=21
```

**Response**: HTML report page with detailed analysis, maps, impact data

## RSS/XML Feed Endpoints

### All Events (Time-Based)

| Feed | URL | Description |
|------|-----|-------------|
| Last 24 hours | `xml/rss_24h.xml` | All disaster types |
| Last 7 days | `xml/rss_7d.xml` | All disaster types |

**Full URLs**:
```
https://www.gdacs.org/xml/rss_24h.xml
https://www.gdacs.org/xml/rss_7d.xml
```

### Earthquake Feeds

| Feed | URL | Criteria |
|------|-----|----------|
| Last 24h | `xml/rss_eq_24h.xml` | All earthquakes |
| M≥4.5 (48h) | `xml/rss_eq_48h_low.xml` | Magnitude ≥4.5 |
| M≥5.5 (48h) | `xml/rss_eq_48h_med.xml` | Magnitude ≥5.5 |
| Orange/Red (3M) | `xml/rss_eq_3M.xml` | Orange or Red alerts |
| M≥5.5 (3M) | `xml/rss_eq_5.5_3m.xml` | Magnitude ≥5.5 |

**Full URLs**:
```
https://www.gdacs.org/xml/rss_eq_24h.xml
https://www.gdacs.org/xml/rss_eq_48h_low.xml
https://www.gdacs.org/xml/rss_eq_48h_med.xml
https://www.gdacs.org/xml/rss_eq_3M.xml
https://www.gdacs.org/xml/rss_eq_5.5_3m.xml
```

### Tropical Cyclone Feeds

| Feed | URL | Coverage |
|------|-----|----------|
| Last week | `xml/rss_tc_7d.xml` | 7 days |
| Last 3 months | `xml/rss_tc_3m.xml` | 3 months |

**Full URLs**:
```
https://www.gdacs.org/xml/rss_tc_7d.xml
https://www.gdacs.org/xml/rss_tc_3m.xml
```

### Flood Feeds

| Feed | URL | Coverage |
|------|-----|----------|
| Last week | `xml/rss_fl_7d.xml` | 7 days |
| Last 3 months | `xml/rss_fl_3m.xml` | 3 months |

**Full URLs**:
```
https://www.gdacs.org/xml/rss_fl_7d.xml
https://www.gdacs.org/xml/rss_fl_3m.xml
```

### Volcano Feeds

Not explicitly documented, likely available at:
```
https://www.gdacs.org/xml/rss_vo_7d.xml
https://www.gdacs.org/xml/rss_vo_3m.xml
```

### Wildfire Feeds

Not explicitly documented, likely available at:
```
https://www.gdacs.org/xml/rss_wf_7d.xml
https://www.gdacs.org/xml/rss_wf_3m.xml
```

### Drought Feeds

Not explicitly documented, likely available at:
```
https://www.gdacs.org/xml/rss_dr_7d.xml
https://www.gdacs.org/xml/rss_dr_3m.xml
```

## RSS Feed Format

**Standard**: RSS 2.0 with custom namespaces

**Namespaces**:
- `gdacs:` - GDACS-specific attributes (alert level, severity)
- `geo:` / `georss:` - Geographic coordinates
- `dc:` - Dublin Core metadata

**Update Frequency**: Every 6 minutes

## KML Endpoint

**URL**: `http://www.gdacs.org/kml.aspx`

**Query Parameters**: Not documented (likely similar to API filters)

**Format**: KML/KMZ for Google Earth and GIS applications

## Documentation Endpoints

### Swagger API Documentation

**URL**: `https://www.gdacs.org/gdacsapi/swagger/index.html`

**Content**: Interactive API documentation (may have loading issues)

### CAP Feeds

**Pattern**: Available per event

**Example**: From search results
```
https://www.gdacs.org/cap.aspx?profile=archive&eventid=1000149&eventtype=TC
```

**Format**: Common Alerting Protocol (CAP) XML

## Endpoint Selection Guide

### Use JSON API (`/geteventlist/SEARCH`) when:
- Building data feeds for applications
- Need structured GeoJSON data
- Filtering by multiple criteria
- Pagination required
- Programmatic access

### Use RSS Feeds when:
- Simple time-based monitoring
- RSS reader integration
- Webhook/polling systems
- Don't need filtering beyond time/type

### Use Event Detail endpoints when:
- Need full event descriptions
- Accessing detailed impact data
- Retrieving affected area geometries
- Generating reports

## Rate Limits

**Not explicitly documented**. Based on:
- RSS feeds update every 6 minutes
- Public API with no authentication
- Recommendation: Poll no more than once per 5 minutes
- Use pagination for large datasets
- Implement caching for frequently accessed data

## Best Practices

1. **Use specific filters**: Reduce payload with `eventlist` and `alertlevel` parameters
2. **Implement pagination**: Don't assume all data fits in one request
3. **Cache responses**: Especially for historical data
4. **Handle empty results**: Check `features` array length
5. **Parse dates correctly**: ISO 8601 format with timezone (UTC)
6. **Follow redirects**: Some URLs may redirect
7. **Error handling**: Handle network errors, invalid responses
8. **Attribution**: Include GDACS and data source attribution in your application
