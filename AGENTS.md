# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Korrosync is a self-hosted KOReader synchronization server written in Rust. It implements the KOReader sync protocol,
allowing users to sync reading progress across devices. Uses an embedded `redb` database (no external DB required).

## Build & Development Commands

```bash
cargo build                              # Debug build
cargo build --release --features tls     # Release build with TLS support
cargo test                               # All tests (unit + integration)
cargo test --lib                         # Unit tests only
cargo test --test '*'                    # Integration tests only
cargo test <test_name>                   # Single test by name
cargo fmt --check                        # Check formatting
cargo clippy -- -D warnings              # Lint (warnings are errors in CI)
cargo machete                            # Detect unused dependencies
cargo deny check advisories              # Security audit
cargo tarpaulin --out Html               # Coverage report
```

The project uses Nix flakes with direnv (`use flake` in `.envrc`) for the development environment, which provides all
necessary tooling including rust-analyzer, cargo-tarpaulin, cargo-machete, and cargo-deny.

## Architecture

Three-layer architecture: **Model → Service → API**, connected through a trait-based service abstraction.

### Service Layer (`src/service/`)

- `KorrosyncService` trait (`src/service/db/mod.rs`) defines the storage interface — all database operations go through this trait
- `KorrosyncServiceRedb` (`src/service/db/redb.rs`) is the only implementation, using embedded `redb` with two tables: `users-v2` and `progress-v2`
- `Rkyv<T>` wrapper (`src/service/serialization.rs`) bridges `rkyv` zero-copy serialization with redb's `Value`/`Key` traits

### API Layer (`src/api/`)

- Axum-based with Tower middleware stack
- Two route groups in `router.rs`: public (register, robots.txt) and authenticated (auth, progress sync, healthcheck)
- Auth middleware (`api/middleware/auth.rs`) validates `x-auth-user`/`x-auth-key` headers against the database
- Rate limiter (`api/middleware/ratelimiter.rs`) uses Governor (2 req/sec, burst 5)
- `AppState` holds `Arc<dyn KorrosyncService + Send + Sync>`

### Model Layer (`src/model/`)

- `User`: username + Argon2 password hash + last_activity timestamp, rkyv-serializable
- `Progress`: device_id, device, percentage, progress, timestamp, rkyv-serializable

### CLI (`src/cli.rs`, `src/main.rs`)

- Three subcommands: `serve`, `user` (create/list/remove/reset-password), `db` (info/backup)
- Global `--db-path` flag overrides `KORROSYNC_DB_PATH` env var
- Passwords accept `-` to read from stdin

### Configuration (`src/config.rs`)

All config via environment variables:

- `KORROSYNC_DB_PATH` (default: `data/db.redb`)
- `KORROSYNC_SERVER_ADDRESS` (default: `0.0.0.0:3000`)
- TLS (behind `tls` feature flag): `KORROSYNC_USE_TLS`, `KORROSYNC_CERT_PATH`, `KORROSYNC_KEY_PATH`

## Testing Patterns

- Integration tests use `tower::ServiceExt::oneshot()` to call handlers directly (no real server)
- `tests/common/mod.rs` provides `spawn_app()`, `spawn_app_with_users()` helpers and request builders
- CLI tests (`tests/cli.rs`) use `#[serial_test::serial]` because they bind to port 3000
- Unit tests are inline `#[cfg(test)]` modules in `src/model/user.rs` and `src/service/db/redb.rs`

## Key Dependencies

- **axum** + **tower** + **tower-http**: Web framework and middleware
- **redb**: Embedded ACID key-value database
- **rkyv**: Zero-copy serialization for database values
- **argon2**: Password hashing
- **clap** (derive): CLI parsing
- **governor** / **tower_governor**: Rate limiting

## CI Pipeline

Runs on PRs and pushes to master: `cargo test`, `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo machete`, and
coverage via `cargo-tarpaulin` uploaded to Codecov (target: 85-100%).
