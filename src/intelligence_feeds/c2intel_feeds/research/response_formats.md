# C2IntelFeeds Response Formats

## Content-Type

All feeds return plain text CSV:

```http
HTTP/1.1 200 OK
Content-Type: text/plain; charset=utf-8
```

## CSV Format

### General Structure

- **Header row**: Starts with `#` (comment marker)
- **Delimiter**: Comma (`,`)
- **Encoding**: UTF-8
- **Line endings**: Unix-style (`\n`)
- **Quoting**: None observed (fields not quoted)

## Feed Type Examples

### 1. IP Feed (IPC2s-30day.csv)

**URL**: `https://raw.githubusercontent.com/drb-ra/C2IntelFeeds/master/feeds/IPC2s-30day.csv`

**Response**:

```csv
#ip,ioc
1.12.231.30,Possible Cobaltstrike C2 IP
1.12.66.17,Possible Cobaltstrike C2 IP
1.14.157.231,Possible Cobaltstrike C2 IP
1.14.241.63,Possible Cobaltstrike C2 IP
1.15.171.190,Possible Cobaltstrike C2 IP
1.15.246.91,Possible Cobaltstrike C2 IP
1.15.25.138,Possible Cobaltstrike C2 IP
1.15.25.148,Possible Cobaltstrike C2 IP
1.94.183.238,Possible Cobaltstrike C2 IP
101.126.144.111,Possible Cobaltstrike C2 IP
101.132.167.9,Possible Cobaltstrike C2 IP
101.132.173.62,Possible Cobaltstrike C2 IP
101.133.225.51,Possible Cobaltstrike C2 IP
101.200.193.211,Possible Cobaltstrike C2 IP
101.201.180.191,Possible Cobaltstrike C2 IP
```

**Typical row count**: 500-1000 IPs (30-day window)

**Structure**:
- Column 1: IPv4 address
- Column 2: IOC classification

### 2. Domain Feed (domainC2s.csv)

**URL**: `https://raw.githubusercontent.com/drb-ra/C2IntelFeeds/master/feeds/domainC2s.csv`

**Response**:

```csv
#domain,ioc
2458ccd60cc54149bb05537717d831f0--8000.ap-shanghai2.cloudstudio.club,Possible Cobalt Strike C2 Domain
accesserdsc.com,Possible Cobalt Strike C2 Domain
api.cryptoprot.info,Possible Cobalt Strike C2 Fronting Domain
api.shenzhenschool.fun,Possible Cobalt Strike C2 Domain
asusupdateserver.asuscomm.com,Possible Cobalt Strike C2 Domain
auth.inmediavault.com,Possible Cobalt Strike C2 Domain
bgfi-groupe.com,Possible Cobalt Strike C2 Fronted Domain
check.judicical.ml,Possible Cobalt Strike C2 Domain
check1.judicical.ml,Possible Cobalt Strike C2 Domain
cryptoprot.info,Possible Cobalt Strike C2 Fronted Domain
d14hh3kwt0vf8s.cloudfront.net,Possible Cobalt Strike C2 Fronting Domain
dakk5rnsax46s.cfc-execute.su.baidubce.com,Possible Cobalt Strike C2 Domain
```

**Structure**:
- Column 1: FQDN
- Column 2: IOC classification (includes "Fronting" and "Fronted" variants)

### 3. Domain + URL Feed (domainC2swithURL-30day.csv)

**URL**: `https://raw.githubusercontent.com/drb-ra/C2IntelFeeds/master/feeds/domainC2swithURL-30day.csv`

**Response** (example structure):

```csv
#domain,ioc,url_path
accesserdsc.com,Possible Cobalt Strike C2 Domain,/en_US/all.js
api.cryptoprot.info,Possible Cobalt Strike C2 Fronting Domain,/dhl
api.shenzhenschool.fun,Possible Cobalt Strike C2 Domain,/api/x
check.judicical.ml,Possible Cobalt Strike C2 Domain,/jquery-3.3.1.min.js
asusupdateserver.asuscomm.com,Possible Cobalt Strike C2 Domain,/submit.php
```

**Structure**:
- Column 1: FQDN
- Column 2: IOC classification
- Column 3: URI path

### 4. Domain + URL + IP Feed (domainC2swithURLwithIP-30day.csv)

**URL**: `https://raw.githubusercontent.com/drb-ra/C2IntelFeeds/master/feeds/domainC2swithURLwithIP-30day.csv`

**Response**:

```csv
#domain,ioc,url_path,ip
120vip.top,Possible Cobalt Strike C2 Fronting Domain,/ptj,106.55.188.70
1258922563-2333n6dmlx.ap-guangzhou.tencentscf.com,Possible Cobalt Strike C2 Domain,/Test/protect/JZJ8DALCUB,43.139.50.42
1401675222-3ywn7qjp3t.ap-guangzhou.tencentscf.com,Possible Cobalt Strike C2 Fronted Domain,/s/ref=nb_sb_noss_1/...,123.56.226.71
2458ccd60cc54149bb05537717d831f0--8000.ap-shanghai2.cloudstudio.club,Possible Cobalt Strike C2 Domain,/s/58462514417,27.124.30.18
5b0rgq8mxzxgv.cfc-execute.bj.baidubce.com,Possible Cobalt Strike C2 Domain,/jquery-3.3.1.min.js,38.147.172.196
5ndg65b68274v.cfc-execute.bj.baidubce.com,Possible Cobalt Strike C2 Domain,/api/x,101.34.66.77
95mfmnebv9a1r.cfc-execute.gz.baidubce.com,Possible Cobalt Strike C2 Domain,/api/x,111.229.43.212
abyssestrinity.com,Possible Cobalt Strike C2 Fronted Domain,/match,216.126.224.23
accesserdsc.com,Possible Cobalt Strike C2 Domain,/en_US/all.js,154.201.74.112
```

