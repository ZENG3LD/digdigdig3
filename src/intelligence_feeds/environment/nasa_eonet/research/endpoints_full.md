# NASA EONET API v3 Endpoints

Base URL: `https://eonet.gsfc.nasa.gov/api/v3`

## Events Endpoints

### GET /events

Returns events in standard JSON format.

**URL**: `https://eonet.gsfc.nasa.gov/api/v3/events`

**Query Parameters**:

| Parameter | Type | Values | Description |
|-----------|------|--------|-------------|
| `status` | string | `open`, `closed`, `all` | Filter by event status. Default: `open` |
| `days` | integer | 1-365+ | Limit to events from past N days (including today) |
| `limit` | integer | 1-1000+ | Cap on number of events returned |
| `source` | string | Source ID(s) | Filter by source. Comma-separated for multiple (OR logic) |
| `category` | string | Category ID(s) | Filter by category. Comma-separated for multiple |
| `start` | string | YYYY-MM-DD | Start date for event range (inclusive) |
| `end` | string | YYYY-MM-DD | End date for event range (inclusive) |
| `bbox` | string | min_lon,min_lat,max_lon,max_lat | Bounding box filter for event coordinates |
| `magID` | string | Magnitude ID | Filter by magnitude type |
| `magMin` | decimal | e.g., 0.0 | Minimum magnitude value |
| `magMax` | decimal | e.g., 10.0 | Maximum magnitude value |

**Response Structure**:

```json
{
  "title": "EONET Events",
  "description": "Natural events from EONET",
  "link": "https://eonet.gsfc.nasa.gov/api/v3/events",
  "events": [
    {
      "id": "EONET_12345",
      "title": "Event Title",
      "description": "Event description or null",
      "link": "https://eonet.gsfc.nasa.gov/api/v3/events/EONET_12345",
      "closed": null,
      "categories": [
        {
          "id": "wildfires",
          "title": "Wildfires"
        }
      ],
      "sources": [
        {
          "id": "InciWeb",
          "url": "http://example.com/source"
        }
      ],
      "geometry": [
        {
          "magnitudeValue": 1000.0,
          "magnitudeUnit": "acres",
          "date": "2026-02-15T12:00:00Z",
          "type": "Point",
          "coordinates": [-120.5, 38.5]
        }
      ]
    }
  ]
}
```

**Examples**:

```
# Open events from last 30 days
GET /api/v3/events?status=open&days=30

# Wildfires in California (bbox approximation)
GET /api/v3/events?category=wildfires&bbox=-124.4,32.5,-114.1,42.0

# Multiple sources
GET /api/v3/events?source=CALFIRE,InciWeb&status=open

# Date range query
GET /api/v3/events?start=2026-01-01&end=2026-01-31
```

---

### GET /events/geojson

Returns events in GeoJSON FeatureCollection format.

**URL**: `https://eonet.gsfc.nasa.gov/api/v3/events/geojson`

**Query Parameters**: Same as `/events` endpoint

**Response Structure**:

```json
{
  "type": "FeatureCollection",
  "features": [
    {
      "type": "Feature",
      "properties": {
        "id": "EONET_12345",
        "title": "Event Title",
        "description": null,
        "date": "2026-02-15T12:00:00Z",
        "magnitudeValue": 1000.0,
        "magnitudeUnit": "acres",
        "categories": [
          {
            "id": "wildfires",
            "title": "Wildfires"
          }
        ],
        "sources": [
          {
            "id": "InciWeb",
            "url": "http://example.com"
          }
        ],
        "closed": null
      },
      "geometry": {
        "type": "Point",
        "coordinates": [-120.5, 38.5]
      }
    }
  ]
}
```

---

## Categories Endpoint

### GET /categories

Returns all event categories.

**URL**: `https://eonet.gsfc.nasa.gov/api/v3/categories`

**Query Parameters**: None

**Response Structure**:

