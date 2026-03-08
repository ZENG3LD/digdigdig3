# FAA NASSTATUS API - Response Formats

## Primary Format: XML

### Content-Type
```
Content-Type: application/xml; charset=utf-8
```

### Character Encoding
UTF-8 (standard)

---

## Complete XML Schema Example

### Airport Status Information Response

```xml
<?xml version="1.0" encoding="UTF-8"?>
<AIRPORT_STATUS_INFORMATION>
    <Update_Time>Mon Feb 16 09:01:29 2026 GMT</Update_Time>
    <Dtd_File>http://nasstatus.faa.gov/xml/airport_status.dtd</Dtd_File>

    <!-- Airport Closures -->
    <Delay_type>
        <Name>Airport Closures</Name>
        <Airport_Closure_List>
            <Airport>
                <ARPT>MMU</ARPT>
                <Reason><![CDATA[MMU 02/014 MMU AD AP CLSD TO ALL ACFT 2602160900-2602161500 PHONE: 973-555-1234]]></Reason>
                <Start>2602160900</Start>
                <Reopen>2602161500</Reopen>
            </Airport>
            <Airport>
                <ARPT>SAN</ARPT>
                <Reason><![CDATA[SAN 02/023 SAN RWY 09/27 CLSD DUE TO MAINT 2602160800-2602161200]]></Reason>
                <Start>2602160800</Start>
                <Reopen>2602161200</Reopen>
            </Airport>
            <Airport>
                <ARPT>STX</ARPT>
                <Reason><![CDATA[STX 02/007 STX AD AP CLSD TO ALL ACFT EXC EMERG 2602160700-2602162359]]></Reason>
                <Start>2602160700</Start>
                <Reopen>2602162359</Reopen>
            </Airport>
            <Airport>
                <ARPT>LAS</ARPT>
                <Reason><![CDATA[LAS 02/041 LAS RWY 08L/26R CLSD DUE TO WX 2602161000-2602161400]]></Reason>
                <Start>2602161000</Start>
                <Reopen>2602161400</Reopen>
            </Airport>
        </Airport_Closure_List>
    </Delay_type>

    <!-- Ground Delay Programs -->
    <Delay_type>
        <Name>Ground Delay Programs</Name>
        <Ground_Delay_List>
            <Ground_Delay>
                <ARPT>ORD</ARPT>
                <Reason>WX</Reason>
                <Avg_Delay>45</Avg_Delay>
                <Start>Mon Feb 16 08:30:00 2026 GMT</Start>
                <End>Mon Feb 16 14:00:00 2026 GMT</End>
            </Ground_Delay>
            <Ground_Delay>
                <ARPT>EWR</ARPT>
                <Reason>VOL / WX</Reason>
                <Avg_Delay>62</Avg_Delay>
                <Start>Mon Feb 16 09:15:00 2026 GMT</Start>
                <End>Mon Feb 16 16:30:00 2026 GMT</End>
            </Ground_Delay>
        </Ground_Delay_List>
    </Delay_type>

    <!-- Ground Stops -->
    <Delay_type>
        <Name>Ground Stops</Name>
        <Ground_Stop_List>
            <Ground_Stop>
                <ARPT>ATL</ARPT>
                <Reason>WX / THUNDERSTORMS</Reason>
                <End_Time>Mon Feb 16 10:30:00 2026 GMT</End_Time>
            </Ground_Stop>
        </Ground_Stop_List>
    </Delay_type>

    <!-- Arrival/Departure Delays -->
    <Delay_type>
        <Name>Arrival/Departure Delay Info</Name>
        <Arrival_Departure_Delay_List>
            <Delay>
                <ARPT>LAX</ARPT>
                <Arrival_Delay>
                    <Min>15</Min>
                    <Max>30</Max>
                    <Trend>Decreasing</Trend>
                </Arrival_Delay>
                <Departure_Delay>
                    <Min>10</Min>
                    <Max>25</Max>
                    <Trend>Stable</Trend>
                </Departure_Delay>
                <Reason>WX / LOW VISIBILITY</Reason>
            </Delay>
            <Delay>
                <ARPT>JFK</ARPT>
                <Arrival_Delay>
                    <Min>20</Min>
                    <Max>45</Max>
                    <Trend>Increasing</Trend>
                </Arrival_Delay>
                <Departure_Delay>
                    <Min>15</Min>
                    <Max>35</Max>
                    <Trend>Stable</Trend>
                </Departure_Delay>
                <Reason>VOL / HIGH TRAFFIC</Reason>
            </Delay>
        </Arrival_Departure_Delay_List>
    </Delay_type>

    <!-- Airspace Flow Programs -->
    <Delay_type>
        <Name>Airspace Flow Programs</Name>
        <Airspace_Flow_Program_List>
            <Program>
                <Name>ZNY AFP</Name>
                <Region>New York Center</Region>
                <Reason>WX / CONVECTIVE ACTIVITY</Reason>
                <Affected_Airports>
                    <ARPT>JFK</ARPT>
                    <ARPT>LGA</ARPT>
                    <ARPT>EWR</ARPT>
                </Affected_Airports>
                <Start>Mon Feb 16 10:00:00 2026 GMT</Start>
                <End>Mon Feb 16 18:00:00 2026 GMT</End>
            </Program>
        </Airspace_Flow_Program_List>
    </Delay_type>
</AIRPORT_STATUS_INFORMATION>
```

