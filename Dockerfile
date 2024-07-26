# syntax = docker/dockerfile:1

FROM rust:1.80-slim AS rustbase
WORKDIR /app


FROM node:20-bookworm-slim AS nodebase
WORKDIR /app


FROM nodebase AS frontend-builder
WORKDIR /app
COPY package.json ./package.json
COPY frontend/package.json ./frontend/package.json
COPY yarn.lock ./yarn.lock
RUN yarn install --frozen-lockfile
COPY frontend ./
RUN yarn build


FROM rustbase AS backend-builder
COPY Cargo.lock .
COPY Cargo.toml .
COPY migration migration
COPY backend backend
COPY --from=frontend-builder /app/dist /app/frontend/dist
RUN cargo build --release


FROM debian:bookworm-slim AS runtime
RUN addgroup --system --gid 1001 chamsae
RUN adduser --system --uid 1001 chamsae
USER chamsae
COPY --from=backend-builder /app/target/release/chamsae /usr/local/bin
COPY --from=backend-builder /app/target/release/migration /usr/local/bin
CMD ["chamsae"]
