all: build

build:
	cargo build --release

install: target/release/crosspub
	cp target/release/crosspub /usr/local/bin
	mkdir -p /usr/share/crosspub
	cp -r templates /usr/share/crosspub/
	cp config.toml /usr/share/crosspub/

uninstall:
	rm /usr/local/bin/crosspub
	rm -r /usr/share/crosspub