```json
{
  "title": "EONET Event Categories",
  "description": "List of all event categories in EONET",
  "link": "https://eonet.gsfc.nasa.gov/api/v3/categories",
  "categories": [
    {
      "id": "wildfires",
      "title": "Wildfires",
      "description": "Wildland fires includes all nature of fire, in forest and plains, urban areas",
      "link": "https://eonet.gsfc.nasa.gov/api/v3/categories/wildfires",
      "layers": "https://eonet.gsfc.nasa.gov/api/v3/layers/wildfires"
    }
  ]
}
```

**Category IDs**:
- `drought`
- `dustHaze`
- `earthquakes`
- `floods`
- `landslides`
- `manmade`
- `seaLakeIce`
- `severeStorms`
- `snow`
- `tempExtremes`
- `volcanoes`
- `waterColor`
- `wildfires`

---

### GET /categories/{id}

Returns events for a specific category.

**URL**: `https://eonet.gsfc.nasa.gov/api/v3/categories/{id}`

**Path Parameters**:
- `id`: Category ID (e.g., `wildfires`)

**Query Parameters**: Same filtering options as `/events`

**Example**:
```
GET /api/v3/categories/wildfires?status=open&limit=10
```

---

## Sources Endpoint

### GET /sources

Returns all event sources.

**URL**: `https://eonet.gsfc.nasa.gov/api/v3/sources`

**Query Parameters**: None

**Response Structure**:

```json
{
  "title": "EONET Event Sources",
  "description": "List of all sources in EONET",
  "link": "https://eonet.gsfc.nasa.gov/api/v3/sources",
  "sources": [
    {
      "id": "InciWeb",
      "title": "InciWeb",
      "source": "http://inciweb.nwcg.gov/",
      "link": "https://eonet.gsfc.nasa.gov/api/v3/events?source=InciWeb"
    }
  ]
}
```

**Major Source IDs**:
- `InciWeb`, `CALFIRE`, `IRWIN`, `BCWILDFIRE` (Wildfires)
- `SIVolcano`, `AVO` (Volcanoes)
- `NOAA_NHC`, `JTWC` (Storms)
- `USGS_EHP` (Earthquakes)
- `FloodList`, `AU_BOM` (Floods)
- `GDACS`, `PDC`, `ReliefWeb` (Multi-hazard)

---

## Layers Endpoint

### GET /layers

Returns web service layers for visualization.

**URL**: `https://eonet.gsfc.nasa.gov/api/v3/layers`

**Query Parameters**: None

**Response Structure**:

```json
{
  "title": "EONET Layers",
  "description": "List of web service layers in EONET",
  "link": "https://eonet.gsfc.nasa.gov/api/v3/layers",
  "categories": [
    {
      "id": "wildfires",
      "title": "Wildfires",
      "layers": [
        {
          "name": "VIIRS_SNPP_CorrectedReflectance_TrueColor",
          "serviceUrl": "https://gibs.earthdata.nasa.gov/wmts/epsg4326/best/wmts.cgi",
          "serviceTypeId": "WMTS",
          "parameters": [...]
        }
      ]
    }
  ]
}
```

**Purpose**: Provides NASA imagery layers (WMTS/WMS) for overlaying with events.

---

### GET /layers/{category_id}

Returns visualization layers for a specific category.

**URL**: `https://eonet.gsfc.nasa.gov/api/v3/layers/{category_id}`

**Path Parameters**:
- `category_id`: Category ID (e.g., `wildfires`)

---

## Magnitudes Endpoint

### GET /magnitudes

Returns magnitude types used in events.

**URL**: `https://eonet.gsfc.nasa.gov/api/v3/magnitudes`

**Query Parameters**: None

**Response Structure**: List of magnitude types with IDs and units.

---

## Error Responses

**404 Not Found**:
```json
{
  "error": "Resource not found"
}
```

**400 Bad Request**:
```json
{
  "error": "Invalid parameter value"
}
```

---

## Notes

- All coordinates use **[longitude, latitude]** format (GeoJSON standard)
- Dates in **ISO 8601 format** with UTC timezone
- Multiple geometry points per event possible (e.g., storm path)
- `closed` field is `null` for open events, contains date string when closed
- Bounding box format: `min_lon,min_lat,max_lon,max_lat`
