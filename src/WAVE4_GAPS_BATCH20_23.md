# Wave 4 Endpoint Gap Analysis — Batches 20–23
## Cyber/Threat Intel + Environment Connectors

**Date:** 2026-03-13
**Base path:** `digdigdig3/src/intelligence_feeds/`

---

## Batch 20 — Cyber/Threat Intel

### 1. AbuseIPDB (`cyber/abuseipdb/`)

Base URL: `https://api.abuseipdb.com/api/v2`

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| IP Lookup | Check IP | `GET /check` | YES (`Check`) | Core endpoint |
| IP Lookup | Check Block (CIDR) | `GET /check-block` | YES (`CheckBlock`) | |
| Reporting | Report IP | `POST /report` | YES (`Report`) | |
| Reporting | Bulk Report | `POST /bulk-report` | YES (`BulkReport`) | |
| Reporting | Clear Address | `DELETE /clear-address` | YES (`ClearAddress`) | |
| Lists | Blacklist | `GET /blacklist` | YES (`Blacklist`) | |
| **MISSING** | **Reports (paginated)** | **`GET /reports`** | **NO** | Returns paginated list of reports for a specific IP; distinct from Check |
| **MISSING** | **Categories** | **`GET /categories`** | Partial | Enum `Categories` exists in code but is NOT a real AbuseIPDB endpoint — the categories are static constants, not an API call |

**Summary:** 1 real missing endpoint (`/reports` for paginated IP reports). The `Categories` variant in the enum is a false entry — AbuseIPDB does not expose a `/categories` REST endpoint; category IDs are documented as static constants.

---

### 2. AlienVault OTX (`cyber/alienvault_otx/`)

Base URL: `https://otx.alienvault.com/api/v1`

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Pulses | Subscribed Pulses | `GET /pulses/subscribed` | YES | |
| Pulses | Pulse Activity | `GET /pulses/activity` | YES | |
| **MISSING** | **Get Pulse by ID** | **`GET /pulses/{id}`** | **NO** | Fetch a single pulse's full details |
| **MISSING** | **Create Pulse** | **`POST /pulses/create`** | **NO** | Submit new threat intelligence pulse |
| **MISSING** | **User Pulses** | **`GET /pulses/user/{username}`** | **NO** | Get all pulses by a specific user |
| **MISSING** | **Search Pulses** | **`GET /pulses/search`** | **NO** | Full-text search across pulses |
| **MISSING** | **My Pulses** | **`GET /pulses/me`** | **NO** | Get own authored pulses |
| Indicators | IPv4 General | `GET /indicators/IPv4/{ip}/general` | YES (`IpReputation`) | Only `/general` section covered |
| **MISSING** | **IPv4 Sections** | **`GET /indicators/IPv4/{ip}/{section}`** | **NO** | Sections: geo, malware, url_list, passive_dns, http_scans, reputation |
| **MISSING** | **IPv6 Indicators** | **`GET /indicators/IPv6/{ip}/general`** | **NO** | IPv6 addresses not covered |
| Indicators | Domain General | `GET /indicators/domain/{domain}/general` | YES (`DomainReputation`) | Only `/general` |
| **MISSING** | **Domain Sections** | **`GET /indicators/domain/{domain}/{section}`** | **NO** | Sections: malware, url_list, passive_dns, whois, http_scans |
| **MISSING** | **Hostname Sections** | **`GET /indicators/hostname/{host}/{section}`** | **NO** | Same section pattern as domain |
| Indicators | File General | `GET /indicators/file/{hash}/general` | YES (`FileReputation`) | Only `/general` |
| **MISSING** | **File Sections** | **`GET /indicators/file/{hash}/{section}`** | **NO** | Sections: analysis, malware-samples |
| Indicators | URL General | `GET /indicators/url/{url}/general` | YES (`UrlReputation`) | |
| **MISSING** | **CVE Indicators** | **`GET /indicators/cve/{cve}/general`** | **NO** | CVE lookup by ID |
| **MISSING** | **NIDS Rule Indicators** | **`GET /indicators/nids/{rule}/general`** | **NO** | Suricata/Snort rule lookup |

**Summary:** Significant gaps. Current implementation only covers the `/general` section of each indicator type and basic pulse listing. Missing: pulse CRUD, search, per-section indicator detail, IPv6, CVE, and NIDS indicator types.

---

### 3. Censys (`cyber/censys/`)

Base URL: `https://search.censys.io/api/v2`

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Hosts | Search Hosts | `POST /hosts/search` | YES | |
| Hosts | View Host | `GET /hosts/{ip}` | YES | |
| Hosts | Aggregate Hosts | `POST /hosts/aggregate` | YES | |
| Hosts | Diff Hosts | `GET /hosts/{ip}/diff` | YES | |
| **MISSING** | **Host Events** | **`GET /hosts/{ip}/events`** | **NO** | Historical change events for a host |
| **MISSING** | **Host Names** | **`GET /hosts/{ip}/names`** | **NO** | Hostnames the IP responded to during scanning |
| Certificates | Search Certs | `POST /certificates/search` | YES | |
| **MISSING** | **View Certificate** | **`GET /certificates/{fp}`** | **NO** | Full cert details by fingerprint |
| **MISSING** | **Hosts by Cert** | **`GET /certificates/{fp}/hosts`** | **NO** | List hosts currently presenting a certificate |
| **MISSING** | **Aggregate Certs** | **`POST /certificates/aggregate`** | **NO** | Report on cert field values |
| **MISSING** | **Account Info** | **`GET /account`** | **NO** | API quota, allowances, and plan info |

