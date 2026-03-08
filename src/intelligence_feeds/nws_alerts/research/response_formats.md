# NWS Alerts API - Response Formats & Examples

## Response Format Overview

The NWS API supports multiple response formats via content negotiation:

| Format | Accept Header | Default | Use Case |
|--------|---------------|---------|----------|
| GeoJSON | `application/geo+json` | Yes | Modern web apps |
| JSON-LD | `application/ld+json` | No | Linked data/semantic web |
| CAP XML | `application/cap+xml` | No | Legacy systems, CAP parsers |
| ATOM | `application/atom+xml` | No | RSS feed readers |

This document focuses on **GeoJSON format** (default and most common).

---

## Example 1: GET /alerts/active

Full response with multiple active alerts.

### Request
```http
GET /alerts/active HTTP/1.1
Host: api.weather.gov
User-Agent: (MyApp/1.0, contact@example.com)
Accept: application/geo+json
```

### Response
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
  "features": [
    {
      "id": "https://api.weather.gov/alerts/urn:oid:2.49.0.1.840.0.1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c",
      "type": "Feature",
      "geometry": null,
      "properties": {
        "@id": "https://api.weather.gov/alerts/urn:oid:2.49.0.1.840.0.1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c",
        "@type": "wx:Alert",
        "id": "urn:oid:2.49.0.1.840.0.1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c",
        "areaDesc": "Bexar; Guadalupe; Comal",
        "geocode": {
          "SAME": ["048029", "048187", "048091"],
          "UGC": ["TXZ253", "TXZ254", "TXZ255"]
        },
        "affectedZones": [
          "https://api.weather.gov/zones/forecast/TXZ253",
          "https://api.weather.gov/zones/forecast/TXZ254",
          "https://api.weather.gov/zones/forecast/TXZ255"
        ],
        "references": [],
        "sent": "2026-02-16T02:01:00-06:00",
        "effective": "2026-02-16T02:01:00-06:00",
        "onset": "2026-02-16T23:00:00-06:00",
        "expires": "2026-02-17T03:15:00-06:00",
        "ends": "2026-02-17T17:00:00-06:00",
        "status": "Actual",
        "messageType": "Alert",
        "category": "Met",
        "severity": "Moderate",
        "certainty": "Likely",
        "urgency": "Expected",
        "event": "Winter Weather Advisory",
        "sender": "w-nws.webmaster@noaa.gov",
        "senderName": "NWS Austin/San Antonio TX",
        "headline": "Winter Weather Advisory issued February 16 at 2:01AM CST until February 17 at 5:00PM CST by NWS Austin/San Antonio TX",
        "description": "* WHAT...Snow expected. Total snow accumulations of 1 to 3 inches.\n\n* WHERE...Portions of south central Texas.\n\n* WHEN...From 5 PM this afternoon to 5 PM CST Monday.\n\n* IMPACTS...Plan on slippery road conditions. The hazardous conditions could impact the morning commute.",
        "instruction": "Slow down and use caution while traveling.\n\nThe latest road conditions for the state you are calling from can be obtained by calling 5 1 1.",
        "response": "Prepare",
        "parameters": {
          "AWIPSidentifier": ["WSWTX"],
          "WMOidentifier": ["WWTX"],
          "NWSheadline": ["WINTER WEATHER ADVISORY IN EFFECT FROM 5 PM THIS AFTERNOON TO 5 PM CST MONDAY"],
          "BLOCKCHANNEL": ["EAS", "NWEM"],
          "VTEC": ["/O.NEW.KEWX.WW.Y.0003.260216T2300Z-260217T2300Z/"],
          "eventEndingTime": ["2026-02-17T23:00:00+00:00"]
        }
      }
    },
    {
      "id": "https://api.weather.gov/alerts/urn:oid:2.49.0.1.840.0.9a8b7c6d5e4f3a2b1c0d9e8f7a6b5c4d",
      "type": "Feature",
      "geometry": null,
      "properties": {
        "@id": "https://api.weather.gov/alerts/urn:oid:2.49.0.1.840.0.9a8b7c6d5e4f3a2b1c0d9e8f7a6b5c4d",
        "@type": "wx:Alert",
        "id": "urn:oid:2.49.0.1.840.0.9a8b7c6d5e4f3a2b1c0d9e8f7a6b5c4d",
        "areaDesc": "Oklahoma County; Cleveland County",
        "geocode": {
          "SAME": ["040109", "040027"],
          "UGC": ["OKZ025", "OKZ026"]
        },
        "affectedZones": [
          "https://api.weather.gov/zones/forecast/OKZ025",
          "https://api.weather.gov/zones/forecast/OKZ026"
        ],
        "references": [],
        "sent": "2026-02-16T03:45:00-06:00",
        "effective": "2026-02-16T03:45:00-06:00",
        "onset": "2026-02-16T03:45:00-06:00",
        "expires": "2026-02-16T04:15:00-06:00",
        "ends": "2026-02-16T04:15:00-06:00",
        "status": "Actual",
        "messageType": "Alert",
        "category": "Met",
        "severity": "Severe",
        "certainty": "Observed",
        "urgency": "Immediate",
        "event": "Severe Thunderstorm Warning",
        "sender": "w-nws.webmaster@noaa.gov",
        "senderName": "NWS Norman OK",
        "headline": "Severe Thunderstorm Warning issued February 16 at 3:45AM CST until February 16 at 4:15AM CST by NWS Norman OK",
        "description": "At 344 AM CST, a severe thunderstorm was located near Moore, moving northeast at 40 mph.\n\nHAZARD...60 mph wind gusts and quarter size hail.\n\nSOURCE...Radar indicated.\n\nIMPACT...Hail damage to vehicles is expected. Expect wind damage to roofs, siding, and trees.\n\nLocations impacted include...\nOklahoma City, Norman, Moore, and Midwest City.",
        "instruction": "For your protection move to an interior room on the lowest floor of a building.",
        "response": "Shelter",
        "parameters": {
          "AWIPSidentifier": ["SVROK"],
          "WMOidentifier": ["WWOK"],
          "NWSheadline": ["SEVERE THUNDERSTORM WARNING REMAINS IN EFFECT UNTIL 415 AM CST FOR CENTRAL OKLAHOMA AND CLEVELAND COUNTIES"],
          "BLOCKCHANNEL": ["EAS", "NWEM", "CMAS"],
          "VTEC": ["/O.CON.KOUN.SV.W.0012.000000T0000Z-260216T1015Z/"],
          "eventEndingTime": ["2026-02-16T10:15:00+00:00"],
          "maxWindGust": ["60 MPH"],
          "maxHailSize": ["1.00"]
        }
      }
    }
  ],
  "title": "Current watches, warnings, and advisories",
  "updated": "2026-02-16T09:45:00+00:00"
}
```

### Key Observations
- Top-level is a GeoJSON `FeatureCollection`
- Each alert is a `Feature` with `properties` containing CAP data
- `geometry` is typically `null` for NWS alerts
- Geographic info in `geocode` (SAME/UGC codes) and `affectedZones`
- Temporal fields use ISO 8601 with timezone offsets
- `references` array empty for new alerts (populated for updates)

---

## Example 2: GET /alerts/active/area/KS

Filtered alerts for Kansas only.

### Request
```http
GET /alerts/active/area/KS HTTP/1.1
Host: api.weather.gov
User-Agent: (KansasWeather/1.0, dev@ksweather.com)
Accept: application/geo+json
```

### Response
```json
{
  "@context": [...],
  "type": "FeatureCollection",
  "features": [
    {
      "id": "https://api.weather.gov/alerts/urn:oid:2.49.0.1.840.0.3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f",
      "type": "Feature",
      "geometry": null,
      "properties": {
        "id": "urn:oid:2.49.0.1.840.0.3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f",
        "areaDesc": "Johnson; Wyandotte; Douglas",
        "geocode": {
          "SAME": ["020091", "020209", "020045"],
          "UGC": ["KSZ104", "KSZ105", "KSZ106"]
        },
        "affectedZones": [
          "https://api.weather.gov/zones/forecast/KSZ104",
          "https://api.weather.gov/zones/forecast/KSZ105",
          "https://api.weather.gov/zones/forecast/KSZ106"
        ],
        "references": [],
        "sent": "2026-02-16T08:00:00-06:00",
        "effective": "2026-02-16T08:00:00-06:00",
        "onset": "2026-02-16T18:00:00-06:00",
        "expires": "2026-02-16T18:00:00-06:00",
        "ends": null,
        "status": "Actual",
        "messageType": "Alert",
        "category": "Met",
        "severity": "Minor",
        "certainty": "Likely",
        "urgency": "Expected",
        "event": "Wind Advisory",
        "sender": "w-nws.webmaster@noaa.gov",
        "senderName": "NWS Kansas City/Pleasant Hill MO",
        "headline": "Wind Advisory issued February 16 at 8:00AM CST until February 16 at 6:00PM CST by NWS Kansas City/Pleasant Hill MO",
        "description": "* WHAT...West winds 20 to 25 mph with gusts up to 40 mph expected.\n\n* WHERE...Portions of northeast and east central Kansas.\n\n* WHEN...From 10 AM this morning to 6 PM CST this evening.\n\n* IMPACTS...Gusty winds could blow around unsecured objects. Tree limbs could be blown down and a few power outages may result.",
        "instruction": "Use extra caution when driving, especially if operating a high profile vehicle. Secure outdoor objects.",
        "response": "Execute",
        "parameters": {
          "AWIPSidentifier": ["NPWKS"],
          "WMOidentifier": ["WWKS"],
          "NWSheadline": ["WIND ADVISORY IN EFFECT FROM 10 AM THIS MORNING TO 6 PM CST THIS EVENING"],
          "BLOCKCHANNEL": ["EAS", "NWEM"],
          "VTEC": ["/O.NEW.KEAX.WI.Y.0003.260216T1600Z-260217T0000Z/"],
          "eventEndingTime": ["2026-02-17T00:00:00+00:00"]
        }
      }
    }
  ],
  "title": "Current watches, warnings, and advisories for Kansas",
  "updated": "2026-02-16T14:00:00+00:00"
}
```

---

## Example 3: Alert with Update (messageType: "Update")

Updated alert referencing previous version.

### Response Excerpt
```json
{
  "id": "https://api.weather.gov/alerts/urn:oid:2.49.0.1.840.0.7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b",
  "type": "Feature",
  "geometry": null,
  "properties": {
    "id": "urn:oid:2.49.0.1.840.0.7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b",
    "areaDesc": "Harris",
    "geocode": {
      "SAME": ["048201"],
      "UGC": ["TXZ213"]
    },
    "affectedZones": ["https://api.weather.gov/zones/forecast/TXZ213"],
    "references": [
      {
        "@id": "https://api.weather.gov/alerts/urn:oid:2.49.0.1.840.0.5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c",
        "identifier": "urn:oid:2.49.0.1.840.0.5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c",
        "sender": "w-nws.webmaster@noaa.gov",
        "sent": "2026-02-16T06:00:00-06:00"
      }
    ],
    "sent": "2026-02-16T09:30:00-06:00",
    "effective": "2026-02-16T09:30:00-06:00",
    "onset": "2026-02-16T12:00:00-06:00",
    "expires": "2026-02-16T18:00:00-06:00",
    "ends": "2026-02-16T21:00:00-06:00",
    "status": "Actual",
    "messageType": "Update",
    "category": "Met",
    "severity": "Severe",
    "certainty": "Likely",
    "urgency": "Expected",
    "event": "Flash Flood Watch",
    "sender": "w-nws.webmaster@noaa.gov",
    "senderName": "NWS Houston/Galveston TX",
    "headline": "Flash Flood Watch issued February 16 at 9:30AM CST until February 16 at 9:00PM CST by NWS Houston/Galveston TX",
    "description": "...FLASH FLOOD WATCH NOW IN EFFECT THROUGH THIS EVENING...\n\nThe Flash Flood Watch is now in effect for Harris County through 9 PM this evening.\n\nPrevious rainfall amounts have been updated. Expect an additional 1 to 2 inches of rain through the evening.",
    "instruction": "You should monitor later forecasts and be prepared to take action should Flash Flood Warnings be issued.",
    "response": "Prepare",
    "parameters": {
      "AWIPSidentifier": ["FFATX"],
      "WMOidentifier": ["WWTX"],
      "NWSheadline": ["FLASH FLOOD WATCH NOW IN EFFECT THROUGH THIS EVENING"],
      "BLOCKCHANNEL": ["EAS", "NWEM"],
      "VTEC": ["/O.EXT.KHGX.FA.A.0002.260216T1800Z-260217T0300Z/"],
      "eventEndingTime": ["2026-02-17T03:00:00+00:00"]
    }
  }
}
```

### Key Differences for Updates
- `messageType`: `"Update"` (not `"Alert"`)
- `references` array populated with previous alert info
- `description` may reference previous alert ("...NOW IN EFFECT...")
- VTEC code shows action code `O.EXT` (extended) or `O.COR` (corrected)

---

## Example 4: Alert Cancellation (messageType: "Cancel")

### Response Excerpt
```json
{
  "id": "https://api.weather.gov/alerts/urn:oid:2.49.0.1.840.0.2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7a",
  "type": "Feature",
  "geometry": null,
  "properties": {
    "id": "urn:oid:2.49.0.1.840.0.2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7a",
    "areaDesc": "Miami-Dade",
    "geocode": {
      "SAME": ["012086"],
      "UGC": ["FLZ173"]
    },
    "affectedZones": ["https://api.weather.gov/zones/forecast/FLZ173"],
    "references": [
      {
        "@id": "https://api.weather.gov/alerts/urn:oid:2.49.0.1.840.0.8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b",
        "identifier": "urn:oid:2.49.0.1.840.0.8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b",
        "sender": "w-nws.webmaster@noaa.gov",
        "sent": "2026-02-16T05:00:00-05:00"
      }
    ],
    "sent": "2026-02-16T11:00:00-05:00",
    "effective": "2026-02-16T11:00:00-05:00",
    "onset": null,
    "expires": "2026-02-16T11:00:00-05:00",
    "ends": null,
    "status": "Actual",
    "messageType": "Cancel",
    "category": "Met",
    "severity": "Unknown",
    "certainty": "Unknown",
    "urgency": "Unknown",
    "event": "Tornado Warning",
    "sender": "w-nws.webmaster@noaa.gov",
    "senderName": "NWS Miami FL",
    "headline": "Tornado Warning cancelled",
    "description": "The Tornado Warning for southern Miami-Dade County has been cancelled. The storm which prompted the warning has weakened below severe limits and no longer appears capable of producing a tornado.",
    "instruction": null,
    "response": "AllClear",
    "parameters": {
      "AWIPSidentifier": ["TORFL"],
      "WMOidentifier": ["WWFL"],
      "NWSheadline": ["TORNADO WARNING IS CANCELLED"],
      "BLOCKCHANNEL": ["EAS", "NWEM", "CMAS"],
      "VTEC": ["/O.CAN.KMFL.TO.W.0005.000000T0000Z-260216T1600Z/"],
      "eventEndingTime": ["2026-02-16T16:00:00+00:00"]
    }
  }
}
```

### Key Differences for Cancellations
- `messageType`: `"Cancel"`
- `references` points to cancelled alert
- `response`: `"AllClear"`
- `severity`, `certainty`, `urgency` often `"Unknown"`
- `expires` may equal `sent` (immediate cancellation)

---

## Example 5: GET /alerts/active/zone/TXZ253

Single zone query.

### Request
```http
GET /alerts/active/zone/TXZ253 HTTP/1.1
Host: api.weather.gov
User-Agent: (LocalWeather/1.0, dev@local.com)
Accept: application/geo+json
```

### Response
```json
{
  "@context": [...],
  "type": "FeatureCollection",
  "features": [
    {
      "id": "https://api.weather.gov/alerts/urn:oid:2.49.0.1.840.0.1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d",
      "type": "Feature",
      "geometry": null,
      "properties": {
        "id": "urn:oid:2.49.0.1.840.0.1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d",
        "areaDesc": "Bexar",
        "geocode": {
          "SAME": ["048029"],
          "UGC": ["TXZ253"]
        },
        "affectedZones": ["https://api.weather.gov/zones/forecast/TXZ253"],
        "references": [],
        "sent": "2026-02-16T02:01:00-06:00",
        "effective": "2026-02-16T02:01:00-06:00",
        "onset": "2026-02-16T23:00:00-06:00",
        "expires": "2026-02-17T03:15:00-06:00",
        "ends": "2026-02-17T17:00:00-06:00",
        "status": "Actual",
        "messageType": "Alert",
        "category": "Met",
        "severity": "Moderate",
        "certainty": "Likely",
        "urgency": "Expected",
        "event": "Winter Weather Advisory",
        "sender": "w-nws.webmaster@noaa.gov",
        "senderName": "NWS Austin/San Antonio TX",
        "headline": "Winter Weather Advisory issued February 16 at 2:01AM CST until February 17 at 5:00PM CST by NWS Austin/San Antonio TX",
        "description": "* WHAT...Snow expected. Total snow accumulations of 1 to 3 inches.\n\n* WHERE...Bexar County.\n\n* WHEN...From 5 PM this afternoon to 5 PM CST Monday.\n\n* IMPACTS...Plan on slippery road conditions.",
        "instruction": "Slow down and use caution while traveling.",
        "response": "Prepare",
        "parameters": {
          "AWIPSidentifier": ["WSWTX"],
          "WMOidentifier": ["WWTX"],
          "NWSheadline": ["WINTER WEATHER ADVISORY IN EFFECT FROM 5 PM THIS AFTERNOON TO 5 PM CST MONDAY"],
          "BLOCKCHANNEL": ["EAS", "NWEM"],
          "VTEC": ["/O.NEW.KEWX.WW.Y.0003.260216T2300Z-260217T2300Z/"],
          "eventEndingTime": ["2026-02-17T23:00:00+00:00"]
        }
      }
    }
  ],
  "title": "Current watches, warnings, and advisories for TXZ253",
  "updated": "2026-02-16T08:01:00+00:00"
}
```

---

## Example 6: GET /alerts/{id}

Fetch specific alert by ID.

### Request
```http
GET /alerts/urn:oid:2.49.0.1.840.0.1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d HTTP/1.1
Host: api.weather.gov
User-Agent: (AlertViewer/1.0, dev@alerts.com)
Accept: application/geo+json
```

### Response
```json
{
  "@context": [...],
  "id": "https://api.weather.gov/alerts/urn:oid:2.49.0.1.840.0.1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d",
  "type": "Feature",
  "geometry": null,
  "properties": {
    "@id": "https://api.weather.gov/alerts/urn:oid:2.49.0.1.840.0.1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d",
    "@type": "wx:Alert",
    "id": "urn:oid:2.49.0.1.840.0.1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d",
    "areaDesc": "Bexar",
    "geocode": {
      "SAME": ["048029"],
      "UGC": ["TXZ253"]
    },
    "affectedZones": ["https://api.weather.gov/zones/forecast/TXZ253"],
    "references": [],
    "sent": "2026-02-16T02:01:00-06:00",
    "effective": "2026-02-16T02:01:00-06:00",
    "onset": "2026-02-16T23:00:00-06:00",
    "expires": "2026-02-17T03:15:00-06:00",
    "ends": "2026-02-17T17:00:00-06:00",
    "status": "Actual",
    "messageType": "Alert",
    "category": "Met",
    "severity": "Moderate",
    "certainty": "Likely",
    "urgency": "Expected",
    "event": "Winter Weather Advisory",
    "sender": "w-nws.webmaster@noaa.gov",
    "senderName": "NWS Austin/San Antonio TX",
    "headline": "Winter Weather Advisory issued February 16 at 2:01AM CST until February 17 at 5:00PM CST by NWS Austin/San Antonio TX",
    "description": "* WHAT...Snow expected. Total snow accumulations of 1 to 3 inches.\n\n* WHERE...Bexar County.\n\n* WHEN...From 5 PM this afternoon to 5 PM CST Monday.\n\n* IMPACTS...Plan on slippery road conditions.",
    "instruction": "Slow down and use caution while traveling.",
    "response": "Prepare",
    "parameters": {
      "AWIPSidentifier": ["WSWTX"],
      "WMOidentifier": ["WWTX"],
      "NWSheadline": ["WINTER WEATHER ADVISORY IN EFFECT FROM 5 PM THIS AFTERNOON TO 5 PM CST MONDAY"],
      "BLOCKCHANNEL": ["EAS", "NWEM"],
      "VTEC": ["/O.NEW.KEWX.WW.Y.0003.260216T2300Z-260217T2300Z/"],
      "eventEndingTime": ["2026-02-17T23:00:00+00:00"]
    }
  }
}
```

**Note**: Returns single Feature (not FeatureCollection) when querying by ID.

---

## Example 7: Empty Result Set

When no active alerts match criteria.

### Request
```http
GET /alerts/active/area/HI HTTP/1.1
Host: api.weather.gov
User-Agent: (HawaiiWeather/1.0, dev@hi.com)
Accept: application/geo+json
```

### Response
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
  "features": [],
  "title": "Current watches, warnings, and advisories for Hawaii",
  "updated": "2026-02-16T14:00:00+00:00"
}
```

