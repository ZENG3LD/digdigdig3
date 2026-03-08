# GDACS Data Types and Classifications

## Disaster Types

GDACS monitors seven disaster types, identified by two-letter codes:

| Code | Disaster Type | Automated | Data Source | Update Frequency |
|------|---------------|-----------|-------------|------------------|
| **EQ** | Earthquake | Yes | USGS NEIC | Real-time (seconds-minutes) |
| **TC** | Tropical Cyclone | Yes | Multiple met agencies | Every 6 hours (forecasts) |
| **TS** | Tsunami | Yes | PTWC, JMA, others | Real-time (minutes) |
| **FL** | Flood | No | GLOFAS | Daily (model runs) + manual |
| **VO** | Volcano | No | VAAC (DARWIN, TOKYO) | Manual (hours-days) |
| **WF** | Wildfire | Yes | GWIS | Daily |
| **DR** | Drought | Yes | GDO | Weekly/monthly |

### Disaster Type Details

#### EQ - Earthquake
**Description**: Seismic events with potential humanitarian impact

**Data Sources**:
- USGS National Earthquake Information Center (NEIC)
- USGS ShakeMaps (MMI intensity data)

**Key Metrics**:
- **Magnitude**: Moment magnitude scale (Mw) or Richter scale (RS)
- **Depth**: Hypocentre depth in km
- **Intensity**: Modified Mercalli Intensity (MMI) scale (I-XII)
- **Population Exposure**: People exposed at each MMI level (VII+)
- **Shakemap**: Spatial distribution of ground shaking

**Alert Criteria**:
- Magnitude ≥ 5.5 (or lower in populated/vulnerable areas)
- Population exposure at MMI ≥ VII (Very Strong Shaking)
- Country vulnerability factor (INFORM Index)
- Casualty estimates based on historical data

**Example Severity**:
- "Magnitude 7.2M earthquake at 15km depth"
- "15,000 people exposed to MMI VIII-IX shaking"

#### TC - Tropical Cyclone
**Description**: Hurricanes, typhoons, cyclones with wind and storm surge hazards

**Data Sources**:
- NHC (National Hurricane Center) - Atlantic
- JTWC (Joint Typhoon Warning Center) - Pacific
- IMD (India Meteorological Department) - Indian Ocean
- Multiple regional meteorological agencies

**Key Metrics**:
- **Wind Speed**: 1-minute sustained winds (kt or km/h)
- **Category**: Saffir-Simpson Hurricane Wind Scale (Cat 1-5)
- **Track**: Forecast path with uncertainty cone
- **Storm Surge**: Estimated surge height (meters)
- **Affected Population**: People in forecast path

**Saffir-Simpson Scale**:
| Category | Wind Speed (kt) | Wind Speed (km/h) | Damage Potential |
|----------|-----------------|-------------------|------------------|
| 1 | 64-82 | 119-153 | Minimal |
| 2 | 83-95 | 154-177 | Moderate |
| 3 | 96-112 | 178-208 | Extensive |
| 4 | 113-136 | 209-251 | Extreme |
| 5 | ≥137 | ≥252 | Catastrophic |

**GDACS Classifications**:
- Tropical Depression: <34 kt
- Tropical Storm: 34-63 kt
- Severe Tropical Storm: ~100-150 km/h (varies by agency)
- Hurricane/Typhoon/Cyclone: ≥64 kt

**Alert Criteria**:
- Wind speed (hazard)
- Population in forecast path
- Country vulnerability
- **Special Rule**: Red alerts >3 days in advance downgraded to Orange (reduce false alarms)

**Example Severity**:
- "Severe Tropical Storm (maximum wind speed of 211 km/h)"
- "Category 4 hurricane approaching populated coast"

#### FL - Flood
**Description**: Riverine floods, flash floods, coastal floods

**Data Sources**:
- GLOFAS (Global Flood Awareness System)
- Manual input from field reports

**Key Metrics**:
- **Affected Area**: Square kilometers flooded
- **Population Displaced**: Number of people evacuated/affected
- **Casualty Estimates**: Dead and missing
- **Duration**: Days/weeks of flooding

**Alert Criteria**:
- **Red**: >1,000 dead OR >800,000 displaced
- **Orange**: >100 dead OR >80,000 displaced
- **Green**: All other floods

**Example Severity**:
- "Flooding affecting 250,000 people"
- "85 casualties reported, 150,000 displaced"

#### VO - Volcano
**Description**: Volcanic eruptions with aviation hazards and ground impacts