**Summary:** Missing host detail sub-endpoints (events, names) and 3 certificate endpoints. Account endpoint useful for quota monitoring.

---

### 4. Cloudflare Radar (`cyber/cloudflare_radar/`)

Base URL: `https://api.cloudflare.com/client/v4/radar`

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| HTTP | Top Locations | `/http/top/locations` | YES | |
| HTTP | Top ASes | `/http/top/ases` | YES | |
| HTTP | Summary Bot Class | `/http/summary/bot_class` | YES | |
| HTTP | Summary Device Type | `/http/summary/device_type` | YES | |
| HTTP | Summary HTTP Protocol | `/http/summary/http_protocol` | YES | |
| HTTP | Summary OS | `/http/summary/os` | YES | |
| HTTP | Summary Browser | `/http/summary/browser` | YES | |
| HTTP | Timeseries | `/http/timeseries` | YES | |
| Attacks | Layer 3 Summary | `/attacks/layer3/summary` | YES | |
| Attacks | Layer 7 Summary | `/attacks/layer7/summary` | YES | |
| Attacks | Layer 3 Timeseries | `/attacks/layer3/timeseries` | YES | |
| **MISSING** | **L7 Attack Timeseries** | **`/attacks/layer7/timeseries`** | **NO** | Layer 7 over-time data |
| **MISSING** | **L7 Top Locations** | **`/attacks/layer7/top/locations`** | **NO** | |
| **MISSING** | **L7 Top Ases** | **`/attacks/layer7/top/ases`** | **NO** | |
| **MISSING** | **L3 Top Locations** | **`/attacks/layer3/top/locations`** | **NO** | |
| DNS | Top DNS Locations | `/dns/top/locations` | YES | |
| **MISSING** | **DNS Top Domains** | **`/dns/top/domains`** | **NO** | Top queried domains |
| **MISSING** | **DNS Summary** | **`/dns/summary/dnssec`** | **NO** | DNSSEC adoption summary |
| **MISSING** | **BGP Top ASes** | **`/bgp/top/ases`** | **NO** | Top BGP announcing ASes |
| **MISSING** | **BGP Top Prefixes** | **`/bgp/top/prefixes`** | **NO** | Most announced prefixes |
| **MISSING** | **BGP Hijacks** | **`/bgp/hijacks/events`** | **NO** | BGP hijack detection events |
| **MISSING** | **BGP Route Leaks** | **`/bgp/leaks/events`** | **NO** | BGP route leak events |
| **MISSING** | **BGP Timeseries** | **`/bgp/timeseries`** | **NO** | BGP update volume over time |
| **MISSING** | **Email Summary** | **`/email/security/summary/spf`** | **NO** | SPF/DKIM/DMARC adoption |
| **MISSING** | **Email Timeseries** | **`/email/security/timeseries`** | **NO** | Email security trends |
| **MISSING** | **Netflows Top ASes** | **`/netflows/top/ases`** | **NO** | Netflow traffic top ASes |
| **MISSING** | **Internet Quality** | **`/quality/iqi/summary`** | **NO** | Internet Quality Index |
| **MISSING** | **Search Entities** | **`/entities/asns/search`** | **NO** | Search ASNs by name |
| **MISSING** | **Annotations** | **`/annotations`** | **NO** | Notable internet events/outages |
| Ranking | Top Domains | `/ranking/top` | YES | |
| **MISSING** | **Ranking Timeseries** | **`/ranking/domain/{domain}/history`** | **NO** | Domain rank history over time |

**Summary:** Large gap. Current implementation covers basic HTTP, attacks, and 1 DNS endpoint. Missing entire BGP, Email Security, Netflows, Quality, Annotations, and Entity search verticals — roughly 60% of available API surface.

---

### 5. NVD (`cyber/nvd/`)

Base URL: `https://services.nvd.nist.gov/rest/json`

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| CVEs | Search CVEs | `GET /cves/2.0` | YES | Supports 20+ filter params |
| CPEs | Search CPEs | `GET /cpes/2.0` | YES | |
| CPEs | CPE Match Strings | `GET /cpematch/2.0` | YES | |
| **MISSING** | **CVE Change History** | **`GET /cvehistory/2.0`** | **NO** | Monitor when/why CVEs change; essential for tracking severity updates |
| **MISSING** | **Data Sources** | **`GET /source/2.0`** | **NO** | Lists NVD data sources/contributors |

**Summary:** 2 missing endpoints. `cvehistory/2.0` is important for production use — it lets consumers detect when CVE scores or affected products change without re-fetching the entire CVE database.

---

## Batch 21 — Cyber/Threat Intel (continued)

### 6. RIPE NCC (`cyber/ripe_ncc/`)

