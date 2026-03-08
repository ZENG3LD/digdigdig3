# NASA EONET Coverage

## Geographic Coverage

### Global Scope

EONET provides **worldwide coverage** of natural events with no geographic restrictions.

**Coverage areas**:
- All continents (including Antarctica)
- All oceans and seas
- Remote islands and territories
- Arctic and Antarctic regions
- Urban and rural areas

**Coordinate system**: WGS84 (World Geodetic System 1984)
- Longitude: -180° to +180°
- Latitude: -90° to +90°

### Regional Distribution

Event frequency varies by region and category:

| Region | Primary Event Types | Key Sources |
|--------|---------------------|-------------|
| **North America** | Wildfires, severe storms, floods | CALFIRE, InciWeb, IRWIN, NOAA_NHC, FEMA |
| **Pacific Ring of Fire** | Volcanoes, earthquakes | SIVolcano, AVO, USGS_EHP, JTWC |
| **Atlantic/Caribbean** | Hurricanes, severe storms | NOAA_NHC, GDACS |
| **Asia-Pacific** | Typhoons, floods, earthquakes | JTWC, AU_BOM, GDACS |
| **Antarctica** | Sea ice, icebergs | NATICE, BYU_ICE |
| **Global** | All event types | GDACS, ReliefWeb, NASA_DISP |

### Bounding Box Queries

Filter events by geographic area:

```
# California
GET /api/v3/events?bbox=-124.4,32.5,-114.1,42.0

# Australia
GET /api/v3/events?bbox=113.0,-44.0,154.0,-10.0

# Atlantic Hurricane Basin
GET /api/v3/events?bbox=-100.0,5.0,-20.0,45.0&category=severeStorms
```

**Format**: `bbox=min_lon,min_lat,max_lon,max_lat`

---

## Event Type Coverage

### Complete Event Categories (13 Total)

EONET tracks **13 distinct natural event categories**:

#### 1. Wildfires
- **Description**: Forest fires, grass fires, brush fires, urban fires
- **Global coverage**: All continents except Antarctica
- **Primary regions**: North America (US, Canada), Australia, Mediterranean, South America, Africa
- **Data sources**: InciWeb, CALFIRE, IRWIN, BCWILDFIRE, DFES_WA, MBFIRE, ABFIRE
- **Update frequency**: Hourly to daily
- **Magnitude units**: acres, NM² (nautical square miles)
- **Typical count**: 50-200 open events during fire season

#### 2. Severe Storms
- **Description**: Hurricanes, typhoons, cyclones, tornadoes, tropical storms
- **Global coverage**: All ocean basins, continental regions
- **Primary regions**: Atlantic, Pacific, Indian Ocean, Caribbean, Southeast Asia
- **Data sources**: NOAA_NHC, JTWC, GDACS, NASA_HURR, UNISYS
- **Update frequency**: Hourly (during active storms)
- **Magnitude units**: kts (wind speed), mb (pressure), mph
- **Typical count**: 5-30 open events (seasonal)

#### 3. Volcanoes
- **Description**: Volcanic eruptions, ash plumes, lava flows
- **Global coverage**: Pacific Ring of Fire, Mediterranean, Caribbean, Iceland, Hawaii
- **Primary regions**: Indonesia, Philippines, Japan, Alaska, Central America, Kamchatka
- **Data sources**: SIVolcano (Smithsonian), AVO (Alaska Volcano Observatory)
- **Update frequency**: Daily
- **Magnitude units**: VEI (Volcanic Explosivity Index)
- **Typical count**: 20-40 active volcanoes

#### 4. Floods
- **Description**: River flooding, flash floods, coastal flooding, inundation
- **Global coverage**: All continents
- **Primary regions**: Monsoon regions (Asia), major river basins, coastal areas
- **Data sources**: FloodList, AU_BOM, GDACS, ReliefWeb
- **Update frequency**: Daily
- **Magnitude units**: km² (area), sometimes null
- **Typical count**: 20-60 events

