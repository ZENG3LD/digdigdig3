# Interactive Brokers Client Portal Web API - Error Handling

## HTTP Status Codes

### Success Codes (2xx)

#### 200 OK

**Description:** Request successful

**Example Response:**
```json
{
  "data": { ... }
}
```

**Use Cases:**
- Successful GET requests
- Successful POST requests
- Data retrieval
- Order placement accepted

#### 201 Created

**Description:** Resource created successfully

**Rare in IBKR API** - typically returns 200 for creation

### Client Error Codes (4xx)

#### 400 Bad Request

**Description:** Invalid request parameters or malformed request

**Common Causes:**
- Invalid JSON format
- Missing required parameters
- Invalid parameter values
- Parameter type mismatch
- Validation failures

**Example Response:**
```json
{
  "error": "Invalid conid parameter",
  "statusCode": 400
}
```

**Specific Scenarios:**

**Invalid Contract ID:**
```json
{
  "error": "Invalid conid: 999999999",
  "statusCode": 400
}
```

**Missing Required Field:**
```json
{
  "error": "Missing required parameter: quantity",
  "statusCode": 400
}
```

**Invalid Order Type:**
```json
{
  "error": "Invalid orderType: INVALID_TYPE",
  "statusCode": 400
}
```

**Order Validation Failure:**
```json
{
  "error": "Order rejected: Insufficient funds",
  "statusCode": 400,
  "orderId": 0,
  "details": {
    "reason": "INSUFFICIENT_FUNDS",
    "requiredMargin": 18551.00,
    "availableMargin": 10000.00
  }
}
```

**Handling:**
```python
def handle_400(response):
    error = response.json().get('error', 'Unknown error')

    if 'invalid conid' in error.lower():
        # Handle invalid contract
        logger.error(f"Invalid contract ID: {error}")
        return ErrorCode.INVALID_CONTRACT

    elif 'insufficient funds' in error.lower():
        # Handle insufficient funds
        logger.error(f"Insufficient funds: {error}")
        return ErrorCode.INSUFFICIENT_FUNDS

    elif 'missing required parameter' in error.lower():
        # Handle missing parameter
        logger.error(f"Missing parameter: {error}")
        return ErrorCode.MISSING_PARAMETER

    else:
        # Generic bad request
        logger.error(f"Bad request: {error}")
        return ErrorCode.BAD_REQUEST
```

#### 401 Unauthorized

**Description:** Authentication required or authentication failed

**Common Causes:**
- No active session
- Session expired
- Brokerage session not authenticated
- Invalid OAuth token
- OAuth token expired

**Example Response:**
```json
{
  "error": "Not authenticated",
  "statusCode": 401
}
```

**Handling:**
```python
def handle_401(response):
    # Check auth status
    auth_status = check_auth_status()

    if not auth_status['authenticated']:
        # Re-authenticate
        logger.warning("Session expired, re-authenticating...")

        # Initialize brokerage session
        init_response = initialize_session()

        if init_response['connected']:
            # Retry original request
            return retry_request()
        else:
            # Authentication failed
            logger.error("Re-authentication failed")
            return ErrorCode.AUTH_FAILED

    return ErrorCode.UNAUTHORIZED
```

**Re-Authentication Flow:**
```python
def ensure_authenticated():
    """Ensure session is authenticated, re-auth if needed"""

    status = get_auth_status()

    if not status['authenticated']:
        # Initialize session
        init_result = initialize_session()

        if not init_result['connected']:
            raise AuthenticationError("Failed to authenticate")

        # Wait for session to be ready
        time.sleep(1)

        # Verify authentication
        status = get_auth_status()

        if not status['authenticated']:
            raise AuthenticationError("Session not authenticated after init")

    return True
```

#### 403 Forbidden

**Description:** Access denied, insufficient permissions

**Common Causes:**
- Account does not have permission for operation
- Market data subscription required
- Trading permissions not enabled
- Geographic restrictions
- Account type restrictions

