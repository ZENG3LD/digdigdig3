# C2IntelFeeds Data Types

## Overview

C2IntelFeeds provides threat intelligence data in CSV format with varying column structures depending on the feed type.

## CSV Column Structures

### 1. IP Feeds (IPC2s-*.csv)

**Columns**: 2
**Format**: `ip,ioc`

| Column | Type | Description | Example |
|--------|------|-------------|---------|
| ip | IPv4 Address | C2 server IP address | `101.34.205.214` |
| ioc | String | Threat classification | `Possible Cobaltstrike C2 IP` |

**File examples**:
- `IPC2s.csv`
- `IPC2s-30day.csv`
- `IPC2s-90day.csv`

### 2. Domain Feeds (domainC2s-*.csv)

**Columns**: 2
**Format**: `domain,ioc`

| Column | Type | Description | Example |
|--------|------|-------------|---------|
| domain | FQDN | Fully qualified domain name | `api.cryptoprot.info` |
| ioc | String | Threat classification | `Possible Cobalt Strike C2 Fronting Domain` |

**File examples**:
- `domainC2s.csv`
- `domainC2s-30day.csv`
- `domainC2s-90day-filter-abused.csv`

### 3. Domain + URL Feeds (domainC2swithURL-*.csv)

**Columns**: 3
**Format**: `domain,ioc,url_path`

| Column | Type | Description | Example |
|--------|------|-------------|---------|
| domain | FQDN | Fully qualified domain name | `accesserdsc.com` |
| ioc | String | Threat classification | `Possible Cobalt Strike C2 Domain` |
| url_path | String | URI path component | `/en_US/all.js` |

**File examples**:
- `domainC2swithURL.csv`
- `domainC2swithURL-30day.csv`
- `domainC2swithURL-90day-filter-abused.csv`

### 4. Domain + URL + IP Feeds (domainC2swithURLwithIP-*.csv)

**Columns**: 4
**Format**: `domain,ioc,url_path,ip`

| Column | Type | Description | Example |
|--------|------|-------------|---------|
| domain | FQDN | Fully qualified domain name | `120vip.top` |
| ioc | String | Threat classification | `Possible Cobalt Strike C2 Fronting Domain` |
| url_path | String | URI path component | `/ptj` |
| ip | IPv4 Address | Resolved IP address | `106.55.188.70` |

**File examples**:
- `domainC2swithURLwithIP.csv`
- `domainC2swithURLwithIP-30day.csv`
- `domainC2swithURLwithIP-90day-filter-abused.csv`

### 5. IP:Port Feeds (IPPortC2s-*.csv)

**Format**: IP:port pairs (column structure not fully documented)

**File examples**:
- `IPPortC2s.csv`
- `IPPortC2s-30day.csv`
- `IPPortC2s-90day.csv`

### 6. DNS Feeds (DNSC2Domains-*.csv)

**Format**: DNS-specific C2 domains (column structure similar to domain feeds)

**File examples**:
- `DNSC2Domains.csv`
- `DNSC2Domains-30day.csv`

## Data Field Specifications

### IP Address Format

- **Type**: IPv4 only (no IPv6 observed)
- **Format**: Dotted decimal notation
- **Examples**:
  - `1.12.231.30`
  - `101.126.144.111`
  - `216.126.224.23`
- **Validation**: Standard IPv4 regex: `^\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}$`

### Domain Format

- **Type**: FQDN (Fully Qualified Domain Names)
- **Format**: Standard DNS naming
- **Examples**:
  - `api.cryptoprot.info`
  - `d14hh3kwt0vf8s.cloudfront.net`
  - `2458ccd60cc54149bb05537717d831f0--8000.ap-shanghai2.cloudstudio.club`
- **Special cases**:
  - Serverless function domains (Tencent Cloud, Baidu Cloud)
  - CDN domains (CloudFront, Cloudflare)
  - Dynamic DNS (asuscomm.com, etc.)

### IOC (Indicator of Compromise) Field

**Type**: String (enumerated threat classifications)

**Common values**:

| IOC Value | Framework | Confidence |
|-----------|-----------|------------|
| `Possible Cobaltstrike C2 IP` | Cobalt Strike | Medium |
| `Possible Cobalt Strike C2 Domain` | Cobalt Strike | Medium |
| `Possible Cobalt Strike C2 Fronting Domain` | Cobalt Strike (domain fronting) | Medium |
| `Possible Cobalt Strike C2 Fronted Domain` | Cobalt Strike (fronted) | Medium |
| `Possible Metasploit C2 IP` | Metasploit | Medium |

**Notes**:
- All classifications use "Possible" prefix (not definitive)
- Fronting/Fronted variants indicate domain fronting techniques
- Case-sensitive strings

### URL Path Format

- **Type**: URI path component
- **Format**: String starting with `/`
- **Examples**:
  - `/ptj`
  - `/Test/protect/JZJ8DALCUB`
  - `/jquery-3.3.1.min.js`
  - `/s/ref=nb_sb_noss_1/...`
  - `/api/x`
- **Common patterns**:
  - JavaScript file paths (`.js`)
  - API endpoints (`/api/...`)
  - Random/encoded paths
  - Legitimate-looking paths (obfuscation)

## CSV Format Specifications

### Header Row

All feeds include a header row starting with `#`:

```csv
#ip,ioc
```

**Note**: The `#` prefix indicates a comment line in some CSV parsers.

### Delimiter

**Type**: Comma (`,`)

### Quoting

**Not observed**: Fields do not appear to be quoted (even with commas in data)

**Recommendation**: Implement defensive parsing for potential edge cases

### Line Endings

**Type**: Unix-style (`\n`) - standard for GitHub

### Encoding

**Type**: UTF-8

## Data Validation Rules

### IP Address Validation

```rust
fn is_valid_ipv4(ip: &str) -> bool {
    ip.split('.')
        .filter_map(|octet| octet.parse::<u8>().ok())
        .count() == 4
}
```

### Domain Validation

```rust
fn is_valid_domain(domain: &str) -> bool {
    // Basic validation: contains at least one dot, no spaces
    domain.contains('.') && !domain.contains(' ') && domain.len() > 3
}
```

### URL Path Validation

```rust
fn is_valid_url_path(path: &str) -> bool {
    path.starts_with('/') && !path.contains(' ')
}
```

## Time Windows

Feeds are provided in multiple time windows:

| Suffix | Time Window | Description |
|--------|-------------|-------------|
| (none) | All-time | Complete historical dataset |
| `-30day` | 30 days | Last 30 days of data |
| `-90day` | 90 days | Last 90 days of data |
| `-7day` | 7 days | Last 7 days (if available) |

## Filtering Types

### Standard Feeds

Raw C2 indicators without filtering (may include legitimate services being abused).

### Filtered Feeds (`-filter-abused`)

Removes known abused legitimate services:
- CDNs (CloudFront, Cloudflare, Akamai)
- Cloud providers (AWS, Azure, GCP)
- Shared hosting platforms
- Popular SaaS platforms

**Use case**: Lower false positive rate for blocking/alerting.

## C2 Framework Types

Based on observed IOC classifications:

### Cobalt Strike (Primary)

- Commercial penetration testing framework
- Commonly used by APT groups and red teams
- Most prevalent in feeds (>95% of indicators)

### Metasploit (Secondary)

- Open-source penetration testing framework
- Less prevalent in feeds (<5%)

### Other Frameworks

Not explicitly labeled in current feed structure.

## Summary

| Feed Type | Columns | Primary Use Case |
|-----------|---------|------------------|
| IP feeds | 2 (ip, ioc) | Quick IP blocking |
| Domain feeds | 2 (domain, ioc) | DNS-based blocking |
| Domain+URL | 3 (domain, ioc, path) | HTTP/HTTPS filtering |
| Domain+URL+IP | 4 (domain, ioc, path, ip) | Full context analysis |
| Filtered feeds | Same + filtering | Low false positive blocking |
| Time-windowed | Same + recency | Fresh threat intelligence |
