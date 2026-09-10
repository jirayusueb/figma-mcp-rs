.PHONY: build build-rs build-ts test test-rs test-ts clean release

build: build-rs build-ts

build-rs:
	cargo build --release

build-ts:
	cd plugin && bun install && bun run build

test: test-rs test-ts

test-rs:
	cargo test

test-ts:
	cd plugin && bun install && bun test

clean:
	cargo clean
	rm -rf plugin/dist plugin/node_modules

release: build
	@echo "Release binary: target/release/figma-mcp-rs"
