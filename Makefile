ifneq (,$(wildcard ./.env))
	include .env
	export
endif

.PHONY: build clean dev gen-keys init-db install migrate send-test test

build:
	cargo build -r

clean:
	rm -rf .env
	rm -rf subscriptions.db
	rm -rf target

dev: .env subscriptions.db
	make init-db
	cargo run

gen-keys:
	cargo run --bin gen-keys

.env:
	cp deb/push-server.conf .env
	sed -i /VAPID_PUBLIC_KEY/d .env
	sed -i /DATABASE_ENCRYPTION_KEY/d .env
	cargo run --bin push-keygen >> .env

init-db: subscriptions.db
	make migrate
	cargo sqlx prepare

install:
	cargo install --path .

migrate:
	sqlx migrate run

send-test: .env subscriptions.db
	cargo run --bin send -- --title "test title" --body "this is the body"

subscriptions.db:
	sqlx database create

test:
	cargo test
