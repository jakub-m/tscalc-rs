default: build
build:
	cargo build
test:
	RUST_BACKTRACE=1 cargo test
test-nocapture:
	RUST_BACKTRACE=1 cargo test -- --nocapture
release:
	cargo build --release
.phony: build run test release

