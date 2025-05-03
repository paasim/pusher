.PHONY: check clean gen-keys run send-test

.env:
	cp deb/push-server.conf .env
	sed -i '/VAPID_PUBLIC_KEY/d' .env
	sed -i '/PUSH_TEST_ADDR/d' .env
	sed -i '/DATABASE_ENCRYPTION_KEY/d' .env
	sed -i 's/\/usr\/share\/pusher\///' .env
	cargo run --bin push-keygen >> .env
	grep VAPID_SUBJECT deb/push-send.conf >> .env

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

run: .env
	cargo run

send-test: .env
	echo "this is the body" | cargo run --bin push-send -- --title "test title"

.git/hooks/pre-commit:
	curl -o $@ https://gist.githubusercontent.com/paasim/317a1fd91a6236ca36d1c1c00c2a02d5/raw/315eb5b4e242684d64deb07a0c1597057af29f90/rust-pre-commit.sh
	echo "" >> $@
	chmod +x $@
