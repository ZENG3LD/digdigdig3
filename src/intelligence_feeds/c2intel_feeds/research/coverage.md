# C2IntelFeeds Coverage

## Data Sources

### Primary Source: Censys

C2IntelFeeds primarily aggregates data from **Censys**, a large-scale internet scanning platform.

**Censys capabilities**:
- Continuous internet-wide scanning
- IPv4 address space coverage
- Service fingerprinting
- Certificate transparency monitoring
- Banner grabbing
- HTTP/HTTPS service analysis

### Detection Methods

C2IntelFeeds identifies C2 infrastructure through multiple signals from Censys data:

| Detection Method | Description | C2 Frameworks |
|------------------|-------------|---------------|
| TLS Certificate Fields | Self-signed certs, known CN patterns | Cobalt Strike, Metasploit |
| JARM Fingerprints | TLS handshake fingerprinting | Cobalt Strike |
| HTTP Headers | Server headers, Host headers | Multiple frameworks |
| HTTP Response Bodies | Body hashes, known HTML patterns | Cobalt Strike |
| Page Titles | Specific page title signatures | Multiple frameworks |
| Service Banners | Port banners with known patterns | Metasploit, custom C2 |
| Public Keys | Known RSA/EC public keys | Cobalt Strike |

### Additional Sources (Implied)

While Censys is the primary source, the project may aggregate from:
- Community submissions
- Open-source threat intelligence
- Security research feeds

**Note**: Specific additional sources not explicitly documented.

## C2 Framework Coverage

### Cobalt Strike (Primary Focus)

**Coverage**: Extensive (>95% of indicators)

**Detection signals**:
- Default TLS certificate patterns
- Known malleable C2 profiles
- JARM fingerprints
- HTTP response characteristics
- jQuery stager patterns
- Known URI paths

**Indicator types**:
- `Possible Cobaltstrike C2 IP`
- `Possible Cobalt Strike C2 Domain`
- `Possible Cobalt Strike C2 Fronting Domain`
- `Possible Cobalt Strike C2 Fronted Domain`

### Metasploit (Secondary)

**Coverage**: Limited (<5% of indicators)

**Detection signals**:
- Default framework fingerprints
- Known service banners
- HTTP response patterns

**Indicator types**:
- `Possible Metasploit C2 IP`

### Other Frameworks

**Coverage**: Unknown/Limited

Frameworks like Havoc, Mythic, Sliver, Brute Ratel, Empire, etc. may be present but not explicitly labeled.

## Geographic Coverage

### Global Scope

C2IntelFeeds covers infrastructure worldwide, based on Censys's global internet scanning.

**Observed regions** (based on sample data):
- **China**: Heavy presence (Tencent Cloud, Baidu Cloud, Alibaba Cloud serverless functions)
- **North America**: CloudFront, AWS, standard hosting
- **Europe**: Various hosting providers
- **Asia-Pacific**: APAC cloud providers

### Cloud Provider Coverage

**Heavily represented**:
- Tencent Cloud (China)
- Baidu Cloud (China)
- Alibaba Cloud (China)
- Amazon Web Services (Global)
- Amazon CloudFront (CDN)
- Cloudflare (CDN)

**Note**: Serverless functions and CDNs are commonly abused for C2 infrastructure.

## Network Coverage

### AS Numbers (ASN)

Feeds do not directly provide ASN data, but underlying Censys data includes:
- Internet Service Providers (ISPs)
- Cloud hosting providers
- Data center networks
- CDN providers
- Residential ISPs (less common for C2)

### Port Coverage

IP:Port feeds (`IPPortC2s-*.csv`) cover non-standard C2 ports, but specific port distribution not documented.

**Common C2 ports**:
- 80 (HTTP)
- 443 (HTTPS)
- 8080, 8443 (Alternative HTTP/HTTPS)
- 50050 (Cobalt Strike default)
- Custom high ports (ephemeral range)

## Temporal Coverage

### Time Windows

Feeds are available in multiple time-based variants:

| Time Window | Feed Suffix | Coverage Period | Use Case |
|-------------|-------------|-----------------|----------|
| All-time | (none) | Complete historical dataset | Historical analysis, research |
| 90-day | `-90day` | Last 90 days | Medium-term trends |
| 30-day | `-30day` | Last 30 days | Recent threats (recommended) |
| 7-day | `-7day` | Last 7 days | Emerging threats (if available) |

### Data Freshness

**Update frequency**: Feeds are "updated automatically" but exact schedule not documented.

**Estimated update frequency**: Daily or more frequent, based on:
- Censys scan frequency
- Automated processing pipeline
- GitHub repository commit patterns

**Recommendation**: Poll feeds every 15-60 minutes for near-real-time threat intelligence.

### Historical Depth

**All-time feeds**: Appear to contain complete historical data from project inception.

**Note**: Exact start date of project not documented, but likely 2020-2023 based on GitHub trends.

## Data Quality

### Confidence Levels

All indicators use "Possible" prefix:
- **High confidence**: Not explicitly marked (no "Confirmed" classification)
- **Medium confidence**: All indicators (implicit)
- **Low confidence**: Not included (filtered out)

**Interpretation**: Indicators are based on automated detection and should be treated as medium-confidence.

### False Positive Rate

#### Unfiltered Feeds

