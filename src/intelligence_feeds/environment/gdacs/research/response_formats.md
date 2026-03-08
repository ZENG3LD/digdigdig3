# GDACS Response Formats

## Overview

GDACS API returns data in **GeoJSON FeatureCollection** format following RFC 7946 specification.

**Base Structure**:
```json
{
  "type": "FeatureCollection",
  "features": [...],
  "bbox": null
}
```

## Complete Response Example

### Earthquake Event

```json
{
  "type": "FeatureCollection",
  "bbox": null,
  "features": [
    {
      "type": "Feature",
      "geometry": {
        "type": "Point",
        "coordinates": [154.5612, 48.3271]
      },
      "properties": {
        "eventtype": "EQ",
        "eventid": 1522345,
        "episodeid": 1685234,
        "eventname": "",
        "glide": "EQ-2026-000123-RUS",
        "name": "RUSSIA",
        "country": "Russia",
        "iso3": "RUS",
        "iso2": "RU",
        "alertlevel": "Orange",
        "alertscore": 1.8,
        "episodealertlevel": "Orange",
        "episodealertscore": 1.8,
        "fromdate": "2026-02-15T05:11:00+00:00",
        "todate": "2026-02-15T05:11:00+00:00",
        "datemodified": "2026-02-15T05:45:23+00:00",
        "source": "NEIC",
        "icon": "https://www.gdacs.org/Images/gdacs_icons/eq_orange.png",
        "iconoverall": "https://www.gdacs.org/Images/gdacs_icons/alerts/eq_orange.png",
        "iscurrent": true,
        "htmldescription": "<p>On 2/15/2026 5:11:00 AM UTC (2/15/2026 5:11:00 AM local time) an earthquake of magnitude 7.2 occurred in RUSSIA...</p>",
        "affectedcountries": [
          {
            "iso2": "RU",
            "iso3": "RUS",
            "countryname": "Russia"
          }
        ],
        "severitydata": {
          "severity": 7.2,
          "severitytext": "Magnitude 7.2M earthquake at 55km depth",
          "severityunit": "M"
        },
        "url": {
          "geometry": "https://www.gdacs.org/gdacsapi/api/polygons/geteventdata?eventtype=EQ&eventid=1522345&episodeid=1685234",
          "report": "https://www.gdacs.org/report.aspx?eventtype=EQ&eventid=1522345&episodeid=1685234",
          "details": "https://www.gdacs.org/gdacsapi/api/events/getdetails?eventtype=EQ&eventid=1522345"
        }
      }
    }
  ]
}
```

### Tropical Cyclone Event

```json
{
  "type": "Feature",
  "geometry": {
    "type": "Point",
    "coordinates": [40.2, -18.5]
  },
  "properties": {
    "eventtype": "TC",
    "eventid": 1001256,
    "episodeid": 21,
    "eventname": "GEZANI-26",
    "glide": "TC-2026-000005-MOZ",
    "name": "MOZAMBIQUE, MADAGASCAR",
    "country": "Mozambique, Madagascar",
    "iso3": "MOZ",
    "iso2": "MZ",
    "alertlevel": "Red",
    "alertscore": 2.5,
    "episodealertlevel": "Orange",
    "episodealertscore": 1.7,
    "fromdate": "2026-02-06T00:00:00+00:00",
    "todate": "2026-02-12T18:00:00+00:00",
    "datemodified": "2026-02-09T12:30:45+00:00",
    "source": "JTWC",
    "icon": "https://www.gdacs.org/Images/gdacs_icons/tc_red.png",
    "iconoverall": "https://www.gdacs.org/Images/gdacs_icons/alerts/tc_red.png",
    "iscurrent": true,
    "htmldescription": "<p>Tropical Cyclone GEZANI-26 approaching Mozambique with maximum sustained winds of 211 km/h...</p>",
    "affectedcountries": [
      {
        "iso2": "MZ",
        "iso3": "MOZ",
        "countryname": "Mozambique"
      },
      {
        "iso2": "MG",
        "iso3": "MDG",
        "countryname": "Madagascar"
      }
    ],
    "severitydata": {
      "severity": 211,
      "severitytext": "Severe Tropical Storm (maximum wind speed of 211 km/h)",
      "severityunit": "km/h"
    },
    "url": {
      "geometry": "https://www.gdacs.org/gdacsapi/api/polygons/geteventdata?eventtype=TC&eventid=1001256&episodeid=21",
      "report": "https://www.gdacs.org/report.aspx?eventtype=TC&eventid=1001256&episodeid=21",
      "details": "https://www.gdacs.org/gdacsapi/api/events/getdetails?eventtype=TC&eventid=1001256"
    }
  }
}
```

