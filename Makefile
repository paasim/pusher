.PHONY: check clean gen-keys migrate run send-socket send-test

.env:
	cp deb/push-server.conf .env
	sed -i '/VAPID_PUBLIC_KEY/d' .env
	sed -i '/PUSH_SOCKET_ADDR/d' .env
	sed -i '/DATABASE_ENCRYPTION_KEY/d' .env
	sed -i 's/\/usr\/share\/pusher\///' .env
	cargo run --bin push-keygen >> .env
	grep VAPID_SUBJECT deb/push-send.conf >> .env
	echo "PUSH_SOCKET_ADDR=push-test-socket" >> .env

include .env
export

check: .git/hooks/pre-commit
	. $<

clean:
	rm -rf .env
	rm -rf subscriptions.db
	rm -rf target

gen-keys:
	cargo run --bin push-keygen

migrate: migrations/migrate.sh .env
	. $< $(DATABASE_PATH)

run: .env migrate
	cargo run

send-socket: .env
	cargo run --bin push-send -- --server "push-from-socket"

send-test: .env
	echo -e "this is the body\nand so on and so on" \
		| cargo run --bin push-send -- "push-from-stdin"

.git/hooks/pre-commit:
	curl -o $@ https://gist.githubusercontent.com/paasim/317a1fd91a6236ca36d1c1c00c2a02d5/raw/315eb5b4e242684d64deb07a0c1597057af29f90/rust-pre-commit.sh
	echo "" >> $@
	chmod +x $@
