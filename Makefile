default: run
run:
	echo 123abc | cargo run
build:
	cargo build
test:
	RUST_BACKTRACE=1 cargo test
test-nocapture:
	RUST_BACKTRACE=1 cargo test -- --nocapture
.phony: build run test

