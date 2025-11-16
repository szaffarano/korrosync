# Korrosync

<!-- markdownlint-disable-next-line no-inline-html -->
<div align="center">

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![codecov](https://codecov.io/gh/szaffarano/korrosync/graph/badge.svg?token=4D0LOB1XSV)](https://codecov.io/gh/szaffarano/korrosync)
[![CI](https://github.com/szaffarano/korrosync/actions/workflows/ci.yml/badge.svg)](https://github.com/szaffarano/korrosync/actions/workflows/ci.yml)
[![Dependency Status](https://deps.rs/repo/github/szaffarano/korrosync/status.svg)](https://deps.rs/repo/github/szaffarano/korrosync)

</div>

> A modern, high-performance KOReader sync server written in Rust

## Overview

Korrosync is a self-hosted synchronization server for [KOReader](https://github.com/koreader/koreader). It enables
seamless reading progress synchronization across multiple devices, allowing you to pick up where you left off on any
device.

## Installation

### Building from Source

```bash
# Clone the repository
git clone https://github.com/szaffarano/korrosync.git
cd korrosync

# Build with cargo
cargo build --release

# Run the server
./target/release/korrosync
```

### Using Nix Flakes

```bash
# Run directly with Nix
nix run github:szaffarano/korrosync

# Or enter development shell
nix develop
cargo run
```

### Pre-built Binaries

Pre-built binaries for Linux and macOS are available in the
[releases](https://github.com/szaffarano/korrosync/releases) section.

### Docker

Docker images are uploaded in [Docker Hub](https://hub.docker.com/r/szaffarano/korrosync/tags).

If you want to build the image yourself, use the following instructions.

```bash
# Build for your current platform
docker build -t korrosync .

# Multi-arch builds (requires buildx)
# First, create a buildx builder if you haven't already
docker buildx create --name multiarch --use

# e.g., build for Raspberry Pi 3 and load locally
docker buildx build \
  --platform linux/arm/v7 \
  -t korrosync:arm32 \
  --load .

# Run container (HTTP)
docker run -d \
  -p 3000:3000 \
  -v $(pwd)/data:/data \
  -e KORROSYNC_DB_PATH=/data/db.redb \
  --name korrosync \
  korrosync

# Run container with TLS enabled
docker run -d \
  -p 3000:3000 \
  -v $(pwd)/data:/data \
  -v $(pwd)/tls:/tls \
  -e KORROSYNC_DB_PATH=/data/db.redb \
  -e KORROSYNC_USE_TLS=true \
  -e KORROSYNC_CERT_PATH=/tls/cert.pem \
  -e KORROSYNC_KEY_PATH=/tls/key.pem \
  --name korrosync \
  korrosync
```

## Configuration

Korrosync is configured through environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `KORROSYNC_DB_PATH` | Path to the redb database file | `data/db.redb` |
| `KORROSYNC_SERVER_ADDRESS` | Server bind address | `0.0.0.0:3000` |
| `KORROSYNC_USE_TLS` | Enable TLS/HTTPS support (true/1/yes/on or false/0/no/off, case-insensitive) | `false` |
| `KORROSYNC_CERT_PATH` | Path to TLS certificate file (PEM format) | `tls/cert.pem` |
| `KORROSYNC_KEY_PATH` | Path to TLS private key file (PEM format) | `tls/key.pem` |

### Example

```bash
# Basic configuration
export KORROSYNC_DB_PATH=/var/lib/korrosync/db.redb
export KORROSYNC_SERVER_ADDRESS=127.0.0.1:8080
korrosync

# With TLS enabled
export KORROSYNC_DB_PATH=/var/lib/korrosync/db.redb
export KORROSYNC_SERVER_ADDRESS=0.0.0.0:3000
export KORROSYNC_USE_TLS=true
export KORROSYNC_CERT_PATH=/etc/korrosync/tls/cert.pem
export KORROSYNC_KEY_PATH=/etc/korrosync/tls/key.pem
korrosync
```

## Usage

### Starting the Server

```bash
# Start with default configuration
korrosync

# Or with custom configuration
KORROSYNC_SERVER_ADDRESS=0.0.0.0:8080 korrosync
```

### API Endpoints

- `POST /users/create` — Register a new user
- `GET /users/auth` — Verify authentication status
- `PUT /syncs/progress` — Update reading progress for a document
- `GET /syncs/progress/{document}` — Retrieve progress for a specific document
- `GET /healthcheck` — Health check endpoint
- `GET /robots.txt` — Robots exclusion file

## Deployment

### Systemd Service

Create a systemd service file at `/etc/systemd/system/korrosync.service`:

```ini
[Unit]
Description=Korrosync - KOReader Sync Server
After=network.target

[Service]
Type=simple
User=korrosync
Group=korrosync
Environment="KORROSYNC_DB_PATH=/var/lib/korrosync/db.redb"
Environment="KORROSYNC_SERVER_ADDRESS=0.0.0.0:3000"
# Uncomment to enable TLS
#Environment="KORROSYNC_USE_TLS=true"
#Environment="KORROSYNC_CERT_PATH=/etc/korrosync/tls/cert.pem"
#Environment="KORROSYNC_KEY_PATH=/etc/korrosync/tls/key.pem"
ExecStart=/usr/local/bin/korrosync
Restart=on-failure
RestartSec=5s

# Security
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/korrosync

[Install]
WantedBy=multi-user.target
```

Enable and start the service:

```bash
sudo systemctl enable korrosync
sudo systemctl start korrosync
sudo systemctl status korrosync
```

### Native TLS Support

Korrosync supports built-in TLS/HTTPS without requiring a reverse proxy:

```bash
# Enable TLS with environment variables
export KORROSYNC_USE_TLS=true
export KORROSYNC_CERT_PATH=/etc/korrosync/tls/cert.pem
export KORROSYNC_KEY_PATH=/etc/korrosync/tls/key.pem
korrosync
```

**Note:** Certificate and private key files must be in PEM format. For production, use certificates from a trusted CA
(e.g., Let's Encrypt). For testing, self-signed certificates are provided in the `tls/` directory. See `tls/README.md`
for more information.

### Reverse Proxy (Nginx)

Alternatively, use a reverse proxy with TLS termination:

```nginx
server {
    listen 443 ssl;
    http2 on;
    server_name sync.example.com;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

## Development

### Setting Up Development Environment

Using Nix flakes (recommended):

```bash
# Enter development shell with all tools
nix develop

# Pre-commit hooks will be installed automatically
```

### Running Tests

```bash
# Run all tests
cargo test

# Run with coverage
cargo tarpaulin --out Html

# Run integration tests only
cargo test --test '*'
```

## Architecture

Korrosync is built with:

- **[Axum](https://github.com/tokio-rs/axum)** — Fast, ergonomic web framework
- **[redb](https://github.com/cberner/redb)** — Embedded key-value database (ACID compliant)
- **[Argon2](https://github.com/RustCrypto/password-hashes)** — Secure password hashing
- **[Tokio](https://tokio.rs/)** — Async runtime
- **[Tower](https://github.com/tower-rs/tower)** — Middleware and service abstractions
- **[Governor](https://github.com/beltram/governor)** — Rate limiting

## TODO

The following features and improvements are planned:

### API & Features

- [ ] OpenAPI/Swagger documentation

### Infrastructure

- [x] TLS/HTTPS configuration support
- [ ] Configurable rate limiting via environment variables
- [ ] Metrics and observability (Prometheus/OpenTelemetry)
- [ ] Structured logging with log levels
- [ ] CLI tool for administrative tasks (user management, database maintenance)

### Deployment & Distribution

- [x] Automated binary releases for multiple platforms
- [x] Official Docker images on Docker Hub
- [ ] Helm chart for Kubernetes deployment
- [ ] NixOS module for declarative configuration

### Documentation

- [x] API documentation
- [ ] Deployment guides

### Testing & Quality

- [x] Increase test coverage to >80%
- [ ] Improve logging and error handling

## License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [KOReader](https://github.com/koreader/koreader) — The amazing e-reader application this server supports
- [KOReader progress sync server](https://github.com/lzyor/kosync) — Inspiration for this implementation
- The Rust community for excellent tools and libraries