### Flood Event

```json
{
  "type": "Feature",
  "geometry": {
    "type": "Point",
    "coordinates": [100.5, 13.75]
  },
  "properties": {
    "eventtype": "FL",
    "eventid": 1028456,
    "episodeid": 5,
    "eventname": "",
    "glide": "FL-2025-000089-THA",
    "name": "THAILAND",
    "country": "Thailand",
    "iso3": "THA",
    "iso2": "TH",
    "alertlevel": "Orange",
    "alertscore": 1.5,
    "episodealertlevel": "Orange",
    "episodealertscore": 1.5,
    "fromdate": "2025-09-15T00:00:00+00:00",
    "todate": "2025-10-02T00:00:00+00:00",
    "datemodified": "2025-10-02T08:23:11+00:00",
    "source": "GLOFAS",
    "icon": "https://www.gdacs.org/Images/gdacs_icons/fl_orange.png",
    "iconoverall": "https://www.gdacs.org/Images/gdacs_icons/alerts/fl_orange.png",
    "iscurrent": false,
    "htmldescription": "<p>Flooding in Thailand affecting 250,000 people, 85 casualties reported...</p>",
    "affectedcountries": [
      {
        "iso2": "TH",
        "iso3": "THA",
        "countryname": "Thailand"
      }
    ],
    "severitydata": {
      "severity": null,
      "severitytext": "85 casualties, 150,000 displaced",
      "severityunit": ""
    },
    "url": {
      "geometry": "https://www.gdacs.org/gdacsapi/api/polygons/geteventdata?eventtype=FL&eventid=1028456&episodeid=5",
      "report": "https://www.gdacs.org/report.aspx?eventtype=FL&eventid=1028456&episodeid=5",
      "details": "https://www.gdacs.org/gdacsapi/api/events/getdetails?eventtype=FL&eventid=1028456"
    }
  }
}
```

### Volcano Event

```json
{
  "type": "Feature",
  "geometry": {
    "type": "Point",
    "coordinates": [112.92, -8.11]
  },
  "properties": {
    "eventtype": "VO",
    "eventid": 1023789,
    "episodeid": 3,
    "eventname": "Semeru",
    "glide": "VO-2025-000012-IDN",
    "name": "INDONESIA",
    "country": "Indonesia",
    "iso3": "IDN",
    "iso2": "ID",
    "alertlevel": "Orange",
    "alertscore": 1.3,
    "episodealertlevel": "Orange",
    "episodealertscore": 1.3,
    "fromdate": "2025-11-10T06:00:00+00:00",
    "todate": "2025-11-15T12:00:00+00:00",
    "datemodified": "2025-11-15T14:22:33+00:00",
    "source": "DARWIN",
    "icon": "https://www.gdacs.org/Images/gdacs_icons/vo_orange.png",
    "iconoverall": "https://www.gdacs.org/Images/gdacs_icons/alerts/vo_orange.png",
    "iscurrent": true,
    "htmldescription": "<p>Eruption of Semeru volcano with ash cloud to 30,000 feet, 5,000 people evacuated...</p>",
    "affectedcountries": [
      {
        "iso2": "ID",
        "iso3": "IDN",
        "countryname": "Indonesia"
      }
    ],
    "severitydata": {
      "severity": null,
      "severitytext": "Ash cloud to 30,000 feet",
      "severityunit": ""
    },
    "url": {
      "geometry": "https://www.gdacs.org/gdacsapi/api/polygons/geteventdata?eventtype=VO&eventid=1023789&episodeid=3",
      "report": "https://www.gdacs.org/report.aspx?eventtype=VO&eventid=1023789&episodeid=3",
      "details": "https://www.gdacs.org/gdacsapi/api/events/getdetails?eventtype=VO&eventid=1023789"
    }
  }
}
```

