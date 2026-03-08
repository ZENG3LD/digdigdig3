# NWS Alerts API - Coverage & Scope

## Geographic Coverage

### United States Territory

The NWS Alerts API provides comprehensive coverage for:

#### 50 US States
All states from Alabama to Wyoming, including:
- Continental United States (48 states)
- Alaska
- Hawaii

**State Codes**: Standard 2-letter postal abbreviations
- Examples: `TX` (Texas), `CA` (California), `NY` (New York), `FL` (Florida)

#### US Territories
- **Puerto Rico** (`PR`)
- **US Virgin Islands** (`VI`)
- **Guam** (`GU`)
- **American Samoa** (`AS`)
- **Northern Mariana Islands** (`MP`)

#### US Marine Regions

**Atlantic Ocean**:
- North Atlantic (`AN`)
- South Atlantic (`AS`)
- Atlantic region (`AT`)

**Pacific Ocean**:
- Pacific region (`PA`)
- Pacific Islands (`PI`)

**Gulf of Mexico** (`GM`)

**Great Lakes** (`GL`)

**Alaska Marine** (`AL`)

---

## Administrative Boundaries

### County Coverage

**Total Counties**: ~3,200 US counties, parishes, boroughs, and census areas

**Identification**:
- **SAME Codes**: 6-digit codes (e.g., `048029` for Bexar County, TX)
  - Format: State FIPS (2) + County FIPS (3) + Subcounty (1)
- **UGC Codes**: County UGC (e.g., `TXC029`)
  - Format: State (2 letters) + `C` + County number (3 digits)

**Coverage**:
- All US counties
- Louisiana parishes
- Alaska boroughs and census areas
- Independent cities (Virginia, Maryland, Missouri, Nevada)

### NWS Forecast Zones

**Total Zones**: ~4,000 public forecast zones

**Types**:
- **Land Zones**: Public forecast zones for weather forecasts
- **Fire Weather Zones**: Specialized zones for wildfire weather
- **Marine Zones**: Coastal and offshore marine areas

**Identification**:
- **UGC Codes**: Zone UGC (e.g., `TXZ253`)
  - Format: State (2 letters) + `Z` + Zone number (3 digits)

**Coverage**: More granular than counties, designed for weather forecast areas

**Example Zone Breakdown** (Texas):
- `TXZ253` - Bexar County area (San Antonio)
- `TXZ209` - Houston metropolitan area
- `TXZ418` - Northwestern Edwards Plateau

### NWS Offices

**Total Offices**: 122 Weather Forecast Offices (WFOs)

**Coverage**: Each office responsible for specific geographic regions
- Offices identified in alerts via `senderName` field
- Example: "NWS Austin/San Antonio TX" (`KEWX` identifier)

**Office Identifiers**:
- 4-letter codes (ICAO-like): `KEWX`, `KOUN`, `KMFL`, etc.
- Used in VTEC codes and AWIPS identifiers

---

## Alert Type Coverage

### Weather Phenomena

#### Severe Weather
- **Tornadoes**: Tornado Watch, Tornado Warning, Tornado Emergency
- **Severe Thunderstorms**: Severe Thunderstorm Watch/Warning
- **Hail**: Large Hail Warning
- **Damaging Winds**: High Wind Warning, Wind Advisory, Extreme Wind Warning

#### Winter Weather
- **Snow**: Winter Storm Watch/Warning, Blizzard Warning, Snow Advisory
- **Ice**: Ice Storm Warning, Freezing Rain Advisory
- **Wind Chill**: Wind Chill Watch/Warning/Advisory
- **Lake Effect**: Lake Effect Snow Watch/Warning

#### Flooding
- **Flash Floods**: Flash Flood Watch/Warning/Emergency
- **River Floods**: Flood Watch/Warning, River Flood Statement
- **Coastal Floods**: Coastal Flood Watch/Warning/Advisory
- **Areal Floods**: Areal Flood Watch/Warning/Advisory

#### Tropical Systems
- **Hurricanes**: Hurricane Watch/Warning, Hurricane Force Wind Warning
- **Tropical Storms**: Tropical Storm Watch/Warning
- **Storm Surge**: Storm Surge Watch/Warning

#### Heat & Cold
- **Heat**: Excessive Heat Watch/Warning, Heat Advisory
- **Cold**: Freeze Watch/Warning, Frost Advisory
- **Extreme Cold**: Extreme Cold Watch/Warning

#### Fire Weather
- **Fire Conditions**: Red Flag Warning, Fire Weather Watch
- **Smoke**: Air Quality Alert (smoke-related)

#### Marine Weather
- **Marine Warnings**: Small Craft Advisory, Gale Warning, Storm Warning
- **Marine Storms**: Hurricane Force Wind Warning (marine)
- **Marine Hazards**: Special Marine Warning

