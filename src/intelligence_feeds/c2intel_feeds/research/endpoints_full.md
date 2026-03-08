# C2IntelFeeds Endpoints (Feed Files)

## Base URL

```
https://raw.githubusercontent.com/drb-ra/C2IntelFeeds/master/feeds/
```

## Available Feed Files (26 total)

### IP-Based Feeds

#### IPC2s.csv
- **URL**: `https://raw.githubusercontent.com/drb-ra/C2IntelFeeds/master/feeds/IPC2s.csv`
- **Description**: All-time C2 IP addresses
- **Columns**: ip, ioc
- **Format**: IPv4 address, threat classification

#### IPC2s-30day.csv
- **URL**: `https://raw.githubusercontent.com/drb-ra/C2IntelFeeds/master/feeds/IPC2s-30day.csv`
- **Description**: C2 IPs from last 30 days
- **Columns**: ip, ioc

#### IPC2s-90day.csv
- **URL**: `https://raw.githubusercontent.com/drb-ra/C2IntelFeeds/master/feeds/IPC2s-90day.csv`
- **Description**: C2 IPs from last 90 days
- **Columns**: ip, ioc

### Domain-Based Feeds

#### domainC2s.csv
- **URL**: `https://raw.githubusercontent.com/drb-ra/C2IntelFeeds/master/feeds/domainC2s.csv`
- **Description**: All-time C2 domains
- **Columns**: domain, ioc
- **Format**: FQDN, threat classification

#### domainC2s-30day.csv
- **URL**: `https://raw.githubusercontent.com/drb-ra/C2IntelFeeds/master/feeds/domainC2s-30day.csv`
- **Description**: C2 domains from last 30 days
- **Columns**: domain, ioc

#### domainC2s-90day.csv
- **URL**: `https://raw.githubusercontent.com/drb-ra/C2IntelFeeds/master/feeds/domainC2s-90day.csv`
- **Description**: C2 domains from last 90 days
- **Columns**: domain, ioc

### Filtered Domain Feeds (Abused Services Removed)

#### domainC2s-filter-abused.csv
- **URL**: `https://raw.githubusercontent.com/drb-ra/C2IntelFeeds/master/feeds/domainC2s-filter-abused.csv`
- **Description**: All-time domains, filtered to remove known abused legitimate services
- **Columns**: domain, ioc

#### domainC2s-30day-filter-abused.csv
- **URL**: `https://raw.githubusercontent.com/drb-ra/C2IntelFeeds/master/feeds/domainC2s-30day-filter-abused.csv`
- **Description**: 30-day domains, filtered
- **Columns**: domain, ioc

#### domainC2s-90day-filter-abused.csv
- **URL**: `https://raw.githubusercontent.com/drb-ra/C2IntelFeeds/master/feeds/domainC2s-90day-filter-abused.csv`
- **Description**: 90-day domains, filtered
- **Columns**: domain, ioc

### Domain + URL Path Feeds

#### domainC2swithURL.csv
- **URL**: `https://raw.githubusercontent.com/drb-ra/C2IntelFeeds/master/feeds/domainC2swithURL.csv`
- **Description**: All-time C2 domains with URL paths
- **Columns**: domain, ioc, url_path

#### domainC2swithURL-30day.csv
- **URL**: `https://raw.githubusercontent.com/drb-ra/C2IntelFeeds/master/feeds/domainC2swithURL-30day.csv`
- **Description**: 30-day domains with URL paths
- **Columns**: domain, ioc, url_path

#### domainC2swithURL-90day.csv
- **URL**: `https://raw.githubusercontent.com/drb-ra/C2IntelFeeds/master/feeds/domainC2swithURL-90day.csv`
- **Description**: 90-day domains with URL paths
- **Columns**: domain, ioc, url_path

### Domain + URL + IP Feeds (Full Context)

#### domainC2swithURLwithIP.csv
- **URL**: `https://raw.githubusercontent.com/drb-ra/C2IntelFeeds/master/feeds/domainC2swithURLwithIP.csv`
- **Description**: All-time C2 domains with URL paths and resolved IP addresses
- **Columns**: domain, ioc, url_path, ip
- **Most comprehensive**: Provides full C2 infrastructure mapping

#### domainC2swithURLwithIP-30day.csv
- **URL**: `https://raw.githubusercontent.com/drb-ra/C2IntelFeeds/master/feeds/domainC2swithURLwithIP-30day.csv`
- **Description**: 30-day domains with URLs and IPs
- **Columns**: domain, ioc, url_path, ip