#### 5. Landslides
- **Description**: Mudslides, rockslides, avalanches, debris flows
- **Global coverage**: Mountainous regions, coastal cliffs, earthquake zones
- **Primary regions**: Himalayas, Andes, Alps, Pacific Northwest, Southeast Asia
- **Data sources**: GDACS, ReliefWeb, IDC
- **Update frequency**: Daily
- **Magnitude units**: Usually null
- **Typical count**: 5-20 events

#### 6. Drought
- **Description**: Long-lasting precipitation deficits affecting agriculture
- **Global coverage**: All continents (except Antarctica)
- **Primary regions**: Africa (Sahel), Middle East, Australia, Southwest US, South America
- **Data sources**: GDACS, ReliefWeb, NOAA_CPC
- **Update frequency**: Weekly to monthly
- **Magnitude units**: Usually null
- **Typical count**: 10-30 events (slow-evolving)

#### 7. Dust and Haze
- **Description**: Dust storms, sandstorms, air pollution, non-volcanic aerosols
- **Global coverage**: Arid/semi-arid regions globally
- **Primary regions**: Sahara, Middle East, Central Asia, Australia, Southwest US
- **Data sources**: NASA_DISP, EO, Earthdata
- **Update frequency**: Daily
- **Magnitude units**: Usually null
- **Typical count**: 10-40 events

#### 8. Snow
- **Description**: Extreme or anomalous snowfall events
- **Global coverage**: Mid-to-high latitudes, mountainous regions
- **Primary regions**: North America, Europe, Asia, high-altitude tropics
- **Data sources**: NASA_DISP, EO, GDACS
- **Update frequency**: Daily (seasonal)
- **Magnitude units**: Usually null
- **Typical count**: 5-20 events (winter months)

#### 9. Temperature Extremes
- **Description**: Heat waves, cold snaps, anomalous temperatures
- **Global coverage**: All continents
- **Primary regions**: Variable (follows seasonal patterns)
- **Data sources**: GDACS, NOAA_CPC, ReliefWeb
- **Update frequency**: Daily to weekly
- **Magnitude units**: Usually null (qualitative)
- **Typical count**: 10-30 events

#### 10. Sea and Lake Ice
- **Description**: Sea ice extent, icebergs, lake ice
- **Global coverage**: Polar regions (Arctic, Antarctic), high-latitude lakes
- **Primary regions**: Antarctica, Arctic Ocean, Great Lakes, Baltic Sea
- **Data sources**: NATICE, BYU_ICE
- **Update frequency**: Daily to weekly
- **Magnitude units**: km² (area), or null
- **Typical count**: 50-200 icebergs tracked

#### 11. Water Color
- **Description**: Phytoplankton blooms, algae, sediment, water appearance changes
- **Global coverage**: Coastal waters, lakes, rivers globally
- **Primary regions**: Nutrient-rich coastal zones, large lakes
- **Data sources**: EO, Earthdata, NASA satellites
- **Update frequency**: Daily (satellite-dependent)
- **Magnitude units**: Usually null
- **Typical count**: 10-40 events

#### 12. Manmade
- **Description**: Human-induced events extreme in extent
- **Global coverage**: All continents
- **Primary regions**: Industrial areas, conflict zones, urban areas
- **Data sources**: GDACS, IDC, ReliefWeb
- **Update frequency**: Variable
- **Magnitude units**: Usually null
- **Typical count**: 5-15 events (rare)

#### 13. Earthquakes
- **Description**: Seismic events of all magnitudes and types
- **Global coverage**: Worldwide, concentrated at tectonic boundaries
- **Primary regions**: Pacific Ring of Fire, Mediterranean, Middle East, Himalayas
- **Data sources**: USGS_EHP, GDACS
- **Update frequency**: Minutes to hours (real-time)
- **Magnitude units**: Richter scale, moment magnitude
- **Typical count**: 100-500 events (depending on magnitude threshold)

**Note**: While EONET includes earthquakes, the project spec mentions GDACS handles earthquakes, so this connector may filter them out or handle separately.

