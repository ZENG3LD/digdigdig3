# GDACS API Overview

## What is GDACS?

**Global Disaster Alert and Coordination System (GDACS)** is a cooperation framework under the United Nations providing real-time alerts and impact estimations for natural disasters worldwide. Operated by the European Commission's Joint Research Centre (JRC) and the United Nations Office for Coordination of Humanitarian Affairs (UN OCHA).

## Purpose

GDACS provides:
- Near real-time disaster alerts with humanitarian impact potential
- Automated impact calculations for earthquakes, tsunamis, and tropical cyclones
- Manual monitoring for floods and volcanic eruptions
- Population exposure and vulnerability assessments
- Coordination support for disaster response

## API Access Methods

### 1. JSON/GeoJSON API (Recommended)
- **Base URL**: `https://www.gdacs.org/gdacsapi/api/`
- **Format**: GeoJSON FeatureCollection
- **Authentication**: None (public API)
- **Primary Endpoint**: `/events/geteventlist/SEARCH`

### 2. RSS/XML Feeds
- **Base URL**: `https://www.gdacs.org/xml/`
- **Format**: RSS 2.0 with GDACS/GeoRSS extensions
- **Update Frequency**: Every 6 minutes
- **Multiple feeds**: Time-based and disaster-specific

### 3. Other Formats
- **KML**: `http://www.gdacs.org/kml.aspx`
- **CAP (Common Alerting Protocol)**: Available per event
- **Shapefile**: Available via Python library

## Data Update Frequency

- **RSS Feeds**: Updated every 6 minutes
- **API Endpoints**: Real-time (no stated lag)
- **Automated Calculations**: Immediate for EQ, TC, TS
- **Manual Events**: Variable lag for FL, VO

## Disaster Types Monitored

| Code | Disaster Type | Automated |
|------|---------------|-----------|
| EQ | Earthquake | Yes |
| TC | Tropical Cyclone | Yes |
| TS | Tsunami | Yes |
| FL | Flood | No (manual) |
| VO | Volcano | No (manual) |
| WF | Wildfire | Yes |
| DR | Drought | Yes |

## Key Features

### Alert System
- **Three-tier alerts**: Red, Orange, Green
- **Risk-based**: Considers hazard + population + vulnerability
- **Country-specific**: Uses INFORM Index for coping capacity
- **Empirical**: Fast rough estimates, not precise predictions

### Impact Modeling
- Population exposure calculations
- Vulnerability assessments
- Affected area estimations
- Casualty and displacement predictions

### Data Sources
- **Earthquakes**: USGS NEIC, shakemaps, MMI calculations
- **Tropical Cyclones**: Multiple meteorological agencies, Saffir-Simpson scale
- **Floods**: GLOFAS (Global Flood Awareness System)
- **Wildfires**: GWIS (Global Wildfire Information System)
- **Volcanoes**: DARWIN, TOKYO VAACs
- **Droughts**: GDO (Global Drought Observatory)

## Use Cases

1. **Disaster Monitoring**: Real-time tracking of global disasters
2. **Early Warning Systems**: Integration into alerting platforms
3. **Humanitarian Response**: Impact assessment for aid coordination
4. **Risk Analysis**: Historical disaster data analysis
5. **Data Feeds**: Integration into mapping/GIS applications

## Documentation Resources

- **Swagger UI**: `https://www.gdacs.org/gdacsapi/swagger/index.html`
- **API Quick Start**: `https://www.gdacs.org/Documents/2025/GDACS_API_quickstart_v2.pdf`
- **Feed Reference**: `https://www.gdacs.org/feed_reference.aspx`
- **Terms of Use**: `https://www.gdacs.org/documents/2025/GDACS_Terms_of_use_Mar_25.pdf`

## Important Notes

### Disclaimer
GDACS alerts are **purely indicative** and should not be used for decision making without alternate sources of information. The system prioritizes awareness and coordination rather than precise forecasting.

### Data Reliability
- Automated alerts are empirical and provide fast rough warnings
- Cannot always reliably predict humanitarian impact
- Alert levels may change as more data becomes available
- Red alerts >3 days in advance are downgraded to Orange to reduce false alarms

### Coverage
- **Global**: All countries and regions
- **24/7 Monitoring**: Continuous operation
- **Historical Data**: Events stored for analysis (pagination required for large queries)

## Technical Characteristics

- **No Authentication Required**: Fully public API
- **Rate Limits**: Not explicitly documented (use responsibly)
- **Response Format**: Standard GeoJSON with GDACS extensions
- **Pagination**: Max 100 records per request (use `pagenumber` and `pagesize` parameters)
- **Date Filtering**: ISO 8601 format (YYYY-MM-DD)
- **CORS**: Not documented (likely restricted)

## License and Attribution

Access governed by GDACS Terms of Use. Always attribute data to GDACS and underlying data sources (USGS, GLOFAS, GWIS, etc.).
