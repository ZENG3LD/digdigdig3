# NASA EONET Data Types

## Event Categories

EONET tracks **13 natural event categories**. Each event belongs to at least one category (can have multiple).

### Category Definitions

| ID | Title | Description | Typical Sources | Update Frequency |
|----|-------|-------------|-----------------|------------------|
| `drought` | Drought | Long lasting absence of precipitation affecting agriculture | GDACS, ReliefWeb | Weekly-Monthly |
| `dustHaze` | Dust and Haze | Dust storms, air pollution, non-volcanic aerosols | NASA_DISP, EO | Daily |
| `earthquakes` | Earthquakes | All manner of shaking and displacement | USGS_EHP, GDACS | Minutes-Hours |
| `floods` | Floods | Inundation, water extending beyond normal extents | FloodList, AU_BOM, GDACS | Daily |
| `landslides` | Landslides | Mudslides, avalanches, rockslides | GDACS, ReliefWeb | Daily |
| `manmade` | Manmade | Human-induced events extreme in extent | GDACS, IDC | Varies |
| `seaLakeIce` | Sea and Lake Ice | Ice on oceans and lakes, including sea ice | NATICE, BYU_ICE | Daily-Weekly |
| `severeStorms` | Severe Storms | Hurricanes, cyclones, tornadoes, typhoons | NOAA_NHC, JTWC, GDACS | Hourly-Daily |
| `snow` | Snow | Extreme/anomalous snowfall in timing or extent | NASA_DISP, EO | Daily |
| `tempExtremes` | Temperature Extremes | Anomalous land temperatures (heat or cold) | GDACS, NOAA_CPC | Daily-Weekly |
| `volcanoes` | Volcanoes | Physical and atmospheric effects of eruptions | SIVolcano, AVO | Daily |
| `waterColor` | Water Color | Phytoplankton, algae, sediment affecting water appearance | EO, Earthdata | Daily-Weekly |
| `wildfires` | Wildfires | All wildland fires (forest, plains, urban areas) | InciWeb, CALFIRE, IRWIN | Hourly-Daily |

### Category Relationships

- **Single category**: Most events (e.g., wildfire = `wildfires` only)
- **Multiple categories**: Some events span categories (e.g., volcano eruption = `volcanoes` + `dustHaze`)
- **Category hierarchy**: None (flat structure)

### Category Filtering

Query events by category:
```
GET /api/v3/events?category=wildfires
GET /api/v3/events?category=wildfires,severeStorms  # OR logic
GET /api/v3/categories/wildfires  # All wildfire events
```

---

## Geometry Types

Events use **GeoJSON geometry types** to represent spatial data.

### Point Geometry

Most common type. Represents single location.

```json
{
  "magnitudeValue": 1500.0,
  "magnitudeUnit": "acres",
  "date": "2026-02-15T14:30:00Z",
  "type": "Point",
  "coordinates": [-120.5234, 38.7645]
}
```

**Coordinate Format**: `[longitude, latitude]` (GeoJSON standard)

**Examples**:
- Wildfire ignition point
- Volcano summit
- Earthquake epicenter
- Flood observation location

### Polygon Geometry

Used for area-based events.

```json
{
  "magnitudeValue": null,
  "magnitudeUnit": null,
  "date": "2026-02-14T00:00:00Z",
  "type": "Polygon",
  "coordinates": [
    [
      [-120.5, 38.7],
      [-120.3, 38.7],
      [-120.3, 38.5],
      [-120.5, 38.5],
      [-120.5, 38.7]
    ]
  ]
}
```

**Structure**: Array of linear rings (first = exterior, rest = holes)

**Examples**:
- Wildfire perimeter
- Flood extent
- Sea ice boundary
- Storm warning area

### Multiple Geometries

Events can have **multiple geometry entries** representing:
- **Temporal progression**: Event moving/growing over time
- **Multiple observations**: Different sources reporting same event
- **Different geometry types**: Point + Polygon for same event