Base URL: `https://stat.ripe.net/data`

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Resources | Country Resource Stats | `/country-resource-stats/data.json` | YES | |
| Resources | Country Resource List | `/country-resource-list/data.json` | YES | |
| AS | AS Overview | `/as-overview/data.json` | YES | |
| AS | Announced Prefixes | `/announced-prefixes/data.json` | YES | |
| AS | ASN Neighbours | `/asn-neighbours/data.json` | YES | |
| Routing | Routing Status | `/routing-status/data.json` | YES | |
| Routing | BGP State | `/bgp-state/data.json` | YES | |
| Network | Network Info | `/network-info/data.json` | YES | |
| Regional | RIR Stats Country | `/rir-stats-country/data.json` | YES | |
| Contact | Abuse Contact Finder | `/abuse-contact-finder/data.json` | YES | |
| **MISSING** | **Routing History** | **`/routing-history/data.json`** | **NO** | Historical announcements for a prefix |
| **MISSING** | **BGP Updates** | **`/bgp-updates/data.json`** | **NO** | BGP update activity for a resource |
| **MISSING** | **BGP Update Activity** | **`/bgp-update-activity/data.json`** | **NO** | Statistical BGP activity |
| **MISSING** | **BGPlay** | **`/bgplay/data.json`** | **NO** | BGP routing animation data |
| **MISSING** | **AS Path Length** | **`/as-path-length/data.json`** | **NO** | Path length distribution for an AS |
| **MISSING** | **AS Routing Consistency** | **`/as-routing-consistency/data.json`** | **NO** | IRR vs RIS consistency check |
| **MISSING** | **ASN Neighbours History** | **`/asn-neighbours-history/data.json`** | **NO** | Time-series of neighbour changes |
| **MISSING** | **Prefix Overview** | **`/prefix-overview/data.json`** | **NO** | Summary information for a prefix |
| **MISSING** | **Prefix Routing Consistency** | **`/prefix-routing-consistency/data.json`** | **NO** | IRR vs routing table check for prefix |
| **MISSING** | **Related Prefixes** | **`/related-prefixes/data.json`** | **NO** | Find topologically related prefixes |
| **MISSING** | **Address Space Hierarchy** | **`/address-space-hierarchy/data.json`** | **NO** | Parent/child prefix relationships |
| **MISSING** | **Address Space Usage** | **`/address-space-usage/data.json`** | **NO** | Utilisation stats for address space |
| **MISSING** | **RPKI Validation** | **`/rpki-validation/data.json`** | **NO** | RPKI validity for prefix-ASN pair |
| **MISSING** | **Country ASNs** | **`/country-asns/data.json`** | **NO** | Per-country registered+routed ASNs |
| **MISSING** | **DNS Blocklists** | **`/dns-blocklists/data.json`** | **NO** | DNS-based block list lookups |
| **MISSING** | **Reverse DNS** | **`/reverse-dns/data.json`** | **NO** | Reverse DNS for an IP/prefix |
| **MISSING** | **Looking Glass** | **`/looking-glass/data.json`** | **NO** | Live BGP table from RIS collectors |
| **MISSING** | **Whois** | **`/whois/data.json`** | **NO** | WHOIS data for any internet resource |
| **MISSING** | **IANA Registry Info** | **`/iana-registry-info/data.json`** | **NO** | IANA registration details |
| **MISSING** | **Visibility** | **`/visibility/data.json`** | **NO** | BGP visibility of a prefix across RIS |

**Summary:** Current implementation covers 10 of 60+ available data calls (~17%). The 20+ missing entries listed above are the highest-value additions. BGP history, RPKI validation, whois, and prefix consistency checks are the most actionable gaps.

---

### 7. Shodan (`cyber/shodan/`)

Base URL: `https://api.shodan.io`

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Host | Host Info | `GET /shodan/host/{ip}` | YES | |
| Search | Host Count | `GET /shodan/host/count` | YES | |
| Search | Host Search | `GET /shodan/host/search` | YES | |
| DNS | DNS Resolve | `GET /dns/resolve` | YES | |
| DNS | DNS Reverse | `GET /dns/reverse` | YES | |
| Utility | My IP | `GET /tools/myip` | YES | |
| Account | API Info | `GET /api-info` | YES | |
| Meta | Ports | `GET /shodan/ports` | YES | |
| Meta | Protocols | `GET /shodan/protocols` | YES | |
| **MISSING** | **Host Search Facets** | **`GET /shodan/host/search/facets`** | **NO** | List valid facets for facet analysis |
| **MISSING** | **Host Search Tokens** | **`GET /shodan/host/search/tokens`** | **NO** | Break search query into tokens |
| **MISSING** | **Scan Submit** | **`POST /shodan/scan`** | **NO** | Request on-demand scan of IPs |
| **MISSING** | **Scan Status** | **`GET /shodan/scan/{id}`** | **NO** | Check status of a submitted scan |
| **MISSING** | **Scan Internet** | **`POST /shodan/scan/internet`** | **NO** | Crawl entire internet for a port/protocol (requires credits) |
| **MISSING** | **Alert Create** | **`POST /shodan/alert`** | **NO** | Create network alert for IP ranges |
| **MISSING** | **Alert List** | **`GET /shodan/alert/info`** | **NO** | List active network alerts |
| **MISSING** | **Alert Delete** | **`DELETE /shodan/alert/{id}`** | **NO** | Delete a network alert |
| **MISSING** | **Saved Searches** | **`GET /shodan/query`** | **NO** | List/search saved queries directory |
| **MISSING** | **Saved Search Tags** | **`GET /shodan/query/tags`** | **NO** | List popular tags for saved queries |
| **MISSING** | **DNS Domain** | **`GET /dns/domain/{domain}`** | **NO** | DNS entries and historical records for domain |
| **MISSING** | **Exploit Search** | **`GET https://exploits.shodan.io/api/search`** | **NO** | Exploit database search (different base URL) |
| **MISSING** | **InternetDB Lookup** | **`GET https://internetdb.shodan.io/{ip}`** | **NO** | Fast IP lookup (separate lightweight API) |

**Summary:** Core search/lookup covered but missing entire monitoring (alerts), scanning, saved queries, and the exploit/internetdb auxiliary APIs.

