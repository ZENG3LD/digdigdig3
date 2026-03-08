# NWS Weather Alerts API Overview

## Provider Information

- **Name**: National Weather Service (NWS)
- **Organization**: National Oceanic and Atmospheric Administration (NOAA)
- **Country**: United States
- **Service Type**: Public Weather Alert Distribution
- **Base URL**: `https://api.weather.gov`
- **API Version**: CAP v1.2 (Common Alerting Protocol)

## Purpose

The NWS Alerts API distributes official weather watches, warnings, advisories, and other hazard notifications issued by the National Weather Service. It provides programmatic access to critical, life-saving weather information using the standardized Common Alerting Protocol (CAP) v1.2 format.

## Key Capabilities

### Alert Distribution
- **Active Alerts**: Real-time access to all currently active weather alerts across the United States
- **Historical Alerts**: Query alerts issued over the past seven days
- **Geographic Filtering**: Filter by state, county, forecast zone, coordinates, or marine region
- **Alert Types**: Comprehensive list of all NWS alert event types

### Data Formats
- **GeoJSON** (default): `application/geo+json` - modern, developer-friendly format
- **JSON-LD**: `application/ld+json` - linked data format
- **CAP XML**: `application/cap+xml` - original CAP 1.2 format
- **ATOM**: `application/atom+xml` - syndication format

### Alert Categories
- Watches: Conditions are favorable for hazardous weather
- Warnings: Hazardous weather is occurring or imminent
- Advisories: Less serious conditions causing inconvenience
- Statements: Follow-up information on ongoing events
- Outlooks: Potential for hazardous weather in the future

## Coverage

- **Geographic**: Continental United States, Alaska, Hawaii, Puerto Rico, Guam, US Virgin Islands, and other US territories
- **Marine Areas**: Atlantic, Pacific, and Gulf of Mexico marine regions
- **Temporal**: Active alerts plus 7-day historical archive
- **Official Archive**: For alerts older than 7 days, contact National Centers for Environmental Information (NCEI)

## Use Cases

1. **Emergency Management**: Real-time alert monitoring for public safety agencies
2. **Weather Applications**: Integration into mobile apps and websites
3. **Geographic Targeting**: Location-based alert notifications
4. **Data Analysis**: Weather pattern and alert frequency analysis
5. **Public Safety**: Automated alert distribution systems

## API Philosophy

The API is designed with:
- **Cache-Friendly**: Content expires based on information lifecycle
- **Open Data**: Free to use for any purpose, no API keys required
- **REST-Style**: Standard HTTP methods and status codes
- **JSON-Based**: Modern web-friendly data format
- **Machine Discovery**: JSON-LD promotes automated data understanding

## Authentication & Access

- **Authentication**: None required (public API)
- **API Keys**: Not needed
- **Cost**: Free government service
- **User-Agent**: Required header to identify your application
  - Format: `(myweatherapp.com, contact@myweatherapp.com)`
  - Purpose: Application identification and abuse prevention

## Rate Limits

- **Recommended**: No more than one request per 30 seconds
- **Enforcement**: Rate-limiting firewalls protect against abuse
- **Retry**: If rate limited, retry after 5 seconds
- **Impact**: Direct client requests less likely to hit limits than proxy requests

## Technical Foundation

- **Protocol**: HTTPS REST API
- **Standards**: CAP v1.2, GeoJSON, JSON-LD
- **Specification**: OpenAPI v3.0
  - JSON: `https://api.weather.gov/openapi.json`
  - YAML: `https://api.weather.gov/openapi.yaml`
- **Cache Strategy**: Expires headers based on alert lifecycle

## Alert Metadata

Each alert includes:
- **Classification**: Status, message type, category
- **Impact**: Severity, urgency, certainty
- **Temporal**: Sent, effective, onset, expires, ends timestamps
- **Geographic**: Area description, geocodes (SAME, UGC), affected zones
- **Content**: Headline, description, instructions
- **References**: Links to related/superseded alerts

## Data Reliability

- **Source**: Official US government weather service
- **Authority**: Issued by trained NWS meteorologists
- **Standards Compliance**: Adheres to CAP v1.2 international standard
- **Legal Status**: Official warnings carry legal weight for emergency management
- **Timeliness**: Real-time distribution of life-safety information

## Support & Contacts

- **Operational Issues**: nco.ops@noaa.gov, 301-713-0902
- **Technical Support**: sdb.support@noaa.gov
- **General Inquiries**: mike.gerber@noaa.gov
- **Community Discussion**: https://github.com/weather-gov/api
- **Documentation**: https://www.weather.gov/documentation/services-web-api