**Key Points**:
- Still returns valid FeatureCollection
- `features` array is empty
- HTTP status 200 (not 404)

---

## Example 8: Error Response (Rate Limited)

### Request
```http
GET /alerts/active HTTP/1.1
Host: api.weather.gov
User-Agent: (SpamBot/1.0, spam@evil.com)
Accept: application/geo+json
```

### Response
```http
HTTP/1.1 429 Too Many Requests
Retry-After: 5
Content-Type: application/problem+json
```

```json
{
  "correlationId": "abc123def456",
  "title": "Too Many Requests",
  "type": "https://api.weather.gov/problems/RateLimited",
  "status": 429,
  "detail": "Rate limit exceeded. Please retry after 5 seconds.",
  "instance": "/alerts/active"
}
```

**Error Fields**:
- `correlationId` - Request tracking ID
- `title` - Human-readable error title
- `type` - Error type URI
- `status` - HTTP status code
- `detail` - Detailed error message
- `instance` - Request path that failed

---

## Example 9: Error Response (Invalid Zone)

### Request
```http
GET /alerts/active/zone/INVALID HTTP/1.1
Host: api.weather.gov
User-Agent: (MyApp/1.0, dev@app.com)
Accept: application/geo+json
```

### Response
```http
HTTP/1.1 400 Bad Request
Content-Type: application/problem+json
```

