# Feodo Tracker Response Formats

## JSON Format (Primary)

### Endpoint
```
GET https://feodotracker.abuse.ch/downloads/ipblocklist.json
```

### Content-Type
```
Content-Type: application/json
```

### Structure

**Root Element**: JSON array of objects

```json
[
  {
    "ip_address": "162.243.103.246",
    "port": 8080,
    "status": "offline",
    "hostname": null,
    "as_number": 14061,
    "as_name": "DIGITALOCEAN-ASN",
    "country": "US",
    "first_seen": "2022-06-04 21:24:53",
    "last_online": "2026-02-06",
    "malware": "Emotet"
  }
]
```

### Field Specifications

| Field | Type | Required | Nullable | Example | Description |
|-------|------|----------|----------|---------|-------------|
| ip_address | string | Yes | No | "162.243.103.246" | IPv4 address of C2 server |
| port | integer | Yes | No | 8080 | TCP port number (1-65535) |
| status | string | Yes | No | "offline" | "online" or "offline" |
| hostname | string/null | Yes | Yes | null | Reverse DNS hostname |
| as_number | integer | Yes | No | 14061 | Autonomous System Number |
| as_name | string | Yes | No | "DIGITALOCEAN-ASN" | ISP/hosting provider |
| country | string | Yes | No | "US" | ISO 3166-1 alpha-2 code |
| first_seen | string | Yes | No | "2022-06-04 21:24:53" | First detection timestamp |
| last_online | string | Yes | No | "2026-02-06" | Most recent activity date |
| malware | string | Yes | No | "Emotet" | Botnet family name |

### Example Responses

#### Populated Dataset (Historical Example)

```json
[
  {
    "ip_address": "162.243.103.246",
    "port": 8080,
    "status": "offline",
    "hostname": null,
    "as_number": 14061,
    "as_name": "DIGITALOCEAN-ASN",
    "country": "US",
    "first_seen": "2022-06-04 21:24:53",
    "last_online": "2026-02-06",
    "malware": "Emotet"
  },
  {
    "ip_address": "185.14.30.61",
    "port": 443,
    "status": "online",
    "hostname": "vmi424242.contaboserver.net",
    "as_number": 51167,
    "as_name": "CONTABO",
    "country": "DE",
    "first_seen": "2023-09-12 14:32:01",
    "last_online": "2026-02-16",
    "malware": "TrickBot"
  },
  {
    "ip_address": "192.0.2.100",
    "port": 2222,
    "status": "offline",
    "hostname": null,
    "as_number": 16509,
    "as_name": "AMAZON-02",
    "country": "US",
    "first_seen": "2021-03-15 09:45:22",
    "last_online": "2025-12-20",
    "malware": "QakBot"
  }
]
```

#### Empty Dataset (Current Status)

```json
[]
```

**Note**: As of February 2026, datasets are empty due to law enforcement takedowns.

### Malware Field Values

Possible values for `malware` field:
- `"Dridex"`
- `"Emotet"`
- `"TrickBot"`
- `"QakBot"`
- `"BazarLoader"`

### Status Field Values

Possible values for `status` field:
- `"online"` - C2 currently active
- `"offline"` - C2 not responding

### Country Code Format

ISO 3166-1 alpha-2 codes (2 letters, uppercase):
- `"US"` - United States
- `"DE"` - Germany
- `"RU"` - Russia
- `"CN"` - China
- etc.

### Timestamp Formats

**first_seen**:
```
Format: "YYYY-MM-DD HH:MM:SS"
Timezone: UTC (implied, not specified in string)
Example: "2022-06-04 21:24:53"
```

**last_online**:
```
Format: "YYYY-MM-DD"
Timezone: UTC (implied)
Example: "2026-02-06"
```

## CSV Format

### Endpoint
```
GET https://feodotracker.abuse.ch/downloads/ipblocklist.csv
```

### Content-Type
```
Content-Type: text/csv
```

### Structure

**Header Row**: Yes

```csv
first_seen,dst_ip,dst_port,c2_status,last_online,malware
2022-06-04 21:24:53,162.243.103.246,8080,offline,2026-02-06,Emotet
2023-09-12 14:32:01,185.14.30.61,443,online,2026-02-16,TrickBot
```

### Field Mapping (CSV vs JSON)

| CSV Column | JSON Field | Notes |
|------------|------------|-------|
| first_seen | first_seen | Same format |
| dst_ip | ip_address | Renamed in CSV |
| dst_port | port | Renamed in CSV |
| c2_status | status | Renamed in CSV |
| last_online | last_online | Same format |
| malware | malware | Same format |

**Missing in CSV**: hostname, as_number, as_name, country

### CSV Limitations

CSV format does **not** include:
- Hostname
- AS Number
- AS Name
- Country Code

**Recommendation**: Use JSON format for full metadata.

## Plain Text Format

### Endpoint
```
GET https://feodotracker.abuse.ch/downloads/ipblocklist.txt
```

### Content-Type
```
Content-Type: text/plain
```

### Structure

**Format**: One IP address per line, no headers, no metadata

```
162.243.103.246
185.14.30.61
192.0.2.100
```

### Use Cases

