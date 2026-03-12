# ── Build stage ───────────────────────────────────────────────────────────────
FROM rust:1.85-slim AS builder

WORKDIR /build

# Cache dependency compilation by copying manifests first
COPY Cargo.toml Cargo.lock ./
COPY crates/core/Cargo.toml        crates/core/Cargo.toml
COPY crates/registry/Cargo.toml    crates/registry/Cargo.toml
COPY crates/compliance/Cargo.toml  crates/compliance/Cargo.toml
COPY crates/routing/Cargo.toml     crates/routing/Cargo.toml
COPY crates/audit/Cargo.toml       crates/audit/Cargo.toml
COPY crates/cli/Cargo.toml         crates/cli/Cargo.toml
COPY crates/service/Cargo.toml     crates/service/Cargo.toml

# Stub sources so Cargo can resolve the workspace without full source
RUN mkdir -p crates/core/src crates/registry/src crates/compliance/src \
             crates/routing/src crates/audit/src crates/cli/src crates/service/src && \
    for d in core registry compliance routing audit; do \
      echo "pub fn _stub() {}" > crates/$d/src/lib.rs; \
    done && \
    echo "fn main() {}" > crates/cli/src/main.rs && \
    echo "pub fn _stub() {}" > crates/cli/src/lib.rs && \
    echo "fn main() {}" > crates/service/src/main.rs && \
    echo "pub fn _stub() {}" > crates/service/src/lib.rs

RUN cargo build --release -p postcad-service 2>/dev/null || true

# Now copy real source and build for real
COPY crates/ crates/
RUN touch crates/*/src/*.rs && \
    cargo build --release -p postcad-service

# ── Runtime stage ─────────────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /build/target/release/postcad-service /app/postcad-service
COPY examples/pilot/ /app/examples/pilot/

EXPOSE 8080

CMD ["/app/postcad-service"]