---

### 8. URLhaus (`cyber/urlhaus/`)

Base URL: `https://urlhaus-api.abuse.ch/v1`

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| URLs | Recent URLs | `GET /urls/recent/limit/{n}/` | YES | |
| URLs | URL Lookup | `POST /url/` | YES | |
| Hosts | Host Lookup | `POST /host/` | YES | |
| Payloads | Payload Lookup | `POST /payload/` | YES | |
| Tags | Tag Lookup | `POST /tag/` | YES | |
| **MISSING** | **Recent Payloads** | **`GET /payloads/recent/`** | **NO** | Latest malware samples collected by URLhaus |
| **MISSING** | **Signature Lookup** | **`POST /signature/`** | **NO** | Lookup URLs/payloads by malware signature name |
| **MISSING** | **Download Sample** | **`GET /download/{sha256hash}`** | **NO** | Download a malware sample by SHA256 |
| **MISSING** | **Daily Batch** | **`GET /downloads/daily/`** | **NO** | Batch download of daily malware samples (zip) |
| **MISSING** | **Hourly Batch** | **`GET /downloads/hourly/`** | **NO** | Batch download of hourly malware samples |

**Summary:** 5 missing endpoints, most notably recent payloads, signature lookup, and malware sample download — these are central to URLhaus's value proposition as a malware sample repository.

---

### 9. VirusTotal (`cyber/virustotal/`)

Base URL: `https://www.virustotal.com/api/v3`

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Files | File Report | `GET /files/{hash}` | YES | |
| URLs | URL Report | `GET /urls/{id}` | YES | |
| Domains | Domain Report | `GET /domains/{domain}` | YES | |
| IPs | IP Report | `GET /ip_addresses/{ip}` | YES | |
| Search | Search | `GET /search` | YES | |
| **MISSING** | **File Scan (Upload)** | **`POST /files`** | **NO** | Upload file for scanning |
| **MISSING** | **File Rescan** | **`POST /files/{hash}/analyse`** | **NO** | Request fresh scan of known file |
| **MISSING** | **URL Scan (Submit)** | **`POST /urls`** | **NO** | Submit URL for scanning |
| **MISSING** | **URL Rescan** | **`POST /urls/{id}/analyse`** | **NO** | Re-analyse a known URL |
| **MISSING** | **File Behaviour** | **`GET /files/{hash}/behaviours`** | **NO** | Sandbox execution reports |
| **MISSING** | **File Comments** | **`GET /files/{hash}/comments`** | **NO** | Community comments on file |
| **MISSING** | **Domain Comments** | **`GET /domains/{domain}/comments`** | **NO** | Comments on domain |
| **MISSING** | **IP Comments** | **`GET /ip_addresses/{ip}/comments`** | **NO** | Comments on IP |
| **MISSING** | **Add Comment** | **`POST /{object_type}/{id}/comments`** | **NO** | Add community comment |
| **MISSING** | **File Votes** | **`GET /files/{hash}/votes`** | **NO** | Community votes (malicious/harmless) |
| **MISSING** | **Submit Vote** | **`POST /{object_type}/{id}/votes`** | **NO** | Submit malicious/harmless vote |
| **MISSING** | **Related Objects** | **`GET /files/{hash}/relationships`** | **NO** | Related files, dropped files, contacted domains |
| **MISSING** | **Collections** | **`GET /collections/{id}`** | **NO** | Threat intelligence collections (Premium) |
| **MISSING** | **Threat Actors** | **`GET /attack_techniques/{id}`** | **NO** | ATT&CK technique info |

**Summary:** Current implementation is read-only reporting only. Missing: file/URL submission, rescan, behavioral analysis (sandbox), community interactions (comments, votes), relationship graph traversal, and Premium intelligence objects.

---

### 10. Feodo Tracker (`feodo_tracker/`)

Base URL: `https://feodotracker.abuse.ch`

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Blocklists | IP Blocklist JSON | `/downloads/ipblocklist.json` | YES | Full IOC metadata |
| Blocklists | IP Blocklist CSV | `/downloads/ipblocklist.csv` | YES | |
| Blocklists | Aggressive Blocklist CSV | `/downloads/ipblocklist_aggressive.csv` | YES | All historical |
| Blocklists | Recommended JSON | `/downloads/ipblocklist_recommended.json` | YES | Curated list |
| **MISSING** | **Plain-text Blocklist** | **`/downloads/ipblocklist.txt`** | **NO** | IPs-only text format (firewall-friendly) |
| **MISSING** | **Recommended TXT** | **`/downloads/ipblocklist_recommended.txt`** | **NO** | IPs-only recommended list |
| **MISSING** | **Recommended McAfee** | **`/downloads/ipblocklist_recommended_mcafee.txt`** | **NO** | McAfee Web Gateway format |
| **MISSING** | **Recommended Palo Alto** | **`/downloads/ipblocklist_recommended_paloalto.txt`** | **NO** | Palo Alto firewall format |
| **MISSING** | **Aggressive TXT** | **`/downloads/ipblocklist_aggressive.txt`** | **NO** | Aggressive IPs-only text |
| **MISSING** | **Suricata Rules** | **`/downloads/feodotracker.rules`** | **NO** | Suricata/Snort IDS ruleset |
| **MISSING** | **Suricata Rules (tar.gz)** | **`/downloads/feodotracker.tar.gz`** | **NO** | |
| **MISSING** | **Suricata Aggressive Rules** | **`/downloads/feodotracker_aggressive.rules`** | **NO** | Aggressive IDS ruleset |