**Example Response:**
```json
{
  "error": "Access denied: Market data subscription required",
  "statusCode": 403
}
```

**Scenarios:**

**Market Data Not Subscribed:**
```json
{
  "error": "Real-time market data not available for this exchange",
  "statusCode": 403
}
```

**Trading Not Permitted:**
```json
{
  "error": "Trading not permitted for this account type",
  "statusCode": 403
}
```

**Geographic Restriction:**
```json
{
  "error": "Canadian residents cannot trade Canadian exchanges programmatically",
  "statusCode": 403
}
```

**Handling:**
```python
def handle_403(response):
    error = response.json().get('error', 'Access denied')

    if 'market data' in error.lower():
        logger.error("Market data subscription required")
        # Notify user to subscribe to market data
        return ErrorCode.MARKET_DATA_REQUIRED

    elif 'trading not permitted' in error.lower():
        logger.error("Trading not permitted for account")
        return ErrorCode.TRADING_NOT_PERMITTED

    else:
        logger.error(f"Access forbidden: {error}")
        return ErrorCode.FORBIDDEN
```

#### 404 Not Found

**Description:** Resource not found

**Common Causes:**
- Invalid endpoint URL
- Order ID not found
- Account ID not found
- Watchlist not found
- Alert not found

**Example Response:**
```json
{
  "error": "Order not found",
  "statusCode": 404
}
```

**Handling:**
```python
def handle_404(response, resource_type):
    logger.warning(f"{resource_type} not found")

    if resource_type == 'order':
        # Order may have already been filled/cancelled
        # Check order history
        return ErrorCode.ORDER_NOT_FOUND

    elif resource_type == 'endpoint':
        # Invalid API endpoint
        logger.error("Invalid API endpoint")
        return ErrorCode.INVALID_ENDPOINT

    else:
        return ErrorCode.NOT_FOUND
```

#### 429 Too Many Requests

**Description:** Rate limit exceeded

**Common Causes:**
- Too many requests per second (global limit: 10 req/s)
- Endpoint-specific limit exceeded
- Burst limit exceeded

**Example Response:**
```json
{
  "error": "Request was throttled. Expected available in 15 seconds.",
  "statusCode": 429
}
```

**Rate Limit Details:**
- **Global:** 10 requests per second (Gateway), 50 req/s (OAuth)
- **Penalty:** 15-minute IP block
- **Repeat Offenders:** Permanent IP block possible

**Handling with Exponential Backoff:**
```python
import time
import random

def handle_429(response, attempt=1, max_attempts=5):
    """Handle rate limit with exponential backoff"""

    if attempt > max_attempts:
        logger.error("Max retry attempts exceeded")
        raise RateLimitError("Rate limit exceeded, max retries reached")

    # Extract wait time from error message if available
    error_msg = response.json().get('error', '')
    wait_time = parse_wait_time(error_msg)  # Parse "15 seconds" from message

    if wait_time is None:
        # Exponential backoff: 2^attempt + random jitter
        wait_time = (2 ** attempt) + random.uniform(0, 1)

    logger.warning(f"Rate limit exceeded, waiting {wait_time:.2f} seconds (attempt {attempt}/{max_attempts})")
    time.sleep(wait_time)

    # Retry request
    return retry_request(attempt + 1)

def parse_wait_time(error_msg):
    """Parse wait time from error message"""
    import re

    match = re.search(r'(\d+)\s*seconds?', error_msg)
    if match:
        return int(match.group(1))
    return None
```

