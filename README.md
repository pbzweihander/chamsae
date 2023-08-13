# chamsae

**chamsae** is a lightweight ActivityPub server for a single user.
chamsae is for a single user who wants to join the fediverse, but not want to run a massive software.
[_chamsae_ means an Eurasian tree sparrow.](https://en.wikipedia.org/wiki/Eurasian_tree_sparrow)

## Features

- ~~Mastodon or Misskey like microblogging service~~ TODO
- ~~Misskey like emoji reactions~~ TODO

## Usage

> **NOTE**
> You have to serve chamsae with HTTPS.
> This is not optional.
> For development, [Caddy](https://caddyserver.com/) is helpful.

### Requirements

- PostgreSQL

### Backend

#### Example: Using local filesystem as object store

```
DEBUG=true \
  DOMAIN=localhost \
  USER_HANDLE=admin \
  USER_PASSWORD_BCRYPT={your_password_hash} \
  DATABASE_HOST={host} \
  DATABASE_PORT={port} \
  DATABASE_USER={user} \
  DATABASE_PASSWORD={password} \
  DATABASE_DATABASE={db} \
  OBJECT_STORE_TYPE=local_filesystem \
  cargo run --bin chamsae
```

#### Example: Using CloudFlare R2 as object store

```
DEBUG=true \
  DOMAIN=localhost \
  USER_HANDLE=admin \
  USER_PASSWORD_BCRYPT={your_password_hash} \
  DATABASE_HOST={host} \
  DATABASE_PORT={port} \
  DATABASE_USER={user} \
  DATABASE_PASSWORD={password} \
  DATABASE_DATABASE={db} \
  OBJECT_STORE_TYPE=s3 \
  AWS_DEFAULT_REGION=auto \
  AWS_ENDPOINT=https://{account_id}.r2.cloudflarestorage.com \
  AWS_ACCESS_KEY_ID={access_key} \
  AWS_SECRET_ACCESS_KEY={secret_key} \
  cargo run --bin chamsae
```

#### Serve HTTPS with caddy

To serve HTTPS with caddy:

```
caddy reverse-proxy --to :3000
```

You may need to run `sudo setcap cap_net_bind_service=+ep $(which caddy)` first.

### Frontend

TODO

## License

_chamsae_ is licensed under the terms of [AGPL v3.0](https://www.gnu.org/licenses/agpl-3.0.html).
See [LICENSE file](./LICENSE) for details.
