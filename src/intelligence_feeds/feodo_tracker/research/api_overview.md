# Feodo Tracker API Overview

## Service Description

Feodo Tracker is a project operated by abuse.ch with the goal of sharing botnet C&C (Command and Control) servers associated with major banking trojans and malware families. The service provides free, publicly accessible threat intelligence feeds for network security and malware research.

## Provider Information

- **Organization**: abuse.ch
- **Service**: Feodo Tracker
- **Website**: https://feodotracker.abuse.ch/
- **Type**: Threat Intelligence Feed / Botnet C2 Tracker
- **Cost**: Free (CC0 License)
- **Authentication**: None required

## Purpose

Feodo Tracker monitors and tracks command-and-control infrastructure for major botnet families that facilitate cybercrime activities such as:
- Banking trojans
- Ransomware delivery
- Credential theft
- Spam campaigns
- Lateral movement

## Tracked Malware Families

The service specifically monitors C&C servers for five major botnet families:

1. **Dridex** - Banking trojan
2. **Emotet (Heodo)** - Modular trojan/loader
3. **TrickBot** - Banking trojan and botnet
4. **QakBot (QuakBot/Qbot)** - Banking trojan
5. **BazarLoader (BazarBackdoor)** - Backdoor/loader

## Data Offering

Feodo Tracker provides several key resources:

1. **IP Blocklists** - Lists of identified botnet C&C server IP addresses
2. **Indicators of Compromise (IOCs)** - Detailed metadata about C2 infrastructure
3. **IDS/IPS Rulesets** - Suricata/Snort rules for network detection
4. **Historical Data** - Archive of past C2 activity
5. **Statistics** - Tracking information on monitored threats

## Use Cases

- **Network Security**: Block malicious C2 communications at firewall/proxy level
- **SIEM Integration**: Enrich security events with threat intelligence
- **Threat Hunting**: Identify compromised hosts communicating with known C2s
- **Research**: Analyze botnet infrastructure and evolution
- **Incident Response**: Validate indicators during investigations

## Current Status (February 2026)

According to the official FAQ, Feodo Tracker datasets are currently empty due to successful law enforcement takedowns:
- **Operation Emotet** (2021) - Dismantled Emotet infrastructure
- **Operation Endgame** (2024) - Targeted remaining malware families

Despite empty datasets, the infrastructure remains active and will track new C2 servers if they emerge.

## License & Terms

- **License**: CC0 (Creative Commons Zero)
- **Commercial Use**: Permitted without limitations
- **Non-Commercial Use**: Permitted without limitations
- **Liability**: Data provided "as is" on best effort basis
- **Attribution**: Not required but appreciated

## Key Features

- **Real-time Updates**: Blocklists generated every 5 minutes
- **Multiple Formats**: JSON, CSV, TXT, and firewall-specific formats
- **Historical Context**: Tracks first seen and last online timestamps
- **Global Coverage**: Worldwide C2 infrastructure monitoring
- **Enriched Metadata**: Includes ASN, hostname, geolocation data
- **Low False Positives**: Recommended blocklist filtered for accuracy

## Integration Options

1. **Direct Download**: HTTP GET from public endpoints
2. **Periodic Polling**: Recommended every 5-15 minutes
3. **Spamhaus BCL**: Alternative feed via Spamhaus (requires contact)
4. **IDS/IPS Rules**: Native integration with Suricata/Snort

## Related Services

abuse.ch operates several complementary threat intelligence services:
- URLhaus - Malware URL tracking
- ThreatFox - IOC sharing platform
- MalwareBazaar - Malware sample repository
- SSL Blacklist - Malicious SSL certificate tracking
- YARAify - YARA rule matching service