**Rate Limiter Implementation:**
```python
from collections import deque
from threading import Lock
import time

class RateLimiter:
    def __init__(self, max_requests=10, time_window=1.0):
        """
        max_requests: Maximum requests per time window
        time_window: Time window in seconds
        """
        self.max_requests = max_requests
        self.time_window = time_window
        self.requests = deque()
        self.lock = Lock()

    def acquire(self):
        """Wait if necessary to comply with rate limit"""
        with self.lock:
            now = time.time()

            # Remove requests outside time window
            while self.requests and self.requests[0] < now - self.time_window:
                self.requests.popleft()

            # Check if rate limit reached
            if len(self.requests) >= self.max_requests:
                # Calculate wait time
                oldest_request = self.requests[0]
                wait_time = self.time_window - (now - oldest_request)

                if wait_time > 0:
                    time.sleep(wait_time)
                    now = time.time()

                    # Clean up old requests after wait
                    while self.requests and self.requests[0] < now - self.time_window:
                        self.requests.popleft()

            # Record this request
            self.requests.append(now)

# Usage
rate_limiter = RateLimiter(max_requests=10, time_window=1.0)

def make_request(url):
    rate_limiter.acquire()  # Wait if necessary
    return requests.get(url)
```

### Server Error Codes (5xx)

#### 500 Internal Server Error

**Description:** Server-side error

**Common Causes:**
- IBKR backend issue
- Unexpected server condition
- Database error
- Service temporarily unavailable

**Example Response:**
```json
{
  "error": "Internal server error",
  "statusCode": 500
}
```

**Handling:**
```python
def handle_500(response, attempt=1, max_attempts=3):
    """Handle server error with retry"""

    if attempt > max_attempts:
        logger.error("Server error persists after retries")
        raise ServerError("Internal server error")

    # Wait before retry (linear backoff for server errors)
    wait_time = attempt * 2  # 2, 4, 6 seconds
    logger.warning(f"Server error, retrying in {wait_time} seconds (attempt {attempt}/{max_attempts})")
    time.sleep(wait_time)

    # Retry request
    return retry_request(attempt + 1)
```

#### 502 Bad Gateway

**Description:** Gateway error, upstream server issue

**Handling:** Similar to 500, implement retry with backoff

#### 503 Service Unavailable

**Description:** Service temporarily unavailable (maintenance, overload)

**Handling:**
```python
def handle_503(response):
    logger.error("Service temporarily unavailable")

    # Check Retry-After header if present
    retry_after = response.headers.get('Retry-After')

    if retry_after:
        wait_time = int(retry_after)
        logger.info(f"Service unavailable, retry after {wait_time} seconds")
        time.sleep(wait_time)
        return retry_request()
    else:
        # Wait longer for service unavailable
        wait_time = 60  # 1 minute
        logger.info(f"Service unavailable, waiting {wait_time} seconds")
        time.sleep(wait_time)
        return retry_request()
```

## Application-Level Errors

### Order Rejection Errors

**Insufficient Funds:**
```json
{
  "error": "Order rejected: Insufficient funds",
  "statusCode": 400,
  "orderId": 0
}
```

**Handling:**
```python
class OrderError(Exception):
    pass

class InsufficientFundsError(OrderError):
    pass

def handle_order_rejection(response):
    error = response.json().get('error', '')

    if 'insufficient funds' in error.lower():
        raise InsufficientFundsError(error)
    elif 'market closed' in error.lower():
        raise MarketClosedError(error)
    elif 'invalid price' in error.lower():
        raise InvalidPriceError(error)
    else:
        raise OrderError(error)
```

### Market Data Errors

**No Subscription:**
```json
{
  "error": "Market data not available",
  "statusCode": 403
}
```

**Delayed Data:**
```json
{
  "conid": 265598,
  "31": null,
  "84": null,
  "86": null,
  "_updated": 1706282450123,
  "server_id": "m1"
}
```

**Handling:**
```python
def handle_market_data(data):
    # Check for null values
    if data.get('31') is None:
        logger.warning(f"No market data for conid {data['conid']}")

        # Check if delayed data available
        delayed_last = data.get('7289')  # Delayed last price
        if delayed_last is not None:
            logger.info(f"Using delayed data: {delayed_last}")
            return delayed_last
        else:
            raise MarketDataUnavailable(f"No market data for conid {data['conid']}")

    return data['31']
```

