# Rust API Gateway

An API Gateway built with Rust. This provides a robust solution for managing API traffic, featuring authentication, rate limiting and caching.

##  Features

-  **Authentication**
  - Bearer token authentication
  - Configurable token validation
  - Secure token management

- **Rate Limiting**
  - Per-client rate limiting
  - Configurable time windows
  - Protection against DoS attacks

- **Caching**
  - In-memory caching for GET requests
  - Configurable cache duration
  - Automatic cache cleanup

- **High Performance**
  - Built with Rust's async/await
  - Efficient memory usage
  - Connection pooling

- **Proxy Capabilities**
  - Request/Response transformation
  - Path-based routing
  - Backend service proxying

-  **Monitoring**
  - Request/Response logging
  - Performance metrics
  - Error tracking

## Architecture

```
api-gateway/
├── src/
│   ├── services/           # Core business logic
│   │   ├── mod.rs
│   │   └── tests.rs
│   ├── handlers/          # Request handlers
│   │   ├── mod.rs
│   │   └── tests.rs
│   ├── middleware/        # HTTP middleware
│   │   ├── mod.rs
│   │   └── tests.rs
│   ├── lib.rs            # Library definitions
│   ├── main.rs           # Application entry point
│   ├── config.rs         # Configuration
│   ├── error.rs          # Error handling
│   └── models.rs         # Data structures
└── tests/
    └── integration_tests.rs
```

## Quick Start

### Prerequisites

- Rust 1.75 or higher
- Cargo package manager
- A backend service to proxy to

### Installation

1. Clone the repository:
```bash
git clone https://github.com/miky-rola/api-gateway
cd api-gateway
```

2. Build the project:
```bash
cargo build --release
```

3. Configure the gateway in `config.rs`:
```rust
pub const BACKEND_BASE: &str = "http://localhost:8081";
pub const RATE_LIMIT_REQUESTS: u32 = 100;
pub const RATE_LIMIT_WINDOW_SECS: u64 = 60;
```

4. Run the gateway:
```bash
cargo run --release
```

The gateway will start on `http://127.0.0.1:3030`

## 🔧 Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `BACKEND_BASE` | Backend service URL | `http://localhost:8081` |
| `RATE_LIMIT_REQUESTS` | Requests per window | 100 |
| `RATE_LIMIT_WINDOW_SECS` | Rate limit window | 60 seconds |
| `REQUEST_TIMEOUT_SECS` | Request timeout | 30 seconds |
| `CACHE_DURATION_SECS` | Cache duration | 300 seconds |
| `STRIP_PATH_PREFIX` | Path prefix to strip | `/api` |

## API Usage

### Health Check
```bash
curl http://localhost:3030/health
```

### Authenticated Request
```bash
curl -H "Authorization: Bearer example-token" \
     http://localhost:3030/api/your-endpoint
```

### Cached GET Request
```bash
curl -H "Authorization: Bearer example-token" \
     http://localhost:3030/api/cached-endpoint
```

## Testing

Run all tests:
```bash
cargo test
```

Run specific test categories:
```bash
# Unit tests
cargo test --lib

# Integration tests
cargo test --test integration_tests

# With logging
RUST_LOG=debug cargo test
```


### Monitoring
```bash
# Enable debug logging
RUST_LOG=debug cargo run
```

## Security

- Bearer token authentication
- Rate limiting protection
- Request timeouts
- CORS protection
- No sensitive data logging

## 🔧 Local Development

1. Clone and install dependencies:
```bash
git clone https://github.com/miky-rola/api-gateway
cd rust-api-gateway
cargo build
```

2. Run tests:
```bash
cargo test
```

3. Run with logging:
```bash
RUST_LOG=debug cargo run
```

## Contributing

1. Fork the repository
2. Create your feature branch:
```bash
git checkout -b new-feature
```

3. Commit your changes:
```bash
git commit -m 'Add amazing feature'
```

4. Push to the branch:
```bash
git push origin new-feature
```

5. Open a Pull Request

## Acknowledgments

- [Warp](https://github.com/seanmonstar/warp) - Web framework
- [Tokio](https://tokio.rs) - Async runtime
- [Hyper](https://hyper.rs) - HTTP client/server

## Contact

miky rola - [mikyrola8@gmail.com](mikyrola8@gmail.com)

Project Link: [https://github.com/miky-rola/api-gateway](https://github.com/miky-rola/api-gateway)

---

Star us on GitHub — it motivates us to make the gateway even better!