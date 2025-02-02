# Rust API Gateway

An API Gateway built with Rust, featuring authentication, rate limiting, caching, and proxy capabilities.

## Features

- 🔒 **Authentication** - Bearer token authentication
- 🚦 **Rate Limiting** - Configurable request limits per client
- 💨 **Caching** - In-memory caching for GET requests
- ⚡ **High Performance** - Built with Rust and async/await
- 🔄 **Proxy** - Forward requests to backend services
- ⏱️ **Timeout Handling** - Configurable request timeouts
- 🔍 **Request Logging** - Performance monitoring
- 🌐 **CORS Support** - Configurable CORS headers

## Architecture

```
src/
├── lib.rs         # Library definitions and exports
├── main.rs        # Application entry point
├── config.rs      # Configuration constants
├── error.rs       # Error handling
├── models.rs      # Data structures
├── services.rs    # Core business logic
├── middleware.rs  # HTTP middleware functions
└── handlers.rs    # Request handlers
```

## Quick Start

1. Clone the repository:
```bash
git clone https://github.com/miky-rola/api-gateway
cd api-gateway
```

2. Configure the gateway in `config.rs`:
```rust
pub const BACKEND_BASE: &str = "http://localhost:8081";
pub const RATE_LIMIT_REQUESTS: u32 = 100;
pub const RATE_LIMIT_WINDOW_SECS: u64 = 60;
```

3. Run the gateway:
```bash
cargo run
```

The gateway will start on `http://127.0.0.1:3030`

## Configuration

- `BACKEND_BASE`: Base URL of your backend service
- `RATE_LIMIT_REQUESTS`: Number of requests allowed per window
- `RATE_LIMIT_WINDOW_SECS`: Rate limit window size in seconds
- `REQUEST_TIMEOUT_SECS`: Request timeout in seconds
- `CACHE_DURATION_SECS`: Cache duration for GET requests
- `STRIP_PATH_PREFIX`: Path prefix to strip before forwarding

## API Usage

1. Health Check:
```bash
curl http://localhost:3030/health
```

2. Proxy Request with Authentication:
```bash
curl -H "Authorization: Bearer example-token" \
     http://localhost:3030/api/your-endpoint
```

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Performance

The gateway is built with performance in mind:
- Async/await for non-blocking I/O
- In-memory caching for frequently accessed endpoints
- Efficient request handling with warp
- Minimal memory footprint

## Security

- Bearer token authentication
- Rate limiting per client IP
- Request timeouts
- CORS protection
- No sensitive data logging

## Local Development

Requirements:
- Rust 1.75 or higher
- Cargo package manager

Build for development:
```bash
cargo build
```

Run tests:
```bash
cargo test
```

Run with logging:
```bash
RUST_LOG=debug cargo run
```