**Data Sources**:
- VAAC (Volcanic Ash Advisory Centers): DARWIN, TOKYO, others
- Manual input from volcanological observatories

**Key Metrics**:
- **Eruption Status**: Active, dormant, extinct
- **Ash Cloud Height**: Altitude in feet/meters
- **Aviation Alert Level**: Color code (Green, Yellow, Orange, Red)
- **Affected Population**: People in hazard zones (lava, ash, pyroclastic flows)
- **Exclusion Zones**: Radius in km

**Alert Criteria**:
- Not explicitly documented (manual assessment)
- Based on eruption intensity, population proximity, ash cloud extent

**Example Severity**:
- "Eruption with ash cloud to 30,000 feet"
- "5,000 people evacuated from 10km exclusion zone"

#### WF - Wildfire / Forest Fire
**Description**: Large-scale wildfires threatening populated areas

**Data Sources**:
- GWIS (Global Wildfire Information System)
- Satellite detection (MODIS, VIIRS)

**Key Metrics**:
- **Burned Area**: Hectares (ha) or square kilometers
- **Active Fire Perimeter**: Current fire boundary
- **Duration**: Days since detection
- **Population Threatened**: People in affected areas
- **Structures Threatened**: Buildings, infrastructure

**Alert Criteria**:
- Not explicitly documented
- Based on burned area, rate of spread, population proximity

**Example Severity**:
- "6,044 hectares burned"
- "200 people affected, ongoing"

#### DR - Drought
**Description**: Agricultural and hydrological drought with food security impacts

**Data Sources**:
- GDO (Global Drought Observatory)
- Satellite soil moisture, vegetation indices

**Key Metrics**:
- **Affected Area**: Square kilometers in drought condition
- **Intensity**: CDI (Combined Drought Indicator), SPI (Standardized Precipitation Index)
- **Impact Type**: Agricultural, hydrological, meteorological
- **Duration**: Months/years of drought conditions
- **Population Affected**: People in food-insecure regions

**CDI Levels**:
- Watch (yellow): Developing drought
- Warning (orange): Moderate drought
- Alert (red): Severe drought

**Alert Criteria**:
- Based on CDI, affected area, population impact

**Example Severity**:
- "Medium impact for agricultural drought in 788110 km²"
- "Severe drought affecting 2 million people"

#### TS - Tsunami
**Description**: Ocean waves generated by earthquakes, landslides, volcanoes

**Data Sources**:
- PTWC (Pacific Tsunami Warning Center)
- JMA (Japan Meteorological Agency)
- Regional tsunami warning centers

**Key Metrics**:
- **Wave Height**: Estimated maximum height (meters)
- **Origin**: Earthquake epicenter, landslide location
- **Travel Time**: Minutes/hours to reach coastlines
- **Affected Coastlines**: Countries/regions threatened
- **Magnitude**: Source earthquake magnitude

**Alert Criteria**:
- Earthquake magnitude ≥7.0 (shallow, under ocean)
- Predicted wave height >1 meter
- Population in coastal zones

**Example Severity**:
- "Tsunami waves 1-3 meters predicted for Pacific coasts"
- "Generated by M8.2 earthquake, arrival in 2 hours"

## Alert Levels

GDACS uses a three-tier color-coded alert system:

### Alert Level Definitions

| Alert Level | Color | Meaning | Humanitarian Response |
|-------------|-------|---------|----------------------|
| **Green** | 🟢 | Minor impact | Local response sufficient |
| **Orange** | 🟠 | Moderate impact | National response, possible int'l support |
| **Red** | 🔴 | Major impact | International response likely needed |

### Alert Level Criteria

Alert levels are calculated based on a **risk matrix**:
```
Alert Level = f(Hazard, Exposure, Vulnerability)
```

**Components**:
1. **Hazard**: Disaster intensity (magnitude, wind speed, area)
2. **Exposure**: Population in affected area
3. **Vulnerability**: Country coping capacity (INFORM Index)

### Earthquake Alert Calculation

**Pre-September 2017** (simplified):
- Magnitude
- Depth
- Population within radii from epicenter
- Distance weighting (closer = higher weight)

**Post-September 2017** (enhanced):
- **ShakeMap-based**: MMI intensity grid
- **Population exposure**: People at each MMI level (VII-XII)
- **Casualty calibration**: Based on historical data since 2006
- **Vulnerability factor**: Country-specific INFORM Index
- **Coping capacity**: National disaster response capability

