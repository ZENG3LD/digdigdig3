# Feodo Tracker Data Types

## Malware Families Tracked

Feodo Tracker specifically monitors Command & Control (C2) infrastructure for **five major botnet families**:

### 1. Dridex
- **Type**: Banking Trojan
- **Active**: Historically active, targeted by law enforcement
- **Capabilities**: Banking credential theft, money mule recruitment
- **C2 Protocol**: HTTPS (typically)
- **Ports**: Various (443, 8080, 8443, etc.)
- **First Seen**: 2014
- **Status**: Disrupted by law enforcement operations

### 2. Emotet (Heodo)
- **Type**: Modular Trojan / Botnet Loader
- **Active**: Dismantled in January 2021 (Operation Emotet)
- **Capabilities**: Email spam, malware delivery, banking trojans, ransomware
- **C2 Protocol**: HTTP/HTTPS
- **Ports**: 80, 443, 8080, 7080, 20, 22, 143, 465, 587, 993, 995, 2222, 3389
- **First Seen**: 2014
- **Status**: Infrastructure seized by law enforcement (January 2021)

### 3. TrickBot
- **Type**: Banking Trojan / Modular Botnet
- **Active**: Disrupted by Operation Endgame (2024)
- **Capabilities**: Banking credential theft, ransomware delivery, lateral movement
- **C2 Protocol**: HTTPS (443)
- **Ports**: 443 (primary)
- **First Seen**: 2016
- **Status**: Disrupted by Operation Endgame (2024)

### 4. QakBot (QuakBot / Qbot)
- **Type**: Banking Trojan / Worm
- **Active**: Disrupted by Operation Endgame (2024)
- **Capabilities**: Banking credential theft, ransomware delivery, network propagation
- **C2 Protocol**: HTTPS (443) and custom protocols
- **Ports**: 443, 2222, 61201, 65400
- **First Seen**: 2008 (one of the oldest)
- **Status**: Disrupted by Operation Endgame (2024)

### 5. BazarLoader (BazarBackdoor)
- **Type**: Backdoor / Loader
- **Active**: Disrupted by Operation Endgame (2024)
- **Capabilities**: Initial access, backdoor, malware delivery
- **C2 Protocol**: HTTPS (443)
- **Ports**: 443 (primary)
- **First Seen**: 2020
- **Status**: Disrupted by Operation Endgame (2024)

## C2 Status Types

Each C2 server entry has a `status` field indicating current operational state:

### Status Values

| Status | Description | Meaning | Action |
|--------|-------------|---------|--------|
| `online` | C2 is currently active | Server is responding to C2 traffic | **HIGH PRIORITY BLOCK** |
| `offline` | C2 is not responding | Server down or decommissioned | Block recommended (may return) |

### Status Lifecycle

```
First Seen → online → offline → (possible) online → offline (final)
```

- C2 servers may cycle between online/offline
- Offline doesn't mean permanently dead
- Recommended blocklist filters for recently active C2s

## Data Collection Types

Feodo Tracker provides three main collection types:

### 1. Recommended Blocklist
- **Filter Criteria**: Active or recently active C2s
- **False Positive Rate**: Lowest
- **Use Case**: Production blocking
- **Update Reason**: Status changes, new detections
- **Typical Size**: Smallest dataset

### 2. IOCs (30-Day)
- **Filter Criteria**: C2s seen in past 30 days
- **False Positive Rate**: Low
- **Use Case**: SIEM, threat intelligence platforms
- **Update Reason**: New detections, status updates
- **Typical Size**: Medium dataset

### 3. Aggressive (Historical)
- **Filter Criteria**: All C2s ever tracked
- **False Positive Rate**: Higher
- **Use Case**: Research, maximum protection
- **Update Reason**: New detections only (never removes)
- **Typical Size**: Largest dataset

## Data Fields

Each C2 entry contains the following fields:

### ip_address
- **Type**: String (IPv4)
- **Format**: Dotted decimal notation
- **Example**: "162.243.103.246"
- **Required**: Yes

### port
- **Type**: Integer (u16)
- **Range**: 1-65535
- **Example**: 8080, 443, 7080
- **Required**: Yes
- **Common Ports**:
  - 443 (HTTPS)
  - 8080 (HTTP alternate)
  - 7080 (Emotet common)
  - 2222 (SSH alternate, QakBot)

### status
- **Type**: String (enum)
- **Values**: "online", "offline"
- **Example**: "offline"
- **Required**: Yes

### hostname
- **Type**: String or null
- **Format**: FQDN or null
- **Example**: null, "example.com"
- **Required**: Yes (can be null)
- **Note**: Reverse DNS lookup result