### Wildfire Event

```json
{
  "type": "Feature",
  "geometry": {
    "type": "Point",
    "coordinates": [-71.5, -33.5]
  },
  "properties": {
    "eventtype": "WF",
    "eventid": 1034567,
    "episodeid": 2,
    "eventname": "",
    "glide": "WF-2026-000003-CHL",
    "name": "CHILE",
    "country": "Chile",
    "iso3": "CHL",
    "iso2": "CL",
    "alertlevel": "Orange",
    "alertscore": 1.6,
    "episodealertlevel": "Orange",
    "episodealertscore": 1.6,
    "fromdate": "2026-01-20T00:00:00+00:00",
    "todate": "2026-02-05T00:00:00+00:00",
    "datemodified": "2026-02-05T10:15:42+00:00",
    "source": "GWIS",
    "icon": "https://www.gdacs.org/Images/gdacs_icons/wf_orange.png",
    "iconoverall": "https://www.gdacs.org/Images/gdacs_icons/alerts/wf_orange.png",
    "iscurrent": true,
    "htmldescription": "<p>Forest fires in Chile affecting 12,340 hectares, 200 people affected...</p>",
    "affectedcountries": [
      {
        "iso2": "CL",
        "iso3": "CHL",
        "countryname": "Chile"
      }
    ],
    "severitydata": {
      "severity": 12340,
      "severitytext": "12,340 hectares burned",
      "severityunit": "ha"
    },
    "url": {
      "geometry": "https://www.gdacs.org/gdacsapi/api/polygons/geteventdata?eventtype=WF&eventid=1034567&episodeid=2",
      "report": "https://www.gdacs.org/report.aspx?eventtype=WF&eventid=1034567&episodeid=2",
      "details": "https://www.gdacs.org/gdacsapi/api/events/getdetails?eventtype=WF&eventid=1034567"
    }
  }
}
```

### Drought Event

```json
{
  "type": "Feature",
  "geometry": {
    "type": "Point",
    "coordinates": [39.5, 2.0]
  },
  "properties": {
    "eventtype": "DR",
    "eventid": 1031234,
    "episodeid": 8,
    "eventname": "East Africa-2025",
    "glide": "DR-2025-000008-ETH",
    "name": "ETHIOPIA, KENYA, SOMALIA",
    "country": "Ethiopia, Kenya, Somalia",
    "iso3": "ETH",
    "iso2": "ET",
    "alertlevel": "Orange",
    "alertscore": 1.4,
    "episodealertlevel": "Orange",
    "episodealertscore": 1.4,
    "fromdate": "2025-06-01T00:00:00+00:00",
    "todate": "2026-01-31T00:00:00+00:00",
    "datemodified": "2026-02-01T09:10:20+00:00",
    "source": "GDO",
    "icon": "https://www.gdacs.org/Images/gdacs_icons/dr_orange.png",
    "iconoverall": "https://www.gdacs.org/Images/gdacs_icons/alerts/dr_orange.png",
    "iscurrent": true,
    "htmldescription": "<p>Agricultural drought affecting East Africa, 788,110 km² affected...</p>",
    "affectedcountries": [
      {
        "iso2": "ET",
        "iso3": "ETH",
        "countryname": "Ethiopia"
      },
      {
        "iso2": "KE",
        "iso3": "KEN",
        "countryname": "Kenya"
      },
      {
        "iso2": "SO",
        "iso3": "SOM",
        "countryname": "Somalia"
      }
    ],
    "severitydata": {
      "severity": 788110,
      "severitytext": "Medium impact for agricultural drought in 788110 km2",
      "severityunit": "km²"
    },
    "url": {
      "geometry": "https://www.gdacs.org/gdacsapi/api/polygons/geteventdata?eventtype=DR&eventid=1031234&episodeid=8",
      "report": "https://www.gdacs.org/report.aspx?eventtype=DR&eventid=1031234&episodeid=8",
      "details": "https://www.gdacs.org/gdacsapi/api/events/getdetails?eventtype=DR&eventid=1031234"
    }
  }
}
```

## Field Definitions

### Root Level

