# NWS Alerts API - Data Types Reference

## Core Data Types

### 1. AlertCollection (GeoJSON FeatureCollection)

Top-level response object for alert queries.

**Structure**:
```json
{
  "@context": [
    "https://geojson.org/geojson-ld/geojson-context.jsonld",
    {
      "wx": "https://api.weather.gov/ontology#",
      "@vocab": "https://api.weather.gov/ontology#"
    }
  ],
  "type": "FeatureCollection",
  "features": [Alert, Alert, ...],
  "title": "Current watches, warnings, and advisories",
  "updated": "2026-02-16T12:00:00+00:00",
  "pagination": {
    "next": "https://api.weather.gov/alerts?cursor=..."
  }
}
```

**Fields**:
- `@context` (array) - JSON-LD context for semantic web
- `type` (string) - Always "FeatureCollection"
- `features` (array of Alert) - Alert objects
- `title` (string) - Human-readable collection title
- `updated` (ISO8601 timestamp) - Last update time
- `pagination` (object, optional) - Pagination info

---

### 2. Alert (GeoJSON Feature)

Individual weather alert object.

**Structure**:
```json
{
  "id": "https://api.weather.gov/alerts/urn:oid:2.49.0.1.840.0.abc123...",
  "type": "Feature",
  "geometry": null | GeoJSON Geometry,
  "properties": AlertProperties
}
```

**Fields**:
- `id` (string) - Full URL to alert resource
- `type` (string) - Always "Feature"
- `geometry` (GeoJSON Geometry | null) - Geographic shape (often null for NWS alerts)
- `properties` (AlertProperties) - Alert details

---

### 3. AlertProperties

Core alert information following CAP 1.2 specification.

**Structure**:
```json
{
  "@id": "https://api.weather.gov/alerts/urn:oid:...",
  "@type": "wx:Alert",
  "id": "urn:oid:2.49.0.1.840.0.abc123...",
  "areaDesc": "Bexar; Guadalupe; Comal",
  "geocode": {
    "SAME": ["048029", "048187", "048091"],
    "UGC": ["TXZ253", "TXZ254"]
  },
  "affectedZones": [
    "https://api.weather.gov/zones/forecast/TXZ253",
    "https://api.weather.gov/zones/forecast/TXZ254"
  ],
  "references": [
    {
      "identifier": "urn:oid:...",
      "sender": "w-nws.webmaster@noaa.gov",
      "sent": "2026-02-16T00:00:00-06:00"
    }
  ],
  "sent": "2026-02-16T02:01:00-06:00",
  "effective": "2026-02-16T02:01:00-06:00",
  "onset": "2026-02-16T23:00:00-06:00",
  "expires": "2026-02-17T03:15:00-06:00",
  "ends": "2026-02-17T17:00:00-06:00",
  "status": "Actual",
  "messageType": "Update",
  "category": "Met",
  "severity": "Moderate",
  "certainty": "Likely",
  "urgency": "Expected",
  "event": "Winter Weather Advisory",
  "sender": "w-nws.webmaster@noaa.gov",
  "senderName": "NWS Austin/San Antonio TX",
  "headline": "Winter Weather Advisory issued February 16 at 2:01AM CST until February 17 at 5:00PM CST by NWS Austin/San Antonio TX",
  "description": "Detailed alert description...",
  "instruction": "Action guidance for public...",
  "response": "Prepare",
  "parameters": {
    "AWIPSidentifier": ["WSWTX"],
    "WMOidentifier": ["WWTX"],
    "NWSheadline": ["WINTER WEATHER ADVISORY IN EFFECT..."],
    "BLOCKCHANNEL": ["EAS", "NWEM", "CMAS"],
    "VTEC": ["/O.NEW.KEWX.WW.Y.0003.260216T2300Z-260217T2300Z/"],
    "eventEndingTime": ["2026-02-17T23:00:00+00:00"]
  }
}
```

**Field Descriptions**:

#### Identifiers
- `@id` (string) - Semantic web identifier (full URL)
- `@type` (string) - Semantic type ("wx:Alert")
- `id` (string) - Alert URN (Uniform Resource Name)

#### Geographic
- `areaDesc` (string) - Human-readable affected area description
- `geocode` (Geocode) - Machine-readable location codes
- `affectedZones` (array of strings) - API URLs for affected forecast zones