- Simple firewall rules (iptables, nftables)
- IP blacklisting scripts
- Basic blocking without metadata

### Limitations

**No Metadata**: Only IP addresses, no:
- Port numbers
- Status
- Malware family
- Timestamps
- Geolocation

## Recommended Blocklist JSON

### Endpoint
```
GET https://feodotracker.abuse.ch/downloads/ipblocklist_recommended.json
```

### Content-Type
```
Content-Type: application/json
```

### Structure

**Root Element**: JSON array of strings (IP addresses only)

```json
[
  "162.243.103.246",
  "185.14.30.61",
  "192.0.2.100"
]
```

**Note**: This is a **simplified format** compared to the full IOC JSON endpoint.

### Differences from Full JSON

| Feature | Full JSON (ipblocklist.json) | Recommended JSON |
|---------|------------------------------|------------------|
| Structure | Array of objects | Array of strings |
| Metadata | Full (port, status, etc.) | None (IPs only) |
| Coverage | Past 30 days | Active/recent only |
| Use Case | SIEM, analytics | Simple blocking |

## HTTP Response Headers

### Successful Response (200 OK)

```http
HTTP/1.1 200 OK
Content-Type: application/json
Content-Length: 1234
Last-Modified: Wed, 15 Feb 2026 12:05:00 GMT
ETag: "abc123def456"
Cache-Control: public, max-age=300
```

### Not Modified (304)

```http
HTTP/1.1 304 Not Modified
Last-Modified: Wed, 15 Feb 2026 12:05:00 GMT
ETag: "abc123def456"
```

### Error Response (500)

```http
HTTP/1.1 500 Internal Server Error
Content-Type: text/html
```

## Rust Parsing Implementation

### JSON Deserialization

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct C2Entry {
    pub ip_address: String,
    pub port: u16,
    pub status: String,
    pub hostname: Option<String>,
    pub as_number: u32,
    pub as_name: String,
    pub country: String,
    pub first_seen: String,
    pub last_online: String,
    pub malware: String,
}

// Parse JSON response
pub fn parse_blocklist(json: &str) -> Result<Vec<C2Entry>, serde_json::Error> {
    serde_json::from_str(json)
}

// Handle empty dataset
pub fn parse_blocklist_safe(json: &str) -> Result<Vec<C2Entry>, serde_json::Error> {
    let entries: Vec<C2Entry> = serde_json::from_str(json)?;
    Ok(entries) // Empty vec is valid
}
```

### Enhanced Types with Enums

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct C2Entry {
    pub ip_address: String,
    pub port: u16,
    pub status: C2Status,
    pub hostname: Option<String>,
    pub as_number: u32,
    pub as_name: String,
    pub country: String,
    pub first_seen: String,
    pub last_online: String,
    pub malware: MalwareFamily,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum C2Status {
    Online,
    Offline,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum MalwareFamily {
    Dridex,
    Emotet,
    TrickBot,
    QakBot,
    BazarLoader,
}
```

## Error Handling

### Invalid JSON

```json
{
  "error": "Invalid request"
}
```

**Unlikely**: Feodo Tracker returns valid JSON arrays or empty arrays.

### Empty Dataset

```json
[]
```

**Expected**: Normal response when no C2s are tracked (current state).

**Handling**: Treat as valid response, not an error.

### Malformed Data

If a field is missing or has wrong type:
```rust
match serde_json::from_str::<Vec<C2Entry>>(json) {
    Ok(entries) => Ok(entries),
    Err(e) => {
        eprintln!("Failed to parse Feodo Tracker response: {}", e);
        Err(e.into())
    }
}
```

## Data Validation

### IP Address Validation

```rust
use std::net::IpAddr;

pub fn validate_ip(ip: &str) -> bool {
    ip.parse::<IpAddr>().is_ok()
}
```

### Port Range Validation

```rust
pub fn validate_port(port: u16) -> bool {
    port > 0 && port <= 65535
}
```

### Country Code Validation

```rust
pub fn validate_country_code(code: &str) -> bool {
    code.len() == 2 && code.chars().all(|c| c.is_ascii_uppercase())
}
```

### Timestamp Parsing

```rust
use chrono::NaiveDateTime;

pub fn parse_first_seen(ts: &str) -> Result<NaiveDateTime, chrono::ParseError> {
    NaiveDateTime::parse_from_str(ts, "%Y-%m-%d %H:%M:%S")
}

pub fn parse_last_online(date: &str) -> Result<chrono::NaiveDate, chrono::ParseError> {
    chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
}
```

## Summary

| Format | Endpoint | Content-Type | Use Case | Metadata |
|--------|----------|--------------|----------|----------|
| JSON (Full) | `/downloads/ipblocklist.json` | application/json | **Primary for connectors** | Complete |
| CSV | `/downloads/ipblocklist.csv` | text/csv | Excel, SIEM | Partial |
| TXT | `/downloads/ipblocklist.txt` | text/plain | Firewall scripts | None |
| JSON (Simple) | `/downloads/ipblocklist_recommended.json` | application/json | Simple blocking | None |

**Recommendation**: Use `/downloads/ipblocklist.json` for the v5 Rust connector.
