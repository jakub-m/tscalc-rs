default: run
run:
	echo "1h + 2h + 2000-01-01T00:00:00Z" | cargo run
	# echo "1h + 2h + now" | cargo run
build:
	cargo build
test:
	RUST_BACKTRACE=1 cargo test
test-nocapture:
	RUST_BACKTRACE=1 cargo test -- --nocapture
release:
	cargo build --release
.phony: build run test release