| Field | Type | Description |
|-------|------|-------------|
| `type` | string | Always "FeatureCollection" |
| `bbox` | array\|null | Bounding box [minLon, minLat, maxLon, maxLat] (usually null) |
| `features` | array | Array of Feature objects |

### Feature Object

| Field | Type | Description |
|-------|------|-------------|
| `type` | string | Always "Feature" |
| `geometry` | object | GeoJSON Geometry object |
| `properties` | object | Event properties (see below) |

### Geometry Object

| Field | Type | Description |
|-------|------|-------------|
| `type` | string | "Point" (sometimes "Polygon" or "LineString") |
| `coordinates` | array | [longitude, latitude] for Point |

### Properties Object

#### Core Identification

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `eventtype` | string | Yes | Disaster type code (EQ, TC, FL, VO, WF, DR, TS) |
| `eventid` | integer | Yes | Unique event identifier (unchanging for event) |
| `episodeid` | integer | Yes | Episode number (increments with updates) |
| `eventname` | string | No | Named event (cyclones, volcanoes, drought regions) |
| `glide` | string | No | Global Identifier for Disasters (format: TYPE-YEAR-SEQ-ISO3) |

#### Location

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Primary location name (country/region) |
| `country` | string | Yes | Affected country/countries (comma-separated) |
| `iso3` | string | Yes | ISO 3166-1 alpha-3 country code |
| `iso2` | string | No | ISO 3166-1 alpha-2 country code |
| `affectedcountries` | array | Yes | Array of country objects (see below) |

#### Alert Information

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `alertlevel` | string | Yes | "Green", "Orange", or "Red" |
| `alertscore` | float | Yes | Numeric severity score (0.0-3.0) |
| `episodealertlevel` | string | Yes | Alert level for this episode |
| `episodealertscore` | float | Yes | Alert score for this episode |

#### Temporal Data

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `fromdate` | string | Yes | Event start (ISO 8601 with timezone) |
| `todate` | string | Yes | Event end or last update (ISO 8601) |
| `datemodified` | string | Yes | Last modification timestamp (ISO 8601) |

#### Metadata

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `source` | string | Yes | Data source (NEIC, GLOFAS, GWIS, JTWC, etc.) |
| `iscurrent` | boolean | Yes | True if event is active/current |
| `icon` | string | Yes | URL to event icon |
| `iconoverall` | string | Yes | URL to overall alert icon |
| `htmldescription` | string | No | HTML-formatted event description |

#### Severity Data

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `severitydata` | object | Yes | Severity information (see below) |

**severitydata Object**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `severity` | float\|null | No | Numeric severity (magnitude, wind speed, area) |
| `severitytext` | string | Yes | Human-readable severity description |
| `severityunit` | string | No | Unit of measurement (M, km/h, ha, km²) |

#### URLs

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `url` | object | Yes | URLs for additional data (see below) |

**url Object**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `geometry` | string | Yes | API endpoint for event geometry (polygons, tracks) |
| `report` | string | Yes | HTML report page URL |
| `details` | string | Yes | API endpoint for detailed event data |

#### Affected Countries Array

**Structure**:
```json
{
  "affectedcountries": [
    {
      "iso2": "TH",
      "iso3": "THA",
      "countryname": "Thailand"
    }
  ]
}
```

| Field | Type | Description |
|-------|------|-------------|
| `iso2` | string | ISO 3166-1 alpha-2 code |
| `iso3` | string | ISO 3166-1 alpha-3 code |
| `countryname` | string | Full country name |

## Empty Response

When no events match the query:

```json
{
  "type": "FeatureCollection",
  "bbox": null,
  "features": []
}
```

## Error Responses

### HTTP 400 - Bad Request

```json
{
  "message": "Invalid parameter: eventlist",
  "details": "Expected one of: EQ, TC, FL, VO, WF, DR, TS"
}
```

(Note: Exact error format not documented, may vary or return HTML)

### HTTP 404 - Not Found

```html
<!DOCTYPE html>
<html>
<head><title>404 Not Found</title></head>
<body>
<h1>404 Not Found</h1>
<p>The requested URL was not found on this server.</p>
</body>
</html>
```

### HTTP 500 - Internal Server Error

```html
<!DOCTYPE html>
<html>
<head><title>500 Internal Server Error</title></head>
<body>
<h1>500 Internal Server Error</h1>
</body>
</html>
```