**Summary:** Plain-text and IDS ruleset formats entirely missing. The `.txt` and `.rules` formats are the most commonly consumed by security tools and firewalls.

---

### 11. C2Intel Feeds (`c2intel_feeds/`)

Base URL: `https://raw.githubusercontent.com/drb-ra/C2IntelFeeds/master/feeds`

| Category | Endpoint / Feed File | Path | We Have? | Notes |
|----------|----------------------|------|----------|-------|
| IP Feeds | All-time IPs | `/IPC2s.csv` | YES | |
| IP Feeds | 30-day IPs | `/IPC2s-30day.csv` | YES | |
| IP Feeds | 7-day IPs | `/IPC2s-7day.csv` | YES | |
| IP Feeds | 90-day IPs | `/IPC2s-90day.csv` | YES | |
| Domain Feeds | All-time domains | `/domainC2s.csv` | YES | |
| Domain Feeds | 30-day domains | `/domainC2s-30day.csv` | YES | |
| Domain Feeds | 90-day domains | `/domainC2s-90day.csv` | YES | |
| **MISSING** | **Domains + Filter (abused)** | **`/domainC2s-filter-abused.csv`** | **NO** | Domains filtered to remove known-abused hosting |
| **MISSING** | **Domains + URL** | **`/domainC2swithURL.csv`** | **NO** | Domains with specific C2 URI paths |
| **MISSING** | **Domains + URL + IP** | **`/domainC2swithURLwithIP-filter-abused.csv`** | **NO** | Full records: domain + URL + resolved IP |
| **MISSING** | **Domains + URL + IP (30d)** | **`/domainC2swithURLwithIP-30day-filter-abused.csv`** | **NO** | 30-day variant of above |
| **MISSING** | **Unverified IP Feed** | **`/unverified/IPC2s.csv`** | **NO** | Potential C2 IPs not yet validated |
| **MISSING** | **Unverified Domain Feed** | **`/unverified/domainC2s.csv`** | **NO** | Potential C2 domains not yet validated |

**Summary:** Time-windowed IP/domain feeds covered but missing the filtered/enriched variants (with URL paths, resolved IPs) and the unverified sub-feeds. The `domainC2swithURL` feeds are particularly valuable for SIEM enrichment.

---

## Batch 22 — Environment

### 12. GDACS (`environment/gdacs/`)

Base URL: `https://www.gdacs.org/gdacsapi/api`

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Events | Event List | `GET /events/geteventlist/SEARCH` | YES | Supports eventlist, fromdate, todate, alertlevel, pagenumber params |
| Events | Event by ID | `GET /events/geteventdata/GetByEventId` | YES | |
| **MISSING** | **Event Data (Type+ID)** | **`GET /events/geteventdata?eventtype={type}&eventid={id}`** | **NO** | Alternative event fetch using type+ID params (vs path-based) |
| **MISSING** | **Event Geometry** | **`GET /events/getgeometry`** | **NO** | Geospatial polygon/geometry for an event |
| **MISSING** | **Event Reports** | **`GET /events/getreportlist`** | **NO** | GDACS situation reports for an event |
| **MISSING** | **Resources/Contacts** | **`GET /events/getresource`** | **NO** | Humanitarian resources linked to event |

**Summary:** Core event list/lookup present. Missing geospatial, situation reports, and resource endpoints useful for comprehensive disaster monitoring.

---

### 13. Global Forest Watch (`environment/global_forest_watch/`)

Base URL: `https://data-api.globalforestwatch.org`

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Datasets | List Datasets | `GET /dataset` | YES | |
| Datasets | Get Dataset | `GET /dataset/{id}` | YES | |
| Datasets | Dataset Latest | `GET /dataset/{id}/latest` | YES | |
| Datasets | Query Dataset | `GET /dataset/{id}/{version}/query` | YES | |
| Forest Change | Statistics | `GET /forest-change/statistics` | YES | |
| Forest Change | Tree Cover Loss | `GET /forest-change/loss` | YES | |
| Forest Change | Tree Cover Gain | `GET /forest-change/gain` | YES | |
| Alerts | Fire Alerts | `GET /fire-alerts` | YES | |
| Alerts | Deforestation Alerts | `GET /deforestation-alerts` | YES | |
| **MISSING** | **GLAD-L Alerts** | **`GET /dataset/gfw_integrated_alerts/latest/query`** | **NO** | Integrated deforestation alerts (GLAD Landsat) via dataset query |
| **MISSING** | **VIIRS Fire Alerts** | **`GET /dataset/nasa_viirs_fire_alerts/latest/query`** | **NO** | VIIRS active fire data via dataset query |
| **MISSING** | **Geostore Create** | **`POST /geostore`** | **NO** | Create a geometry for area-based analysis |
| **MISSING** | **Geostore Lookup** | **`GET /geostore/{id}`** | **NO** | Retrieve a stored geometry |
| **MISSING** | **Download Dataset** | **`GET /dataset/{id}/{version}/download/{format}`** | **NO** | Download dataset in CSV/JSON/Shapefile |

**Summary:** Good baseline coverage. Missing: direct GLAD/VIIRS alert queries (these are the primary real-time deforestation signals), geostore for custom polygon analysis, and download endpoint.

---

### 14. NASA EONET (`environment/nasa_eonet/`)

