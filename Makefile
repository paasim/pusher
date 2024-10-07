ifneq (,$(wildcard ./.env))
	include .env
	export
endif

.PHONY: build clean dev gen-keys install send-test test

build:
	cargo build -r

clean:
	rm -rf .env
	rm -rf subscriptions.db
	rm -rf target

dev: .env
	cargo run

gen-keys:
	cargo run --bin push-keygen

.env:
	cp deb/push-server.conf .env
	sed -i '/VAPID_PUBLIC_KEY/d' .env
	sed -i '/PUSH_TEST_ADDR/d' .env
	sed -i '/DATABASE_ENCRYPTION_KEY/d' .env
	sed -i 's/\/usr\/share\/pusher\///' .env
	cargo run --bin push-keygen >> .env

install:
	cargo install --path .

send-test: .env
	echo "this is the body" | cargo run --bin push-send -- --title "test title"

test:
	cargo test