**Note**: The above example shows the expected full schema. Actual responses vary based on current NAS conditions. An empty response (no delays) may contain only `Update_Time` and `Dtd_File`.

---

## Field Descriptions

### Root Element

| Field | Type | Description | Required |
|-------|------|-------------|----------|
| `AIRPORT_STATUS_INFORMATION` | Element | Root container | Yes |

### Top-Level Fields

| Field | Type | Description | Required |
|-------|------|-------------|----------|
| `Update_Time` | String | Last data update timestamp (RFC 2822 format, GMT) | Yes |
| `Dtd_File` | String | URL to XML DTD schema file | Yes |
| `Delay_type` | Element (array) | Container for each delay category | Conditional |

### Delay_type Structure

| Field | Type | Description | Required |
|-------|------|-------------|----------|
| `Name` | String | Delay category name | Yes |
| `Airport_Closure_List` | Element | Container for airport closures | Conditional |
| `Ground_Delay_List` | Element | Container for ground delay programs | Conditional |
| `Ground_Stop_List` | Element | Container for ground stops | Conditional |
| `Arrival_Departure_Delay_List` | Element | Container for arrival/departure delays | Conditional |
| `Airspace_Flow_Program_List` | Element | Container for airspace flow programs | Conditional |

### Airport Closure Fields

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| `ARPT` | String | 3-letter IATA airport code | MMU, SAN, ATL |
| `Reason` | String (CDATA) | NOTAM-format closure reason | MMU 02/014 MMU AD AP CLSD... |
| `Start` | String | Closure start time (NOTAM format or RFC 2822) | 2602160900 |
| `Reopen` | String | Expected reopening time | 2602161500 |

### Ground Delay Fields

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| `ARPT` | String | 3-letter IATA airport code | ORD, EWR |
| `Reason` | String | Delay reason (abbreviated) | WX, VOL / WX |
| `Avg_Delay` | Integer | Average delay in minutes | 45, 62 |
| `Start` | String | GDP start time (RFC 2822) | Mon Feb 16 08:30:00 2026 GMT |
| `End` | String | Expected end time (RFC 2822) | Mon Feb 16 14:00:00 2026 GMT |

### Ground Stop Fields

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| `ARPT` | String | 3-letter IATA airport code | ATL |
| `Reason` | String | Ground stop reason | WX / THUNDERSTORMS |
| `End_Time` | String | Expected end time (RFC 2822) | Mon Feb 16 10:30:00 2026 GMT |

### Arrival/Departure Delay Fields

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| `ARPT` | String | 3-letter IATA airport code | LAX, JFK |
| `Arrival_Delay` | Object | Arrival delay details | See below |
| `Departure_Delay` | Object | Departure delay details | See below |
| `Reason` | String | Delay reason | WX / LOW VISIBILITY |

**Delay Object Structure**:
| Field | Type | Description | Example |
|-------|------|-------------|---------|
| `Min` | Integer | Minimum delay minutes | 15 |
| `Max` | Integer | Maximum delay minutes | 30 |
| `Trend` | String | Delay trend | Increasing, Decreasing, Stable |

### Airspace Flow Program Fields

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| `Name` | String | AFP identifier | ZNY AFP |
| `Region` | String | Affected airspace region | New York Center |
| `Reason` | String | Program reason | WX / CONVECTIVE ACTIVITY |
| `Affected_Airports` | Array | List of impacted airports | JFK, LGA, EWR |
| `Start` | String | Program start time | Mon Feb 16 10:00:00 2026 GMT |
| `End` | String | Expected end time | Mon Feb 16 18:00:00 2026 GMT |

---

## Minimal Response (No Delays)

```xml
<?xml version="1.0" encoding="UTF-8"?>
<AIRPORT_STATUS_INFORMATION>
    <Update_Time>Mon Feb 16 09:01:29 2026 GMT</Update_Time>
    <Dtd_File>http://nasstatus.faa.gov/xml/airport_status.dtd</Dtd_File>
</AIRPORT_STATUS_INFORMATION>
```

**When no delays are active**, the response contains only metadata fields.

---

## CDATA Sections

### Why CDATA?
The `Reason` field in airport closures contains special characters (slashes, spaces, hyphens) that could break XML parsing. CDATA prevents this:

```xml
<Reason><![CDATA[MMU 02/014 MMU AD AP CLSD TO ALL ACFT 2602160900-2602161500]]></Reason>
```

### Parsing CDATA in Rust

