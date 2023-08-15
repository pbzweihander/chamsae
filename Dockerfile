# syntax = docker/dockerfile:1

FROM lukemathwalker/cargo-chef:latest-rust-1.71-slim-bookworm AS chef
WORKDIR /app

FROM node:20-bookworm-slim AS nodebase
WORKDIR /app


FROM chef AS backend-planner
COPY Cargo.lock .
COPY Cargo.toml .
COPY migration migration
COPY backend backend
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS backend-builder
COPY --from=backend-planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY Cargo.lock .
COPY Cargo.toml .
COPY migration migration
COPY backend backend
RUN cargo build --release


FROM nodebase AS frontend-builder
# Check https://github.com/nodejs/docker-node/tree/b4117f9333da4138b03a546ec926ef50a31506c3#nodealpine to understand why libc6-compat might be needed.
#RUN apk add --no-cache libc6-compat
WORKDIR /app

COPY frontend/package.json ./package.json
COPY frontend/yarn.lock ./yarn.lock
COPY frontend/.yarnrc.yml ./.yarnrc.yml
COPY frontend/.yarn ./.yarn
RUN --mount=type=cache,target=/.yarn/berry/cache \
    yarn install --immutable

COPY frontend ./

RUN NEXT_TELEMETRY_DISABLED=1 yarn build


FROM nodebase AS runtime
ENV NODE_ENV production
ENV NEXT_TELEMETRY_DISABLED 1

RUN addgroup --system --gid 1001 chamsae
RUN adduser --system --uid 1001 chamsae

RUN apt-get update && \
    apt-get install -y libssl-dev

USER chamsae

COPY --from=backend-builder /app/target/release/chamsae /usr/local/bin
COPY --from=backend-builder /app/target/release/migration /usr/local/bin

COPY --from=frontend-builder /app/.next/standalone /app/
COPY --from=frontend-builder /app/.next/static /app/.next/static