```json
{
  "correlationId": "xyz789abc",
  "title": "Bad Request",
  "type": "https://api.weather.gov/problems/BadRequest",
  "status": 400,
  "detail": "Invalid zone identifier: INVALID",
  "instance": "/alerts/active/zone/INVALID"
}
```

---

## Example 10: Paginated Response

When more than 50 alerts returned.

### Request
```http
GET /alerts?severity=severe HTTP/1.1
Host: api.weather.gov
User-Agent: (SevereWeather/1.0, dev@severe.com)
Accept: application/geo+json
```

### Response
```json
{
  "@context": [...],
  "type": "FeatureCollection",
  "features": [
    // ... 50 alerts ...
  ],
  "title": "Severe weather alerts",
  "updated": "2026-02-16T12:00:00+00:00",
  "pagination": {
    "next": "https://api.weather.gov/alerts?severity=severe&cursor=eyJzb3J0IjpbMTY0NTA1NjAwMDAwMCwiOC41Il0sInNlYXJjaEFmdGVyIjpbMTY0NTA1NjAwMDAwMCwiOC41Il0sImlkIjoiMTIzNDU2In0"
  }
}
```

**To Get Next Page**:
```http
GET /alerts?severity=severe&cursor=eyJzb3J0IjpbMTY0NTA1NjAwMDAwMCwiOC41Il0sInNlYXJjaEFmdGVyIjpbMTY0NTA1NjAwMDAwMCwiOC41Il0sImlkIjoiMTIzNDU2In0 HTTP/1.1
```