### Session Errors

**Session Timeout:**
```python
class SessionError(Exception):
    pass

class SessionTimeoutError(SessionError):
    pass

def check_session_health():
    """Periodically check session health"""

    status = get_auth_status()

    if not status['authenticated']:
        raise SessionTimeoutError("Session timed out")

    if status['competing']:
        raise SessionCompetingError("Competing session detected")

    return True
```

**Competing Session:**
```json
{
  "authenticated": false,
  "competing": true,
  "connected": true,
  "message": "Competing session detected"
}
```

**Handling:**
```python
def handle_competing_session():
    logger.error("Competing session detected")

    # Options:
    # 1. Logout other session (if accessible)
    # 2. Wait for other session to timeout
    # 3. Alert user

    # For automated systems, typically wait and retry
    logger.info("Waiting for competing session to close...")
    time.sleep(30)

    # Check again
    status = get_auth_status()
    if status['competing']:
        raise SessionCompetingError("Competing session still active")

    # Re-authenticate
    initialize_session()
```

## WebSocket Errors

### Connection Errors

**Connection Refused:**
```python
def on_error(ws, error):
    if isinstance(error, ConnectionRefusedError):
        logger.error("WebSocket connection refused - is Gateway running?")
        # Verify Gateway is running
        # Check port configuration

    elif isinstance(error, SSLError):
        logger.error("SSL certificate error")
        # For localhost Gateway, disable SSL verification

    else:
        logger.error(f"WebSocket error: {error}")
```

**SSL Certificate Error:**
```python
import ssl
import websocket

# Disable SSL verification for localhost Gateway
ssl_options = {"cert_reqs": ssl.CERT_NONE}

ws = websocket.WebSocketApp(
    "wss://localhost:5000/v1/api/ws",
    on_error=on_error,
    on_close=on_close,
    on_message=on_message
)

ws.run_forever(sslopt=ssl_options)
```

### Disconnection Handling

```python
class WebSocketManager:
    def __init__(self, url):
        self.url = url
        self.ws = None
        self.should_reconnect = True
        self.reconnect_attempts = 0
        self.max_reconnect_attempts = 10
        self.reconnect_delay = 5
        self.subscriptions = []

    def on_close(self, ws, close_status_code, close_msg):
        logger.warning(f"WebSocket closed: {close_status_code} - {close_msg}")

        if self.should_reconnect and self.reconnect_attempts < self.max_reconnect_attempts:
            self.reconnect()
        else:
            logger.error("Max reconnection attempts reached")

    def reconnect(self):
        self.reconnect_attempts += 1

        # Exponential backoff with max delay
        delay = min(self.reconnect_delay * (2 ** (self.reconnect_attempts - 1)), 60)

        logger.info(f"Reconnecting in {delay} seconds (attempt {self.reconnect_attempts})...")
        time.sleep(delay)

        try:
            self.connect()
            self.reconnect_attempts = 0  # Reset on successful connection
            self.resubscribe()
        except Exception as e:
            logger.error(f"Reconnection failed: {e}")
            self.on_close(None, None, str(e))

    def resubscribe(self):
        """Re-subscribe to all topics after reconnection"""
        for subscription in self.subscriptions:
            self.ws.send(subscription)
            logger.info(f"Re-subscribed: {subscription}")
```

### Message Parsing Errors

```python
def on_message(ws, message):
    try:
        data = json.loads(message)
        process_message(data)

    except json.JSONDecodeError as e:
        logger.error(f"Invalid JSON: {message}, error: {e}")
        # Don't crash on parsing errors, continue processing

    except KeyError as e:
        logger.error(f"Missing key in message: {e}, data: {data}")
        # Message structure unexpected

    except Exception as e:
        logger.error(f"Error processing message: {e}, data: {message}")
        # Generic error, don't crash WebSocket
```