#### Temporal
- `sent` (ISO8601) - When alert was sent by NWS
- `effective` (ISO8601) - When alert becomes effective
- `onset` (ISO8601, optional) - Expected onset of event
- `expires` (ISO8601) - When alert expires (no longer displayed)
- `ends` (ISO8601, optional) - Expected end of event

#### Classification
- `status` (Status) - Alert status (Actual, Exercise, System, Test, Draft)
- `messageType` (MessageType) - Type of message (Alert, Update, Cancel, Ack, Error)
- `category` (Category) - Alert category (Met, Geo, Safety, etc.)
- `severity` (Severity) - Impact severity (Extreme, Severe, Moderate, Minor, Unknown)
- `certainty` (Certainty) - Confidence level (Observed, Likely, Possible, Unlikely, Unknown)
- `urgency` (Urgency) - Time to respond (Immediate, Expected, Future, Past, Unknown)
- `event` (string) - Event type (e.g., "Tornado Warning")

#### Content
- `sender` (string) - Alert sender email
- `senderName` (string) - NWS office name
- `headline` (string) - Brief alert headline
- `description` (string) - Detailed alert description
- `instruction` (string, optional) - Recommended actions

#### Metadata
- `response` (ResponseType) - Recommended response (Shelter, Evacuate, Prepare, etc.)
- `references` (array of Reference) - Prior related alerts
- `parameters` (Parameters) - Additional NWS-specific metadata

---

### 4. Geocode

Geographic location codes for machine processing.

**Structure**:
```json
{
  "SAME": ["048029", "048187"],
  "UGC": ["TXZ253", "TXZ254", "TXC253"]
}
```

**Fields**:
- `SAME` (array of strings) - SAME (Specific Area Message Encoding) codes
  - 6-digit county codes for NOAA Weather Radio
  - Format: State FIPS (2) + County FIPS (3) + Subcounty (1)
  - Example: "048029" = Texas (48) + Bexar County (029) + whole county (0)
- `UGC` (array of strings) - Universal Geographic Code
  - NWS zone identifiers
  - Format: State (2 letters) + Type (Z/C) + Number (3 digits)
  - Examples: "TXZ253" (Texas zone 253), "TXC029" (Texas county 029)

---

### 5. Reference

Reference to a previous related alert (for updates/cancellations).

**Structure**:
```json
{
  "@id": "https://api.weather.gov/alerts/urn:oid:...",
  "identifier": "urn:oid:2.49.0.1.840.0.xyz789...",
  "sender": "w-nws.webmaster@noaa.gov",
  "sent": "2026-02-15T18:00:00-06:00"
}
```

**Fields**:
- `@id` (string) - Full URL to referenced alert
- `identifier` (string) - URN of referenced alert
- `sender` (string) - Sender of referenced alert
- `sent` (ISO8601) - When referenced alert was sent

**Usage**: Links Update/Cancel messages to original alerts

---

### 6. Parameters

Additional NWS-specific metadata fields.

**Structure**:
```json
{
  "AWIPSidentifier": ["WSWTX"],
  "WMOidentifier": ["WWTX"],
  "NWSheadline": ["WINTER WEATHER ADVISORY IN EFFECT..."],
  "BLOCKCHANNEL": ["EAS", "NWEM", "CMAS"],
  "VTEC": ["/O.NEW.KEWX.WW.Y.0003.260216T2300Z-260217T2300Z/"],
  "EAS-ORG": ["WXR"],
  "PILID": ["WSWTX"],
  "eventEndingTime": ["2026-02-17T23:00:00+00:00"],
  "expiredReferences": ["..."],
  "maxWindGust": ["50 MPH"],
  "maxHailSize": ["1.00"],
  "waterspoutDetection": ["POSSIBLE"],
  "flashFloodDetection": ["OBSERVED"]
}
```

**Common Fields**:
- `AWIPSidentifier` (array) - Automated Weather Information Processing System ID
- `WMOidentifier` (array) - World Meteorological Organization ID
- `NWSheadline` (array) - NWS-formatted headline
- `BLOCKCHANNEL` (array) - Distribution channels (EAS, NWEM, CMAS)
- `VTEC` (array) - Valid Time Event Code (P-VTEC)
  - Format: `/Action.Office.Phenomena.Significance.Number.StartTime-EndTime/`
  - Example: `/O.NEW.KEWX.WW.Y.0003.260216T2300Z-260217T2300Z/`
- `PILID` (array) - Product Identifier
- `eventEndingTime` (array) - ISO8601 timestamp for event end
- `expiredReferences` (array) - Expired alerts superseded by this one