**Example** (Hurricane path):
```json
"geometry": [
  {
    "date": "2026-02-10T00:00:00Z",
    "type": "Point",
    "coordinates": [-80.5, 15.2]
  },
  {
    "date": "2026-02-11T00:00:00Z",
    "type": "Point",
    "coordinates": [-81.2, 16.5]
  },
  {
    "date": "2026-02-12T00:00:00Z",
    "type": "Point",
    "coordinates": [-82.1, 18.3]
  }
]
```

**Sorting**: Geometries ordered chronologically by `date` field.

---

## Magnitude Types

Magnitudes quantify event intensity/size. Type varies by event category.

### Common Magnitude Units

| Unit | Categories | Description | Example Value |
|------|------------|-------------|---------------|
| `acres` | Wildfires | Fire burn area | 1500.0 |
| `NM^2` | Wildfires | Nautical square miles | 2.5 |
| `kts` | Severe Storms | Wind speed in knots | 120.0 |
| `mph` | Severe Storms | Wind speed in miles/hour | 140.0 |
| `mb` | Severe Storms | Pressure in millibars | 950.0 |
| `Richter` | Earthquakes | Earthquake magnitude | 6.5 |
| `VEI` | Volcanoes | Volcanic Explosivity Index | 4.0 |
| `km^2` | Floods, Ice | Area in square kilometers | 250.0 |

### Magnitude Structure in Geometry

```json
{
  "magnitudeValue": 1200.5,
  "magnitudeUnit": "acres",
  "date": "2026-02-15T10:00:00Z",
  "type": "Point",
  "coordinates": [-119.5, 37.2]
}
```

**Null values**: Common when magnitude not measured or not applicable.

```json
{
  "magnitudeValue": null,
  "magnitudeUnit": null,
  "date": "2026-02-14T12:00:00Z",
  "type": "Point",
  "coordinates": [-118.3, 36.8]
}
```

### Magnitude Filtering

Query events by magnitude:
```
GET /api/v3/events?magID=acres&magMin=1000&magMax=5000
GET /api/v3/events?category=wildfires&magMin=500  # All wildfires > 500 acres
```

---

## Event Status Types

Events have three lifecycle statuses.

### Status Values

| Status | Description | Use Case |
|--------|-------------|----------|
| `open` | Event is ongoing/active | Current monitoring |
| `closed` | Event has ended | Historical analysis |
| `all` | Both open and closed | Full dataset queries |

### Status in API

**Query parameter**:
```
GET /api/v3/events?status=open   # Default
GET /api/v3/events?status=closed
GET /api/v3/events?status=all
```

**Event field**:
```json
{
  "id": "EONET_12345",
  "closed": null  // Open event
}
```

```json
{
  "id": "EONET_12346",
  "closed": "2026-02-10T18:00:00Z"  // Closed event with end date
}
```

**Interpretation**:
- `closed: null` → Event is open
- `closed: "2026-02-10T..."` → Event closed on that date

---

## Date Formats

All timestamps use **ISO 8601 format** with UTC timezone.

### Format Pattern

```
YYYY-MM-DDTHH:MM:SSZ
```

**Examples**:
- `2026-02-15T14:30:00Z`
- `2026-01-01T00:00:00Z`
- `2025-12-31T23:59:59Z`

### Date Fields

| Field | Location | Format | Example |
|-------|----------|--------|---------|
| `date` | `geometry[].date` | ISO 8601 | `2026-02-15T14:30:00Z` |
| `closed` | `events[].closed` | ISO 8601 or null | `2026-02-10T18:00:00Z` |

### Date Range Queries

```
GET /api/v3/events?start=2026-02-01&end=2026-02-15
GET /api/v3/events?days=7  # Last 7 days including today
```

**Format for query params**: `YYYY-MM-DD` (date only, no time)

---

## Source Types

Events reference **33 different sources** across multiple organizations.

### Source Categories