## Error Recovery Strategies

### Automatic Retry

```python
from functools import wraps
import time

def retry_on_error(max_attempts=3, delay=1, backoff=2, exceptions=(Exception,)):
    """Decorator for automatic retry with exponential backoff"""

    def decorator(func):
        @wraps(func)
        def wrapper(*args, **kwargs):
            attempt = 1
            current_delay = delay

            while attempt <= max_attempts:
                try:
                    return func(*args, **kwargs)

                except exceptions as e:
                    if attempt == max_attempts:
                        logger.error(f"Max attempts ({max_attempts}) reached for {func.__name__}")
                        raise

                    logger.warning(f"{func.__name__} failed (attempt {attempt}/{max_attempts}): {e}")
                    time.sleep(current_delay)

                    attempt += 1
                    current_delay *= backoff

        return wrapper
    return decorator

# Usage
@retry_on_error(max_attempts=3, delay=1, backoff=2, exceptions=(RequestException, ServerError))
def place_order(order_data):
    response = requests.post(url, json=order_data)
    response.raise_for_status()
    return response.json()
```

### Circuit Breaker Pattern

```python
from enum import Enum
import time
from threading import Lock

class CircuitState(Enum):
    CLOSED = "closed"      # Normal operation
    OPEN = "open"          # Errors exceeded threshold, reject requests
    HALF_OPEN = "half_open"  # Testing if service recovered

class CircuitBreaker:
    def __init__(self, failure_threshold=5, timeout=60, expected_exception=Exception):
        self.failure_threshold = failure_threshold
        self.timeout = timeout
        self.expected_exception = expected_exception
        self.failure_count = 0
        self.last_failure_time = None
        self.state = CircuitState.CLOSED
        self.lock = Lock()

    def call(self, func, *args, **kwargs):
        with self.lock:
            if self.state == CircuitState.OPEN:
                # Check if timeout expired
                if time.time() - self.last_failure_time >= self.timeout:
                    logger.info("Circuit breaker entering half-open state")
                    self.state = CircuitState.HALF_OPEN
                else:
                    raise CircuitBreakerOpenError("Circuit breaker is open")

        try:
            result = func(*args, **kwargs)
            self.on_success()
            return result

        except self.expected_exception as e:
            self.on_failure()
            raise

    def on_success(self):
        with self.lock:
            self.failure_count = 0
            if self.state == CircuitState.HALF_OPEN:
                logger.info("Circuit breaker closing")
                self.state = CircuitState.CLOSED

    def on_failure(self):
        with self.lock:
            self.failure_count += 1
            self.last_failure_time = time.time()

            if self.failure_count >= self.failure_threshold:
                logger.error(f"Circuit breaker opening after {self.failure_count} failures")
                self.state = CircuitState.OPEN

# Usage
circuit_breaker = CircuitBreaker(failure_threshold=5, timeout=60, expected_exception=RequestException)

def make_api_call():
    return circuit_breaker.call(requests.get, 'https://localhost:5000/v1/api/...')
```

### Graceful Degradation

```python
def get_market_data_with_fallback(conid):
    """Attempt to get real-time data, fallback to delayed if unavailable"""

    try:
        # Try real-time data
        data = get_realtime_market_data(conid)
        return {
            'price': data['31'],
            'source': 'realtime',
            'timestamp': data['_updated']
        }

    except MarketDataUnavailable:
        logger.warning(f"Real-time data unavailable for {conid}, using delayed data")

        try:
            # Fallback to delayed data
            data = get_delayed_market_data(conid)
            return {
                'price': data['7289'],  # Delayed last price
                'source': 'delayed',
                'timestamp': data['_updated']
            }

        except Exception as e:
            logger.error(f"No market data available for {conid}: {e}")

            # Final fallback: Use last known price from cache
            cached_price = get_cached_price(conid)
            if cached_price:
                return {
                    'price': cached_price['price'],
                    'source': 'cache',
                    'timestamp': cached_price['timestamp']
                }

            raise MarketDataUnavailable(f"No market data available for {conid}")
```

