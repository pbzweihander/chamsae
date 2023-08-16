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

You can also use `.env` file.

#### Example: Using local filesystem as object store

```
DEBUG=true \
  DOMAIN=localhost \
  DATABASE_URL=postgresql://postgres:postgres@localhost:5432 \
  OBJECT_STORE_TYPE=local_filesystem \
  OBJECT_STORE_LOCAL_FILE_BASE_PATH=./files/ \
  cargo run --bin chamsae
```

#### Example: Using CloudFlare R2 as object store

```
DEBUG=true \
  DOMAIN=localhost \
  DATABASE_URL=postgresql://postgres:postgres@localhost:5432 \
  OBJECT_STORE_TYPE=s3 \
  OBJECT_STORE_BUCKET=bucket \
  OBJECT_STORE_PUBLIC_URL_BASE=https://example.com/bucket \
  AWS_DEFAULT_REGION=auto \
  AWS_ENDPOINT=https://{account_id}.r2.cloudflarestorage.com \
  AWS_ACCESS_KEY_ID={access_key} \
  AWS_SECRET_ACCESS_KEY={secret_key} \
  cargo run --bin chamsae
```

#### Initialize instance

```
xh -v POST :3000/api/setting/initial \
  instanceName=my-instance \
  userHandle=admin \
  userPassword={your_password}
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
