# this_file: Makefile
.PHONY: all build build-python test examples docs clean
all: build
build:
	cargo build --release -p uubed-core --features capi
build-python:
	uvx maturin build --release
test:
	cargo test --workspace --features capi
examples: build
	$(CC) -Wall -Wextra -Iinclude examples/c_api_demo.c target/release/libuubed_core.a -lpthread -ldl -lm -o target/c_api_demo
	./target/c_api_demo
docs:
	cargo doc --no-deps -p uubed-core -p uubed-tm
clean:
	cargo clean
