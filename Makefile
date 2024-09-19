default: build
build:
	cargo build
test:
	RUST_BACKTRACE=1 cargo test
test-nocapture:
	RUST_BACKTRACE=1 cargo test -- --nocapture
release:
	cargo build --release
clean:
	rm -rf target
install: release
	sudo cp ./target/release/tscalc /usr/local/bin
.phony: build run test release

