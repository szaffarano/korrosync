FROM rust:1-slim AS builder

WORKDIR /app
COPY . .
RUN rustup target add x86_64-unknown-linux-musl && \
  cargo build --release --target x86_64-unknown-linux-musl

FROM scratch

ENV KORROSYNC_DB_PATH=/db.redb
ENV KORROSYNC_SERVER_ADDRESS=0.0.0.0:3000

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/korrosync /app/
EXPOSE 3000

ENTRYPOINT ["/app/korrosync"]