---

## Data Source Coverage

### 33 Authoritative Sources

EONET aggregates events from **33 distinct sources** across multiple organizations:

#### Government Agencies (US)
- **USGS** (Earthquake Hazards Program, CMT)
- **NOAA** (National Hurricane Center, CPC)
- **FEMA** (Federal Emergency Management Agency)
- **CALFIRE** (California Department of Forestry)

#### Government Agencies (International)
- **AU_BOM** (Australia Bureau of Meteorology)
- **BCWILDFIRE** (British Columbia Wildfire Service)
- **DFES_WA** (Western Australia Emergency Services)

#### Research Institutions
- **Smithsonian Institution** (Global Volcanism Program)
- **Alaska Volcano Observatory**
- **BYU_ICE** (Brigham Young University Ice Research)

#### International Organizations
- **GDACS** (Global Disaster Alert Coordination System)
- **ReliefWeb** (UN humanitarian information)
- **GLIDE** (Global Identifier Number for Disasters)

#### NASA Programs
- **NASA_DISP** (Disasters Program)
- **NASA_HURR** (Hurricane tracking)
- **NASA_ESRS** (Earth Science Research)
- **Earthdata** (Earth Observing System)
- **EO** (Earth Observatory)

#### Specialized Systems
- **InciWeb** (Incident Information System - wildfires)
- **IRWIN** (Integrated Reporting of Wildland-Fire Information)
- **JTWC** (Joint Typhoon Warning Center)
- **NATICE** (National Ice Center)
- **FloodList** (Global flood reporting)
- **CEMS** (Copernicus Emergency Management Service)
- **PDC** (Pacific Disaster Center)
- **IDC**, **HDDS**, **MRR** (various disaster systems)

### Source Filtering

Query events from specific sources:

```
# Only CALFIRE wildfires
GET /api/v3/events?source=CALFIRE

# Multiple sources (OR logic)
GET /api/v3/events?source=InciWeb,CALFIRE,IRWIN

# Volcano observatories only
GET /api/v3/events?source=SIVolcano,AVO&category=volcanoes
```

---

## Temporal Coverage

### Historical Data

**Start date**: Approximately **2000** (varies by category)
- Earliest events: ~2000-2005
- Most complete coverage: 2015-present
- No documented end date for historical queries

### Query by Date Range

```
# All 2025 events
GET /api/v3/events?start=2025-01-01&end=2025-12-31

# Events from last 7 days
GET /api/v3/events?days=7

# Events from specific month
GET /api/v3/events?start=2026-02-01&end=2026-02-29
```

### Update Frequency by Category

| Category | Update Frequency | Latency |
|----------|------------------|---------|
| Earthquakes | Minutes | Near real-time |
| Severe Storms | Hourly | 1-3 hours |
| Wildfires | Hourly-Daily | 2-12 hours |
| Volcanoes | Daily | 12-24 hours |
| Floods | Daily | 12-48 hours |
| Sea/Lake Ice | Daily-Weekly | 1-7 days |
| Drought | Weekly-Monthly | 1-4 weeks |
| Other | Daily | 12-48 hours |

**Note**: Update frequency depends on source reporting cadence.

---

## Data Completeness

### Global vs Regional

**Comprehensive coverage**:
- North America (US, Canada)
- Australia
- Pacific Ocean (typhoons, volcanoes)
- Atlantic Ocean (hurricanes)

**Moderate coverage**:
- Europe
- Asia (major events)
- South America

**Variable coverage**:
- Africa (depends on source availability)
- Remote regions
- Some developing countries

**Factors affecting coverage**:
- Source availability in region
- Reporting infrastructure
- Event magnitude/significance threshold

### Magnitude Thresholds

Events typically included if:
- **Wildfires**: > 100 acres (varies by source)
- **Earthquakes**: Usually > M5.0 (major earthquakes)
- **Hurricanes**: Named storms (tropical storm strength+)
- **Volcanoes**: Active eruptions with observable effects
- **Floods**: Significant impact or newsworthy

