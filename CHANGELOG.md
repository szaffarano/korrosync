# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0](https://github.com/szaffarano/korrosync/compare/v0.1.5...v0.2.0) - 2026-01-09

### Fixed

- *(deps)* update rust crate axum-server to 0.8.0 ([#51](https://github.com/szaffarano/korrosync/pull/51))

### Other

- *(deps)* update rust crate tempfile to v3.24.0 ([#68](https://github.com/szaffarano/korrosync/pull/68))
- *(deps)* update rust crate serial_test to v3.3.1 ([#67](https://github.com/szaffarano/korrosync/pull/67))
- *(deps)* update rust crate tokio to v1.49.0 ([#69](https://github.com/szaffarano/korrosync/pull/69))
- *(deps)* update rust crate tokio-util to v0.7.18 ([#66](https://github.com/szaffarano/korrosync/pull/66))
- *(config)* migrate config renovate.json ([#70](https://github.com/szaffarano/korrosync/pull/70))
- *(deps)* update docker/metadata-action digest to ed95091 ([#65](https://github.com/szaffarano/korrosync/pull/65))
- *(deps)* update rust crate tracing to v0.1.44 ([#58](https://github.com/szaffarano/korrosync/pull/58))
- *(deps)* update rust crate reqwest to 0.13.0 ([#62](https://github.com/szaffarano/korrosync/pull/62))
- *(deps)* update rust crate tower-http to v0.6.8 ([#52](https://github.com/szaffarano/korrosync/pull/52))
- *(deps)* update rust crate governor to v0.10.4 ([#56](https://github.com/szaffarano/korrosync/pull/56))
- *(deps)* update rust crate serde_json to v1.0.149 ([#61](https://github.com/szaffarano/korrosync/pull/61))
- *(deps)* update axum monorepo ([#60](https://github.com/szaffarano/korrosync/pull/60))
- *(deps)* update docker/build-push-action digest to 64c9b14 ([#64](https://github.com/szaffarano/korrosync/pull/64))
- *(deps)* update actions/cache action to v5 ([#54](https://github.com/szaffarano/korrosync/pull/54))
- *(deps)* update docker/login-action digest to 916386b ([#59](https://github.com/szaffarano/korrosync/pull/59))
- *(deps)* update actions/upload-artifact action to v6 ([#55](https://github.com/szaffarano/korrosync/pull/55))
- migrate from bincode to rkyv for serialization ([#63](https://github.com/szaffarano/korrosync/pull/63))
- *(deps)* update rust crate uuid to v1.19.0 ([#50](https://github.com/szaffarano/korrosync/pull/50))
- *(deps)* update docker/metadata-action digest to c299e40 ([#49](https://github.com/szaffarano/korrosync/pull/49))
- *(deps)* update tokio-tracing monorepo ([#47](https://github.com/szaffarano/korrosync/pull/47))
- *(deps)* update actions/checkout action to v6 ([#45](https://github.com/szaffarano/korrosync/pull/45))

## [0.1.5](https://github.com/szaffarano/korrosync/compare/v0.1.4...v0.1.5) - 2025-11-16

### Added

- *(server)* Add tls feature flag ([#44](https://github.com/szaffarano/korrosync/pull/44))
- *(server)* Support TLS ([#43](https://github.com/szaffarano/korrosync/pull/43))

### Fixed

- *(docs)* Update TODOs
- *(docs)* Update readme

## [0.1.4](https://github.com/szaffarano/korrosync/compare/v0.1.3...v0.1.4) - 2025-11-16

### Added

- *(docker)* Publish on release

### Fixed

- *(docker)* Update tag resolution

### Other

- *(deps)* update docker/login-action digest to 28fdb31 ([#37](https://github.com/szaffarano/korrosync/pull/37))
- *(deps)* update docker/build-push-action digest to 9e436ba ([#36](https://github.com/szaffarano/korrosync/pull/36))
- *(deps)* update docker/metadata-action digest to 8d8c7c1 ([#40](https://github.com/szaffarano/korrosync/pull/40))
- *(ci)* Cleanup workflow definition ([#39](https://github.com/szaffarano/korrosync/pull/39))

## [0.1.3](https://github.com/szaffarano/korrosync/compare/v0.1.2...v0.1.3) - 2025-11-16

### Added

- *(refactor)* Improve error handling ([#35](https://github.com/szaffarano/korrosync/pull/35))

## [0.1.2](https://github.com/szaffarano/korrosync/compare/v0.1.1...v0.1.2) - 2025-11-14

### Added

- *(renovate)* Add zig to renovate config ([#26](https://github.com/szaffarano/korrosync/pull/26))

### Other

- Improve error handling ([#32](https://github.com/szaffarano/korrosync/pull/32))
- *(deps)* update rust crate governor to v0.10.2 ([#29](https://github.com/szaffarano/korrosync/pull/29))
- Improve coverage ([#31](https://github.com/szaffarano/korrosync/pull/31))
- *(deps)* update rust crate tokio-retry2 to 0.7.0 ([#30](https://github.com/szaffarano/korrosync/pull/30))
- *(deps)* update dependency ziglang/zig to v0.15.1 ([#27](https://github.com/szaffarano/korrosync/pull/27))

## [0.1.1](https://github.com/szaffarano/korrosync/releases/tag/v0.1.0) - 2025-11-13

### Added

- *(model,sync)* Extract model layer with documentation and tests ([#8](https://github.com/szaffarano/korrosync/pull/8))
- Configure CD ([#10](https://github.com/szaffarano/korrosync/pull/10))
- *(docker)* Add multiplatform support ([#6](https://github.com/szaffarano/korrosync/pull/6))
- Add renovate.json ([#3](https://github.com/szaffarano/korrosync/pull/3))
- *(wip)* KOReader sync API implementation ([#1](https://github.com/szaffarano/korrosync/pull/1))
- Project scaffolding with Cargo and Nix

### Other

- Update cargo.toml ([#13](https://github.com/szaffarano/korrosync/pull/13))
- Update cargo.toml ([#12](https://github.com/szaffarano/korrosync/pull/12))
- *(deps)* update rust crate tokio-retry2 to v0.6.1 ([#7](https://github.com/szaffarano/korrosync/pull/7))
- *(deps)* update actions/checkout action to v5 ([#4](https://github.com/szaffarano/korrosync/pull/4))
- Create LICENSE ([#2](https://github.com/szaffarano/korrosync/pull/2))
