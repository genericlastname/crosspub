all: build

build:
	cargo build --release

install: target/release/crosspub
	cp target/release/crosspub /usr/local/bin
	mkdir /usr/share/crosspub
	cp -r templates /usr/share/crosspub/templates

uninstall:
	rm /usr/local/bin/crosspub
	rm -r /usr/share/crosspub