Base URL: `https://eonet.gsfc.nasa.gov/api/v3`

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Events | Events | `GET /events` | YES | |
| Metadata | Categories | `GET /categories` | YES | |
| Metadata | Sources | `GET /sources` | YES | |
| **MISSING** | **Events GeoJSON** | **`GET /events/geojson`** | **NO** | Same events structured as GeoJSON FeatureCollection |
| **MISSING** | **Layers** | **`GET /layers`** | **NO** | WMS/WMTS web service references for NASA imagery by category |
| **MISSING** | **Events by Category** | **`GET /categories/{id}`** | **NO** | Events filtered to a single category |
| **MISSING** | **Events RSS** | **`GET /events/rss`** | **NO** | RSS feed of natural events |
| **MISSING** | **Events ATOM** | **`GET /events/atom`** | **NO** | ATOM feed of natural events |

**Summary:** Core JSON event endpoint covered. Missing GeoJSON variant (important for map integration), layers (NASA imagery links), category-filtered events, and feed formats.

---

### 15. NASA FIRMS (`environment/nasa_firms/`)

Base URL: `https://firms.modaps.eosdis.nasa.gov/api`

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Fire Data | Area Query | `GET /area` | YES | BBox-based fire hotspots |
| Fire Data | Country Query | `GET /country` | YES | Country-code fire hotspots |
| **MISSING** | **Data Availability** | **`GET /data_availability/`** | **NO** | Date ranges available per sensor/dataset |
| **MISSING** | **Map Key Status** | **`GET /map_key/{MAP_KEY}`** | **NO** | Verify key validity and check transaction count |
| **MISSING** | **KML Fire Footprints** | **`GET /kml_fire_footprints/{source}/{days}/{date}`** | **NO** | KML-formatted fire detection footprints |
| **MISSING** | **Missing Data** | **`GET /missing_data`** | **NO** | Dates with missing satellite coverage |

**Summary:** Only the 2 data-retrieval endpoints covered; missing all API management endpoints (data availability, key status, missing data calendar) and KML format output.

---

### 16. NOAA (`environment/noaa/`)

Base URL: `https://www.ncei.noaa.gov/cdo-web/api/v2`

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Data | Climate Data | `GET /data` | YES | Core observations |
| Datasets | List Datasets | `GET /datasets` | YES | |
| Datasets | Get Dataset | `GET /datasets/{id}` | YES | |
| Datatypes | List Datatypes | `GET /datatypes` | YES | |
| Datatypes | Get Datatype | `GET /datatypes/{id}` | YES | |
| Locations | Location Categories | `GET /locationcategories` | YES | |
| Locations | List Locations | `GET /locations` | YES | |
| Locations | Get Location | `GET /locations/{id}` | YES | |
| Stations | List Stations | `GET /stations` | YES | |
| Stations | Get Station | `GET /stations/{id}` | YES | |
| **MISSING** | **Location Category by ID** | **`GET /locationcategories/{id}`** | **NO** | Get specific location category |
| **NOTE** | **CDO v2 Deprecated** | — | — | `ncei.noaa.gov/cdo-web/api/v2` deprecated; new endpoint is `ncei.noaa.gov/access/services/data/v1` |
| **MISSING** | **NCEI Data Service v1** | **`GET https://www.ncei.noaa.gov/access/services/data/v1`** | **NO** | Replacement API with different query params |

**Summary:** CDO v2 coverage is essentially complete (minor gap: `locationcategories/{id}`). However, the entire CDO v2 API is now deprecated by NOAA — the replacement is `ncei.noaa.gov/access/services/data/v1` with a different query interface. This is a major migration concern.

---

## Batch 23 — Environment (continued)

### 17. NWS Alerts (`environment/nws_alerts/`)

Base URL: `https://api.weather.gov`

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Alerts | Active Alerts | `GET /alerts/active` | YES | |
| Alerts | Alert by ID | `GET /alerts/{id}` | YES | |
| Alerts | Alerts by Zone | `GET /alerts/active/zone/{zone}` | YES | |
| Alerts | Alerts by Area | `GET /alerts/active/area/{area}` | YES | |
| **MISSING** | **Alert Types** | **`GET /alerts/types`** | **NO** | List all valid alert event types |
| **MISSING** | **Historical Alerts** | **`GET /alerts`** | **NO** | Past 7 days of all alerts (not just active) |
| **MISSING** | **Forecast by Point** | **`GET /points/{lat},{lon}`** | **NO** | Resolve coordinates to grid; returns forecast URLs |
| **MISSING** | **Grid Forecast** | **`GET /gridpoints/{office}/{x},{y}/forecast`** | **NO** | 12-hour period forecast for a grid cell |
| **MISSING** | **Hourly Forecast** | **`GET /gridpoints/{office}/{x},{y}/forecast/hourly`** | **NO** | Hourly forecast for a grid cell |
| **MISSING** | **Observations** | **`GET /stations/{stationId}/observations/latest`** | **NO** | Latest actual observed weather |
| **MISSING** | **Stations** | **`GET /stations`** | **NO** | List/search observation stations |
| **MISSING** | **Zone Forecast** | **`GET /zones/{type}/{zoneId}/forecast`** | **NO** | Text forecast for a zone |
| **MISSING** | **Radar Queues** | **`GET /radar/queues`** | **NO** | Radar station queue status |

**Summary:** Implementation is alerts-only. The NWS API is a full weather platform — forecast, observation, and station endpoints are entirely missing.

---

### 18. OpenWeatherMap (`environment/open_weather_map/`)

