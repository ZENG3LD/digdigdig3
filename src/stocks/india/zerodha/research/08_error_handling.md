# Zerodha Kite Connect - Error Handling and Exceptions

## Error Response Structure

All API errors follow a consistent JSON structure:

```json
{
  "status": "error",
  "message": "Human-readable error message",
  "error_type": "ExceptionType",
  "data": null
}
```

**Fields**:

| Field | Type | Description |
|-------|------|-------------|
| status | string | Always "error" for errors |
| message | string | Human-readable error description |
| error_type | string | Exception type identifier |
| data | null/object | Additional error data (usually null) |

---

## Exception Types

Kite Connect defines **9 specific exception categories** for structured error handling:

### 1. TokenException

**HTTP Status**: 403 Forbidden

**Cause**: Session expiry or invalidation requiring re-authentication

**Common Scenarios**:
- access_token expired (daily at 6 AM IST)
- Manual logout
- Token invalidated
- Invalid access_token

**Example Response**:
```json
{
  "status": "error",
  "message": "Invalid session credentials",
  "error_type": "TokenException",
  "data": null
}
```

**Resolution**:
1. Redirect user to login flow
2. Obtain new request_token
3. Exchange for new access_token
4. Resume API operations

**Handling**:
```python
try:
    response = kite.get_orders()
except TokenException:
    # Redirect to login
    login_url = kite.login_url()
    print(f"Session expired. Please login: {login_url}")
    # Wait for request_token from callback
    # Generate new access_token
```

---

### 2. UserException

**HTTP Status**: 400-499 (varies)

**Cause**: Account-related issues

**Common Scenarios**:
- Inactive trading account
- Account suspended
- Unauthorized access
- Permissions issues

**Example Response**:
```json
{
  "status": "error",
  "message": "Account is not active for trading",
  "error_type": "UserException",
  "data": null
}
```

**Resolution**:
- Check account status on Zerodha platform
- Contact Zerodha support if needed
- Verify account permissions

---

### 3. OrderException

**HTTP Status**: 400-499 (varies)

**Cause**: Order placement or retrieval failures

**Common Scenarios**:
- Invalid order parameters
- Order already executed/cancelled
- Market closed
- Circuit limits hit
- Invalid instrument

**Example Responses**:

```json
{
  "status": "error",
  "message": "Order cannot be modified as it is already cancelled",
  "error_type": "OrderException",
  "data": null
}
```

```json
{
  "status": "error",
  "message": "Market is closed",
  "error_type": "OrderException",
  "data": null
}
```

**Resolution**:
- Validate order parameters before submission
- Check market hours
- Verify instrument is tradable
- Handle order state transitions correctly

---

### 4. InputException

**HTTP Status**: 400 Bad Request

**Cause**: Missing or invalid parameters

**Common Scenarios**:
- Missing required parameters
- Invalid parameter values
- Incorrect parameter types
- Invalid format

**Example Responses**:

```json
{
  "status": "error",
  "message": "Missing parameter: quantity",
  "error_type": "InputException",
  "data": null
}
```

```json
{
  "status": "error",
  "message": "Invalid value for parameter: order_type",
  "error_type": "InputException",
  "data": null
}
```

**Resolution**:
- Validate all parameters client-side
- Check parameter types and formats
- Refer to API documentation for required fields

**Validation Example**:
```python
def validate_order_params(order):
    required = ["exchange", "tradingsymbol", "transaction_type",
                "order_type", "quantity", "product"]

    for param in required:
        if param not in order:
            raise ValueError(f"Missing required parameter: {param}")

    if order["transaction_type"] not in ["BUY", "SELL"]:
        raise ValueError("transaction_type must be BUY or SELL")

    if order["order_type"] == "LIMIT" and "price" not in order:
        raise ValueError("LIMIT orders require price parameter")

    # ... more validations
```

---

### 5. MarginException

**HTTP Status**: 400 Bad Request

**Cause**: Insufficient funds for order

**Common Scenarios**:
- Not enough cash/margin
- Margin blocked in other positions
- Leveraged positions exceeding limits

**Example Response**:
```json
{
  "status": "error",
  "message": "Insufficient funds. Available: 10000.00, Required: 15000.00",
  "error_type": "MarginException",
  "data": null
}
```

**Resolution**:
1. Check available margins: `GET /user/margins`
2. Use margin calculator: `POST /margins/orders`
3. Add funds or close positions
4. Reduce order quantity