#### Other Hazards
- **Fog**: Dense Fog Advisory
- **Air Quality**: Air Quality Alert
- **Dust**: Blowing Dust Advisory, Dust Storm Warning
- **Avalanche**: Avalanche Watch/Warning
- **Volcano**: Volcano Warning
- **Tsunami**: Tsunami Watch/Warning
- **Earthquake**: Earthquake information statements (rare)

### Non-Weather Alerts

While NWS primarily issues weather alerts, the CAP format supports other categories (rarely used by NWS):

- **Geophysical** (`Geo`): Earthquakes, tsunamis, volcanic activity
- **Safety** (`Safety`): General public safety
- **Security** (`Security`): Law enforcement (rare, usually handled by EAS)
- **Civil Emergency**: Managed by other agencies, not NWS

**NWS Focus**: 99%+ of NWS alerts are category `Met` (Meteorological)

---

## Temporal Coverage

### Historical Data

**Available**: Past 7 days via API

**Endpoint**: `/alerts` with `start` and `end` parameters
```
GET /alerts?start=2026-02-10T00:00:00Z&end=2026-02-16T23:59:59Z
```

**Limitation**: Alerts older than 7 days not available via API

**Archive**: For historical data beyond 7 days, contact National Centers for Environmental Information (NCEI)

### Real-Time Coverage

**Latency**: Near-real-time (typically <1 minute from issuance)

**Updates**:
- Alerts updated as conditions change
- Updates use `messageType: "Update"` with `references` field
- Cancellations use `messageType: "Cancel"`

**Alert Lifecycle**:
1. **Issuance**: New alert (`messageType: "Alert"`)
2. **Updates**: Condition changes (`messageType: "Update"`)
3. **Cancellation**: Hazard ended early (`messageType: "Cancel"`)
4. **Expiration**: Alert reaches `expires` timestamp, auto-removed from active set

---

## Coverage by Region

### Continental US (Lower 48)

**Coverage**: Complete
- All states
- All counties/parishes
- All NWS forecast zones
- Urban and rural areas
- Day and night operations

**Alert Types**: Full suite of weather hazards

### Alaska

**Coverage**: Complete
- Unique hazards: extreme cold, avalanches, volcanic activity
- Marine zones for extensive coastline
- Rural areas with sparse population
- Wilderness zones

**Special Considerations**:
- Longer forecast zones due to size
- Marine coverage extensive (Bering Sea, Gulf of Alaska)
- Volcanic warnings from Alaska Volcano Observatory

### Hawaii

**Coverage**: Complete
- All islands: Big Island, Maui, Oahu, Kauai, Molokai, Lanai, Niihau
- Tropical hazards: hurricanes, heavy surf, flash flooding
- Marine zones around islands
- High Wind warnings for mountainous areas

**Special Considerations**:
- Tsunami warnings from Pacific Tsunami Warning Center
- High surf advisories/warnings common
- Trade wind weather patterns

### Puerto Rico & US Virgin Islands

**Coverage**: Complete
- Puerto Rico (main island + Vieques, Culebra)
- US Virgin Islands (St. Croix, St. Thomas, St. John)
- Tropical hazards: hurricanes, heavy rain, flash flooding
- Marine zones (Caribbean waters)

**Language**: Alerts typically in English; some bilingual support

### Guam, American Samoa, Northern Mariana Islands

**Coverage**: Complete
- Pacific tropical islands
- Typhoon (western Pacific hurricane) warnings
- Marine hazards
- Heavy rain and flooding

**Special Considerations**:
- Typhoons (not hurricanes) in western Pacific
- Pacific Tsunami Warning Center coordination
- Limited land area, extensive marine zones

---

## Coverage Limitations

### Non-US Areas

**Not Covered**:
- Canada (see Environment Canada)
- Mexico (see CONAGUA/SMN)
- Caribbean nations (except US territories)
- International waters beyond US EEZ (Exclusive Economic Zone)

**Border Areas**:
- Alerts may reference cross-border impacts in description text
- Formal alert coverage ends at US border

### Private Property

**Access**: Alerts are geographic, not property-based
- No exclusions for private land, military bases, or restricted areas
- Alerts cover all areas within forecast zones

### Tribal Lands

**Coverage**: Complete
- Native American reservations fully covered
- Alerts issued for zones containing tribal lands
- May include specific tribal area references in `areaDesc`

---

## Data Categories

### Alert Data

**Primary Data**:
- Active weather alerts (watches, warnings, advisories)
- Alert metadata (severity, urgency, certainty)
- Geographic targeting (zones, counties)
- Temporal information (effective, onset, expires, ends)
- Event descriptions and instructions

**NOT Included**:
- Weather observations (temperature, wind, etc.)
- Forecast data (7-day forecasts, hourly forecasts)
- Radar imagery
- Satellite imagery
- Model data

