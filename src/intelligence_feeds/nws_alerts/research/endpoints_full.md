# NWS Alerts API - Complete Endpoint Reference

## Base URL

```
https://api.weather.gov
```

## Endpoint Inventory

### 1. GET /alerts

**Purpose**: Returns all alerts with extensive filtering capabilities

**Query Parameters**:
- `active` (boolean, deprecated) - List only active alerts
- `start` (ISO8601 timestamp) - Query start time
- `end` (ISO8601 timestamp) - Query end time
- `status` (string) - Filter by alert status
  - Values: `actual`, `exercise`, `system`, `test`, `draft`
- `message_type` (string) - Filter by message type
  - Values: `alert`, `update`, `cancel`, `ack`, `error`
- `event` (string) - Specific event name (e.g., "Tornado Warning")
- `code` (string) - Alert code identifier
- `area` (string) - State/territory code (e.g., "KS", "PR")
- `point` (string) - Geographic point query (lat,lon format: "39.7456,-97.0892")
- `region` (string) - Marine region code
  - Values: `AL` (Alaska), `AT` (Atlantic), `GL` (Great Lakes), `GM` (Gulf of Mexico), `PA` (Pacific), `PI` (Pacific Islands)
- `region_type` (string) - Type of region filter
  - Values: `land`, `marine`
- `zone` (string) - NWS zone ID (e.g., "KSZ027", "FLC015")
- `urgency` (string) - Filter by urgency level
  - Values: `immediate`, `expected`, `future`, `past`, `unknown`
- `severity` (string) - Filter by severity level
  - Values: `extreme`, `severe`, `moderate`, `minor`, `unknown`
- `certainty` (string) - Filter by certainty level
  - Values: `observed`, `likely`, `possible`, `unlikely`, `unknown`
- `limit` (integer) - Maximum number of results
- `cursor` (string) - Pagination cursor for next page

**Response Format**: GeoJSON FeatureCollection

**Example**:
```
GET /alerts?area=TX&severity=severe&status=actual
```

**HTTP Redirects**: Common queries may be redirected (HTTP 301) for performance

---

### 2. GET /alerts/active

**Purpose**: Returns all currently active alerts across all US regions

**Query Parameters**: None (or same filters as `/alerts` endpoint)

**Response Format**: GeoJSON FeatureCollection with active alerts only

**Example**:
```
GET /alerts/active
```

**Typical Use**: Dashboard displaying current nationwide alert status

---

### 3. GET /alerts/active/count

**Purpose**: Returns count information for active alerts

**Query Parameters**: None

**Response Format**: JSON with alert counts

**Example**:
```
GET /alerts/active/count
```

**Use Case**: Quick status check without retrieving full alert data

---

### 4. GET /alerts/active/zone/{zoneId}

**Purpose**: Returns active alerts for a specific NWS forecast zone or county

**Path Parameters**:
- `zoneId` (string, required) - NWS zone identifier
  - Format: 2-letter state + zone type + number
  - Examples: `KSZ027` (Kansas zone 027), `FLC015` (Florida county 015)

**Response Format**: GeoJSON FeatureCollection

**Example**:
```
GET /alerts/active/zone/KSZ027
```

**Zone Types**:
- `Z` prefix: Public forecast zones
- `C` prefix: County/parish zones
- Marine zones: Use marine area codes

---

### 5. GET /alerts/active/area/{area}

**Purpose**: Returns active alerts for a state, territory, or marine area

**Path Parameters**:
- `area` (string, required) - Two-letter state/territory code
  - US States: `AL`, `AK`, `AZ`, ..., `WY`
  - Territories: `PR` (Puerto Rico), `GU` (Guam), `VI` (US Virgin Islands), `AS` (American Samoa), `MP` (Northern Mariana Islands)
  - Marine: `AM` (Atlantic Marine), `AN` (Atlantic North), `AS` (Atlantic South), `GM` (Gulf of Mexico), etc.

**Response Format**: GeoJSON FeatureCollection

**Examples**:
```
GET /alerts/active/area/MO    # Missouri alerts
GET /alerts/active/area/PR    # Puerto Rico alerts
GET /alerts/active/area/GM    # Gulf of Mexico marine alerts
```

**Common Use**: State-level alert displays

---

### 6. GET /alerts/active/region/{region}

