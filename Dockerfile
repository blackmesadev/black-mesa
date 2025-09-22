FROM rust:1.90-slim AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY black-mesa/Cargo.toml black-mesa/Cargo.lock ./
COPY black-mesa/src ./src
COPY lib ../lib

RUN cargo build --release --bin black-mesa

FROM debian:trixie-slim AS runtime
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app

COPY --from=builder /app/target/release/black-mesa /usr/local/bin/black-mesa

ENTRYPOINT ["black-mesa"]
