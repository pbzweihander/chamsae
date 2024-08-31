# chamsae

**chamsae** is a lightweight ActivityPub server for a single user.
chamsae is for a single user who wants to join the fediverse, but not want to run a massive software.
[_chamsae_ means an Eurasian tree sparrow.](https://en.wikipedia.org/wiki/Eurasian_tree_sparrow)

## How to pronounce chamsae?

In Korean, it is written as '참새'. pronounced `/chɑm-sæ/`, like "charm-sae"

## Features

- ~~Mastodon or Misskey like microblogging service~~ TODO
- ~~Misskey like emoji reactions~~ TODO

## Usage

> [!NOTE]
> You have to serve chamsae with HTTPS.
> This is not optional.
> For development, [Caddy](https://caddyserver.com/) is helpful.

### Requirements

- PostgreSQL

### Backend

First, due to static-serving, you have to build frontend.

```shell
yarn build
```

For the environment variables, you can use `.env` file.

```shell
DEBUG=true \
  PUBLIC_DOMAIN=localhost \
  DATABASE_URL=postgresql://postgres:postgres@localhost:5432 \
  cargo run --bin chamsae
```

#### Serve HTTPS with caddy for debugging

```shell
caddy reverse-proxy --to :3000
```

You may need to run `sudo setcap cap_net_bind_service=+ep $(which caddy)` first.

### Frontend

```
yarn dev
```

### Initialize instance

Open `http://localhost:5173` with your browser.
You can now initialize instance.

## License

_chamsae_ is licensed under the terms of [AGPL v3.0](https://www.gnu.org/licenses/agpl-3.0.html).
See [LICENSE file](./LICENSE) for details.
