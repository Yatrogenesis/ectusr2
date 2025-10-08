# syntax=docker/dockerfile:1

FROM rust:1.75 as builder
WORKDIR /app
COPY . .
# Build in release
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/ectusr2 /usr/local/bin/ectusr2
# Default metrics port if enabled by env METRICS_ADDR
EXPOSE 9900
ENTRYPOINT ["/usr/local/bin/ectusr2"]