## Data Type Specifics

### Nullable Fields

Fields that may be `null`:
- `eventname`: Often empty for EQ, FL, WF
- `glide`: May be missing for minor events
- `iso2`: Sometimes omitted
- `severitydata.severity`: Null for some FL, VO events
- `severitydata.severityunit`: Empty string when severity is descriptive
- `htmldescription`: May be missing

### Optional Fields

Fields that may be absent entirely:
- `glide`
- `eventname`
- `iso2`
- `htmldescription`

### Data Type Variations

**Coordinates**:
- Always [longitude, latitude]
- Decimal degrees
- Range: longitude [-180, 180], latitude [-90, 90]

**Timestamps**:
- Format: `YYYY-MM-DDTHH:MM:SS+00:00` (ISO 8601)
- Timezone: Always UTC (+00:00)
- Resolution: Seconds

**Country Strings**:
- Single country: `"Thailand"`
- Multiple countries: `"Ethiopia, Kenya, Somalia"`
- Separator: comma + space

## Parsing Examples (Rust)

### Serde Structs

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize, Serialize)]
pub struct GdacsResponse {
    #[serde(rename = "type")]
    pub feature_type: String, // "FeatureCollection"
    pub bbox: Option<Vec<f64>>,
    pub features: Vec<Feature>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Feature {
    #[serde(rename = "type")]
    pub feature_type: String, // "Feature"
    pub geometry: Geometry,
    pub properties: Properties,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Geometry {
    #[serde(rename = "type")]
    pub geometry_type: String, // "Point"
    pub coordinates: Vec<f64>, // [lon, lat]
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Properties {
    pub eventtype: String,
    pub eventid: i64,
    pub episodeid: i64,
    #[serde(default)]
    pub eventname: String,
    pub glide: Option<String>,
    pub name: String,
    pub country: String,
    pub iso3: String,
    pub iso2: Option<String>,
    pub alertlevel: String,
    pub alertscore: f64,
    pub episodealertlevel: String,
    pub episodealertscore: f64,
    pub fromdate: DateTime<Utc>,
    pub todate: DateTime<Utc>,
    pub datemodified: DateTime<Utc>,
    pub source: String,
    pub icon: String,
    pub iconoverall: String,
    pub iscurrent: bool,
    pub htmldescription: Option<String>,
    pub affectedcountries: Vec<AffectedCountry>,
    pub severitydata: SeverityData,
    pub url: EventUrls,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AffectedCountry {
    pub iso2: String,
    pub iso3: String,
    pub countryname: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SeverityData {
    pub severity: Option<f64>,
    pub severitytext: String,
    #[serde(default)]
    pub severityunit: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EventUrls {
    pub geometry: String,
    pub report: String,
    pub details: String,
}
```

### Parsing Example

```rust
use reqwest;

pub async fn fetch_events() -> Result<GdacsResponse, Box<dyn std::error::Error>> {
    let url = "https://www.gdacs.org/gdacsapi/api/events/geteventlist/SEARCH";

    let response = reqwest::get(url)
        .await?
        .json::<GdacsResponse>()
        .await?;

    Ok(response)
}

// Usage
let events = fetch_events().await?;
for feature in events.features {
    println!("Event: {} {} (Alert: {})",
        feature.properties.eventtype,
        feature.properties.name,
        feature.properties.alertlevel
    );
}
```

## Content Negotiation

**Accepted Headers**:
```http
Accept: application/json
Accept: application/geo+json
Accept: */*
```

All return the same GeoJSON format.

**Encoding**:
```http
Accept-Encoding: gzip, deflate
```

Recommended for bandwidth reduction (60-80% savings).

## Summary

- **Format**: GeoJSON FeatureCollection (RFC 7946)
- **Structure**: Consistent across all disaster types
- **Coordinates**: WGS84 [longitude, latitude]
- **Timestamps**: ISO 8601 with UTC timezone
- **Optional Fields**: eventname, glide, iso2, htmldescription
- **Nullable Fields**: severitydata.severity, severitydata.severityunit
- **Error Responses**: May return HTML instead of JSON
- **Empty Results**: Valid GeoJSON with empty features array
