# FAA NASSTATUS API - Endpoints

## Base URL
```
https://nasstatus.faa.gov/api
```

## Endpoints

### 1. Airport Status Information (All Airports)
**Endpoint**: `/airport-status-information`
**Method**: GET
**Authentication**: None
**Description**: Returns comprehensive airport status information for all US airports with active delays, closures, or restrictions.

**Request Headers**:
```
Accept: application/xml
```

**Query Parameters**: None (returns all airports with active events)

**Response**: XML document with nested airport status information

**Example**:
```
GET https://nasstatus.faa.gov/api/airport-status-information
Accept: application/xml
```

---

### 2. Airport Events
**Endpoint**: `/airport-events`
**Method**: GET
**Authentication**: None
**Description**: Returns active airport events in the National Airspace System.

**Request Headers**:
```
Accept: application/xml
```

**Query Parameters**: None documented

**Response**: XML or JSON (based on Accept header)

**Example**:
```
GET https://nasstatus.faa.gov/api/airport-events
Accept: application/xml
```

---

### 3. Individual Airport Status (Legacy ASWS - DEPRECATED)
**Endpoint**: `/airport/status/{airportCode}`
**Base URL**: `https://soa.smext.faa.gov/asws/api`
**Method**: GET
**Authentication**: None
**Status**: Connection refused as of Feb 2026 (likely deprecated)

**Path Parameters**:
- `airportCode` (required): 3-letter IATA code (e.g., ATL, SFO, ORD)

**Request Headers**:
```
Accept: application/json  # For JSON response
Accept: application/xml   # For XML response (default)
```

**Response Fields** (documented but endpoint offline):
- `City`: Airport city
- `Delay`: Boolean or delay details
- `DelayCount`: Number of delays
- `IATA`: 3-letter code
- `ICAO`: 4-letter code
- `Name`: Full airport name
- `State`: Two-letter state code
- `Status`: Current status
- `SupportedAirport`: Boolean
- `Weather`: Object with metadata, temperature, visibility, wind

**Example**:
```
GET https://soa.smext.faa.gov/asws/api/airport/status/ATL
Accept: application/json
```

**Note**: This endpoint is no longer accessible. Use the `/airport-status-information` endpoint instead.

---

## Query Capabilities

### Individual Airport Query
The primary endpoint (`/airport-status-information`) does NOT support filtering by airport code in the URL. To query a specific airport:
1. Fetch the full XML response
2. Parse and filter client-side by ARPT field

### Historical Data
**Not available**. The API provides only current/real-time status. No historical delay data endpoints discovered.

### Time Range Queries
**Not supported**. Only current snapshot available.

### Filtering Options
**None documented**. The endpoint returns all airports with active events. Filtering must be done client-side.

---

## Rate Limiting

**Official limits**: Not documented
**Recommended practice**:
- Cache responses for 60 seconds
- Implement stale-while-revalidate (30s)
- Avoid polling more frequently than every 30-60 seconds
- Respect HTTP cache headers if present

**Expected behavior**: The FAA may throttle excessive requests, but no documented limits exist.

---

## Response Codes

| Code | Meaning | Description |
|------|---------|-------------|
| 200 | Success | Valid XML/JSON response |
| 400 | Bad Request | Invalid parameters |
| 404 | Not Found | Invalid endpoint |
| 500 | Server Error | FAA system error |
| 503 | Service Unavailable | Temporary outage |

---

## Error Handling

No documented error response schema. Errors likely returned as HTTP status codes with minimal body content.

**Best practice**:
- Check HTTP status code
- Log full response body on non-200 status
- Implement retry with exponential backoff for 5xx errors
- Treat 404 as permanent failure for deprecated endpoints

---

## Additional Notes

1. **No bulk download**: No endpoint for downloading all historical data
2. **No WebSocket**: Real-time updates require polling
3. **No pagination**: Single response contains all current events
4. **No versioning in URL**: API version not specified in endpoint path
5. **XML is primary format**: JSON support varies by endpoint
6. **CORS**: Unknown, likely requires server-side proxy for browser clients