**Higher false positive rate**: May include:
- Legitimate services being abused
- CDN nodes hosting C2 payloads
- Shared hosting platforms with compromised sites
- Cloud serverless functions (may be temporary/testing)

#### Filtered Feeds (`-filter-abused`)

**Lower false positive rate**: Removes known abused services:
- Major CDNs (CloudFront, Cloudflare, Akamai)
- Cloud providers (AWS, Azure, GCP shared resources)
- Popular SaaS platforms
- Shared hosting providers

**Recommendation**: Use `-filter-abused` variants for blocking/alerting to minimize false positives.

### False Negative Rate

**Unknown**: No documentation on false negative rate.

**Factors affecting coverage**:
- C2 infrastructure using custom/unknown fingerprints
- Encrypted/obfuscated C2 channels
- Non-HTTP/HTTPS C2 protocols (e.g., DNS, ICMP)
- C2 infrastructure not yet scanned by Censys
- Ephemeral C2 infrastructure (short-lived)

## Exclusions and Filtering

### Exclusions File

Repository includes `exclusions.rex` (regex patterns) to remove false positives.

**Common exclusions**:
- `.*\.cloudfront\.net` (Amazon CloudFront)
- `.*\.cloudflare\.com` (Cloudflare)
- `.*\.amazonaws\.com` (AWS shared resources)
- Other major CDNs and cloud providers

### Filter Variants

**Feed types**:
- **Unfiltered**: `domainC2s.csv`, `IPC2s.csv`
- **Filtered**: `domainC2s-filter-abused.csv`, etc.

**Recommendation**: Use filtered feeds for production blocking/alerting.

## Data Completeness

### Included Data

- ✅ IPv4 addresses
- ✅ Domain names (FQDN)
- ✅ URL paths
- ✅ IOC classifications
- ✅ Time-windowed variants

### Missing Data

- ❌ IPv6 addresses (not observed)
- ❌ First seen timestamps (not in CSV feeds)
- ❌ Last seen timestamps
- ❌ ASN information
- ❌ Geolocation data
- ❌ Port numbers (except in `IPPortC2s-*` feeds)
- ❌ Confidence scores (all marked "Possible")
- ❌ C2 configuration metadata (available in separate JSON feeds)

**Note**: Some metadata may be available in separate JSON feeds not covered in this analysis.

## Feed Variants Summary

| Feed Category | Feed Count | Coverage |
|---------------|-----------|----------|
| IP feeds | 3 | IPv4 addresses only |
| Domain feeds | 6 | FQDNs (filtered and unfiltered) |
| Domain+URL feeds | 6 | FQDNs with URI paths |
| Domain+URL+IP feeds | 6 | Complete context (domain, URL, IP) |
| IP:Port feeds | 3 | IP addresses with ports |
| DNS feeds | 2 | DNS-specific C2 domains |
| **Total** | **26 CSV feeds** | Multiple time windows and filtering options |

## Use Case Recommendations

| Use Case | Recommended Feed | Time Window |
|----------|------------------|-------------|
| Network firewall blocking | `IPC2s-30day.csv` | 30-day |
| DNS blackholing | `domainC2s-30day-filter-abused.csv` | 30-day |
| Web proxy filtering | `domainC2swithURL-30day-filter-abused.csv` | 30-day |
| Threat hunting (full context) | `domainC2swithURLwithIP-30day.csv` | 30-day |
| Historical analysis | `domainC2swithURLwithIP.csv` (all-time) | All-time |
| Low false positive alerting | Any `-filter-abused` variant | 30-day |
| Emerging threats | `-30day` variants | 30-day |

## Known Limitations

1. **No real-time streaming**: Static CSV files updated periodically
2. **No confidence scores**: All indicators marked "Possible"
3. **Limited framework coverage**: Primarily Cobalt Strike and Metasploit
4. **No metadata in CSV**: Timestamps, ASN, geolocation require additional enrichment
5. **Potential false positives**: Legitimate services may be included (use filtered variants)
6. **IPv4 only**: No IPv6 coverage observed
7. **Update frequency unknown**: Exact update schedule not documented
8. **No SLA guarantees**: Community project (best-effort basis)

## Data Enrichment Recommendations

To enhance C2IntelFeeds data:

1. **Geolocation**: Enrich IPs with GeoIP databases (MaxMind, IP2Location)
2. **ASN Lookup**: Add ASN and organization data (WHOIS, IPInfo)
3. **Threat intelligence platforms**: Cross-reference with VirusTotal, AlienVault OTX
4. **DNS resolution**: Periodically resolve domains to track infrastructure changes
5. **Port scanning**: Validate active C2 servers on reported IP:port pairs
6. **Reputation scoring**: Combine with other threat feeds for confidence scoring

## Summary

- **Primary source**: Censys internet scanning data
- **Framework focus**: Cobalt Strike (>95%), Metasploit (<5%)
- **Geographic scope**: Global coverage
- **Temporal coverage**: All-time, 90-day, 30-day, 7-day variants
- **Update frequency**: Automated (likely daily)
- **Data quality**: Medium confidence, filtered variants reduce false positives
- **Completeness**: Comprehensive for HTTP/HTTPS C2 infrastructure, limited for other protocols
- **Best for**: Network/DNS blocking, threat hunting, IOC enrichment