**Margin Check Example**:
```python
def check_margin_before_order(kite, order):
    # Calculate required margin
    margin_response = kite.order_margins([order])
    required_margin = margin_response[0]["total"]

    # Get available margin
    margins = kite.margins()
    available = margins["equity"]["net"]

    if required_margin > available:
        raise MarginException(
            f"Insufficient funds. Available: {available}, Required: {required_margin}"
        )

    return required_margin
```

---

### 6. HoldingException

**HTTP Status**: 400 Bad Request

**Cause**: Insufficient holdings for sell order

**Common Scenarios**:
- Trying to sell more than owned
- Holdings not yet settled (T+2)
- Holdings in auction
- CDSL authorization pending

**Example Response**:
```json
{
  "status": "error",
  "message": "Insufficient holdings. Available: 10, Requested: 20",
  "error_type": "HoldingException",
  "data": null
}
```

**Resolution**:
1. Check holdings: `GET /portfolio/holdings`
2. Verify realised_quantity (settled holdings)
3. Check for auction holdings
4. Authorize holdings if needed (CDSL e-DIS)

**Holdings Check Example**:
```python
def check_holdings_before_sell(kite, tradingsymbol, quantity):
    holdings = kite.holdings()

    for holding in holdings:
        if holding["tradingsymbol"] == tradingsymbol:
            available = holding["realised_quantity"] - holding["used_quantity"]

            if quantity > available:
                raise HoldingException(
                    f"Insufficient holdings. Available: {available}, Requested: {quantity}"
                )

            return True

    raise HoldingException(f"No holdings found for {tradingsymbol}")
```

---

### 7. NetworkException

**HTTP Status**: 429, 500-599

**Cause**: Communication failures with Order Management System or rate limiting

**Common Scenarios**:
- Rate limit exceeded (429)
- Server error (500)
- Bad gateway (502)
- Service unavailable (503)
- Gateway timeout (504)
- Network connectivity issues

**Example Responses**:

```json
{
  "status": "error",
  "message": "Too many requests",
  "error_type": "NetworkException",
  "data": null
}
```

```json
{
  "status": "error",
  "message": "Gateway timeout",
  "error_type": "NetworkException",
  "data": null
}
```

**Resolution**:
- Implement exponential backoff
- Respect rate limits (10 req/sec)
- Retry with delay
- Check network connectivity
- Use circuit breaker pattern

**Retry Logic Example**:
```python
import time

def api_call_with_retry(func, max_retries=5):
    delay = 1

    for attempt in range(max_retries):
        try:
            return func()
        except NetworkException as e:
            if "Too many requests" in str(e):
                # Rate limit - longer backoff
                delay = min(delay * 2, 60)
            elif attempt == max_retries - 1:
                raise

            print(f"NetworkException: {e}. Retrying in {delay}s... (attempt {attempt + 1}/{max_retries})")
            time.sleep(delay)
            delay *= 2  # Exponential backoff

    raise Exception("Max retries exceeded")
```

---

### 8. DataException

**HTTP Status**: 500 Internal Server Error

**Cause**: Internal system errors processing OMS responses

**Common Scenarios**:
- Backend data inconsistency
- OMS system error
- Data processing failure
- Unexpected response format

**Example Response**:
```json
{
  "status": "error",
  "message": "Error processing order response",
  "error_type": "DataException",
  "data": null
}
```

**Resolution**:
- Retry the request
- Check if operation succeeded despite error
- Contact support if persistent
- Log error for investigation

---

### 9. GeneralException

**HTTP Status**: Varies (400-599)

**Cause**: Unclassified errors occurring rarely

**Common Scenarios**:
- Unexpected errors
- System maintenance
- Edge cases not covered by specific exceptions

**Example Response**:
```json
{
  "status": "error",
  "message": "An unexpected error occurred",
  "error_type": "GeneralException",
  "data": null
}
```

**Resolution**:
- Log error details
- Retry if appropriate
- Check system status
- Contact support for persistent issues

---

## HTTP Status Codes

| Code | Meaning | Description |
|------|---------|-------------|
| **200** | OK | Success |
| **400** | Bad Request | Missing or bad request parameters or values |
| **403** | Forbidden | Session expired; re-login required (TokenException) |
| **404** | Not Found | Resource not found |
| **405** | Method Not Allowed | Invalid HTTP method for endpoint |
| **410** | Gone | Resource permanently gone |
| **429** | Too Many Requests | Rate limit exceeded |
| **500** | Internal Server Error | Unexpected server error |
| **502** | Bad Gateway | Backend system unreachable |
| **503** | Service Unavailable | Service temporarily unavailable |
| **504** | Gateway Timeout | Backend system timeout |

---

## Rate Limit Errors

### HTTP 429 Response