```rust
use quick_xml::Reader;
use quick_xml::events::Event;

fn parse_reason(xml: &str) -> String {
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::CData(e)) => {
                return e.unescape().unwrap().into_owned();
            }
            Ok(Event::Text(e)) => {
                return e.unescape().unwrap().into_owned();
            }
            Ok(Event::Eof) => break,
            _ => {}
        }
        buf.clear();
    }
    String::new()
}
```

---

## JSON Format (Legacy ASWS - Offline)

The legacy ASWS API (now offline) supported JSON responses. For reference, here's the expected schema:

```json
{
  "City": "Atlanta",
  "Delay": true,
  "DelayCount": 1,
  "IATA": "ATL",
  "ICAO": "KATL",
  "Name": "Hartsfield-Jackson Atlanta International Airport",
  "State": "GA",
  "Status": {
    "AvgDelay": "45",
    "ClosureBegin": "",
    "ClosureEnd": "",
    "EndTime": "14:00 PM EST",
    "MaxDelay": "60",
    "MinDelay": "30",
    "Reason": "WX / THUNDERSTORMS",
    "Trend": "Increasing",
    "Type": "Ground Delay"
  },
  "SupportedAirport": true,
  "Weather": {
    "Meta": {
      "Credit": "NOAA's National Weather Service",
      "Updated": "9:00 AM EST",
      "Url": "http://weather.gov/"
    },
    "Temp": "72 F (22 C)",
    "Visibility": 10.00,
    "Weather": "Partly Cloudy",
    "Wind": "South at 12 mph"
  }
}
```

**Note**: This format is no longer available. Included for historical reference only.

---

## Error Responses

### HTTP 404 (Not Found)
```xml
<?xml version="1.0" encoding="UTF-8"?>
<error>
  <code>404</code>
  <message>Endpoint not found</message>
</error>
```

**More commonly**: Plain text error message or empty body.

### HTTP 500 (Internal Server Error)
```xml
<?xml version="1.0" encoding="UTF-8"?>
<error>
  <code>500</code>
  <message>Internal server error</message>
</error>
```

**More commonly**: HTML error page or empty body.

### HTTP 503 (Service Unavailable)
```
Service Temporarily Unavailable
```

**Plain text response** during outages or maintenance.

---

## Parsing Strategies

### Rust XML Parsing

#### Option 1: quick-xml (Recommended)
```rust
use quick_xml::de::from_str;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename = "AIRPORT_STATUS_INFORMATION")]
struct AirportStatusResponse {
    #[serde(rename = "Update_Time")]
    update_time: String,
    #[serde(rename = "Dtd_File")]
    dtd_file: String,
    #[serde(rename = "Delay_type", default)]
    delay_types: Vec<DelayType>,
}

#[derive(Debug, Deserialize)]
struct DelayType {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Airport_Closure_List", default)]
    closures: Option<AirportClosureList>,
}

#[derive(Debug, Deserialize)]
struct AirportClosureList {
    #[serde(rename = "Airport", default)]
    airports: Vec<AirportClosure>,
}

#[derive(Debug, Deserialize)]
struct AirportClosure {
    #[serde(rename = "ARPT")]
    code: String,
    #[serde(rename = "Reason")]
    reason: String,
    #[serde(rename = "Start")]
    start: String,
    #[serde(rename = "Reopen")]
    reopen: String,
}

let response: AirportStatusResponse = from_str(&xml_string)?;
```

#### Option 2: roxmltree (Lightweight)
```rust
use roxmltree::Document;

let doc = Document::parse(&xml_string)?;
let root = doc.root_element();

let update_time = root
    .children()
    .find(|n| n.has_tag_name("Update_Time"))
    .and_then(|n| n.text())
    .unwrap_or("");

for delay_type in root.children().filter(|n| n.has_tag_name("Delay_type")) {
    let name = delay_type
        .children()
        .find(|n| n.has_tag_name("Name"))
        .and_then(|n| n.text())
        .unwrap_or("");

    // Process delay_type...
}
```

---

## Response Size Estimates

| Scenario | Size (KB) | Example |
|----------|-----------|---------|
| No active delays | 0.3-0.5 | Minimal response |
| 1-5 airport events | 1-3 | Light delay day |
| 10-20 airport events | 5-10 | Moderate delays |
| 50+ airport events | 20-50 | Major weather event |
| Maximum observed | ~50 | Nationwide weather |

**Typical response**: 2-10 KB

---

## Summary

| Feature | Details |
|---------|---------|
| Primary format | XML |
| Character encoding | UTF-8 |
| Root element | `AIRPORT_STATUS_INFORMATION` |
| CDATA usage | `Reason` fields |
| Timestamp formats | RFC 2822 (Update_Time), NOTAM (Start/Reopen) |
| Array handling | Named list elements (e.g., `Airport_Closure_List`) |
| Conditional fields | Delay types only present when active |
| Error format | Varied (XML, HTML, plain text) |
| Typical size | 2-10 KB |
| JSON support | Legacy only (offline) |

**Parsing recommendation**: Use `quick-xml` with `serde` for type-safe deserialization, or `roxmltree` for lightweight manual parsing.