**Event-Specific Parameters**:
- `maxWindGust` (array) - Max wind speed
- `maxHailSize` (array) - Max hail diameter in inches
- `waterspoutDetection` (array) - Waterspout status
- `flashFloodDetection` (array) - Flash flood status
- `tornadoDetection` (array) - Tornado detection status

---

### 7. Pagination

Pagination information for paginated results.

**Structure**:
```json
{
  "next": "https://api.weather.gov/alerts?cursor=eyJzb3J0IjpbMTY..."
}
```

**Fields**:
- `next` (string, optional) - URL for next page of results

**Usage**: Present when more results available; absent on last page

---

## Enumerated Types

### Status

Alert status values (CAP 1.2 standard):

| Value | Description |
|-------|-------------|
| `Actual` | Actionable by all targeted recipients |
| `Exercise` | Actionable only by designated exercise participants |
| `System` | Messages supporting alert network internal functions |
| `Test` | Technical testing only; all recipients disregard |
| `Draft` | Preliminary template or draft, not actionable |

**Most Common**: `Actual` (live operational alerts)

---

### MessageType

Type of alert message:

| Value | Description |
|-------|-------------|
| `Alert` | Initial information requiring attention |
| `Update` | Updates and supersedes earlier message |
| `Cancel` | Cancels earlier message |
| `Ack` | Acknowledges receipt and acceptance |
| `Error` | Indicates rejection of message |

**Usage**:
- `Alert` - New event
- `Update` - Event details changed, check `references` field
- `Cancel` - Event cancelled, check `references` for cancelled alert

---

### Category

Hazard category:

| Value | Description |
|-------|-------------|
| `Met` | Meteorological (weather) |
| `Geo` | Geophysical (earthquake, tsunami) |
| `Safety` | General emergency and public safety |
| `Security` | Law enforcement, military, homeland security |
| `Rescue` | Rescue and recovery |
| `Fire` | Fire suppression and rescue |
| `Health` | Medical and public health |
| `Env` | Pollution and environmental |
| `Transport` | Transportation |
| `Infra` | Utility, telecommunication, infrastructure |
| `CBRNE` | Chemical, Biological, Radiological, Nuclear, Explosive |
| `Other` | Other events |

**NWS Alerts**: Almost always `Met` (meteorological)

---

### Severity

Impact severity:

| Value | Description |
|-------|-------------|
| `Extreme` | Extraordinary threat to life or property |
| `Severe` | Significant threat to life or property |
| `Moderate` | Possible threat to life or property |
| `Minor` | Minimal to no known threat to life or property |
| `Unknown` | Severity unknown |

**Examples**:
- `Extreme` - Tornado Emergency, Extreme Wind Warning
- `Severe` - Tornado Warning, Flash Flood Warning
- `Moderate` - Severe Thunderstorm Warning
- `Minor` - Freeze Warning, Wind Advisory

---

### Urgency

Time available to prepare:

| Value | Description |
|-------|-------------|
| `Immediate` | Responsive action should be taken immediately |
| `Expected` | Responsive action should be taken soon (within next hour) |
| `Future` | Responsive action should be taken in near future |
| `Past` | Responsive action no longer required |
| `Unknown` | Urgency not known |

**Examples**:
- `Immediate` - Tornado Warning (act now)
- `Expected` - Flash Flood Watch (prepare within hour)
- `Future` - Winter Storm Watch (prepare over next 24-48 hours)

---

### Certainty

Confidence in observation or prediction:

| Value | Description | Probability |
|-------|-------------|-------------|
| `Observed` | Determined to have occurred or ongoing | 100% |
| `Likely` | Likely to occur | > 50% |
| `Possible` | Possible but not likely | ≤ 50% |
| `Unlikely` | Not expected to occur | ~ 0% |
| `Unknown` | Certainty unknown | N/A |

**Examples**:
- `Observed` - Warning based on radar-confirmed tornado
- `Likely` - Warning based on strong meteorological indicators
- `Possible` - Watch condition (favorable environment)

---

### ResponseType

Recommended response action:

| Value | Description |
|-------|-------------|
| `Shelter` | Take shelter in place or per instruction |
| `Evacuate` | Relocate as instructed |
| `Prepare` | Make preparations |
| `Execute` | Execute pre-planned activity |
| `Avoid` | Avoid subject event |
| `Monitor` | Attend to information sources |
| `Assess` | Evaluate situation |
| `AllClear` | Hazard has passed |
| `None` | No action recommended |