Base URL: `https://api.openweathermap.org/data/2.5` (current; needs upgrade to 3.0)

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Weather | Current Weather | `GET /weather` | YES | Works for both city and coords |
| Forecast | 5-day Forecast | `GET /forecast` | YES | 3-hour step |
| Air Quality | Air Pollution | `GET /air_pollution` | YES | |
| Air Quality | Air Pollution History | `GET /air_pollution/history` | YES | |
| **MISSING** | **One Call API 3.0** | **`GET https://api.openweathermap.org/data/3.0/onecall`** | **NO** | Unified endpoint: current + minute + hourly + daily + alerts in one call; v2.5 deprecated June 2024 |
| **MISSING** | **One Call Historical** | **`GET https://api.openweathermap.org/data/3.0/onecall/timemachine`** | **NO** | Historical weather for any date since 1979 |
| **MISSING** | **One Call Day Aggregation** | **`GET https://api.openweathermap.org/data/3.0/onecall/day_summary`** | **NO** | Daily aggregated stats for any date |
| **MISSING** | **Geocoding Direct** | **`GET https://api.openweathermap.org/geo/1.0/direct`** | **NO** | City name → coordinates |
| **MISSING** | **Geocoding Reverse** | **`GET https://api.openweathermap.org/geo/1.0/reverse`** | **NO** | Coordinates → city name |
| **MISSING** | **Geocoding by ZIP** | **`GET https://api.openweathermap.org/geo/1.0/zip`** | **NO** | ZIP code → coordinates |
| **MISSING** | **Air Pollution Forecast** | **`GET /air_pollution/forecast`** | **NO** | 4-day air quality forecast |
| **MISSING** | **Historical Weather** | **`GET /history/city`** | **NO** | Past weather (History API, paid) |
| **MISSING** | **16-day Daily Forecast** | **`GET /forecast/daily`** | **NO** | Extended daily forecast (paid) |
| **MISSING** | **UV Index** | **`GET /uvi`** | **NO** | UV index (deprecated, now in One Call) |
| **NOTE** | **API v2.5 Deprecation** | — | — | One Call 2.5 deprecated June 2024; base URL should migrate to `data/3.0` |

**Summary:** Current implementation is basic and built on deprecated v2.5. The One Call API 3.0 is the strategic endpoint to add — it replaces multiple v2.5 calls. Geocoding endpoints are entirely absent, making location resolution impossible without external tools.

---

### 19. OpenAQ (`environment/openaq/`)

Base URL: `https://api.openaq.org/v2` (OUTDATED — v2 retired January 31, 2025)

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Locations | Locations | `GET /v2/locations` | YES (but v2!) | V2 now returns HTTP 410 Gone |
| Locations | Location by ID | `GET /v2/locations/{id}` | YES (but v2!) | Dead endpoint |
| Measurements | Measurements | `GET /v2/measurements` | YES (but v2!) | Dead endpoint |
| Measurements | Latest | `GET /v2/latest` | YES (but v2!) | Dead endpoint |
| Measurements | Averages | `GET /v2/averages` | YES (but v2!) | Dead endpoint |
| Meta | Countries | `GET /v2/countries` | YES (but v2!) | Dead endpoint |
| Meta | Cities | `GET /v2/cities` | YES (but v2!) | Dead endpoint |
| Meta | Parameters | `GET /v2/parameters` | YES (but v2!) | Dead endpoint |
| **CRITICAL** | **V3 Locations** | **`GET https://api.openaq.org/v3/locations`** | **NO** | Current active endpoint |
| **CRITICAL** | **V3 Sensors** | **`GET https://api.openaq.org/v3/sensors/{id}`** | **NO** | New in v3: per-sensor data |
| **CRITICAL** | **V3 Measurements** | **`GET https://api.openaq.org/v3/sensors/{id}/measurements`** | **NO** | Per-sensor measurement series |
| **CRITICAL** | **V3 Latest** | **`GET https://api.openaq.org/v3/sensors/{id}/measurements/latest`** | **NO** | Latest reading per sensor |
| **MISSING** | **V3 Hourly Averages** | **`GET https://api.openaq.org/v3/sensors/{id}/hours`** | **NO** | Hourly averages |
| **MISSING** | **V3 Daily Averages** | **`GET https://api.openaq.org/v3/sensors/{id}/days`** | **NO** | Daily averages |
| **MISSING** | **V3 Yearly Averages** | **`GET https://api.openaq.org/v3/sensors/{id}/years`** | **NO** | Yearly averages |
| **MISSING** | **V3 Providers** | **`GET https://api.openaq.org/v3/providers`** | **NO** | New in v3: list data providers |
| **MISSING** | **V3 Instruments** | **`GET https://api.openaq.org/v3/instruments`** | **NO** | New in v3: sensor instruments |
| **MISSING** | **V3 Manufacturers** | **`GET https://api.openaq.org/v3/manufacturers`** | **NO** | New in v3: instrument manufacturers |
| **MISSING** | **V3 Owners** | **`GET https://api.openaq.org/v3/owners`** | **NO** | New in v3: location owners |
| **MISSING** | **V3 Countries** | **`GET https://api.openaq.org/v3/countries`** | **NO** | |
| **MISSING** | **V3 Parameters** | **`GET https://api.openaq.org/v3/parameters`** | **NO** | |

**Summary:** CRITICAL — entire connector targets a dead API (v2 retired Jan 31, 2025). All endpoints currently return HTTP 410 Gone. The entire connector needs migration to v3, which has a substantially different data model (location → sensor hierarchy replaces location → measurement).

