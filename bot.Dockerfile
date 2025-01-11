FROM lukemathwalker/cargo-chef:latest AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --bin black-mesa --recipe-path recipe.json
COPY --from=mesa-lib:latest /app/target/release/deps/libbm_lib-*.rlib /app/target/release/deps/
COPY . .
RUN cargo build --release --bin black-mesa

FROM rust AS runtime
RUN apt-get update && apt-get install -y libssl3
COPY --from=builder /app/target/release/black-mesa /usr/local/bin/
CMD ["black-mesa"]