**Usage**: Drives automated response systems, UI action buttons

---

## Additional Types

### 8. AlertType

Alert event type definition (from `/alerts/types` endpoint).

**Structure**:
```json
{
  "eventType": "Tornado Warning",
  "eventTypeCode": "TOR",
  "category": "Met"
}
```

**Fields**:
- `eventType` (string) - Full event name
- `eventTypeCode` (string) - Short code
- `category` (Category) - Hazard category

---

### 9. AlertCount

Active alert count information (from `/alerts/active/count` endpoint).

**Structure**:
```json
{
  "total": 147,
  "land": 142,
  "marine": 5,
  "regions": {
    "AL": 5,
    "AT": 12,
    "GM": 3,
    "GL": 8,
    "PA": 15,
    "PI": 0
  },
  "areas": {
    "TX": 18,
    "CA": 12,
    "FL": 9,
    ...
  }
}
```

**Fields**:
- `total` (integer) - Total active alerts
- `land` (integer) - Land-based alerts
- `marine` (integer) - Marine alerts
- `regions` (map) - Count by marine region
- `areas` (map) - Count by state/territory

---

## Geometry Types (GeoJSON)

Alerts may include geographic shapes (though often null for NWS alerts).

### Polygon

**Structure**:
```json
{
  "type": "Polygon",
  "coordinates": [
    [
      [-98.5, 29.4],
      [-98.4, 29.4],
      [-98.4, 29.5],
      [-98.5, 29.5],
      [-98.5, 29.4]
    ]
  ]
}
```

### MultiPolygon

**Structure**:
```json
{
  "type": "MultiPolygon",
  "coordinates": [
    [
      [[-98.5, 29.4], [-98.4, 29.4], ...]
    ],
    [
      [[-98.7, 29.6], [-98.6, 29.6], ...]
    ]
  ]
}
```

**Note**: Most NWS alerts have `geometry: null` and rely on `geocode` fields for location.

---

## Rust Type Mapping

Suggested Rust structs for implementation:

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize)]
pub struct AlertCollection {
    #[serde(rename = "@context")]
    pub context: serde_json::Value,
    #[serde(rename = "type")]
    pub type_: String,
    pub features: Vec<Alert>,
    pub title: Option<String>,
    pub updated: Option<DateTime<Utc>>,
    pub pagination: Option<Pagination>,
}

#[derive(Debug, Deserialize)]
pub struct Alert {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub geometry: Option<serde_json::Value>,  // GeoJSON geometry or null
    pub properties: AlertProperties,
}

#[derive(Debug, Deserialize)]
pub struct AlertProperties {
    pub id: String,
    #[serde(rename = "areaDesc")]
    pub area_desc: String,
    pub geocode: Geocode,
    #[serde(rename = "affectedZones")]
    pub affected_zones: Vec<String>,
    pub references: Option<Vec<Reference>>,
    pub sent: DateTime<Utc>,
    pub effective: DateTime<Utc>,
    pub onset: Option<DateTime<Utc>>,
    pub expires: DateTime<Utc>,
    pub ends: Option<DateTime<Utc>>,
    pub status: Status,
    #[serde(rename = "messageType")]
    pub message_type: MessageType,
    pub category: Category,
    pub severity: Severity,
    pub certainty: Certainty,
    pub urgency: Urgency,
    pub event: String,
    #[serde(rename = "senderName")]
    pub sender_name: String,
    pub headline: Option<String>,
    pub description: String,
    pub instruction: Option<String>,
    pub response: ResponseType,
    pub parameters: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct Geocode {
    #[serde(rename = "SAME")]
    pub same: Vec<String>,
    #[serde(rename = "UGC")]
    pub ugc: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum Severity {
    Extreme,
    Severe,
    Moderate,
    Minor,
    Unknown,
}

// Additional enums for Status, MessageType, Category, etc.
```

---

## JSON-LD Context

The `@context` field provides semantic web metadata:

```json
"@context": [
  "https://geojson.org/geojson-ld/geojson-context.jsonld",
  {
    "wx": "https://api.weather.gov/ontology#",
    "@vocab": "https://api.weather.gov/ontology#"
  }
]
```

**Purpose**: Enables linked data processing and semantic queries

**Usage for Most Developers**: Can be ignored; standard JSON parsing sufficient
