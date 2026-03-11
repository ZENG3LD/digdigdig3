# Phemex API Authentication & Rate Limits

Source: https://phemex-docs.github.io/

---

## Authentication Method

Phemex uses **HMAC-SHA256** for all private endpoint request signatures.

### Signature String Construction

```
HMAC-SHA256(key=apiSecret, message=URLPath + QueryString + Expiry + Body)
```

Components concatenated in this exact order (no separator):

| Component     | Description                                              | Example                        |
|---------------|----------------------------------------------------------|--------------------------------|
| `URLPath`     | Request path without domain                              | `/accounts/accountPositions`   |
| `QueryString` | URL query parameters (without `?`)                       | `currency=BTC`                 |
| `Expiry`      | Unix epoch seconds (now + 60 seconds)                    | `1575735514`                   |
| `Body`        | JSON request body (empty string for GET)                 | `{"symbol":"BTCUSD",...}`      |

### GET Request Example

```
Path:       /accounts/accountPositions
Query:      currency=BTC
Expiry:     1575735514
Message:    /accounts/accountPositionscurrency=BTC1575735514
```

### POST Request Example

```
Path:       /orders
Query:      (empty)
Expiry:     1575735514
Body:       {"symbol":"BTCUSD","side":"Sell","orderQty":1,"ordType":"Market"}
Message:    /orders1575735514{"symbol":"BTCUSD","side":"Sell","orderQty":1,"ordType":"Market"}
```

---

## Required HTTP Headers

Every authenticated request must include these three headers:

| Header Name                    | Value                                         | Required |
|-------------------------------|-----------------------------------------------|----------|
| `x-phemex-access-token`       | API Key (the `id` field from key creation)    | Yes      |
| `x-phemex-request-expiry`     | Unix epoch seconds (current time + 60 sec)    | Yes      |
| `x-phemex-request-signature`  | Computed HMAC-SHA256 signature                | Yes      |
| `x-phemex-request-tracing`    | Custom trace string (<40 bytes) for debugging | No       |

---

## Permission Levels / Scopes

The official Phemex API documentation does not define explicit granular permission scopes (e.g., read-only, trade, withdraw). Authentication is binary:

- **Public endpoints**: No authentication required. Accessible without any headers.
- **Private endpoints**: Require all three auth headers.

API keys are managed through the Phemex web interface. Specific scope configuration per key (read/trade/withdraw) is not documented in the public REST API reference.

VIP access (`https://vapi.phemex.com`) requires IP whitelist registration with Phemex support — this is not self-service.

---

## Base URLs

| Environment | URL                          | Notes                              |
|-------------|------------------------------|------------------------------------|
| Production  | `https://api.phemex.com`     | Standard rate limits apply         |
| VIP         | `https://vapi.phemex.com`    | Higher limits; requires IP whitelist approval |
| Testnet     | `https://testnet-api.phemex.com` | Testing only; shared rate limits |

---

## Rate Limits

### IP-Level Rate Limits

| Scope       | Limit                       |
|-------------|-----------------------------|
| REST API    | 5,000 requests / 5 minutes per IP |
| Testnet     | 500 requests / 5 minutes shared (all users) |
| WebSocket   | 200 requests / 5 minutes per IP |

### User-Level REST Rate Limits (per API key, 60-second window)

Endpoints are grouped into rate limit buckets:

| Group          | Standard Limit | VIP Limit   | Applies To                              |
|----------------|----------------|-------------|------------------------------------------|
| `Contract`     | 500 / min      | 5,000 / min | Futures order placement, management      |
| `Contract` per symbol | —       | 500 / min   | Per-symbol futures operations (VIP only) |
| `CONTACT_ALL_SYM` | —          | 500 / min   | Cancel-all-symbols operations (VIP)      |
| `SpotOrder`    | 500 / min      | —           | Spot order placement and management      |
| `Others`       | 100 / min      | —           | Account queries, market data, etc.       |

### WebSocket Limits

| Limit                    | Value                     |
|--------------------------|---------------------------|
| Concurrent connections   | 5 per client              |
| Subscriptions per conn   | 20 maximum                |
| Message rate             | 20 requests/sec per conn  |

### Rate Limit Response Headers

Phemex returns these headers on every private response:

| Header                                  | Description                                     |
|-----------------------------------------|-------------------------------------------------|
| `x-ratelimit-remaining-<groupName>`     | Remaining capacity for the request's group      |
| `x-ratelimit-capacity-<groupName>`      | Total capacity of the group                     |
| `x-ratelimit-retry-after-<groupName>`   | Seconds until reset (only present on 429 response) |

HTTP `429` is returned when rate limit is exceeded.

---

## IP Restrictions

- **Standard API** (`api.phemex.com`): No IP restriction. Any IP can authenticate.
- **VIP API** (`vapi.phemex.com`): Requires IP whitelist. Must contact Phemex support to register IPs for elevated limits.
- **Testnet**: No IP restrictions. Shared rate limit pool across all testnet users.

---

## Expiry Window

The `x-phemex-request-expiry` timestamp must be within a valid window of server time. Recommended value is `current_unix_time + 60`. If the request arrives after the expiry timestamp, it will be rejected with an authentication error.

---

## Error Codes Related to Auth

| BizError Code | Message              | Description                              |
|---------------|----------------------|------------------------------------------|
| 11023         | TE_REJECT_DUE_TO_BANNED | Account is banned / API key invalid   |
| 10001         | OM_DUPLICATE_ORDERID | Order ID collision (not auth, but relevant) |

---

## Signature Algorithm — Pseudocode

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;

fn sign_request(
    api_secret: &str,
    path: &str,
    query: &str,      // empty string if no query params
    expiry: u64,      // unix seconds
    body: &str,       // empty string for GET
) -> String {
    let message = format!("{}{}{}{}", path, query, expiry, body);
    let mut mac = Hmac::<Sha256>::new_from_slice(api_secret.as_bytes())
        .expect("HMAC accepts any key size");
    mac.update(message.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}
```

Headers to set:

```rust
headers.insert("x-phemex-access-token",      api_key);
headers.insert("x-phemex-request-expiry",    expiry.to_string());
headers.insert("x-phemex-request-signature", signature);
```

---

Sources:
- https://phemex-docs.github.io/