**MMI Shaking Intensity**:
| MMI | Shaking | Damage | Example Alert Contribution |
|-----|---------|--------|---------------------------|
| VII | Very Strong | Considerable damage | Moderate |
| VIII | Severe | Severe damage | High |
| IX | Violent | Widespread severe damage | Very High |
| X-XII | Extreme | Total destruction | Critical |

**Alert Thresholds** (not exact, empirical):
- **Green**: Few casualties expected, local response adequate
- **Orange**: 100-1000 casualties possible, national response needed
- **Red**: >1000 casualties possible, international assistance likely

**Example**:
- M6.5 shallow earthquake in developed country with resilient infrastructure: **Orange**
- M7.0 shallow earthquake in vulnerable country with dense population: **Red**

### Tropical Cyclone Alert Calculation

**Formula**:
```
Alert = f(Wind Speed, Population in Path, Vulnerability, Forecast Uncertainty)
```

**Wind Speed Component**:
- Saffir-Simpson category (1-5)
- 1-minute sustained winds

**Exposure Component**:
- Population in forecast cone (24h, 48h, 72h)
- Urban vs. rural areas
- Coastal vs. inland

**Vulnerability Component**:
- Building codes and construction quality
- Early warning systems
- Evacuation infrastructure
- Country INFORM Index

**Special Rules**:
- **Red alerts >3 days out**: Downgraded to Orange (forecast uncertainty)
- **Storm surge not yet included**: Currently wind-only (as of 2026)

### Flood Alert Thresholds

**Explicit Thresholds**:
- **Red**: >1,000 dead OR >800,000 displaced
- **Orange**: >100 dead OR >80,000 displaced
- **Green**: All other floods

**Note**: GLOFAS provides forecasts, but GDACS alerts often include field-reported data (manual updates).

### Volcano Alert Criteria

**Not explicitly documented**. Likely factors:
- Eruption intensity (VEI - Volcanic Explosivity Index)
- Ash cloud height and extent
- Population in exclusion zones
- Aviation hazards (international flights)
- Lava flows and pyroclastic flows

### Wildfire Alert Criteria

**Not explicitly documented**. Likely factors:
- Burned area (hectares)
- Rate of spread
- Population threatened
- Structures threatened
- Air quality impacts

### Drought Alert Criteria

**CDI-based** (Combined Drought Indicator):
- **Watch**: Early warning, developing conditions
- **Warning** (Orange): Moderate impacts, agricultural losses
- **Alert** (Red): Severe impacts, food security crisis

**Factors**:
- Affected area (km²)
- Duration (months)
- Population in food-insecure regions
- Agricultural production losses

## Alert Scores

**Numeric Score**: `alertscore` field (0.0-3.0)

**Mapping** (approximate):
- **0.0-0.9**: Green
- **1.0-1.9**: Orange
- **2.0-3.0**: Red

**Purpose**:
- Granular severity within alert levels
- Sorting events by severity
- Programmatic thresholds

**Example**:
```json
{
  "alertlevel": "Orange",
  "alertscore": 1.5
}
```

## Episode System

**Episodes**: Multiple alerts for the same ongoing event

**Fields**:
- `eventid`: Unique event identifier (unchanging)
- `episodeid`: Incremented with each update
- `episodealertlevel`: Alert level for this specific episode
- `episodealertscore`: Score for this episode

**Use Cases**:
- **Tropical Cyclones**: Track progression (forecast updates every 6 hours)
- **Floods**: Update as situation develops (more rain, more displaced)
- **Volcanoes**: Multiple eruption phases
- **Wildfires**: Growing fire perimeter

**Example**:
```
Event: Cyclone GEZANI-26
  Episode 1: Orange (forecast 5 days out)
  Episode 2: Orange (forecast 3 days out)
  Episode 3: Red (forecast 1 day out, upgraded)
  Episode 21: Orange (post-landfall, downgraded)
```

## GLIDE Numbers

**GLIDE** = Global Identifier for Disasters

**Format**: `{TYPE}-{YEAR}-{SEQUENCE}-{ISO3}`

**Examples**:
- `EQ-2026-000123-IDN`: Earthquake in Indonesia
- `TC-2026-000005-MOZ`: Tropical Cyclone in Mozambique
- `WF-2026-000003-CHL`: Wildfire in Chile
- `FL-2025-000089-BGD`: Flood in Bangladesh

**Purpose**:
- Globally unique disaster identifier
- Cross-system referencing (OCHA, humanitarian databases)
- Long-term tracking and analysis

**Not all events have GLIDE numbers**: Some minor events may lack them.

## Severity Data Structure

**Field**: `severitydata` (object)

