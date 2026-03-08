# NASA EONET Response Formats

## JSON Format (Default)

### Events Endpoint Response

**URL**: `GET /api/v3/events?status=open&days=30&limit=3`

```json
{
  "title": "EONET Events",
  "description": "Natural events from EONET.",
  "link": "https://eonet.gsfc.nasa.gov/api/v3/events",
  "events": [
    {
      "id": "EONET_17841",
      "title": "Wildfire - TX, Montgomery County - Westwood Prescribed Fire",
      "description": null,
      "link": "https://eonet.gsfc.nasa.gov/api/v3/events/EONET_17841",
      "closed": null,
      "categories": [
        {
          "id": "wildfires",
          "title": "Wildfires"
        }
      ],
      "sources": [
        {
          "id": "IRWIN",
          "url": "https://irwin.doi.gov/observer/7f2b72e2-6e8a-4fc3-a71e-b1b1c8c0123a"
        }
      ],
      "geometry": [
        {
          "magnitudeValue": 500.00,
          "magnitudeUnit": "acres",
          "date": "2026-02-13T16:05:00Z",
          "type": "Point",
          "coordinates": [ -95.10661, 30.87547 ]
        }
      ]
    },
    {
      "id": "EONET_17840",
      "title": "Wildfire - TX, Montgomery County - North Conroe Prescribed Fire",
      "description": null,
      "link": "https://eonet.gsfc.nasa.gov/api/v3/events/EONET_17840",
      "closed": null,
      "categories": [
        {
          "id": "wildfires",
          "title": "Wildfires"
        }
      ],
      "sources": [
        {
          "id": "IRWIN",
          "url": "https://irwin.doi.gov/observer/0c6e3f8e-d3c5-4a1d-8f5e-9a0b1c2d3e4f"
        }
      ],
      "geometry": [
        {
          "magnitudeValue": 1379.00,
          "magnitudeUnit": "acres",
          "date": "2026-02-12T18:19:00Z",
          "type": "Point",
          "coordinates": [ -95.50083, 30.31722 ]
        }
      ]
    },
    {
      "id": "EONET_17832",
      "title": "Iceberg A83b",
      "description": null,
      "link": "https://eonet.gsfc.nasa.gov/api/v3/events/EONET_17832",
      "closed": "2026-02-08T00:00:00Z",
      "categories": [
        {
          "id": "seaLakeIce",
          "title": "Sea and Lake Ice"
        }
      ],
      "sources": [
        {
          "id": "NATICE",
          "url": "https://usicecenter.gov/pub/Iceberg_Tabular.csv"
        }
      ],
      "geometry": [
        {
          "magnitudeValue": null,
          "magnitudeUnit": null,
          "date": "2026-02-08T00:00:00Z",
          "type": "Point",
          "coordinates": [ -46.54, -74.41 ]
        }
      ]
    }
  ]
}
```

**Structure**:
- **Root object**: Metadata + events array
- **Events array**: Variable length (filtered by query params)
- **Nested arrays**: categories, sources, geometry (1+ items each)

---

## GeoJSON Format

### Events GeoJSON Response

**URL**: `GET /api/v3/events/geojson?status=open&limit=2`

```json
{
  "type": "FeatureCollection",
  "features": [
    {
      "type": "Feature",
      "properties": {
        "id": "EONET_17841",
        "title": "Wildfire - TX, Montgomery County - Westwood Prescribed Fire",
        "description": null,
        "date": "2026-02-13T16:05:00Z",
        "magnitudeValue": 500.00,
        "magnitudeUnit": "acres",
        "categories": [
          {
            "id": "wildfires",
            "title": "Wildfires"
          }
        ],
        "sources": [
          {
            "id": "IRWIN",
            "url": "https://irwin.doi.gov/observer/7f2b72e2-6e8a-4fc3-a71e-b1b1c8c0123a"
          }
        ],
        "closed": null
      },
      "geometry": {
        "type": "Point",
        "coordinates": [ -95.10661, 30.87547 ]
      }
    },
    {
      "type": "Feature",
      "properties": {
        "id": "EONET_17840",
        "title": "Wildfire - TX, Montgomery County - North Conroe Prescribed Fire",
        "description": null,
        "date": "2026-02-12T18:19:00Z",
        "magnitudeValue": 1379.00,
        "magnitudeUnit": "acres",
        "categories": [
          {
            "id": "wildfires",
            "title": "Wildfires"
          }
        ],
        "sources": [
          {
            "id": "IRWIN",
            "url": "https://irwin.doi.gov/observer/0c6e3f8e-d3c5-4a1d-8f5e-9a0b1c2d3e4f"
          }
        ],
        "closed": null
      },
      "geometry": {
        "type": "Point",
        "coordinates": [ -95.50083, 30.31722 ]
      }
    }
  ]
}
```

