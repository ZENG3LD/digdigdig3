# FAA NASSTATUS API - Data Types

## Overview
The FAA NASSTATUS API provides structured data about airport delays, closures, and restrictions in the United States National Airspace System.

---

## Airport Identifiers

### IATA Code
- **Format**: 3-letter code
- **Example**: `ATL`, `ORD`, `LAX`, `SFO`
- **Standard**: International Air Transport Association
- **Field name**: `ARPT` (in XML responses)

### ICAO Code
- **Format**: 4-letter code
- **Example**: `KATL`, `KORD`, `KLAX`, `KSFO`
- **Standard**: International Civil Aviation Organization
- **Field name**: Not always present in NASSTATUS responses

### Airport Names
- **Format**: Full text name
- **Example**: "Hartsfield-Jackson Atlanta International Airport"
- **Field name**: Airport name (not present in all response types)

---

## Delay Types

### 1. Airport Closure
**Description**: Airport is closed to traffic
**Field name**: `Airport_Closure_List`
**Severity**: Critical
**Data includes**:
- Airport code
- Closure reason (NOTAM format)
- Start time
- Expected reopening time

**Example scenario**:
- Runway maintenance
- Severe weather
- Emergency situation
- Security incident

---

### 2. Ground Stop (GS)
**Description**: Aircraft must remain on ground at origin airport
**Severity**: Critical
**Duration**: Typically short-term (minutes to hours)
**Reason codes**:
- Weather (WX)
- Runway (RWY)
- Volume (VOL)
- Equipment (EQUIPMENT)

**Characteristics**:
- Immediate effect
- Usually precedes Ground Delay Program
- Zero acceptance rate at affected airport

---

### 3. Ground Delay Program (GDP)
**Description**: Controlled rate of arrivals to manage capacity
**Severity**: Moderate to Major
**Duration**: Hours (typically 2-8 hours)
**Reason codes**:
- Weather (WX)
- Volume (VOL)
- Runway configuration (RWY)
- Wind (WIND)

**Data includes**:
- Airport Arrival Rate (AAR)
- Average delay in minutes
- Delay start time
- Estimated end time
- Affected airports (departure airports may be included)

**Impact**:
- Delays at origin airports before departure
- Reduced acceptance rate at destination

---

### 4. Airspace Flow Program (AFP)
**Description**: Metering of traffic through constrained airspace
**Severity**: Moderate
**Scope**: Regional (affects multiple airports)
**Reason codes**:
- Weather (WX)
- Special Use Airspace (SUA)
- Volume (VOL)

**Characteristics**:
- Affects en-route traffic
- Not airport-specific (but may list affected airports)
- Delays assigned via Expected Departure Clearance Time (EDCT)

---

### 5. Arrival Delay
**Description**: Delays for arriving flights
**Severity**: Minor to Moderate
**Field name**: `arrival_delay`
**Unit**: Minutes (average)

**Causes**:
- Weather at destination
- Congestion
- Runway configuration

---

### 6. Departure Delay
**Description**: Delays for departing flights
**Severity**: Minor to Moderate
**Field name**: `departure_delay`
**Unit**: Minutes (average)

**Causes**:
- Weather at origin
- Ground congestion
- Air traffic control restrictions

---

## Delay Severity Levels

### Normal
- **Value**: `normal`
- **Delay**: 0-15 minutes
- **Color**: Green (in FAA dashboard)
- **Impact**: Minimal operational impact

### Minor
- **Value**: `minor`
- **Delay**: 15-30 minutes
- **Color**: Yellow
- **Impact**: Slight delays, manageable

### Moderate
- **Value**: `moderate`
- **Delay**: 30-60 minutes
- **Color**: Orange
- **Impact**: Significant delays, schedule adjustments needed

### Major
- **Value**: `major`
- **Delay**: 60-120 minutes
- **Color**: Red
- **Impact**: Major disruptions, many flights affected

### Severe
- **Value**: `severe`
- **Delay**: 120+ minutes or closure
- **Color**: Dark Red
- **Impact**: Critical disruptions, possible diversions

---

## Timestamp Formats

### Update_Time
- **Format**: RFC 2822 style
- **Example**: `Mon Feb 16 09:01:29 2026 GMT`
- **Timezone**: Always GMT/UTC
- **Field name**: `Update_Time`

### Start Time
- **Format**: NOTAM format (YYMMDDHHM) or RFC 2822
- **Example**: `2602160900` (26 Feb 16, 09:00 UTC)
- **Field name**: `Start`

### Reopen Time
- **Format**: NOTAM format (YYMMDDHHM) or RFC 2822
- **Example**: `2602161500` (26 Feb 16, 15:00 UTC)
- **Field name**: `Reopen`

**Parsing considerations**:
- Always UTC/GMT (no timezone conversion needed)
- NOTAM format: YY (year), MM (month), DD (day), HH (hour), M (minute/10)
- May include range: `2602160900-2602161500`

---

## Weather Data

