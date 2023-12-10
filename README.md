# pusher

[![build](https://github.com/paasim/pusher/workflows/build/badge.svg)](https://github.com/paasim/pusher/actions)

A server listening for web push subscriptions and a [sending encrypted messages](https://datatracker.ietf.org/doc/html/rfc8291) as push notifictaions.

## install

The [release builds](https://github.com/paasim/pusher/releases) [link to system `openssl`](https://docs.rs/openssl/latest/openssl/), so version mismatches might occur.

```bash
make install
# this builds the binaries under target/release
make build -r
```

## structure

The repository consists of three different binaries:

### gen-keys

Generates [`VAPID`-keys](https://datatracker.ietf.org/doc/html/rfc8292) and salt used for encrypting client authentication secret. Usage:

```bash
./gen-keys > .env
# or without installing first
make gen-keys
```

### server

A simple http-server that allows clients to register for push-notifications. The registrations are stored in `sqlite` database. The server also expects the following environment variables to be defined:
* `VAPID_PUBLIC_KEY`, for server authentication.
* `DATABASE_ENCRYPTION_KEY`: Used for encrypting client authentication secret.
* `DATABASE_URL`: location of the `sqlite`-database.
* `PORT`: port the server listens to.

These can also be automatically generaterated with `make .env` (subject will be incorrect, however). In addition, the server also needs `static` and `migrations` to exist to run. Usage:

```bash
./server
# or without installing first, requires sqlx-cli
make dev
```

The prerequisites are also auto-generated and the server is run with with `make dev`, but this requires [`sqlx-cli`](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md).

### send

An utility to send push messages. Expects the following environment variables to be defined:
* `VAPID_PUBLIC_KEY`, `VAPID_PRIVATE_KEY`, `VAPID_SUBJECT`: for server authentication.
* `DATABASE_ENCRYPTION_KEY`: Used for decrypting client authentication secret.
* `DATABASE_URL`: location of the `sqlite`-database.

Currenly this simply sends the same messages to all the subscribed clients. Usage:

```bash
./send --title "the title" --body "this will be the body"
# or without installing first, requires sqlx-cli
make send-test
```

### other

* `src`: all the functionality that is common among the binaries.
* `assets`: all the client code (`script.js`, `sw.js`).
* `arm-cross`: debian-image for cross-compiling form `arm64`.
