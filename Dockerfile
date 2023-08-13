FROM rust:1.71.1-bookworm AS rustbase
ENV HOME=/home/root
WORKDIR $HOME/app

FROM node:20-bookworm-slim AS nodebase
WORKDIR /app


FROM rustbase AS builder
COPY Cargo.lock .
COPY Cargo.toml .
COPY migration migration
COPY backend backend
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,sharing=private,target=/home/root/app/target \
    cargo install --path ./backend && cargo install --path ./migration


FROM nodebase AS febuilder
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

COPY --from=builder /usr/local/cargo/bin/chamsae /usr/local/bin
COPY --from=builder /usr/local/cargo/bin/migration /usr/local/bin

COPY --from=febuilder /app/.next/standalone /app/
COPY --from=febuilder /app/.next/static /app/.next/static
