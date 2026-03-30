# syntax=docker/dockerfile:1.7

FROM rust:1.94-slim-trixie AS builder
WORKDIR /app/black-mesa

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY ./lib /app/lib
COPY ./black-mesa/Cargo.toml ./black-mesa/Cargo.lock ./black-mesa/build.rs ./
COPY ./black-mesa/src/main.rs ./src/main.rs

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo fetch

COPY ./black-mesa/src ./src

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/cargo-target \
    CARGO_TARGET_DIR=/cargo-target cargo build --release --bin black-mesa \
    && cp /cargo-target/release/black-mesa /usr/local/bin/black-mesa

FROM debian:trixie-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3t64 \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app

COPY --from=builder /usr/local/bin/black-mesa /usr/local/bin/black-mesa

RUN useradd --system --uid 10001 --create-home blackmesa
USER blackmesa

ENTRYPOINT ["black-mesa"]