**Small events**: May not appear in EONET (use source APIs directly for comprehensive coverage)

---

## Excluded Event Types

EONET **does not cover**:
- **Pandemics/epidemics** (health emergencies)
- **Industrial accidents** (unless extreme extent → manmade category)
- **Small-scale incidents** (local fires < threshold)
- **Routine weather** (normal rain, snow, etc.)
- **Astronomical events** (meteor showers, eclipses)
- **Biological events** (unless water color/environmental)

---

## Data Latency

### Real-time vs Curated

EONET is **curated**, not real-time:
- Events are **manually reviewed** before publication
- Multiple sources cross-referenced when possible
- Typical latency: **2-48 hours** from event occurrence
- Earthquakes: Fastest (minutes-hours)
- Droughts: Slowest (weeks)

**Not suitable for**:
- Immediate disaster response (use source APIs directly)
- Sub-hour event detection
- Automated early warning systems

**Suitable for**:
- Event tracking and monitoring
- Historical analysis
- Dashboard visualization
- Multi-source event aggregation

---

## Coverage Summary Table

| Aspect | Coverage |
|--------|----------|
| **Geographic** | Global (all continents, oceans) |
| **Event categories** | 13 natural disaster types |
| **Data sources** | 33 authoritative organizations |
| **Historical** | ~2000 to present (~26 years) |
| **Update frequency** | Minutes to monthly (category-dependent) |
| **Coordinate precision** | ~10m-1m (4-6 decimal places) |
| **Languages** | English (titles, descriptions) |
| **Time zone** | UTC (all timestamps) |
| **Data format** | JSON, GeoJSON |

---

## Use Cases by Coverage Strengths

### Excellent For:
1. **Wildfire tracking** (North America, Australia)
2. **Hurricane/typhoon monitoring** (Atlantic, Pacific basins)
3. **Volcano activity** (Ring of Fire)
4. **Major earthquake tracking** (global)
5. **Sea ice/iceberg tracking** (polar regions)

### Good For:
6. **Flood monitoring** (global major events)
7. **Severe storm tracking** (developed regions)
8. **Multi-source event aggregation**
9. **Historical event analysis**
10. **Geographic event visualization**

### Limited For:
- Small-scale local events
- Real-time disaster response (latency)
- Comprehensive small-magnitude earthquakes
- Events in regions with poor reporting infrastructure

---

## Coverage Gaps

### Known Limitations:
1. **Source-dependent**: Coverage reflects source availability
2. **Curation lag**: Not real-time (2-48hr typical delay)
3. **Magnitude thresholds**: Small events may not appear
4. **Regional disparities**: Better coverage in developed nations
5. **Language**: English-only (international events translated)

### Mitigations:
- Use multiple connectors (EONET + GDACS + source APIs)
- Poll source APIs directly for real-time data
- Combine EONET with regional/national APIs for completeness
- Accept latency for visualization/historical use cases

---

## Data Reliability

### Source Authority Levels

| Tier | Sources | Reliability |
|------|---------|-------------|
| **Highest** | USGS, NOAA, Smithsonian, NASA | Government/institutional |
| **High** | National agencies (AU_BOM, CALFIRE, etc.) | National government |
| **Moderate** | International orgs (GDACS, ReliefWeb) | Aggregated data |
| **Variable** | Specialized systems (FloodList, etc.) | Community-driven |

**Multi-source events**: Higher confidence (cross-validation)

**Single-source events**: Reliability depends on source tier

---

## API Coverage Consistency

### Guaranteed Fields (Always Present):
- `id`, `title`, `link`
- `categories` (1+ items)
- `sources` (1+ items)
- `geometry` (1+ items)
- `geometry[].date`, `geometry[].type`, `geometry[].coordinates`

### Optional Fields (May Be Null):
- `description`
- `closed`
- `geometry[].magnitudeValue`
- `geometry[].magnitudeUnit`

**Implication**: Rust connector must handle `Option<T>` for nullable fields.