For non-alert weather data, use other NWS API endpoints:
- `/points/{lat},{lon}/forecast` - Point forecasts
- `/stations/{stationId}/observations` - Observations
- `/radar/stations/{stationId}` - Radar data

---

## Coverage Quality & Reliability

### Data Completeness

**Alert Coverage**: 100% of NWS-issued alerts
- All watches, warnings, advisories
- All severity levels
- All geographic areas within US jurisdiction

**Missing Data**: Rare, typically due to:
- Temporary system outages
- Network issues
- Extreme event overwhelming systems

**Redundancy**: NWS has backup systems for critical operations

### Update Frequency

**Real-Time Alerts**: Updated as issued by NWS offices
- New alerts appear within ~30-60 seconds
- Updates/cancellations similarly fast

**Active Alert Polling**: Recommended 30-60 second interval
- Balances timeliness with rate limits
- Appropriate for weather event timescales

### Data Quality

**Source**: Direct from NWS, official US government data

**Accuracy**:
- Alerts issued by trained meteorologists
- Based on observations, radar, satellites, models
- Human verification for critical alerts

**Legal Status**: Official warnings for emergency management

---

## Coverage Expansion Plans

### Current Status

The NWS API coverage is **stable and complete** for US jurisdiction.

**No Planned Geographic Expansion**:
- Coverage already includes all US states and territories
- NWS has no jurisdiction outside US

### Potential Enhancements

**Future Improvements** (not confirmed):
- Additional alert types (new hazard categories)
- More granular geographic targeting
- Enhanced metadata fields
- Better integration with other NWS data products

**Community Requests**:
- Follow GitHub repository: https://github.com/weather-gov/api
- Feature requests and discussions welcome

---

## Coverage Comparison

### vs Commercial Weather APIs

**NWS Advantages**:
- Free, no API keys
- Official, legally recognized alerts
- Comprehensive US coverage
- Direct from source (no intermediary)

**NWS Limitations**:
- US-only geographic coverage
- No international alerts
- No custom alerting logic (e.g., "alert me if temp >100F")
- No historical archive >7 days via API

**Commercial APIs** (Weather.com, AccuWeather, etc.):
- Global coverage
- Historical data archives
- Custom alert conditions
- Polished UX/visualizations
- But: Cost, potential delays, not legal-authority alerts

### vs State/Local Alert Systems

**NWS Scope**: National, standardized
**State/Local**: May have additional granular alerts
- Road closures
- School closures
- Local emergency management bulletins

**Integration**: Many state/local systems pull NWS alerts as foundation

---

## Using Coverage Data in Applications

### Geographic Filtering Strategies

#### Strategy 1: State-Level
```
GET /alerts/active/area/TX
```
**Use Case**: State weather app, state dashboards

#### Strategy 2: Zone-Level
```
GET /alerts/active/zone/TXZ253
```
**Use Case**: City-level app, hyper-local notifications

#### Strategy 3: Point-Based
```
GET /alerts/active?point=29.4241,-98.4936
```
**Use Case**: User location-based alerts, mobile apps

#### Strategy 4: Multi-Zone
Fetch multiple zones in parallel (respect rate limits):
```rust
let zones = vec!["TXZ253", "TXZ254", "TXZ255"];
let futures = zones.iter().map(|z| fetch_zone_alerts(z));
let results = join_all(futures).await;
```
**Use Case**: Regional dashboard, multi-city apps

### Coverage Validation

**Before Relying on Coverage**:
1. Check if your target area is within US jurisdiction
2. Identify relevant NWS zones via `/zones` endpoint (separate API)
3. Test alert retrieval for your zones
4. Implement fallback for temporary outages

### Coverage Display

**Inform Users**:
- "Powered by NOAA National Weather Service"
- "Coverage: United States and territories"
- "Data updates every 30-60 seconds"

**Disclaimers**:
- "For emergency purposes, always monitor official channels"
- "API access subject to NWS rate limits"

---

## Related NWS APIs

For complete weather coverage, combine with:

1. **Forecasts**: `/points/{lat},{lon}/forecast`
   - 7-day forecasts, hourly forecasts

2. **Observations**: `/stations/{stationId}/observations`
   - Current conditions, historical observations

3. **Radar**: `/radar/stations/{stationId}`
   - Radar imagery, reflectivity data

4. **Zones**: `/zones`
   - Zone definitions, geometries

5. **Offices**: `/offices/{officeId}`
   - WFO information, contact details

**Documentation**: https://www.weather.gov/documentation/services-web-api

---

## Summary

**Geographic Coverage**: Complete US (50 states + territories + marine regions)

**Alert Types**: All NWS weather hazards (tornados to winter storms)

**Temporal Coverage**: Real-time + 7-day history

**Data Quality**: Official, authoritative, legally recognized

**Limitations**: US-only, no historical archive >7 days, no custom alerting

**Best Use Cases**:
- US weather apps
- Emergency management systems
- Location-based alert notifications
- Government/public safety applications