**Differences from JSON**:
- GeoJSON standard `FeatureCollection` wrapper
- Each event = one `Feature`
- Event metadata in `properties` object
- Geometry promoted to top-level `geometry` field (single geometry per feature)
- **Note**: Events with multiple geometries split into multiple features

---

## Categories Endpoint Response

**URL**: `GET /api/v3/categories`

```json
{
  "title": "EONET Event Categories",
  "description": "List of all Event Categories in the EONET system",
  "link": "https://eonet.gsfc.nasa.gov/api/v3/categories",
  "categories": [
    {
      "id": "drought",
      "title": "Drought",
      "link": "https://eonet.gsfc.nasa.gov/api/v3/categories/drought",
      "description": "Long lasting absence of precipitation affecting agriculture and livestock",
      "layers": "https://eonet.gsfc.nasa.gov/api/v3/layers/drought"
    },
    {
      "id": "dustHaze",
      "title": "Dust and Haze",
      "link": "https://eonet.gsfc.nasa.gov/api/v3/categories/dustHaze",
      "description": "Related to dust storms, air pollution and other non-volcanic aerosols. Volcano-related plumes shall be included with the originating eruption event.",
      "layers": "https://eonet.gsfc.nasa.gov/api/v3/layers/dustHaze"
    },
    {
      "id": "earthquakes",
      "title": "Earthquakes",
      "link": "https://eonet.gsfc.nasa.gov/api/v3/categories/earthquakes",
      "description": "Related to all manner of shaking and displacement. Certain aftermath of earthquakes may also be found under landslides and floods.",
      "layers": "https://eonet.gsfc.nasa.gov/api/v3/layers/earthquakes"
    },
    {
      "id": "floods",
      "title": "Floods",
      "link": "https://eonet.gsfc.nasa.gov/api/v3/categories/floods",
      "description": "Related to aspects of actual flooding--e.g., inundation, water extending beyond river and lake extents.",
      "layers": "https://eonet.gsfc.nasa.gov/api/v3/layers/floods"
    },
    {
      "id": "landslides",
      "title": "Landslides",
      "link": "https://eonet.gsfc.nasa.gov/api/v3/categories/landslides",
      "description": "Related to landslides and variations thereof: mudslides, avalanche.",
      "layers": "https://eonet.gsfc.nasa.gov/api/v3/layers/landslides"
    },
    {
      "id": "manmade",
      "title": "Manmade",
      "link": "https://eonet.gsfc.nasa.gov/api/v3/categories/manmade",
      "description": "Events that have been human-induced and are extreme in their extent.",
      "layers": "https://eonet.gsfc.nasa.gov/api/v3/layers/manmade"
    },
    {
      "id": "seaLakeIce",
      "title": "Sea and Lake Ice",
      "link": "https://eonet.gsfc.nasa.gov/api/v3/categories/seaLakeIce",
      "description": "Related to all ice that resides on oceans and lakes, including sea and lake ice (permanent and seasonal) and icebergs.",
      "layers": "https://eonet.gsfc.nasa.gov/api/v3/layers/seaLakeIce"
    },
    {
      "id": "severeStorms",
      "title": "Severe Storms",
      "link": "https://eonet.gsfc.nasa.gov/api/v3/categories/severeStorms",
      "description": "Related to the atmospheric aspect of storms (hurricanes, cyclones, tornadoes, etc.). Results of storms may be included under floods, landslides, etc.",
      "layers": "https://eonet.gsfc.nasa.gov/api/v3/layers/severeStorms"
    },
    {
      "id": "snow",
      "title": "Snow",
      "link": "https://eonet.gsfc.nasa.gov/api/v3/categories/snow",
      "description": "Related to snow events, particularly extreme/anomalous snowfall in terms of timing or extent.",
      "layers": "https://eonet.gsfc.nasa.gov/api/v3/layers/snow"
    },
    {
      "id": "tempExtremes",
      "title": "Temperature Extremes",
      "link": "https://eonet.gsfc.nasa.gov/api/v3/categories/tempExtremes",
      "description": "Related to anomalous land temperatures, either heat or cold.",
      "layers": "https://eonet.gsfc.nasa.gov/api/v3/layers/tempExtremes"
    },
    {
      "id": "volcanoes",
      "title": "Volcanoes",
      "link": "https://eonet.gsfc.nasa.gov/api/v3/categories/volcanoes",
      "description": "Related to both the physical effects of an eruption (rock, ash, lava) and the atmospheric (ash and gas plumes).",
      "layers": "https://eonet.gsfc.nasa.gov/api/v3/layers/volcanoes"
    },
    {
      "id": "waterColor",
      "title": "Water Color",
      "link": "https://eonet.gsfc.nasa.gov/api/v3/categories/waterColor",
      "description": "Related to events that alter the appearance of water: phytoplankton, algae, sediment, etc.",
      "layers": "https://eonet.gsfc.nasa.gov/api/v3/layers/waterColor"
    },
    {
      "id": "wildfires",
      "title": "Wildfires",
      "link": "https://eonet.gsfc.nasa.gov/api/v3/categories/wildfires",
      "description": "Wildfires includes all nature of fire, including forest and plains fires, as well as urban fires, among others.",
      "layers": "https://eonet.gsfc.nasa.gov/api/v3/layers/wildfires"
    }
  ]
}
```