**Purpose**: Returns active alerts for a marine region

**Path Parameters**:
- `region` (string, required) - Marine region code
  - `AL` - Alaska
  - `AT` - Atlantic
  - `GL` - Great Lakes
  - `GM` - Gulf of Mexico
  - `PA` - Pacific
  - `PI` - Pacific Islands

**Response Format**: GeoJSON FeatureCollection

**Example**:
```
GET /alerts/active/region/AT
```

**Use Case**: Marine weather applications

---

### 7. GET /alerts/types

**Purpose**: Returns a comprehensive list of all NWS alert event types

**Query Parameters**: None

**Response Format**: JSON array of event type objects

**Example**:
```
GET /alerts/types
```

**Response Contains**:
- Event type names (e.g., "Tornado Warning", "Winter Storm Watch")
- Event codes
- Categories

**Use Case**: Building UI filters, validation lists

---

### 8. GET /alerts/{id}

**Purpose**: Returns a specific alert by its unique identifier

**Path Parameters**:
- `id` (string, required) - Alert URN identifier
  - Format: `urn:oid:2.49.0.1.840.0.[unique-id]`

**Response Format**: GeoJSON Feature (single alert)

**Example**:
```
GET /alerts/urn:oid:2.49.0.1.840.0.ff4e4f3e1b6cf5e78d1c4a5c8b2e9f1a
```

**Use Case**: Direct alert lookup, alert detail pages

---

## Query Parameter Details

### Point Query Format

The `point` parameter accepts latitude,longitude format:
```
point=39.7456,-97.0892
```

**Behavior**: Returns alerts affecting the geographic point (not just alerts with geometry containing the point)

**Latitude Range**: -90 to 90
**Longitude Range**: -180 to 180

### Time Query Format

`start` and `end` parameters use ISO 8601 format:
```
start=2026-02-15T00:00:00Z
end=2026-02-16T23:59:59Z
```

**Timezone**: UTC or with timezone offset
**Default**: Returns alerts from past 7 days if no time specified

### Pagination

When results exceed the limit, response includes pagination cursor:
```json
{
  "pagination": {
    "next": "https://api.weather.gov/alerts?cursor=eyJzb3J0IjpbMTY..."
  }
}
```

Use `cursor` parameter to retrieve next page:
```
GET /alerts?cursor=eyJzb3J0IjpbMTY...
```

---

## Content Negotiation

All endpoints support multiple response formats via `Accept` header:

| Format | Accept Header | Use Case |
|--------|---------------|----------|
| GeoJSON (default) | `application/geo+json` | Modern web apps |
| JSON-LD | `application/ld+json` | Linked data apps |
| CAP XML | `application/cap+xml` | Legacy systems |
| ATOM | `application/atom+xml` | RSS-style feeds |

**Example**:
```
GET /alerts/active
Accept: application/cap+xml
```

---

## Response Codes

| Code | Meaning | Action |
|------|---------|--------|
| 200 | Success | Process response |
| 301 | Redirect | Follow redirect URL |
| 400 | Bad Request | Check query parameters |
| 404 | Not Found | Alert/zone doesn't exist |
| 429 | Rate Limited | Retry after 5 seconds |
| 500 | Server Error | Report to nco.ops@noaa.gov |
| 503 | Service Unavailable | Retry later |

---

## Performance Notes

- **Caching**: Responses include `Expires` headers based on alert lifecycle
- **CDN**: Heavily cached for common queries
- **Redirects**: Popular queries redirected to optimized endpoints
- **Pagination**: Use `limit` parameter to control response size
- **Filtering**: Server-side filtering more efficient than client-side

---

## Geographic Identifiers

### SAME Codes
- 6-digit county codes used by NOAA Weather Radio
- Format: State FIPS (2 digits) + County FIPS (3 digits) + subcounty (1 digit)
- Example: `048029` (Texas, Bexar County)

### UGC Codes
- Universal Geographic Code used by NWS
- Format: State (2 letters) + zone type (1 letter) + zone number (3 digits)
- Examples: `TXZ253`, `FLC001`
- Types: `Z` (zone), `C` (county)

### Zone IDs
- API uses zone IDs in path parameters
- Same format as UGC codes
- Lookup via `/zones` endpoint (separate from alerts API)
