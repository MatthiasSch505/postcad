# PostCAD Protocol Node
# Multi-stage build: compile Rust binaries, then copy into a minimal runtime image.

# ── Stage 1: Build ────────────────────────────────────────────────────────────
FROM rust:1.82-slim AS builder

WORKDIR /build

# Cache dependency compilation separately from application code.
# Copy workspace manifests first so the dependency layer is reused when only
# source files change.
COPY Cargo.toml Cargo.lock ./
COPY crates/core/Cargo.toml        crates/core/
COPY crates/registry/Cargo.toml    crates/registry/
COPY crates/compliance/Cargo.toml  crates/compliance/
COPY crates/routing/Cargo.toml     crates/routing/
COPY crates/audit/Cargo.toml       crates/audit/
COPY crates/cli/Cargo.toml         crates/cli/
COPY crates/service/Cargo.toml     crates/service/

# Create stub lib/main files so `cargo build` resolves dependencies without
# the full source tree.
RUN for dir in core registry compliance routing audit; do \
      mkdir -p crates/$dir/src && echo "// stub" > crates/$dir/src/lib.rs; \
    done && \
    mkdir -p crates/cli/src crates/cli/src/bin crates/service/src && \
    echo "// stub" > crates/cli/src/lib.rs && \
    echo "fn main() {}" > crates/cli/src/main.rs && \
    echo "fn main() {}" > crates/cli/src/bin/demo.rs && \
    echo "// stub" > crates/service/src/lib.rs && \
    echo "fn main() {}" > crates/service/src/main.rs

RUN cargo build --release --bin postcad-cli --bin postcad-service --bin demo \
    2>&1 | tail -5

# Now copy real source and rebuild only the application layer.
COPY crates/ crates/
COPY fixtures/ fixtures/
COPY tests/ tests/

RUN touch crates/*/src/*.rs crates/*/src/**/*.rs 2>/dev/null || true && \
    cargo build --release --bin postcad-cli --bin postcad-service --bin demo

# ── Stage 2: Runtime ──────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
      ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy compiled binaries.
COPY --from=builder /build/target/release/postcad-cli     /usr/local/bin/postcad-cli
COPY --from=builder /build/target/release/postcad-service /usr/local/bin/postcad-service
COPY --from=builder /build/target/release/demo            /usr/local/bin/postcad-demo

# Copy entrypoint.
COPY docker/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

# Default address. Override with POSTCAD_ADDR env var.
ENV POSTCAD_ADDR=0.0.0.0:8080
ENV RUST_LOG=info

EXPOSE 8080

ENTRYPOINT ["/entrypoint.sh"]
