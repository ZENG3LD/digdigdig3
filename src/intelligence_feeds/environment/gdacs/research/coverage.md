# GDACS Coverage

## Geographic Coverage

**GDACS provides global coverage for all disaster types.**

### Scope
- **Global**: All countries and territories
- **Oceans**: Earthquakes, tsunamis, tropical cyclones
- **Remote Areas**: Automated systems detect events regardless of location
- **Urban & Rural**: No geographic bias in detection

### Coverage by Disaster Type

| Disaster Type | Geographic Coverage | Notes |
|---------------|---------------------|-------|
| **Earthquake** | Global | USGS NEIC monitors worldwide |
| **Tropical Cyclone** | All ocean basins | Atlantic, Pacific, Indian Ocean |
| **Tsunami** | Coastal regions worldwide | Pacific, Indian, Atlantic, Caribbean |
| **Flood** | Global (limited) | GLOFAS model coverage + manual reports |
| **Volcano** | Global | Focus on populated areas, aviation routes |
| **Wildfire** | Global | Satellite detection (MODIS, VIIRS) |
| **Drought** | Global | GDO monitors all land areas |

## Regional Specifications

### Tropical Cyclones by Basin

| Basin | Coverage | Data Source | Naming Convention |
|-------|----------|-------------|-------------------|
| North Atlantic | 0-100°W, 0-60°N | NHC | Hurricane (e.g., Maria) |
| East Pacific | 90-180°W, 0-60°N | NHC | Hurricane |
| West Pacific | 100°E-180°, 0-60°N | JTWC | Typhoon (e.g., Haiyan) |
| North Indian | 40-100°E, 0-30°N | IMD | Cyclone (e.g., Amphan) |
| South Indian | 20-100°E, 0-40°S | Multiple | Cyclone |
| South Pacific | 100°E-180°, 0-40°S | Multiple | Cyclone |

**Note**: GDACS aggregates data from multiple regional meteorological agencies.

### Earthquake Coverage

**Detection Threshold**:
- **Global**: All M≥4.5 earthquakes detected by USGS NEIC
- **Alert Threshold**: M≥5.5 (or lower in populated/vulnerable areas)
- **ShakeMap Coverage**: Limited to certain regions (primarily US, Japan, parts of Europe, New Zealand)

**ShakeMap Availability**:
- **High Coverage**: United States, Japan, Taiwan, New Zealand
- **Moderate Coverage**: Europe, parts of Latin America, parts of Asia
- **Low Coverage**: Africa, some developing nations

**Impact**: Without ShakeMaps, GDACS uses simplified magnitude-depth-population model (less accurate).

### Flood Coverage

**GLOFAS Model**:
- **Coverage**: Global river basins
- **Resolution**: ~0.1° (~10km)
- **Focus**: Major rivers and flood-prone regions

**Manual Reporting**:
- Enhanced coverage for major flood events
- Relies on field reports, news, humanitarian agencies
- May have delays in remote areas

### Volcano Coverage

**VAAC Coverage**:
- **9 VAACs globally**: Washington, Anchorage, Montreal, London, Toulouse, Buenos Aires, Darwin, Wellington, Tokyo
- **Focus**: Aviation hazards (ash clouds)
- **Ground Impacts**: Manual assessment, may be incomplete

**Notable Gaps**:
- Remote volcanoes without monitoring stations
- Limited real-time data in some developing countries

### Wildfire Coverage

**GWIS (Satellite Detection)**:
- **Global**: All land areas
- **Satellites**: MODIS, VIIRS, Sentinel
- **Resolution**: 375m-1km
- **Frequency**: Daily updates

**Detection Limitations**:
- Cloud cover may obscure fires
- Small fires (<1 hectare) may not be detected
- Understory fires (beneath canopy) difficult to detect

### Drought Coverage

**GDO Coverage**:
- **Global**: All land areas
- **Data Sources**: Satellite soil moisture, precipitation, vegetation indices
- **Resolution**: ~10km
- **Update Frequency**: Weekly to monthly

## Temporal Coverage

### Real-Time Monitoring

| Disaster Type | Latency | Update Frequency |
|---------------|---------|------------------|
| Earthquake | Seconds-minutes | Immediate |
| Tsunami | Minutes | Immediate |
| Tropical Cyclone | Minutes-hours | Every 6 hours |
| Flood | Hours-days | Daily (forecasts) + manual |
| Volcano | Hours-days | Manual (as reported) |
| Wildfire | Hours | Daily (satellite passes) |
| Drought | Days-weeks | Weekly/monthly |

