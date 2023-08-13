FROM lukemathwalker/cargo-chef:latest-rust-1.70.0-bullseye AS chef
WORKDIR /app

FROM chef AS planner
COPY Cargo.lock .
COPY Cargo.toml .
COPY migration migration
COPY backend backend
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY Cargo.lock .
COPY Cargo.toml .
COPY migration migration
COPY backend backend
RUN cargo build --release

FROM debian:bullseye-slim AS runtime
RUN apt-get update &&\
    apt-get install -y libssl-dev
WORKDIR /app
COPY --from=builder /app/target/release/chamsae /usr/local/bin
COPY --from=builder /app/target/release/migration /usr/local/bin
