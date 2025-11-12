# syntax=docker/dockerfile:1
FROM --platform=$BUILDPLATFORM rust:1-slim AS builder

# cross-compilation dependencies
# hadolint ignore=DL3008
RUN apt-get update && apt-get install -y \
    gcc-arm-linux-gnueabihf \
    libc6-dev-armhf-cross \
    gcc-aarch64-linux-gnu \
    libc6-dev-arm64-cross \
    --no-install-recommends \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

ARG TARGETPLATFORM

RUN case "$TARGETPLATFORM" in \
    "linux/amd64") echo "x86_64-unknown-linux-musl" > /rust-target.txt ;; \
    "linux/arm64") echo "aarch64-unknown-linux-gnu" > /rust-target.txt ;; \
    "linux/arm/v7") echo "armv7-unknown-linux-musleabihf" > /rust-target.txt ;; \
    *) echo "Unsupported platform: $TARGETPLATFORM" && exit 1 ;; \
    esac

RUN RUST_TARGET=$(cat /rust-target.txt) && \
    rustup target add "$RUST_TARGET" && \
    case "$TARGETPLATFORM" in \
    "linux/arm/v7") \
    export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_MUSLEABIHF_LINKER=arm-linux-gnueabihf-gcc && \
    export CC_armv7_unknown_linux_musleabihf=arm-linux-gnueabihf-gcc ;; \
    "linux/arm64") \
    export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc && \
    export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc ;; \
    esac && \
    cargo build --release --target "$RUST_TARGET" && \
    mkdir -p /output && \
    cp target/"$RUST_TARGET"/release/korrosync /output/korrosync

FROM scratch

ENV KORROSYNC_DB_PATH=/db.redb
ENV KORROSYNC_SERVER_ADDRESS=0.0.0.0:3000

COPY --from=builder /output/korrosync /app/korrosync
EXPOSE 3000

ENTRYPOINT ["/app/korrosync"]
