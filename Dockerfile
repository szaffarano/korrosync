# syntax=docker/dockerfile:1
FROM --platform=$BUILDPLATFORM rust:1-slim AS builder

ARG BUILDPLATFORM
ARG TARGETPLATFORM
ARG ZIG_VERSION=0.15.1

SHELL ["/bin/bash", "-eo", "pipefail", "-c"]

# cross-compilation dependencies
# hadolint ignore=DL3008
RUN apt-get update && apt-get install -y \
    curl \
    xz-utils \
    --no-install-recommends \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

RUN case "$BUILDPLATFORM" in \
    "linux/amd64") ZIG_ARCH="x86_64" ;; \
    "linux/arm64") ZIG_ARCH="aarch64" ;; \
    "linux/arm/v7") ZIG_ARCH="arm" ;; \
    *) echo "Unsupported platform: $BUILDPLATFORM" && exit 1 ;; \
    esac \
    && curl -L "https://ziglang.org/download/${ZIG_VERSION}/zig-${ZIG_ARCH}-linux-${ZIG_VERSION}.tar.xz" | tar xJ --strip-components=1 -C /usr/local/bin 

RUN case "$TARGETPLATFORM" in \
    "linux/amd64") echo "x86_64-unknown-linux-musl" > /rust-target.txt ;; \
    "linux/arm64") echo "aarch64-unknown-linux-musl" > /rust-target.txt ;; \
    "linux/arm/v7") echo "armv7-unknown-linux-musleabihf" > /rust-target.txt ;; \
    *) echo "Unsupported platform: $TARGETPLATFORM" && exit 1 ;; \
    esac \
    && RUST_TARGET=$(cat /rust-target.txt) \
    && rustup target add "$RUST_TARGET" \
    && cargo install --locked cargo-zigbuild \
    && cargo zigbuild --release --target "$RUST_TARGET" \
    && mkdir -p /output \
    && cp target/"$RUST_TARGET"/release/korrosync /output/korrosync

FROM scratch

ENV KORROSYNC_DB_PATH=/db.redb
ENV KORROSYNC_SERVER_ADDRESS=0.0.0.0:3000

COPY --from=builder /output/korrosync /app/korrosync
EXPOSE 3000

ENTRYPOINT ["/app/korrosync"]