### Weather Information (Legacy ASWS)
Available in legacy ASWS endpoint (now offline), may include:
- **Temperature**: Degrees (Celsius or Fahrenheit)
- **Visibility**: Miles or kilometers
- **Wind**: Speed and direction
- **Conditions**: Clear, cloudy, rain, snow, etc.

**Note**: Current NASSTATUS endpoint focuses on delays, not detailed weather.

---

## Reason Codes

### NOTAM Format Reasons
Airport closures include detailed NOTAM-format reasons:

**Example**:
```
MMU 02/014 MMU AD AP CLSD TO ALL ACFT 2602160900-2602161500
```

**Breakdown**:
- `MMU`: Airport code
- `02/014`: NOTAM number (year/sequence)
- `MMU AD AP`: Airport aerodrome
- `CLSD TO ALL ACFT`: Closed to all aircraft
- `2602160900-2602161500`: Time range (UTC)
- Contact: Phone number may be included

### Common Reason Abbreviations

| Code | Meaning |
|------|---------|
| WX | Weather |
| VOL | Volume (traffic) |
| RWY | Runway |
| WIND | Wind |
| EQUIPMENT | Equipment failure |
| SUA | Special Use Airspace |
| CONST | Construction |
| NOTAM | Notice to Airmen |
| AD | Aerodrome |
| AP | Airport |
| CLSD | Closed |
| ACFT | Aircraft |

---

## Airport Status Values

### Status Types
- **Open**: Normal operations
- **Closed**: Airport closed
- **Delayed**: Active delays (GDP, GS, AFP)
- **Limited**: Reduced capacity

**Note**: Exact status enum not documented in XML schema, but inferred from presence of delay types.

---

## Data Structure Hierarchy

```
AIRPORT_STATUS_INFORMATION (root)
├── Update_Time (string, timestamp)
├── Dtd_File (string, URL to DTD schema)
└── Delay_type (array, repeatable)
    ├── Name (string, e.g., "Airport Closures")
    └── Airport_Closure_List (object)
        └── Airport (array, repeatable)
            ├── ARPT (string, 3-letter IATA code)
            ├── Reason (string, NOTAM format)
            ├── Start (string, timestamp)
            └── Reopen (string, timestamp)
```

**Additional Delay_type variants** (not observed in sample, but documented):
- `Ground_Delay_List`
- `Ground_Stop_List`
- `Arrival_Departure_Delay_List`
- `Airspace_Flow_Program_List`

---

## Coverage Data

### Airport Categories
Not explicitly categorized in API, but coverage includes:
- **Large hub airports**: High traffic (e.g., ATL, ORD, LAX)
- **Medium hub airports**: Moderate traffic (e.g., BNA, RDU, SAN)
- **Small hub airports**: Limited coverage (e.g., BDL, PVD)
- **Non-hub airports**: Rare coverage (only during major events)

### Geographic Coverage
- **United States**: Primary coverage
- **Territories**: Limited (e.g., Puerto Rico - SJU, US Virgin Islands - STX)
- **International**: None

---

## Data Quality Indicators

### SupportedAirport (Legacy ASWS)
Boolean field indicating whether detailed status is available for the airport.
- `true`: Full status data available
- `false`: Limited or no data

**Note**: This field is from the legacy ASWS API (now offline).

---

## Response Format Differences

### XML Response
- Primary format
- Nested structure
- CDATA sections for long text (Reason field)
- Array items wrapped in named lists

### JSON Response (Legacy ASWS)
- Flat object structure
- Easier to parse
- Direct field access
- No CDATA handling needed

**Current NASSTATUS API returns XML only.**

---

## Enumeration Summary

### Delay Types (Enumeration)
```rust
pub enum DelayType {
    AirportClosure,
    GroundStop,
    GroundDelayProgram,
    AirspaceFlowProgram,
    ArrivalDelay,
    DepartureDelay,
}
```

### Severity Levels (Enumeration)
```rust
pub enum DelaySeverity {
    Normal,
    Minor,
    Moderate,
    Major,
    Severe,
}
```

### Reason Codes (Common Patterns)
```rust
pub enum DelayReason {
    Weather,
    Volume,
    Runway,
    Wind,
    Equipment,
    Construction,
    SpecialUseAirspace,
    Security,
    Other(String),
}
```

---

## Summary

| Data Type | Format | Example | Notes |
|-----------|--------|---------|-------|
| Airport code | 3-letter IATA | ATL | Primary identifier |
| Delay type | String enum | Airport Closures, GDP | Category of delay |
| Severity | String enum | normal, minor, major | Impact level |
| Delay minutes | Integer | 45 | Average delay |
| Timestamp | RFC 2822 | Mon Feb 16 09:01:29 2026 GMT | Always UTC |
| NOTAM time | YYMMDDHHM | 2602160900 | Compact format |
| Reason | String (NOTAM format) | WX, VOL, RWY CLSD | Abbreviations common |
| Status | String | Open, Closed, Delayed | Inferred from data |