**Common Properties**:
```json
{
  "severity": 7.2,
  "severitytext": "Magnitude 7.2M earthquake at 15km depth",
  "severityunit": "M"
}
```

### Severity Units by Disaster Type

| Disaster Type | Unit | Example |
|---------------|------|---------|
| Earthquake | `M` (Magnitude) | 7.2 M |
| Tropical Cyclone | `km/h` or `kt` | 211 km/h |
| Flood | (varies) | "85 casualties, 150,000 displaced" |
| Volcano | (descriptive) | "Ash cloud to 30,000 ft" |
| Wildfire | `ha` or `km²` | 6,044 ha |
| Drought | `km²` | 788,110 km² |

### Severity Text Patterns

**Earthquake**:
```
"Magnitude {MAG}M earthquake at {DEPTH}km depth"
"Magnitude 6.5RS earthquake at 10km depth, {POPULATION} people in MMI>=III"
```

**Tropical Cyclone**:
```
"Severe Tropical Storm (maximum wind speed of {SPEED} km/h)"
"Category {CAT} hurricane with winds of {SPEED} kt"
```

**Flood**:
```
"{CASUALTIES} casualties, {DISPLACED} displaced"
"Flooding affecting {POPULATION} people"
```

**Wildfire**:
```
"{AREA} hectares burned, {POPULATION} people affected"
```

**Drought**:
```
"Medium impact for agricultural drought in {AREA} km2"
"Severe drought affecting {POPULATION} people"
```

## Coordinate Systems

**Standard**: WGS84 (EPSG:4326)

**Format**: [longitude, latitude]

**GeoJSON Point**:
```json
{
  "type": "Point",
  "coordinates": [154.5612, 48.3271]
}
```

**Bounding Box** (bbox):
```json
{
  "bbox": [min_lon, min_lat, max_lon, max_lat]
}
```

**For Tropical Cyclones**: Track line (LineString) or forecast cone (Polygon) available via `url.geometry`

## Country Identification

**Fields**:
- `country`: Human-readable name (e.g., "Indonesia, Philippines")
- `iso3`: ISO 3166-1 alpha-3 code (e.g., "IDN")
- `iso2`: ISO 3166-1 alpha-2 code (e.g., "ID")

**Affected Countries Array**:
```json
{
  "affectedcountries": [
    {
      "iso2": "ID",
      "iso3": "IDN",
      "countryname": "Indonesia"
    },
    {
      "iso2": "PH",
      "iso3": "PHL",
      "countryname": "Philippines"
    }
  ]
}
```

## Temporal Data

**Timestamp Format**: ISO 8601 with timezone

**Key Fields**:
- `fromdate`: Event start (e.g., "2026-02-06T00:00:00+00:00")
- `todate`: Event end or last update
- `datemodified`: Last modification timestamp

**Event Lifecycle**:
1. `fromdate`: Disaster occurrence or detection
2. `todate`: Current end time (may be extended for ongoing events)
3. `datemodified`: Last data update (episodes, severity, casualties)

**For Forecasts** (Tropical Cyclones):
- `fromdate`: Current position time
- `todate`: Forecast end (dissipation or landfall + 24h)

## Data Quality Indicators

**Field**: `iscurrent` (boolean)
- `true`: Event is active/current
- `false`: Historical or archived event

**Source Reliability**:
- **Automated** (EQ, TC): High confidence, quantitative
- **Manual** (FL, VO): Medium confidence, may be delayed or incomplete
- **Model-based** (DR): Medium confidence, requires validation

## Summary Table

| Disaster | Code | Key Metric | Unit | Alert Thresholds | Automation |
|----------|------|------------|------|------------------|------------|
| Earthquake | EQ | Magnitude | M | MMI, population, vulnerability | Yes |
| Tropical Cyclone | TC | Wind Speed | km/h, kt | Cat 1-5, population, vulnerability | Yes |
| Flood | FL | Casualties/Displaced | count | >100 dead OR >80k displaced (Orange) | No |
| Volcano | VO | Ash Height | ft, m | Manual assessment | No |
| Wildfire | WF | Burned Area | ha, km² | Area, population | Yes |
| Drought | DR | Affected Area | km² | CDI, duration, population | Yes |
| Tsunami | TS | Wave Height | m | >1m, coastal population | Yes |

**Alert Levels**: Green (minor), Orange (moderate), Red (major)

**Data Sources**: USGS, GLOFAS, GWIS, GDO, VAAC, PTWC, meteorological agencies

**Update Frequency**: Real-time (EQ, TS) to weekly/monthly (DR)