---

### 20. USGS Earthquake (`environment/usgs_earthquake/`)

Base URL: `https://earthquake.usgs.gov/fdsnws/event/1`

| Category | Endpoint | Path | We Have? | Notes |
|----------|----------|------|----------|-------|
| Data | Query | `GET /query` | YES | Main earthquake search |
| Data | Count | `GET /count` | YES | Count matching earthquakes |
| **MISSING** | **Catalogs** | **`GET /catalogs`** | **NO** | List available earthquake catalogs |
| **MISSING** | **Contributors** | **`GET /contributors`** | **NO** | List data contributors |
| **MISSING** | **Version** | **`GET /version`** | **NO** | Service version number |
| **MISSING** | **Application JSON** | **`GET /application.json`** | **NO** | List valid enumerated parameter values |
| **MISSING** | **Application WADL** | **`GET /application.wadl`** | **NO** | WADL service description |
| **MISSING** | **GeoJSON Feed** | **`GET https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/all_hour.geojson`** | **NO** | Real-time GeoJSON feed (separate feed URL) — e.g. all_hour, all_day, 2.5_week, 4.5_month |

**Summary:** Core query/count endpoints present. Missing metadata endpoints (catalogs, contributors, version, application.json) and the real-time GeoJSON feed system which uses a different base URL.

---

## Priority Summary

| Priority | Connector | Issue |
|----------|-----------|-------|
| CRITICAL | OpenAQ | Entire connector targeting dead API (v2 retired Jan 2025) — all calls return 410 |
| HIGH | OpenWeatherMap | Base URL targets deprecated v2.5 (One Call 2.5 gone June 2024); missing One Call 3.0 and Geocoding |
| HIGH | AlienVault OTX | Only `/general` sections covered; missing pulse CRUD, search, per-section indicators |
| HIGH | Cloudflare Radar | BGP, Email Security, Netflows, Quality, Annotations verticals entirely absent |
| HIGH | RIPE NCC | Only 10 of 60+ data calls implemented; missing BGP history, RPKI, whois |
| HIGH | URLhaus | Missing recent payloads, signature lookup, and malware sample download |
| HIGH | VirusTotal | Read-only; missing file/URL submission, behavioral analysis, community interactions |
| MEDIUM | NWS Alerts | Alerts-only; forecast, observation, and station endpoints completely missing |
| MEDIUM | Shodan | Missing alerts, scanning, saved searches, and exploit/internetdb auxiliary APIs |
| MEDIUM | Feodo Tracker | Missing plain-text and IDS ruleset formats |
| MEDIUM | NASA EONET | Missing GeoJSON variant, layers, and feed formats |
| MEDIUM | NASA FIRMS | Missing data_availability, map_key status, KML output |
| MEDIUM | Censys | Missing host events/names and 3 certificate sub-endpoints |
| MEDIUM | C2Intel Feeds | Missing enriched domain feeds (with URL paths + resolved IPs) and unverified sub-feeds |
| LOW | AbuseIPDB | Missing `/reports` paginated endpoint; spurious `Categories` enum entry |
| LOW | NVD | Missing CVE change history and data sources endpoints |
| LOW | GDACS | Missing geometry, reports, and resources endpoints |
| LOW | GFW | Missing GLAD/VIIRS direct queries, geostore, and download endpoint |
| LOW | NOAA | CDO v2 deprecated (new: access/services/data/v1); minor: locationcategories/{id} |
| LOW | USGS Earthquake | Missing metadata endpoints and real-time GeoJSON feed system |

---

## Sources

- [AbuseIPDB API v2 Documentation](https://docs.abuseipdb.com/)
- [AlienVault OTX External API Documentation](https://otx.alienvault.com/assets/static/external_api.html)
- [Censys Search API v2](https://search.censys.io/api)
- [Cloudflare Radar API Reference](https://developers.cloudflare.com/api/resources/radar/)
- [NVD Developers — Start Here](https://nvd.nist.gov/developers/start-here)
- [NVD Vulnerability APIs](https://nvd.nist.gov/developers/vulnerabilities)
- [RIPEstat Data API](https://stat.ripe.net/docs/data-api/ripestat-data-api)
- [Shodan Developer API](https://developer.shodan.io/api)
- [URLhaus Community API](https://urlhaus.abuse.ch/api/)
- [VirusTotal API v3 Overview](https://docs.virustotal.com/reference/overview)
- [Feodo Tracker Blocklist](https://feodotracker.abuse.ch/blocklist/)
- [C2IntelFeeds GitHub Repository](https://github.com/drb-ra/C2IntelFeeds)
- [GDACS API Quick Start](https://www.gdacs.org/Documents/2025/GDACS_API_quickstart_v1.pdf)
- [Global Forest Watch Data API](https://data-api.globalforestwatch.org/)
- [NASA EONET API v3 Documentation](https://eonet.gsfc.nasa.gov/docs/v3)
- [NASA FIRMS API](https://firms.modaps.eosdis.nasa.gov/api/)
- [NOAA CDO Web Services v2](https://www.ncdc.noaa.gov/cdo-web/webservices/v2)
- [NWS Weather API Documentation](https://www.weather.gov/documentation/services-web-api)
- [OpenWeatherMap API](https://openweathermap.org/api)
- [OpenAQ API v3 Documentation](https://docs.openaq.org/)
- [USGS Earthquake Catalog API](https://earthquake.usgs.gov/fdsnws/event/1/)