### as_number
- **Type**: Integer (u32)
- **Format**: Autonomous System Number
- **Example**: 14061
- **Required**: Yes
- **Purpose**: ISP identification

### as_name
- **Type**: String
- **Format**: ISP/hosting provider name
- **Example**: "DIGITALOCEAN-ASN", "AMAZON-02"
- **Required**: Yes
- **Purpose**: Infrastructure attribution

### country
- **Type**: String
- **Format**: ISO 3166-1 alpha-2 country code
- **Example**: "US", "DE", "RU"
- **Required**: Yes
- **Purpose**: Geolocation

### first_seen
- **Type**: String (timestamp)
- **Format**: "YYYY-MM-DD HH:MM:SS"
- **Example**: "2022-06-04 21:24:53"
- **Required**: Yes
- **Timezone**: UTC (implied)

### last_online
- **Type**: String (date)
- **Format**: "YYYY-MM-DD"
- **Example**: "2026-02-06"
- **Required**: Yes
- **Purpose**: Most recent activity date

### malware
- **Type**: String
- **Values**: "Dridex", "Emotet", "TrickBot", "QakBot", "BazarLoader"
- **Example**: "Emotet"
- **Required**: Yes
- **Purpose**: Botnet family attribution

## Derived Data Types

For connector implementation, derive these types:

### Threat Severity
Based on status and recency:
- **Critical**: status=online
- **High**: status=offline, last_online within 7 days
- **Medium**: status=offline, last_online within 30 days
- **Low**: status=offline, last_online > 30 days

### Infrastructure Category
Based on as_name patterns:
- **Cloud Hosting**: AWS, GCP, Azure, DigitalOcean
- **VPS Providers**: OVH, Hetzner, Linode
- **Residential**: Comcast, AT&T, Verizon (rare)
- **Bulletproof**: Known abuse-friendly hosts

### Geographic Distribution
Group by country field for analytics:
- High concentration in: US, DE, FR, NL, RU
- Emerging regions: Asia (CN, HK, SG), Eastern Europe

## Malware Family Statistics

### Common Characteristics by Family

| Family | Typical Ports | Protocol | First Seen Range | Peak Activity |
|--------|--------------|----------|------------------|---------------|
| Emotet | 80, 443, 7080, 8080 | HTTP/HTTPS | 2014-2021 | 2018-2020 |
| TrickBot | 443 | HTTPS | 2016-2024 | 2019-2023 |
| QakBot | 443, 2222, 65400 | HTTPS | 2008-2024 | 2020-2023 |
| Dridex | 443, 8443 | HTTPS | 2014-present | 2015-2019 |
| BazarLoader | 443 | HTTPS | 2020-2024 | 2021-2023 |

## Data Quality Notes

### Accuracy
- **IP Addresses**: Verified through multiple sources
- **Ports**: Observed C2 communication ports
- **Status**: Updated every 5 minutes
- **Geolocation**: Based on IP WHOIS/GeoIP
- **ASN**: Accurate (from routing tables)

### False Positives
- **Recommended List**: Very low FP rate
- **Aggressive List**: Higher FP rate (historical servers)
- **Shared Hosting**: Possible FPs if C2 on shared IP (rare)

### Coverage Completeness
- **Not Exhaustive**: Only tracks known/discovered C2s
- **Detection Lag**: New C2s may not appear immediately
- **Law Enforcement Impact**: Takedowns cause empty datasets

## Current Dataset Status (February 2026)

**All datasets are currently empty** due to successful law enforcement operations:
- **Operation Emotet** (January 2021): Dismantled Emotet infrastructure
- **Operation Endgame** (2024): Targeted TrickBot, QakBot, BazarLoader

### Historical Context
- At peak, datasets contained 100-1000+ C2 entries
- Multiple malware families operated simultaneously
- Regular churn (new C2s added, old removed)

### Future State
- Datasets will repopulate if new C2 infrastructure emerges
- Monitoring infrastructure remains active
- Automated detection continues

## Rust Type Mapping

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct C2Entry {
    pub ip_address: String,          // IPv4 as string
    pub port: u16,                   // TCP port
    pub status: C2Status,            // Enum: Online/Offline
    pub hostname: Option<String>,    // Nullable FQDN
    pub as_number: u32,              // ASN
    pub as_name: String,             // ISP name
    pub country: String,             // ISO 3166-1 alpha-2
    pub first_seen: String,          // "YYYY-MM-DD HH:MM:SS"
    pub last_online: String,         // "YYYY-MM-DD"
    pub malware: MalwareFamily,      // Enum
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum C2Status {
    Online,
    Offline,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MalwareFamily {
    Dridex,
    Emotet,
    TrickBot,
    QakBot,
    BazarLoader,
}
```
