# NASA EONET API v3 Overview

## What is EONET?

The Earth Observatory Natural Event Tracker (EONET) is a repository of metadata about natural events. It provides a RESTful API for accessing global natural disaster and environmental event data curated from multiple authoritative sources.

## Base URL

```
https://eonet.gsfc.nasa.gov/api/v3
```

## API Version

**Current stable version**: v3.0

## Key Features

- **No Authentication Required**: Public API, open access to all endpoints
- **Multiple Output Formats**: JSON, GeoJSON, RSS, ATOM
- **Real-time Event Tracking**: Open, closed, and all events
- **Global Coverage**: Events from 33+ authoritative sources worldwide
- **13 Event Categories**: Wildfires, storms, volcanoes, floods, droughts, etc.
- **Rich Metadata**: Geographic coordinates, magnitude, sources, dates
- **Flexible Filtering**: By status, category, source, date range, bounding box

## Primary Use Cases

1. **Natural Disaster Monitoring**: Track active wildfires, storms, volcanoes globally
2. **Historical Analysis**: Query closed events by date range
3. **Geographic Analysis**: Filter events by bounding box
4. **Multi-source Aggregation**: Events from NASA, NOAA, USGS, and 30+ other sources
5. **Visualization**: GeoJSON support for mapping applications

## Data Sources

Events aggregated from 33 sources including:
- **Volcanic**: Alaska Volcano Observatory, Smithsonian Institution
- **Wildfire**: CALFIRE, InciWeb, IRWIN, BCWILDFIRE, etc.
- **Storms**: NOAA National Hurricane Center, Joint Typhoon Warning Center
- **Earthquakes**: USGS Earthquake Hazards Program
- **Floods**: FloodList, Australia BOM
- **General**: GDACS, GLIDE, FEMA, ReliefWeb, PDC

## Event Categories

13 natural event types:
- Wildfires
- Severe Storms
- Volcanoes
- Floods
- Landslides
- Drought
- Dust and Haze
- Snow
- Temperature Extremes
- Sea and Lake Ice
- Water Color
- Manmade
- Earthquakes

## Response Formats

- **JSON**: Standard structured data
- **GeoJSON**: Geographic feature collections
- **RSS**: Feed format
- **ATOM**: Alternative feed format

## Rate Limiting

- Optional NASA API key for intensive use
- Rate limits enforced with automatic 1-hour block when exceeded
- Headers: `X-RateLimit-Limit`, `X-RateLimit-Remaining`
- Specific EONET limits not documented (general NASA API: hourly limits)

## Documentation

- Official Docs: https://eonet.gsfc.nasa.gov/docs/v3
- How-to Guide: https://eonet.gsfc.nasa.gov/how-to-guide
- About EONET: https://eonet.gsfc.nasa.gov/what-is-eonet

## API Characteristics

- RESTful design
- GET requests only
- Query parameters for filtering
- Consistent JSON structure across endpoints
- ISO 8601 timestamps (UTC)
- Coordinates in [longitude, latitude] format
- Null-safe fields (description, closed, magnitude)
