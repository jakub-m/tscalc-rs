default: run
run:
	echo 123ą | cargo run
build:
	cargo build
test:
	RUST_BACKTRACE=1 cargo test
.phony: build run test