### Historical Data

**Availability**:
- API provides access to historical events
- No documented retention limit
- Pagination required for large queries (max 100 per request)

**Typical Retention**:
- **Current Events**: Last 24 hours (RSS feed)
- **Recent Events**: Last 7 days (RSS feed)
- **Archived Events**: Last 3 months (RSS feed)
- **All Events**: API access via date filters (fromdate, todate)

**Date Range Queries**:
```
# Events from specific year
https://www.gdacs.org/gdacsapi/api/events/geteventlist/SEARCH?fromdate=2023-01-01&todate=2023-12-31

# Events from specific month
https://www.gdacs.org/gdacsapi/api/events/geteventlist/SEARCH?fromdate=2025-09-01&todate=2025-09-30
```

## Data Quality by Region

### High-Quality Coverage

**Regions**:
- North America (US, Canada)
- Europe (EU countries)
- Japan, South Korea, Taiwan
- Australia, New Zealand

**Characteristics**:
- Dense monitoring networks
- Real-time data sharing
- High-resolution models
- ShakeMaps available (EQ)
- Strong early warning systems

### Moderate-Quality Coverage

**Regions**:
- Latin America (major countries)
- China, India, Southeast Asia
- Middle East (parts)
- South Africa

**Characteristics**:
- Regional monitoring networks
- Some real-time data
- Model-based assessments
- May lack detailed impact data

### Lower-Quality Coverage

**Regions**:
- Sub-Saharan Africa (parts)
- Central Asia
- Remote Pacific islands
- Conflict zones

**Characteristics**:
- Limited ground monitoring
- Satellite-based detection only
- Delayed manual reports
- May miss localized events

**Note**: Automated systems (EQ, TC) maintain global quality. Manual systems (FL, VO) have regional variations.

## Alert Coverage by Severity

### Alert Level Distribution

Based on humanitarian impact thresholds:

**Green Alerts** (Minor Impact):
- Often excluded from GDACS feeds (check alertlevel parameter)
- Local response sufficient
- Lower priority for international monitoring

**Orange Alerts** (Moderate Impact):
- National response, possible international support
- Focus of GDACS monitoring
- Regular updates and reports

**Red Alerts** (Major Impact):
- International response likely needed
- Highest priority for GDACS
- Detailed impact assessments
- Frequent updates

**Recommendation**: Filter API requests with `alertlevel=orange;red` to focus on actionable events.

## Population-Based Coverage Bias

### Automated Systems

**Earthquake & Tropical Cyclone**:
- Alert levels consider population exposure
- Events in populated areas more likely to trigger alerts
- Remote events (oceans, deserts) may be Green despite high magnitude/intensity

**Example**:
- M6.5 earthquake in desert: Green alert (low population)
- M6.5 earthquake in city: Orange/Red alert (high population)

**Implication**: GDACS focuses on **humanitarian impact**, not absolute disaster magnitude.

### Wildfire & Drought

**Wildfire**:
- All fires detected by satellite
- Alerts prioritize fires threatening populated areas
- Remote forest fires may not generate alerts

**Drought**:
- All regions monitored
- Alerts focus on agricultural/food security impacts
- Population-dependent severity assessment

## Disaster Type Coverage Summary

### Fully Automated (High Global Coverage)

| Disaster | Detection | Coverage | Quality |
|----------|-----------|----------|---------|
| Earthquake | USGS NEIC | Global | High |
| Tropical Cyclone | Met agencies | All basins | High |
| Tsunami | Warning centers | Coastal | High |
| Wildfire | Satellite | Global land | Moderate |
| Drought | Satellite | Global land | Moderate |

### Manual/Hybrid (Variable Coverage)

| Disaster | Detection | Coverage | Quality |
|----------|-----------|----------|---------|
| Flood | GLOFAS + manual | Global (uneven) | Moderate |
| Volcano | VAAC + manual | Global (focus areas) | Moderate |

## Limitations and Gaps

### Known Limitations

1. **Flood Coverage**:
   - GLOFAS model has regional biases
   - Flash floods may not be detected
   - Urban flooding often missed
   - Manual reporting delays