```json
{
  "status": "error",
  "message": "Too many requests",
  "error_type": "NetworkException",
  "data": null
}
```

**Response Headers** (when available):
```
X-RateLimit-Limit: 10
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1706254380
Retry-After: 1
```

### Rate Limit by Endpoint

| Endpoint Category | Limit | Error Handling |
|------------------|-------|----------------|
| Quote endpoints | 1 req/sec | Wait 1 second before retry |
| Historical data | 3 req/sec | Wait 0.33 seconds before retry |
| General API | 10 req/sec | Wait 0.1 seconds before retry |

### Handling Strategy

**1. Client-side rate limiting** (prevent 429):
```python
import time
from collections import deque

class RateLimiter:
    def __init__(self, requests_per_second):
        self.rate = requests_per_second
        self.timestamps = deque()

    def acquire(self):
        now = time.time()

        # Remove old timestamps
        while self.timestamps and self.timestamps[0] < now - 1:
            self.timestamps.popleft()

        if len(self.timestamps) >= self.rate:
            sleep_time = 1 - (now - self.timestamps[0])
            if sleep_time > 0:
                time.sleep(sleep_time)
            self.timestamps.popleft()

        self.timestamps.append(time.time())

# Usage
limiter = RateLimiter(10)  # 10 req/sec

def make_api_call():
    limiter.acquire()
    # Make API call
```

**2. Exponential backoff on 429**:
```python
def exponential_backoff_on_429(func, max_retries=5):
    delay = 1

    for attempt in range(max_retries):
        try:
            return func()
        except Exception as e:
            if "429" in str(e) or "Too many requests" in str(e):
                if attempt == max_retries - 1:
                    raise

                print(f"Rate limited. Waiting {delay}s...")
                time.sleep(delay)
                delay = min(delay * 2, 60)  # Cap at 60 seconds
            else:
                raise

    raise Exception("Max retries exceeded")
```

---

## Order-Specific Errors

### Order State Errors

```json
{
  "status": "error",
  "message": "Order cannot be modified as it is already cancelled",
  "error_type": "OrderException",
  "data": null
}
```

**Valid Order State Transitions**:
- OPEN → COMPLETE
- OPEN → CANCELLED
- OPEN → REJECTED
- OPEN → (modification) → OPEN
- TRIGGER PENDING → OPEN

**Invalid Transitions** (will cause errors):
- COMPLETE → CANCELLED (cannot cancel executed order)
- CANCELLED → MODIFIED (cannot modify cancelled order)
- REJECTED → MODIFIED (cannot modify rejected order)

**Handling**:
```python
def safe_cancel_order(kite, order_id, variety="regular"):
    try:
        # Get order status first
        orders = kite.orders()
        order = next((o for o in orders if o["order_id"] == order_id), None)

        if not order:
            print("Order not found")
            return False

        if order["status"] in ["COMPLETE", "CANCELLED", "REJECTED"]:
            print(f"Order already in terminal state: {order['status']}")
            return False

        # Safe to cancel
        kite.cancel_order(variety, order_id)
        return True

    except OrderException as e:
        print(f"Cannot cancel order: {e}")
        return False
```

---

## Market Hours Errors

```json
{
  "status": "error",
  "message": "Market is closed",
  "error_type": "OrderException",
  "data": null
}
```

**Indian Market Hours**:
- **Pre-market**: 09:00 - 09:15 IST
- **Regular session**: 09:15 - 15:30 IST
- **Post-market**: 15:40 - 16:00 IST (orders only, no execution)

**After Market Orders (AMO)**:
- Can be placed outside market hours
- Use variety="amo"
- Executed at market open

**Handling**:
```python
from datetime import datetime, time
import pytz

def is_market_open():
    ist = pytz.timezone('Asia/Kolkata')
    now = datetime.now(ist).time()

    market_open = time(9, 15)
    market_close = time(15, 30)

    return market_open <= now <= market_close

def place_order_smart(kite, order_params):
    if is_market_open():
        # Regular order
        return kite.place_order(variety="regular", **order_params)
    else:
        # After Market Order
        return kite.place_order(variety="amo", **order_params)
```

---

## WebSocket Errors

### Connection Errors

**Error Messages** (text JSON):
```json
{
  "type": "error",
  "data": "Invalid subscription"
}
```

**Common WebSocket Errors**:
- Invalid authentication (connection rejected)
- Invalid instrument token
- Subscription limit exceeded (>3,000 instruments)
- Connection limit exceeded (>3 connections per API key)
- Token expired