---

## Sources Endpoint Response

**URL**: `GET /api/v3/sources` (truncated for brevity)

```json
{
  "title": "EONET Event Sources",
  "description": "List of all sources in the EONET system",
  "link": "https://eonet.gsfc.nasa.gov/api/v3/sources",
  "sources": [
    {
      "id": "InciWeb",
      "title": "InciWeb",
      "source": "http://inciweb.nwcg.gov/",
      "link": "https://eonet.gsfc.nasa.gov/api/v3/events?source=InciWeb"
    },
    {
      "id": "CALFIRE",
      "title": "CAL FIRE",
      "source": "http://www.fire.ca.gov/",
      "link": "https://eonet.gsfc.nasa.gov/api/v3/events?source=CALFIRE"
    },
    {
      "id": "SIVolcano",
      "title": "Smithsonian Institution Global Volcanism Program",
      "source": "http://volcano.si.edu/",
      "link": "https://eonet.gsfc.nasa.gov/api/v3/events?source=SIVolcano"
    },
    {
      "id": "USGS_EHP",
      "title": "USGS Earthquake Hazards Program",
      "source": "http://earthquake.usgs.gov/",
      "link": "https://eonet.gsfc.nasa.gov/api/v3/events?source=USGS_EHP"
    },
    {
      "id": "GDACS",
      "title": "Global Disaster Alert and Coordination System",
      "source": "http://www.gdacs.org/",
      "link": "https://eonet.gsfc.nasa.gov/api/v3/events?source=GDACS"
    }
  ]
}
```

**Total sources**: 33 (truncated above for space)

---

## Single Event Response

**URL**: `GET /api/v3/events/EONET_17841`

```json
{
  "id": "EONET_17841",
  "title": "Wildfire - TX, Montgomery County - Westwood Prescribed Fire",
  "description": null,
  "link": "https://eonet.gsfc.nasa.gov/api/v3/events/EONET_17841",
  "closed": null,
  "categories": [
    {
      "id": "wildfires",
      "title": "Wildfires"
    }
  ],
  "sources": [
    {
      "id": "IRWIN",
      "url": "https://irwin.doi.gov/observer/7f2b72e2-6e8a-4fc3-a71e-b1b1c8c0123a"
    }
  ],
  "geometry": [
    {
      "magnitudeValue": 500.00,
      "magnitudeUnit": "acres",
      "date": "2026-02-13T16:05:00Z",
      "type": "Point",
      "coordinates": [ -95.10661, 30.87547 ]
    }
  ]
}
```

**Note**: Returns single event object, not wrapped in array.

---

## Multi-Geometry Event Example

**URL**: Hypothetical severe storm with path

```json
{
  "id": "EONET_99999",
  "title": "Hurricane Example - Atlantic",
  "description": "Category 4 hurricane tracking northwest",
  "link": "https://eonet.gsfc.nasa.gov/api/v3/events/EONET_99999",
  "closed": null,
  "categories": [
    {
      "id": "severeStorms",
      "title": "Severe Storms"
    }
  ],
  "sources": [
    {
      "id": "NOAA_NHC",
      "url": "https://www.nhc.noaa.gov/storm123"
    }
  ],
  "geometry": [
    {
      "magnitudeValue": 130.0,
      "magnitudeUnit": "kts",
      "date": "2026-02-10T00:00:00Z",
      "type": "Point",
      "coordinates": [ -75.5, 22.3 ]
    },
    {
      "magnitudeValue": 135.0,
      "magnitudeUnit": "kts",
      "date": "2026-02-11T00:00:00Z",
      "type": "Point",
      "coordinates": [ -76.8, 24.1 ]
    },
    {
      "magnitudeValue": 120.0,
      "magnitudeUnit": "kts",
      "date": "2026-02-12T00:00:00Z",
      "type": "Point",
      "coordinates": [ -78.2, 26.5 ]
    }
  ]
}
```

**Interpretation**: Geometry array shows storm progression over 3 days with changing wind speeds.

---

## Polygon Geometry Example

