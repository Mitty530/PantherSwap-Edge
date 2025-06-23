# PantherSwap Edge Integration Tests

This directory contains comprehensive integration tests for the PantherSwap Edge REST API and core functionality.

## 🧪 Test Structure

### Test Categories

1. **`integration_tests.rs`** - Core API functionality tests
   - Health endpoints (health, status, metrics, liveness, readiness)
   - Basic authentication validation
   - Rate limiting verification
   - CORS and security headers
   - Request validation

2. **`auth_tests.rs`** - Authentication and authorization tests
   - API key authentication with various scenarios
   - Role-based access control (Admin, Trader, ReadOnly)
   - Malformed authorization headers
   - Authentication bypass attempts
   - Concurrent authentication requests

3. **`rate_limit_tests.rs`** - Rate limiting tests
   - IP-based rate limiting for unauthenticated requests
   - User-based rate limiting for authenticated requests
   - Role-based rate limit differences
   - Burst capacity testing
   - Rate limit reset functionality

4. **`api_endpoint_tests.rs`** - Comprehensive endpoint tests
   - All health and monitoring endpoints
   - Instruments API (CRUD operations)
   - Market data API endpoints
   - Request validation and error handling
   - HTTP method validation
   - Query parameter handling

5. **`database_tests.rs`** - Database integration tests
   - Database connection and basic operations
   - Instrument CRUD operations
   - Market tick operations
   - Performance and connection pooling
   - Error handling and edge cases
   - Database schema validation

6. **`simple_test.rs`** - Basic framework validation
   - Simple health endpoint test
   - Framework functionality verification
   - Quick smoke tests

### Common Test Utilities (`common/mod.rs`)

- **Test app setup** - Creates configured test application instances
- **Test data generators** - Creates realistic test data
- **Assertion helpers** - Common test assertions
- **API key constants** - Test API keys for different roles
- **Endpoint constants** - API endpoint URLs
- **Rate limiting helpers** - Utilities for rate limit testing

## 🚀 Running Tests

### Run All Integration Tests
```bash
cargo test --tests
```

### Run Specific Test Categories
```bash
# Core integration tests
cargo test --test integration_tests

# Authentication tests
cargo test --test auth_tests

# Rate limiting tests
cargo test --test rate_limit_tests

# API endpoint tests
cargo test --test api_endpoint_tests

# Database tests
cargo test --test database_tests

# Simple framework tests
cargo test --test simple_test
```

### Run Specific Tests
```bash
# Run a specific test
cargo test --test integration_tests test_health_check

# Run with output
cargo test --test simple_test -- --nocapture

# Run with verbose logging
RUST_LOG=debug cargo test --test integration_tests
```

### Run Test Runner
```bash
# Run comprehensive test suite
cargo test --test test_runner
```

## 🔧 Test Configuration

### Environment Variables
- `DATABASE_URL` - PostgreSQL/TimescaleDB connection string
- `ALPHA_VANTAGE_API_KEY` - Alpha Vantage API key for market data
- `RUST_LOG` - Logging level (debug, info, warn, error)

### Test API Keys
The tests use predefined API keys for different roles:
- `demo-admin-key` - Full admin access
- `demo-trader-key` - Trading operations access
- `demo-readonly-key` - Read-only access
- `invalid-test-key` - Invalid key for negative testing

### Database Requirements
- Tests can run with or without a real database
- If database is unavailable, tests use mock connections
- TimescaleDB features are tested when available
- Tests handle database errors gracefully

## 📊 Test Coverage

### API Endpoints Tested
- ✅ `GET /health` - Basic health check
- ✅ `GET /status` - System status
- ✅ `GET /metrics` - System metrics
- ✅ `GET /health/liveness` - Kubernetes liveness probe
- ✅ `GET /health/readiness` - Kubernetes readiness probe
- ✅ `GET /api/v1/instruments` - List instruments
- ✅ `GET /api/v1/instruments/{id}` - Get specific instrument
- ✅ `POST /api/v1/instruments` - Create instrument
- ✅ `PUT /api/v1/instruments/{id}` - Update instrument
- ✅ `GET /api/v1/market-data/latest` - Latest market ticks
- ✅ `GET /api/v1/market-data/ticks` - Market ticks with filters
- ✅ `GET /api/v1/market-data/ohlc` - OHLC data
- ✅ `GET /api/v1/market-data/stats` - Market statistics

### Security Features Tested
- ✅ API key authentication
- ✅ Role-based access control
- ✅ Rate limiting (IP and user-based)
- ✅ CORS headers
- ✅ Security headers (XSS, CSRF protection)
- ✅ Request validation
- ✅ Authentication bypass prevention

### Database Operations Tested
- ✅ Connection pooling
- ✅ Instrument CRUD operations
- ✅ Market tick insertion and retrieval
- ✅ Time-range queries
- ✅ Concurrent operations
- ✅ Error handling
- ✅ Schema validation

## 🎯 Test Results

### Expected Behavior
- **Health endpoints** should return 200 OK without authentication
- **Protected endpoints** should return 401 without valid API key
- **Rate limiting** should trigger after configured limits
- **Database operations** should work with real DB or gracefully handle mock DB
- **Security headers** should be present in responses
- **Error responses** should follow consistent format

### Common Test Scenarios
1. **Happy path** - All operations work with valid inputs
2. **Authentication failures** - Invalid/missing API keys rejected
3. **Authorization failures** - Insufficient permissions rejected
4. **Rate limiting** - Excessive requests throttled
5. **Input validation** - Invalid data rejected
6. **Database errors** - Graceful handling of DB issues
7. **Concurrent operations** - System handles multiple simultaneous requests

## 🔍 Debugging Tests

### Verbose Output
```bash
# Show test output
cargo test --test simple_test -- --nocapture

# Debug logging
RUST_LOG=debug cargo test --test integration_tests

# Show all test names
cargo test --test integration_tests -- --list
```

### Common Issues
- **Database connection failures** - Tests will use mock DB and continue
- **Rate limiting not triggered** - Timing-dependent, may need adjustment
- **Authentication errors** - Check API key configuration
- **Compilation errors** - Ensure all dependencies are available

## 📈 Performance Considerations

### Test Performance
- Tests use connection pooling for efficiency
- Concurrent tests verify system scalability
- Database operations are tested for performance
- Rate limiting tests verify throttling effectiveness

### Resource Usage
- Tests create minimal test data
- Database connections are pooled and reused
- Memory usage is monitored in performance tests
- Network requests are minimized

## 🎉 Success Criteria

The integration test suite validates that:
1. **API is functional** - All endpoints respond correctly
2. **Security is working** - Authentication and authorization enforced
3. **Performance is acceptable** - Rate limiting and connection pooling work
4. **Database integration works** - CRUD operations and queries function
5. **Error handling is robust** - System gracefully handles failures
6. **Configuration is correct** - All components integrate properly

This comprehensive test suite ensures the PantherSwap Edge API is production-ready and meets all functional and security requirements.