**Handling**:
```javascript
ws.onmessage = (event) => {
    if (typeof event.data === 'string') {
        const message = JSON.parse(event.data);

        if (message.type === "error") {
            console.error("WebSocket error:", message.data);

            if (message.data.includes("Invalid subscription")) {
                // Handle invalid instrument token
                console.log("Removing invalid instruments...");
            } else if (message.data.includes("limit")) {
                // Handle subscription limit
                console.log("Too many subscriptions");
            }
        }
    }
};

ws.onerror = (error) => {
    console.error("WebSocket connection error:", error);
};

ws.onclose = (event) => {
    console.log("WebSocket closed:", event.code, event.reason);

    if (event.code === 1006) {
        // Abnormal closure - likely token expiry
        console.log("Token may have expired. Re-authenticating...");
        reAuthenticate();
    } else {
        // Implement reconnection logic
        setTimeout(reconnect, 5000);
    }
};
```

---

## Error Handling Best Practices

### 1. Structured Exception Handling

```python
from kiteconnect import KiteConnect
from kiteconnect.exceptions import (
    TokenException, UserException, OrderException,
    InputException, MarginException, HoldingException,
    NetworkException, DataException, GeneralException
)

def handle_api_call(func):
    try:
        return func()

    except TokenException as e:
        print("Session expired. Re-authenticating...")
        # Trigger re-authentication
        reAuthenticate()

    except MarginException as e:
        print(f"Insufficient margin: {e}")
        # Check margins and notify user

    except HoldingException as e:
        print(f"Insufficient holdings: {e}")
        # Check holdings and notify user

    except OrderException as e:
        print(f"Order error: {e}")
        # Handle order-specific errors

    except InputException as e:
        print(f"Invalid input: {e}")
        # Validate and fix parameters

    except NetworkException as e:
        print(f"Network error: {e}")
        # Retry with backoff

    except (DataException, GeneralException) as e:
        print(f"System error: {e}")
        # Log and notify support

    except Exception as e:
        print(f"Unexpected error: {e}")
        # Log and handle gracefully
```

### 2. Validation Before API Calls

```python
class OrderValidator:
    VALID_EXCHANGES = ["NSE", "BSE", "NFO", "BFO", "MCX", "CDS", "BCD"]
    VALID_TRANSACTION_TYPES = ["BUY", "SELL"]
    VALID_ORDER_TYPES = ["MARKET", "LIMIT", "SL", "SL-M"]
    VALID_PRODUCTS = ["CNC", "NRML", "MIS", "MTF"]
    VALID_VALIDITIES = ["DAY", "IOC", "TTL"]

    @staticmethod
    def validate(order):
        errors = []

        # Required fields
        required = ["exchange", "tradingsymbol", "transaction_type",
                    "order_type", "quantity", "product"]
        for field in required:
            if field not in order or not order[field]:
                errors.append(f"Missing required field: {field}")

        # Enum validations
        if order.get("exchange") not in OrderValidator.VALID_EXCHANGES:
            errors.append(f"Invalid exchange: {order.get('exchange')}")

        if order.get("transaction_type") not in OrderValidator.VALID_TRANSACTION_TYPES:
            errors.append(f"Invalid transaction_type: {order.get('transaction_type')}")

        if order.get("order_type") not in OrderValidator.VALID_ORDER_TYPES:
            errors.append(f"Invalid order_type: {order.get('order_type')}")

        if order.get("product") not in OrderValidator.VALID_PRODUCTS:
            errors.append(f"Invalid product: {order.get('product')}")

        # Conditional validations
        if order.get("order_type") == "LIMIT" and "price" not in order:
            errors.append("LIMIT orders require price parameter")

        if order.get("order_type") in ["SL", "SL-M"] and "trigger_price" not in order:
            errors.append("Stop-loss orders require trigger_price parameter")

        # Numeric validations
        if "quantity" in order and order["quantity"] <= 0:
            errors.append("Quantity must be positive")

        if "price" in order and order["price"] <= 0:
            errors.append("Price must be positive")

        if errors:
            raise InputException("; ".join(errors))

        return True

# Usage
try:
    OrderValidator.validate(order_params)
    kite.place_order(**order_params)
except InputException as e:
    print(f"Validation failed: {e}")
```

### 3. Comprehensive Logging

```python
import logging

logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    handlers=[
        logging.FileHandler('kite_api.log'),
        logging.StreamHandler()
    ]
)

logger = logging.getLogger('kite_api')

def logged_api_call(func, *args, **kwargs):
    try:
        logger.info(f"API call: {func.__name__} with args={args}, kwargs={kwargs}")
        result = func(*args, **kwargs)
        logger.info(f"API call successful: {func.__name__}")
        return result

    except Exception as e:
        logger.error(f"API call failed: {func.__name__} - {type(e).__name__}: {e}")
        raise

# Usage
logged_api_call(kite.place_order, variety="regular", **order_params)
```