**URL**: Hypothetical wildfire perimeter

```json
{
  "id": "EONET_88888",
  "title": "Wildfire - CA, Large Forest Fire",
  "description": "Active wildfire with mapped perimeter",
  "link": "https://eonet.gsfc.nasa.gov/api/v3/events/EONET_88888",
  "closed": null,
  "categories": [
    {
      "id": "wildfires",
      "title": "Wildfires"
    }
  ],
  "sources": [
    {
      "id": "CALFIRE",
      "url": "https://www.fire.ca.gov/incident/fire123"
    }
  ],
  "geometry": [
    {
      "magnitudeValue": 5000.0,
      "magnitudeUnit": "acres",
      "date": "2026-02-15T12:00:00Z",
      "type": "Polygon",
      "coordinates": [
        [
          [ -120.5, 38.7 ],
          [ -120.3, 38.7 ],
          [ -120.2, 38.5 ],
          [ -120.4, 38.4 ],
          [ -120.6, 38.5 ],
          [ -120.5, 38.7 ]
        ]
      ]
    }
  ]
}
```

**Structure**:
- `coordinates[0]`: Exterior ring (closed polygon, first = last point)
- `coordinates[1+]`: Holes (if any)

---

## Error Response Examples

### 404 Not Found

```json
{
  "error": "Event not found"
}
```

### 429 Rate Limit Exceeded

```json
{
  "error": {
    "code": "OVER_RATE_LIMIT",
    "message": "API rate limit exceeded. Please try again in an hour."
  }
}
```

### 400 Bad Request

```json
{
  "error": "Invalid parameter: 'days' must be a positive integer"
}
```

---

## HTTP Headers

### Request Headers

```http
GET /api/v3/events?status=open&days=30 HTTP/1.1
Host: eonet.gsfc.nasa.gov
Accept: application/json
User-Agent: RustConnector/1.0
Accept-Encoding: gzip, deflate
```

### Response Headers

```http
HTTP/1.1 200 OK
Content-Type: application/json; charset=utf-8
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 987
Content-Encoding: gzip
Content-Length: 4523
Date: Sun, 16 Feb 2026 10:30:00 GMT
```

---

## Content-Type Variations

| Endpoint | Content-Type |
|----------|-------------|
| `/events` | `application/json` |
| `/events/geojson` | `application/vnd.geo+json` or `application/json` |
| `/categories` | `application/json` |
| `/sources` | `application/json` |

---

## Field Type Summary

| Field | Type | Nullable | Example |
|-------|------|----------|---------|
| `id` | String | No | `"EONET_17841"` |
| `title` | String | No | `"Wildfire - TX"` |
| `description` | String | **Yes** | `"Event details"` or `null` |
| `link` | String | No | `"https://..."` |
| `closed` | String | **Yes** | `"2026-02-10T..."` or `null` |
| `categories[].id` | String | No | `"wildfires"` |
| `categories[].title` | String | No | `"Wildfires"` |
| `sources[].id` | String | No | `"InciWeb"` |
| `sources[].url` | String | No | `"http://..."` |
| `geometry[].magnitudeValue` | Number | **Yes** | `1500.0` or `null` |
| `geometry[].magnitudeUnit` | String | **Yes** | `"acres"` or `null` |
| `geometry[].date` | String | No | `"2026-02-15T..."` |
| `geometry[].type` | String | No | `"Point"` or `"Polygon"` |
| `geometry[].coordinates` | Array | No | `[-120.5, 38.7]` |

---

## Parsing Notes for Rust

### Serde Structures

```rust
#[derive(Deserialize)]
struct EventsResponse {
    title: String,
    description: String,
    link: String,
    events: Vec<Event>,
}

#[derive(Deserialize)]
struct Event {
    id: String,
    title: String,
    description: Option<String>,
    link: String,
    closed: Option<String>,
    categories: Vec<Category>,
    sources: Vec<Source>,
    geometry: Vec<Geometry>,
}

#[derive(Deserialize)]
struct Geometry {
    #[serde(rename = "magnitudeValue")]
    magnitude_value: Option<f64>,
    #[serde(rename = "magnitudeUnit")]
    magnitude_unit: Option<String>,
    date: String,
    #[serde(rename = "type")]
    geometry_type: String,
    coordinates: serde_json::Value,  // Dynamic: [lon, lat] or [[[...]]]
}
```

### Coordinates Handling

**Point**:
```rust
let coords: [f64; 2] = serde_json::from_value(geometry.coordinates)?;
let (lon, lat) = (coords[0], coords[1]);
```

**Polygon**:
```rust
let coords: Vec<Vec<Vec<f64>>> = serde_json::from_value(geometry.coordinates)?;
let exterior_ring = &coords[0];
```