| Type | Examples | Count |
|------|----------|-------|
| Government (US) | USGS, NOAA, FEMA, CALFIRE | 8 |
| Government (International) | AU_BOM, BCWILDFIRE, DFES_WA | 5 |
| Research Institutions | Smithsonian, Alaska VO | 3 |
| International Orgs | GDACS, ReliefWeb, GLIDE | 6 |
| NASA Programs | NASA_DISP, NASA_HURR, NASA_ESRS, Earthdata, EO | 7 |
| Other | InciWeb, FloodList, JTWC, BYU_ICE, NATICE, IDC, CEMS, PDC, USGS_CMT, HDDS, MRR | 11 |

### Source Structure

```json
"sources": [
  {
    "id": "InciWeb",
    "url": "http://inciweb.nwcg.gov/incident/9876/"
  }
]
```

**Fields**:
- `id`: Unique source identifier (matches `/sources` endpoint)
- `url`: Link to source's page for this specific event

### Multi-source Events

Events can have multiple sources:
```json
"sources": [
  { "id": "CALFIRE", "url": "http://..." },
  { "id": "InciWeb", "url": "http://..." }
]
```

---

## Coordinate System

All coordinates use **WGS84** (World Geodetic System 1984).

### Format

**GeoJSON standard**: `[longitude, latitude]`

**Range limits**:
- Longitude: -180.0 to +180.0 (west to east)
- Latitude: -90.0 to +90.0 (south to north)

**Precision**: Typically 4-6 decimal places (~10m-1m accuracy)

### Examples

```json
[-120.5234, 38.7645]  // California
[151.2093, -33.8688]  // Sydney, Australia
[139.6917, 35.6895]   // Tokyo, Japan
[-0.1276, 51.5074]    // London, UK
```

### Bounding Box Format

```
bbox=min_lon,min_lat,max_lon,max_lat
```

**Example** (California):
```
GET /api/v3/events?bbox=-124.4,32.5,-114.1,42.0
```

---

## Data Structures Summary

### Event Object (Full Schema)

```json
{
  "id": "EONET_12345",                    // String: Unique ID
  "title": "Wildfire - CA, Event Name",   // String: Human-readable title
  "description": "Event details...",      // String or null
  "link": "https://eonet.gsfc.../12345",  // String: API URL for this event
  "closed": null,                         // String (ISO 8601) or null
  "categories": [                         // Array: 1+ categories
    {
      "id": "wildfires",                  // String: Category ID
      "title": "Wildfires"                // String: Category name
    }
  ],
  "sources": [                            // Array: 1+ sources
    {
      "id": "InciWeb",                    // String: Source ID
      "url": "http://..."                 // String: External URL
    }
  ],
  "geometry": [                           // Array: 1+ geometries
    {
      "magnitudeValue": 1500.0,           // Number or null
      "magnitudeUnit": "acres",           // String or null
      "date": "2026-02-15T14:30:00Z",     // String: ISO 8601
      "type": "Point",                    // String: "Point" or "Polygon"
      "coordinates": [-120.5, 38.7]       // Array: [lon, lat] or polygon rings
    }
  ]
}
```

### Rust Type Mapping

```rust
struct Event {
    id: String,
    title: String,
    description: Option<String>,
    link: String,
    closed: Option<String>,  // ISO 8601 datetime
    categories: Vec<Category>,
    sources: Vec<Source>,
    geometry: Vec<Geometry>,
}

struct Category {
    id: String,
    title: String,
}

struct Source {
    id: String,
    url: String,
}

struct Geometry {
    magnitude_value: Option<f64>,
    magnitude_unit: Option<String>,
    date: String,  // ISO 8601
    geometry_type: String,  // "Point" or "Polygon"
    coordinates: Vec<f64>,  // [lon, lat] for Point, or Vec<Vec<Vec<f64>>> for Polygon
}
```

---

## Null Handling

EONET API uses `null` for missing/inapplicable data:

| Field | Null Meaning |
|-------|--------------|
| `description` | No description provided |
| `closed` | Event is still open |
| `magnitudeValue` | Magnitude not measured |
| `magnitudeUnit` | Magnitude not applicable |

**Rust implementation**: Use `Option<T>` for all nullable fields.
