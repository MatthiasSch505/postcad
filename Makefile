.PHONY: build test lint fmt pilot demo protocol-info clean docker-build docker-run docker-compose-up dev

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

## Build the Docker image for the protocol node.
docker-build:
	docker build -t postcad-node -f docker/Dockerfile .

## Run the protocol node container (ephemeral).
docker-run:
	docker run --rm -p 8080:8080 -e RUST_LOG=info postcad-node

## Start the protocol node via docker compose.
docker-compose-up:
	docker compose up

## Local dev: build + run all tests.
dev:
	cargo build
	cargo test --workspace
