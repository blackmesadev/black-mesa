FROM lukemathwalker/cargo-chef:latest-rust-1.64 AS chef
WORKDIR /usr/src/black-mesa

FROM chef AS prepare
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS build
COPY --from=prepare /usr/src/black-mesa/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

FROM rust AS runtime
COPY --from=build /usr/src/black-mesa/target/release/black-mesa .
EXPOSE 3000
CMD ["./black-mesa"]