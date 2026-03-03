.PHONY: build install release clean

build:
	cargo build --release

install: build
	cp target/release/termimon /usr/local/bin/termimon

release:
	@if [ -z "$(VERSION)" ]; then echo "Usage: make release VERSION=0.1.0"; exit 1; fi
	git tag -a v$(VERSION) -m "v$(VERSION)"
	git push origin v$(VERSION)

clean:
	cargo clean