## Logging and Monitoring

### Structured Logging

```python
import logging
import json

class JSONFormatter(logging.Formatter):
    def format(self, record):
        log_obj = {
            'timestamp': self.formatTime(record),
            'level': record.levelname,
            'message': record.getMessage(),
            'module': record.module,
            'function': record.funcName,
            'line': record.lineno
        }

        # Add exception info if present
        if record.exc_info:
            log_obj['exception'] = self.formatException(record.exc_info)

        # Add custom fields
        if hasattr(record, 'order_id'):
            log_obj['order_id'] = record.order_id
        if hasattr(record, 'conid'):
            log_obj['conid'] = record.conid
        if hasattr(record, 'account'):
            log_obj['account'] = record.account

        return json.dumps(log_obj)

# Setup logging
handler = logging.StreamHandler()
handler.setFormatter(JSONFormatter())
logger = logging.getLogger('ib_api')
logger.addHandler(handler)
logger.setLevel(logging.INFO)

# Usage with extra fields
logger.info("Order placed", extra={'order_id': 12345, 'conid': 265598})
```

### Error Tracking

```python
class ErrorTracker:
    def __init__(self):
        self.errors = deque(maxlen=1000)  # Keep last 1000 errors
        self.error_counts = {}
        self.lock = Lock()

    def record_error(self, error_type, error_msg, context=None):
        with self.lock:
            error_record = {
                'type': error_type,
                'message': error_msg,
                'context': context,
                'timestamp': time.time()
            }

            self.errors.append(error_record)

            # Track error counts
            if error_type not in self.error_counts:
                self.error_counts[error_type] = 0
            self.error_counts[error_type] += 1

    def get_error_summary(self, last_n_minutes=60):
        """Get error summary for last N minutes"""
        with self.lock:
            cutoff_time = time.time() - (last_n_minutes * 60)
            recent_errors = [e for e in self.errors if e['timestamp'] >= cutoff_time]

            summary = {}
            for error in recent_errors:
                error_type = error['type']
                if error_type not in summary:
                    summary[error_type] = 0
                summary[error_type] += 1

            return summary

# Usage
error_tracker = ErrorTracker()

try:
    place_order(order_data)
except InsufficientFundsError as e:
    error_tracker.record_error('INSUFFICIENT_FUNDS', str(e), {'order': order_data})
    logger.error(f"Order failed: {e}")
```

### Health Checks

```python
class HealthChecker:
    def __init__(self):
        self.last_successful_request = None
        self.consecutive_failures = 0
        self.session_healthy = False
        self.websocket_connected = False

    def check_health(self):
        """Comprehensive health check"""

        issues = []

        # Check session authentication
        try:
            status = get_auth_status()
            if status['authenticated']:
                self.session_healthy = True
            else:
                self.session_healthy = False
                issues.append("Session not authenticated")
        except Exception as e:
            self.session_healthy = False
            issues.append(f"Cannot check auth status: {e}")

        # Check WebSocket connection
        if not self.websocket_connected:
            issues.append("WebSocket disconnected")

        # Check for stale data
        if self.last_successful_request:
            time_since_request = time.time() - self.last_successful_request
            if time_since_request > 300:  # 5 minutes
                issues.append(f"No successful requests in {time_since_request:.0f} seconds")

        # Check consecutive failures
        if self.consecutive_failures > 10:
            issues.append(f"{self.consecutive_failures} consecutive failures")

        health_status = {
            'healthy': len(issues) == 0,
            'session_authenticated': self.session_healthy,
            'websocket_connected': self.websocket_connected,
            'consecutive_failures': self.consecutive_failures,
            'issues': issues,
            'timestamp': time.time()
        }

        return health_status

    def record_success(self):
        self.last_successful_request = time.time()
        self.consecutive_failures = 0

    def record_failure(self):
        self.consecutive_failures += 1

# Periodic health check
def health_check_loop(checker, interval=60):
    """Run health checks periodically"""
    while True:
        health = checker.check_health()

        if health['healthy']:
            logger.info("Health check passed")
        else:
            logger.warning(f"Health issues detected: {health['issues']}")
            # Trigger alerts if needed

        time.sleep(interval)
```