2. **Volcano Coverage**:
   - Focus on aviation hazards (ash clouds)
   - Ground impacts may be incomplete
   - Remote volcanoes less monitored
   - No global eruption catalog integration

3. **Wildfire Coverage**:
   - Cloud cover obscures satellite detection
   - Small fires missed
   - Understory fires invisible
   - Daily update lag (not real-time)

4. **Earthquake Coverage**:
   - ShakeMap gaps in developing countries
   - Simplified models for regions without ShakeMaps
   - May underestimate or overestimate impacts

5. **Drought Coverage**:
   - Weekly/monthly updates (slow-onset disaster)
   - Model-based, requires validation
   - Food security data lags

### Geographic Gaps

**Least Covered**:
- Conflict zones (limited data sharing)
- Failed states (no national reporting)
- Extremely remote areas (no ground truth)
- International waters (except TC, TS)

**Most Covered**:
- Developed nations (dense networks)
- Population centers (high priority)
- International aviation routes (volcano ash)
- Major river basins (flood models)

## Data Sources and Partners

### Primary Data Providers

| Disaster | Source | Organization |
|----------|--------|--------------|
| Earthquake | NEIC | USGS (United States Geological Survey) |
| Earthquake (ShakeMap) | USGS | ShakeMap system |
| Tropical Cyclone | Multiple | NHC, JTWC, IMD, regional met agencies |
| Tsunami | PTWC, JMA | Pacific Tsunami Warning Center, Japan Met Agency |
| Flood | GLOFAS | Global Flood Awareness System (ECMWF) |
| Volcano | VAAC | Volcanic Ash Advisory Centers (9 global) |
| Wildfire | GWIS | Global Wildfire Information System (Copernicus) |
| Drought | GDO | Global Drought Observatory (JRC) |

### Coordination Partners

- **UN OCHA**: United Nations Office for Coordination of Humanitarian Affairs
- **JRC**: Joint Research Centre (European Commission)
- **WMO**: World Meteorological Organization
- **IFRC**: International Federation of Red Cross and Red Crescent Societies

## Coverage Recommendations

### For Global Monitoring Applications

**Strengths**:
- ✅ Earthquakes (M≥5.5)
- ✅ Tropical cyclones (all basins)
- ✅ Tsunamis (coastal areas)
- ✅ Major floods (Orange/Red alerts)
- ✅ Significant wildfires
- ✅ Droughts (regional trends)

**Supplement with**:
- Local/national disaster agencies for detailed data
- USGS Earthquake Hazards Program for comprehensive EQ data
- National Hurricane Center for Atlantic TC details
- National flood warning systems for real-time flood data

### For Regional Applications

**Developed Regions** (US, EU, Japan):
- GDACS provides good overview
- Use local systems for actionable alerts (USGS, NHC, JMA)

**Developing Regions**:
- GDACS may be primary source for automated events (EQ, TC)
- Supplement with local news, humanitarian reports for FL, VO

### For Specific Disaster Types

**Earthquake**: GDACS + USGS Earthquake API
**Tropical Cyclone**: GDACS + NHC/JTWC APIs
**Flood**: GDACS + national hydrology services
**Volcano**: GDACS + VAAC advisories + USGS Volcano Hazards Program
**Wildfire**: GDACS + national fire services (e.g., NIFC in US)
**Drought**: GDACS + FAO food security data

## Future Coverage Improvements

### Planned Enhancements (from documentation)

1. **Storm Surge Alerts**: TC storm surge modeling (not yet in alertscore)
2. **Improved Flood Coverage**: Enhanced GLOFAS integration
3. **Real-time Wildfire**: More frequent updates
4. **Expanded ShakeMap Coverage**: More countries

### Community Contributions

GDACS encourages:
- Field reports from humanitarian organizations
- Crowdsourced impact data (via ReliefWeb, HDX)
- Satellite imagery analysis (post-disaster)

## Summary

- **Global coverage** for all disaster types (7 types monitored)
- **Best coverage**: Earthquakes, tropical cyclones, tsunamis (automated systems)
- **Variable coverage**: Floods, volcanoes (manual + model-based)
- **Population-focused**: Alerts prioritize humanitarian impact over absolute magnitude
- **Data quality**: Highest in developed nations, moderate to lower in developing regions
- **Historical data**: API access via date filters, no documented retention limits
- **Real-time latency**: Seconds (EQ) to days (DR, FL manual reports)
- **Complementary sources recommended**: For detailed local/regional data
