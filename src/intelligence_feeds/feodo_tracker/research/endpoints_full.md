# Feodo Tracker API Endpoints

## Base URL

```
https://feodotracker.abuse.ch
```

## Endpoint Categories

Feodo Tracker provides three main categories of downloads:
1. **Recommended Blocklist** - Active/recent C2s with lowest false positive rate
2. **IOCs (30-day)** - Past 30 days of C2 activity with full metadata
3. **Aggressive Blocklist** - All historical C2s ever tracked

## 1. Recommended Botnet C2 IP Blocklist

**Purpose**: Block active or recently active C2 servers (lowest false positives)

### Plain Text Format
```
GET /downloads/ipblocklist_recommended.txt
```
- **Format**: Plain text, one IP per line
- **Use Case**: Simple firewall/iptables rules
- **Content**: IP addresses only

### JSON Format
```
GET /downloads/ipblocklist_recommended.json
```
- **Format**: JSON array of IP strings
- **Use Case**: Programmatic processing
- **Content**: IP addresses only

### McAfee Web Gateway Format
```
GET /downloads/ipblocklist_recommended_mcafee.txt
```
- **Format**: McAfee-specific format
- **Use Case**: Direct import into McAfee Web Gateway
- **Content**: IP addresses formatted for McAfee

### Palo Alto Firewall Format
```
GET /downloads/ipblocklist_recommended_paloalto.txt
```
- **Format**: Palo Alto-specific format
- **Use Case**: Direct import into Palo Alto firewalls
- **Content**: IP addresses formatted for Palo Alto

## 2. Botnet C2 IOCs (Past 30 Days)

**Purpose**: Full metadata for SIEM/threat intelligence platforms

### CSV Format
```
GET /downloads/ipblocklist.csv
```
- **Format**: CSV with headers
- **Use Case**: Excel, SIEM imports
- **Fields**: first_seen, dst_ip, dst_port, c2_status, last_online, malware
- **Example**:
  ```csv
  first_seen,dst_ip,dst_port,c2_status,last_online,malware
  2022-06-04 21:24:53,162.243.103.246,8080,offline,2026-02-06,Emotet
  ```

### JSON Format
```
GET /downloads/ipblocklist.json
```
- **Format**: JSON array of objects
- **Use Case**: REST APIs, programmatic processing
- **Content**: Full metadata per entry (see response_formats.md)
- **Primary Format for Connectors**: YES

### Plain Text (IPs Only)
```
GET /downloads/ipblocklist.txt
```
- **Format**: Plain text, one IP per line
- **Use Case**: Simple firewall rules
- **Content**: IP addresses only (no metadata)

## 3. Aggressive Blocklist (Historical)

**Purpose**: All C2s ever tracked (higher false positive rate)

### CSV Format
```
GET /downloads/ipblocklist_aggressive.csv
```
- **Format**: CSV with headers
- **Use Case**: Historical analysis, research
- **Fields**: Same as regular CSV
- **Warning**: May include long-dead C2 servers

### Plain Text (IPs Only)
```
GET /downloads/ipblocklist_aggressive.txt
```
- **Format**: Plain text, one IP per line
- **Use Case**: Maximum blocking (higher false positives)
- **Content**: IP addresses only

## 4. IDS/IPS Rulesets

**Purpose**: Network intrusion detection/prevention

### Suricata Rules (Recommended)
```
GET /downloads/feodotracker.rules
```
- **Format**: Suricata rule syntax
- **Use Case**: Suricata/Snort integration
- **Content**: alert rules for recommended C2s

### Suricata Rules Archive
```
GET /downloads/feodotracker.tar.gz
```
- **Format**: Tar.gz archive
- **Content**: Rules + metadata

### Suricata Rules (Aggressive)
```
GET /downloads/feodotracker_aggressive.rules
```
- **Format**: Suricata rule syntax
- **Use Case**: Maximum detection coverage
- **Content**: alert rules for all historical C2s

### Suricata Rules Archive (Aggressive)
```
GET /downloads/feodotracker_aggressive.tar.gz
```
- **Format**: Tar.gz archive
- **Content**: Aggressive rules + metadata

## Query Parameters

**None Required** - All endpoints are static file downloads with no query parameters.

## HTTP Method

All endpoints use **GET** requests only.

## Authentication

**None** - All endpoints are publicly accessible without API keys or authentication.

## Response Codes

- **200 OK** - Successful download
- **404 Not Found** - Invalid endpoint
- **503 Service Unavailable** - Server maintenance (rare)

## Endpoint Selection Guide

| Use Case | Recommended Endpoint |
|----------|---------------------|
| Firewall blocking (production) | `/downloads/ipblocklist_recommended.txt` |
| SIEM/TI platform integration | `/downloads/ipblocklist.json` |
| Rust connector (primary) | `/downloads/ipblocklist.json` |
| Research/analysis | `/downloads/ipblocklist_aggressive.csv` |
| IDS/IPS | `/downloads/feodotracker.rules` |
| Simple blocking script | `/downloads/ipblocklist.txt` |

## Update Frequency

- **Generation Interval**: Every 5 minutes
- **Recommended Poll Rate**: Every 5-15 minutes
- **Minimum Poll Rate**: Every 15 minutes

## File Sizes (Approximate)

When datasets are populated:
- JSON files: 10-100 KB
- CSV files: 10-100 KB
- TXT files: 5-50 KB
- Rules files: 50-500 KB

**Current Status**: All datasets are empty as of February 2026 due to law enforcement takedowns.

## Example URLs

```
https://feodotracker.abuse.ch/downloads/ipblocklist.json
https://feodotracker.abuse.ch/downloads/ipblocklist.csv
https://feodotracker.abuse.ch/downloads/ipblocklist_recommended.txt
https://feodotracker.abuse.ch/downloads/feodotracker.rules
```

## ETags / Caching

- Standard HTTP caching headers are used
- Check `Last-Modified` header for data freshness
- Use conditional requests (`If-Modified-Since`) to reduce bandwidth