**Last Page**:
```json
{
  "features": [...],
  "pagination": {}
  // No "next" field = last page
}
```

---

## Field Nullability Summary

Common fields that can be `null` or omitted:

| Field | Nullable? | When Null/Omitted |
|-------|-----------|-------------------|
| `geometry` | Yes | Most NWS alerts (use geocode instead) |
| `onset` | Yes | Immediate-onset events |
| `ends` | Yes | Unknown end time |
| `headline` | No | Always present |
| `description` | No | Always present |
| `instruction` | Yes | When no specific action needed |
| `references` | No | Empty array for new alerts |
| `parameters` | No | May be empty object |
| `pagination.next` | Yes | Omitted on last page |

---

## Response Headers

Typical response headers from NWS API:

```http
HTTP/1.1 200 OK
Content-Type: application/geo+json
Cache-Control: public, max-age=300
Expires: Sun, 16 Feb 2026 14:05:00 GMT
Vary: Accept, Accept-Encoding
X-Correlation-Id: abc123def456
X-Request-Id: xyz789
ETag: "33a64df551425fcc55e4d42a148795d9f25f89d4"
```

**Important Headers**:
- `Cache-Control` - Caching directives (typically 5-15 min TTL)
- `Expires` - Absolute expiration time
- `X-Correlation-Id` - Request tracking
- `ETag` - Response version (for conditional requests)

---

## Timezone Handling

All timestamps use ISO 8601 format with timezone offset:

```json
"sent": "2026-02-16T02:01:00-06:00"
```

**Format**: `YYYY-MM-DDTHH:MM:SS±HH:MM`

**Timezone**: Local time for affected area with offset
- `-06:00` = Central Standard Time (CST)
- `-05:00` = Eastern Standard Time (EST)
- `-07:00` = Mountain Standard Time (MST)
- `-08:00` = Pacific Standard Time (PST)
- `-10:00` = Hawaii-Aleutian Standard Time

**UTC Conversion**: Subtract offset from time
```
2026-02-16T02:01:00-06:00
  → 2026-02-16T08:01:00Z (UTC)
```

**Rust Parsing**:
```rust
use chrono::{DateTime, Utc};

let sent: DateTime<Utc> = "2026-02-16T02:01:00-06:00".parse()?;
```

This handles timezone conversion automatically.
