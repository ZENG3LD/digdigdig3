# FAA NASSTATUS API Overview

## Service Name
**National Airspace System Status (NASSTATUS) - Airport Status Information API**

## Purpose
Provides real-time airport delay and status information for major United States airports, including:
- Airport closures
- Ground stops (GS)
- Ground delay programs (GDP)
- Airspace flow programs (AFP)
- Departure delays
- Arrival delays
- Weather conditions
- General airport status

## API Type
Public REST API (no authentication required)

## Base URLs
- **Primary endpoint**: `https://nasstatus.faa.gov/api/airport-status-information`
- **Legacy ASWS endpoint**: `https://soa.smext.faa.gov/asws/api/airport/status/{airportCode}` (connection refused as of Feb 2026)
- **Airport events**: `https://nasstatus.faa.gov/api/airport-events`

## Response Formats
- **Primary format**: XML (Content-Type: application/xml)
- **Alternative format**: JSON (request via HTTP Accept header)

## Data Source
Composite data aggregated from:
- FAA fly.faa.gov system
- National Weather Service
- FAA Air Traffic Control System Command Center
- NOTAM (Notice to Airmen) system

## Coverage
- Major United States airports (IATA/ICAO codes)
- Primarily large hub and medium hub airports
- Limited coverage of smaller regional airports

## Use Cases
- Real-time airport delay monitoring
- Flight planning and route optimization
- Airport operations dashboard
- Travel advisory systems
- Aviation weather integration
- Traffic management awareness

## API Characteristics
- **Access**: Public, no authentication
- **Protocol**: HTTPS
- **Rate limits**: Unspecified (reasonable use expected)
- **Cache strategy**: Recommended 60s TTL with 30s stale-while-revalidate
- **Reliability**: Government-operated service, high availability
- **Data freshness**: Real-time updates (typically <5 minutes)

## Official Documentation
- **User Guide**: https://nasstatus.faa.gov/static/media/NASStatusUserGuide.cccc6d48.pdf (PDF format)
- **GitHub Repository**: https://github.com/Federal-Aviation-Administration/ASWS
- **SwaggerHub**: https://app.swaggerhub.com/apis/FAA/ASWS (version 1.1.0)
- **Dashboard**: https://nasstatus.faa.gov/

## Version Information
- **Current API version**: 1.1.0 (ASWS)
- **DTD file reference**: Available in XML responses (Dtd_File field)
- **Last documented update**: September 6, 2022 (User Guide v2.2.1)

## Known Issues
- Legacy ASWS endpoint (`soa.smext.faa.gov`) appears to be deprecated/offline as of 2026
- Primary endpoint uses JavaScript-rendered UI, API access requires direct endpoint calls
- No public documentation of rate limits
- Some responses return only airport closures (depends on current NAS status)