**Structure**:
- Column 1: FQDN
- Column 2: IOC classification
- Column 3: URI path
- Column 4: IPv4 address

**Most comprehensive feed**: Provides full C2 infrastructure context.

## HTTP Response Headers (Typical)

```http
HTTP/1.1 200 OK
Content-Type: text/plain; charset=utf-8
Content-Length: 35142
Cache-Control: max-age=300
Last-Modified: Sat, 15 Feb 2026 14:30:00 GMT
ETag: "a1b2c3d4e5f6..."
X-Content-Type-Options: nosniff
```

### Key Headers

| Header | Description | Example Value |
|--------|-------------|---------------|
| Content-Type | MIME type | `text/plain; charset=utf-8` |
| Content-Length | File size in bytes | `35142` |
| Cache-Control | CDN cache duration | `max-age=300` |
| Last-Modified | File modification timestamp | `Sat, 15 Feb 2026 14:30:00 GMT` |
| ETag | Entity tag for caching | `"a1b2c3d4e5f6..."` |

## Parsing Examples

### Rust CSV Parsing

```rust
use csv::ReaderBuilder;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct IpIndicator {
    #[serde(rename = "#ip")]
    ip: String,
    ioc: String,
}

fn parse_ip_feed(csv_content: &str) -> Result<Vec<IpIndicator>, csv::Error> {
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(csv_content.as_bytes());

    reader.deserialize::<IpIndicator>()
        .collect()
}
```

**Note**: Header column name is `#ip`, not `ip`.

### Alternative: Manual Parsing (Skip Header)

```rust
fn parse_ip_feed_manual(csv_content: &str) -> Vec<(String, String)> {
    csv_content
        .lines()
        .skip(1) // Skip header row (#ip,ioc)
        .filter(|line| !line.is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 2 {
                Some((parts[0].to_string(), parts[1].to_string()))
            } else {
                None
            }
        })
        .collect()
}
```

### Domain + URL + IP Parsing

```rust
#[derive(Debug, Deserialize)]
struct FullIndicator {
    #[serde(rename = "#domain")]
    domain: String,
    ioc: String,
    url_path: String,
    ip: String,
}

fn parse_full_feed(csv_content: &str) -> Result<Vec<FullIndicator>, csv::Error> {
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(csv_content.as_bytes());

    reader.deserialize::<FullIndicator>()
        .collect()
}
```

## Error Responses

### Rate Limit Exceeded (429)

```http
HTTP/1.1 429 Too Many Requests
Retry-After: 3600
Content-Type: text/plain

rate limit exceeded
```

**Response body**: Plain text error message

### Not Found (404)

```http
HTTP/1.1 404 Not Found
Content-Type: text/html

404: Not Found
```

**Cause**: Invalid feed filename or path

### Service Unavailable (503)

```http
HTTP/1.1 503 Service Unavailable
Retry-After: 120
```

**Cause**: GitHub CDN temporary outage

## Conditional Request Responses

### 304 Not Modified

```http
HTTP/1.1 304 Not Modified
Cache-Control: max-age=300
Last-Modified: Sat, 15 Feb 2026 14:30:00 GMT
ETag: "a1b2c3d4e5f6..."
```

**Body**: Empty (no content)

**Meaning**: Feed has not been updated since last check.

## File Size Ranges

| Feed Type | Typical Size | Row Count (30-day) |
|-----------|-------------|-------------------|
| IP feeds | 15-50 KB | 500-1500 rows |
| Domain feeds | 20-80 KB | 300-1000 rows |
| Domain+URL | 30-100 KB | 300-800 rows |
| Domain+URL+IP | 50-150 KB | 300-700 rows |
| Full historical | 100-500 KB | 2000-10000 rows |

## Edge Cases and Validation

### Empty Feeds

Unlikely but possible:

```csv
#ip,ioc
```

**Handling**: Check if CSV has >1 row before processing.

### Malformed Rows

Missing columns or extra commas:

```csv
#ip,ioc
1.2.3.4
5.6.7.8,Possible Cobaltstrike C2 IP,extra_field
```

**Recommendation**:
- Skip rows with incorrect column count
- Log warnings for malformed data
- Implement defensive parsing

### Special Characters in Domains

Domains may contain unusual but valid characters:

```csv
2458ccd60cc54149bb05537717d831f0--8000.ap-shanghai2.cloudstudio.club,Possible Cobalt Strike C2 Domain
```

**Handling**: Don't overly restrict domain validation.

### URL Paths with Commas

If a URL path contains a comma, it could break CSV parsing. Not observed in current feeds, but implement defensive parsing.

## Summary

- **Format**: Plain-text CSV with `#`-prefixed header
- **Encoding**: UTF-8
- **Columns**: 2-4 depending on feed type
- **Size**: 15KB-500KB per feed
- **Parsing**: Standard CSV libraries work (handle `#` prefix in header)
- **Error handling**: Standard HTTP status codes (429, 404, 503)
- **Caching**: Use Last-Modified and ETag headers for efficiency