## Best Practices

### 1. Comprehensive Error Handling

```python
def api_request_wrapper(func):
    """Wrapper for all API requests with comprehensive error handling"""

    @wraps(func)
    def wrapper(*args, **kwargs):
        try:
            response = func(*args, **kwargs)

            if response.status_code == 200:
                return response.json()
            elif response.status_code == 400:
                return handle_400(response)
            elif response.status_code == 401:
                return handle_401(response)
            elif response.status_code == 403:
                return handle_403(response)
            elif response.status_code == 404:
                return handle_404(response)
            elif response.status_code == 429:
                return handle_429(response)
            elif response.status_code >= 500:
                return handle_500(response)
            else:
                logger.error(f"Unexpected status code: {response.status_code}")
                raise APIError(f"Unexpected status: {response.status_code}")

        except requests.exceptions.ConnectionError as e:
            logger.error(f"Connection error: {e}")
            raise ConnectionError("Cannot connect to IBKR API")

        except requests.exceptions.Timeout as e:
            logger.error(f"Request timeout: {e}")
            raise TimeoutError("Request timed out")

        except Exception as e:
            logger.error(f"Unexpected error: {e}", exc_info=True)
            raise

    return wrapper
```

### 2. Fail Fast vs Fail Safe

**Fail Fast:** (For critical operations)
```python
def critical_operation():
    if not session_authenticated():
        raise SessionError("Session not authenticated")

    if not sufficient_funds():
        raise InsufficientFundsError("Insufficient funds")

    # Proceed only if all checks pass
    execute_operation()
```

**Fail Safe:** (For non-critical operations)
```python
def get_market_data_safe(conid):
    try:
        return get_market_data(conid)
    except Exception as e:
        logger.warning(f"Market data fetch failed for {conid}: {e}")
        return None  # Return None instead of crashing
```

### 3. Error Context

Always provide context in error messages:

```python
try:
    place_order(order)
except OrderError as e:
    logger.error(
        f"Order placement failed",
        extra={
            'error': str(e),
            'order_id': order.get('orderId'),
            'conid': order.get('conid'),
            'account': order.get('account'),
            'order_type': order.get('orderType'),
            'quantity': order.get('quantity')
        }
    )
```

### 4. Don't Swallow Exceptions

**Bad:**
```python
try:
    critical_operation()
except:
    pass  # Silent failure
```

**Good:**
```python
try:
    critical_operation()
except SpecificError as e:
    logger.error(f"Operation failed: {e}")
    handle_error(e)
    raise  # Re-raise if cannot recover
```

### 5. Use Custom Exception Hierarchy

```python
class IBAPIError(Exception):
    """Base exception for IB API errors"""
    pass

class AuthenticationError(IBAPIError):
    """Authentication related errors"""
    pass

class SessionTimeoutError(AuthenticationError):
    """Session timeout"""
    pass

class OrderError(IBAPIError):
    """Order related errors"""
    pass

class InsufficientFundsError(OrderError):
    """Insufficient funds"""
    pass

class MarketDataError(IBAPIError):
    """Market data errors"""
    pass

class RateLimitError(IBAPIError):
    """Rate limit exceeded"""
    pass
```

---

**Research Date:** 2026-01-26
**API Version:** v1.0
**Error Handling:** Critical for Production Systems
