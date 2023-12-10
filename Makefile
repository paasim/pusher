include .env
export

.PHONY: build clean dev gen-keys init-db install migrate send-test test

build:
	cargo build -r

clean:
	rm -rf .env
	rm -rf subscriptions.db

dev: .env subscriptions.db
	make init-db
	cargo run

gen-keys:
	cargo run -r --bin gen-keys

.env:
	cargo run -r --bin gen-keys > .env
	echo 'VAPID_SUBJECT=mailto:pusher-test@test.pusher' >> .env
	echo 'DATABASE_URL=sqlite:subscriptions.db' >> .env
	echo 'PORT=3000' >> .env

init-db: subscriptions.db
	make migrate
	cargo sqlx prepare

install:
	cargo install --path .

migrate:
	sqlx migrate run

send-test: .env subscriptions.db
	cargo run -r --bin send -- --title "test title" --body "this is the body"

subscriptions.db:
	sqlx database create

test:
	cargo test -r
