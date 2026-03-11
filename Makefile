.PHONY: build test lint fmt pilot demo protocol-info clean

build:
	cargo build

test:
	cargo test --workspace

lint:
	cargo clippy --all-targets --all-features

fmt:
	cargo fmt

## Run the pilot workflow: registry-backed routing + self-verification.
pilot:
	examples/pilot/run_pilot.sh

## Run the end-to-end demo (route + explicit verify using frozen fixtures).
demo:
	cargo run --bin postcad-cli -- demo-run

## Print the compact protocol version summary.
protocol-info:
	cargo run --bin postcad-cli -- protocol-info

clean:
	cargo clean
	rm -f examples/pilot/receipt.json