#### domainC2swithURLwithIP-90day.csv
- **URL**: `https://raw.githubusercontent.com/drb-ra/C2IntelFeeds/master/feeds/domainC2swithURLwithIP-90day.csv`
- **Description**: 90-day domains with URLs and IPs
- **Columns**: domain, ioc, url_path, ip

### Filtered Domain + URL Path Feeds

#### domainC2swithURL-filter-abused.csv
- **URL**: `https://raw.githubusercontent.com/drb-ra/C2IntelFeeds/master/feeds/domainC2swithURL-filter-abused.csv`
- **Description**: All-time domains with URLs, filtered
- **Columns**: domain, ioc, url_path

#### domainC2swithURL-30day-filter-abused.csv
- **URL**: `https://raw.githubusercontent.com/drb-ra/C2IntelFeeds/master/feeds/domainC2swithURL-30day-filter-abused.csv`
- **Description**: 30-day domains with URLs, filtered
- **Columns**: domain, ioc, url_path

#### domainC2swithURL-90day-filter-abused.csv
- **URL**: `https://raw.githubusercontent.com/drb-ra/C2IntelFeeds/master/feeds/domainC2swithURL-90day-filter-abused.csv`
- **Description**: 90-day domains with URLs, filtered
- **Columns**: domain, ioc, url_path

### Filtered Domain + URL + IP Feeds

#### domainC2swithURLwithIP-filter-abused.csv
- **URL**: `https://raw.githubusercontent.com/drb-ra/C2IntelFeeds/master/feeds/domainC2swithURLwithIP-filter-abused.csv`
- **Description**: All-time domains with URLs and IPs, filtered
- **Columns**: domain, ioc, url_path, ip

#### domainC2swithURLwithIP-30day-filter-abused.csv
- **URL**: `https://raw.githubusercontent.com/drb-ra/C2IntelFeeds/master/feeds/domainC2swithURLwithIP-30day-filter-abused.csv`
- **Description**: 30-day domains with URLs and IPs, filtered
- **Columns**: domain, ioc, url_path, ip

#### domainC2swithURLwithIP-90day-filter-abused.csv
- **URL**: `https://raw.githubusercontent.com/drb-ra/C2IntelFeeds/master/feeds/domainC2swithURLwithIP-90day-filter-abused.csv`
- **Description**: 90-day domains with URLs and IPs, filtered
- **Columns**: domain, ioc, url_path, ip

### IP:Port Pair Feeds

#### IPPortC2s.csv
- **URL**: `https://raw.githubusercontent.com/drb-ra/C2IntelFeeds/master/feeds/IPPortC2s.csv`
- **Description**: All-time C2 IP:port pairs
- **Format**: IP addresses with associated listening ports

#### IPPortC2s-30day.csv
- **URL**: `https://raw.githubusercontent.com/drb-ra/C2IntelFeeds/master/feeds/IPPortC2s-30day.csv`
- **Description**: 30-day IP:port pairs

#### IPPortC2s-90day.csv
- **URL**: `https://raw.githubusercontent.com/drb-ra/C2IntelFeeds/master/feeds/IPPortC2s-90day.csv`
- **Description**: 90-day IP:port pairs

### DNS-Based Feeds

#### DNSC2Domains.csv
- **URL**: `https://raw.githubusercontent.com/drb-ra/C2IntelFeeds/master/feeds/DNSC2Domains.csv`
- **Description**: All-time DNS-based C2 domains

#### DNSC2Domains-30day.csv
- **URL**: `https://raw.githubusercontent.com/drb-ra/C2IntelFeeds/master/feeds/DNSC2Domains-30day.csv`
- **Description**: 30-day DNS-based C2 domains

## Feed Selection Guide

| Use Case | Recommended Feed |
|----------|------------------|
| IP blocking (recent threats) | IPC2s-30day.csv |
| Domain blocking (filtered) | domainC2s-30day-filter-abused.csv |
| Full context investigation | domainC2swithURLwithIP-30day.csv |
| Historical analysis | IPC2s.csv or domainC2s.csv |
| Firewall rules | IPPortC2s-30day.csv |
| Low false positives | Any `-filter-abused.csv` variant |

## Additional Resources

- **VPN Exit Nodes**: `/vpn/` directory (separate from C2 feeds)
- **Exclusions List**: `exclusions.rex` (regex patterns for false positive filtering)
