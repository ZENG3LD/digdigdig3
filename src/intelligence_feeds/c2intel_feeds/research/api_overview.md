# C2IntelFeeds API Overview

## Project Description

C2IntelFeeds is an automated threat intelligence project that generates Command-and-Control (C2) threat intelligence feeds from internet scanning data. The project is hosted as a public GitHub repository and provides free access to C2 infrastructure indicators.

**Repository**: https://github.com/drb-ra/C2IntelFeeds

## Purpose and Use Cases

The feeds are designed for defensive security operations:
- Threat hunting
- Network monitoring
- Detection engineering
- IOC (Indicator of Compromise) enrichment
- Blocking known malicious infrastructure

## Data Source

Primary data source is **Censys** large-scale internet scanning, detecting C2 infrastructure through:
- TLS certificate fields and fingerprints
- JARM fingerprints
- HTTP response headers and titles
- Body hashes
- Service banners
- Known malware implant artifacts

## Feed Types

The repository provides multiple feed categories:

### Verified Feeds
- C2 IP addresses (various time windows: 7, 30, 90-day)
- C2 Domains
- C2 Domains (filtered - abused services removed)
- C2 Domains with URL paths
- C2 Domains with URL paths and IP addresses
- IP:Port pairs

### Additional Data
- VPN exit node lists (separate `/vpn` directory)
- Residential proxy network data
- C2 configuration metadata (CSV and JSON formats)

## Data Formats

Feeds use "simple formats" for automation compatibility:
- **CSV**: Primary format for feeds (tab-separated values)
- **JSON**: Available for some metadata
- **Plain-text**: Simple line-delimited lists

## Access Method

**Type**: Public GitHub repository (raw file access)
**Protocol**: HTTPS
**Base URL**: `https://raw.githubusercontent.com/drb-ra/C2IntelFeeds/master/feeds/`
**Authentication**: None required (public data)

## False Positive Management

The repository includes `exclusions.rex` file containing regex patterns to remove false positives from:
- Known CDNs (Cloudflare, CloudFront, etc.)
- Shared hosting providers
- Legitimate services

## Update Mechanism

Feeds are "updated automatically" via GitHub Actions or similar CI/CD pipeline. Multiple time-window variants provide historical context (7-day, 30-day, 90-day).

## Primary C2 Frameworks Detected

Based on feed data:
- **Cobalt Strike**: Primary framework (majority of indicators)
- **Metasploit**: Secondary framework
- Other APT frameworks (less common)

## License and Usage

Public repository - check LICENSE file for specific terms. Generally intended for defensive security purposes.
