# pusher

[![build](https://github.com/paasim/pusher/workflows/build/badge.svg)](https://github.com/paasim/pusher/actions)

A server listening for web push subscriptions and a [sending encrypted messages](https://datatracker.ietf.org/doc/html/rfc8291) as push notifictaions.

## install

The [release builds](https://github.com/paasim/pusher/releases) [link to system `openssl`](https://docs.rs/openssl/latest/openssl/), so version mismatches might occur.

```bash
# this builds the binaries under target/release
cargo build -r

cargo install --path .
```

## structure

The repository consists of three different binaries:

### keygen

Generates [`VAPID` keys](https://datatracker.ietf.org/doc/html/rfc8292) and salt used for encrypting client authentication secret. Usage:

```bash
./push-keygen > .env
# or without installing first
make gen-keys
```

### push-server

A simple http-server that allows clients to register for push-notifications. The registrations are stored in `sqlite` database. The server also expects the following environment variables to be defined:
* `VAPID_PUBLIC_KEY`, public part of the VAPID key
* `DATABASE_ENCRYPTION_KEY`: Used for encrypting client authentication secret.
* `DATABASE_PATH`: location of the `sqlite`-database.
* `PORT`: port the server listens to.
* `PUSH_SOCKET_ADDR`: **optional** socket path (see [push-send](#push-send)) where test messages are sent to.

These can also be automatically generaterated with `make .env` (subject will be incorrect, however). In addition, the server also needs `static` and `migrations` to exist to run. Usage:

```bash
./push-server
# or
make run
```

The prerequisites are also auto-generated and the server is run with with `make run`.

### push-send

An utility to send push messages. Expects the following environment variables to be defined:
* `VAPID_PUBLIC_KEY`, `VAPID_PRIVATE_KEY`, `VAPID_SUBJECT`: for server authentication.
* `DATABASE_ENCRYPTION_KEY`: Used for decrypting client authentication secret.
* `DATABASE_PATH`: location of the `sqlite`-database.

The utility supports two modes, sending one time message (which is read from stdin)

```bash
make send-test
```
and a server mode, listening to messages from a unix socket. In this case
`PUSH_SOCKET_ADDR` - path to the socket - should also be set and match to the one
set for `push-server`. This enables the test-button in the web app.

```bash
make send-socket
```

See `deb/push-sender.service` and `man push-send` for details.


### other

* `arm-cross`: debian-image for cross-compiling form `arm64`.
* `assets`: all the client code (`script.js`, `sw.js`).
* `migrations`: migrations along with a script to run them (`migrate.sh`).
* `src`: all the functionality that is common among the binaries.
