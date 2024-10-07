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

These can also be automatically generaterated with `make .env` (subject will be incorrect, however). In addition, the server also needs `static` and `migrations` to exist to run. Usage:

```bash
./push-server
# or
make dev
```

The prerequisites are also auto-generated and the server is run with with `make dev`.

### push-send

An utility to send push messages. Expects the following environment variables to be defined:
* `VAPID_PUBLIC_KEY`, `VAPID_PRIVATE_KEY`, `VAPID_SUBJECT`: for server authentication.
* `DATABASE_ENCRYPTION_KEY`: Used for decrypting client authentication secret.
* `DATABASE_PATH`: location of the `sqlite`-database.

Currenly this simply sends the same messages to all the subscribed clients. Usage:

```bash
echo "this is the body" | ./push-send --title "the title"
# or
make send-test
```

#### push-test

The server also has a test-button that triggers a test message. The `.deb`-package installs a systemd socket unit, that listens for a message and calls `push-send`.

The socket address can be specified in the `PUSH_TEST_ADDR`-environment variable (for the server), and if let unset, a message is not triggered. Note, that for development purposes the aforementioned socket unit most likely does not exist and therefore `push-send` needs to be called manually.

See `deb/push-test@.service` and `deb/push-test.socket` for details.


### other

* `src`: all the functionality that is common among the binaries.
* `assets`: all the client code (`script.js`, `sw.js`).
* `arm-cross`: debian-image for cross-compiling form `arm64`.