### 4. Circuit Breaker Pattern

```python
from datetime import datetime, timedelta

class CircuitBreaker:
    def __init__(self, failure_threshold=5, timeout=60):
        self.failure_threshold = failure_threshold
        self.timeout = timeout
        self.failures = 0
        self.last_failure_time = None
        self.state = "CLOSED"  # CLOSED, OPEN, HALF_OPEN

    def call(self, func):
        if self.state == "OPEN":
            if datetime.now() - self.last_failure_time > timedelta(seconds=self.timeout):
                self.state = "HALF_OPEN"
            else:
                raise Exception("Circuit breaker is OPEN")

        try:
            result = func()
            if self.state == "HALF_OPEN":
                self.state = "CLOSED"
                self.failures = 0
            return result

        except Exception as e:
            self.failures += 1
            self.last_failure_time = datetime.now()

            if self.failures >= self.failure_threshold:
                self.state = "OPEN"

            raise

# Usage
breaker = CircuitBreaker()

def make_api_call():
    return breaker.call(lambda: kite.get_quote("NSE:INFY"))
```

### 5. Graceful Degradation

```python
class KiteAPIWrapper:
    def __init__(self, kite):
        self.kite = kite
        self.cache = {}

    def get_quote_with_fallback(self, instruments):
        try:
            # Try live quote
            return self.kite.quote(instruments)

        except NetworkException:
            # Fallback to cached data
            if instruments in self.cache:
                logger.warning(f"Using cached quote for {instruments}")
                return self.cache[instruments]
            raise

        except Exception as e:
            logger.error(f"Error fetching quote: {e}")
            raise

    def place_order_with_retry(self, order_params, max_retries=3):
        for attempt in range(max_retries):
            try:
                return self.kite.place_order(**order_params)

            except NetworkException as e:
                if attempt == max_retries - 1:
                    raise
                time.sleep(2 ** attempt)  # Exponential backoff

            except (MarginException, HoldingException, OrderException) as e:
                # Don't retry these - they won't succeed
                raise
```

---

## Error Recovery Strategies

| Error Type | Recovery Strategy |
|------------|------------------|
| **TokenException** | Re-authenticate, obtain new access_token |
| **NetworkException (429)** | Exponential backoff, respect rate limits |
| **NetworkException (5xx)** | Retry with backoff, check system status |
| **MarginException** | Check margins, reduce quantity, or add funds |
| **HoldingException** | Check holdings, reduce quantity |
| **OrderException** | Validate order state, check market hours |
| **InputException** | Validate parameters, fix and retry |
| **DataException** | Retry, check operation status, log for investigation |
| **GeneralException** | Log error, retry if appropriate, contact support |

---

## Monitoring and Alerting

```python
class APIMonitor:
    def __init__(self):
        self.error_counts = {}
        self.total_requests = 0
        self.failed_requests = 0

    def record_request(self, success, error_type=None):
        self.total_requests += 1

        if not success:
            self.failed_requests += 1
            if error_type:
                self.error_counts[error_type] = self.error_counts.get(error_type, 0) + 1

    def get_stats(self):
        error_rate = (self.failed_requests / self.total_requests * 100) if self.total_requests > 0 else 0

        return {
            "total_requests": self.total_requests,
            "failed_requests": self.failed_requests,
            "error_rate": f"{error_rate:.2f}%",
            "error_breakdown": self.error_counts
        }

    def should_alert(self):
        # Alert if error rate > 10%
        error_rate = (self.failed_requests / self.total_requests) if self.total_requests > 0 else 0
        return error_rate > 0.1

# Usage
monitor = APIMonitor()

def monitored_api_call(func):
    try:
        result = func()
        monitor.record_request(success=True)
        return result
    except Exception as e:
        error_type = type(e).__name__
        monitor.record_request(success=False, error_type=error_type)

        if monitor.should_alert():
            send_alert(f"High error rate detected: {monitor.get_stats()}")

        raise
```

---

## Summary

- **9 exception types** for structured error handling
- **HTTP status codes** indicate error categories
- **Rate limiting** requires client-side management
- **Validation** before API calls reduces errors
- **Retry logic** with exponential backoff for transient errors
- **Circuit breaker** prevents cascading failures
- **Logging and monitoring** essential for production systems
- **WebSocket errors** handled differently from REST errors
- **Token expiry** (6 AM IST) requires daily re-authentication
