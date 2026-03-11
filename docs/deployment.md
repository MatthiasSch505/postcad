# PostCAD Protocol Node — Deployment Guide

The PostCAD Protocol Node is a stateless HTTP service that exposes the PostCAD routing kernel. It is distributed as a single Docker image.

---

## Quick Start

### Build the image

```bash
make docker-build
# or: docker build -t postcad-node -f docker/Dockerfile .
```

### Run a single container

```bash
make docker-run
# or: docker run --rm -p 8080:8080 postcad-node
```

The node starts on port 8080. On startup it prints the protocol version summary.

### Run with docker compose

```bash
make docker-compose-up
# or: docker compose up
```

The compose file mounts `examples/pilot/` at `/data/pilot` inside the container, so the pilot input files are accessible if needed.

---

## Configuration

All configuration is via environment variables.

| Variable | Default | Description |
|----------|---------|-------------|
| `POSTCAD_ADDR` | `0.0.0.0:8080` | Bind address for the HTTP service. |
| `RUST_LOG` | `info` | Log level (`trace`, `debug`, `info`, `warn`, `error`). |

Copy `.env.example` to `.env` and adjust:

```bash
cp .env.example .env
docker run --rm -p 8080:8080 --env-file .env postcad-node
```

To bind on a different port:

```bash
docker run --rm -p 9090:9090 -e POSTCAD_ADDR=0.0.0.0:9090 postcad-node
```

---

## Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `GET`  | `/health` | Liveness check. Returns `{"status":"ok"}`. |
| `GET`  | `/version` | Service and kernel version strings. |
| `POST` | `/route` | Route a case from registry snapshot + config (pilot path). |
| `POST` | `/verify` | Replay-verify a routing receipt (pilot path). |
| `POST` | `/route-case` | Route a case from a policy bundle. |
| `POST` | `/route-case-from-registry` | Route a case from a registry snapshot + config. |
| `POST` | `/verify-receipt` | Replay-verify a routing receipt. |
| `GET`  | `/protocol-manifest` | Return the protocol manifest (versions, schema hashes). |

### Health

```bash
curl -s http://localhost:8080/health
```

Response: `{"status":"ok"}`

### Version

```bash
curl -s http://localhost:8080/version
```

Response:

```json
{"protocol_version":"postcad-v1","routing_kernel_version":"postcad-routing-v1","service":"postcad-service"}
```

### Route a case — pilot path

```bash
curl -s -X POST http://localhost:8080/route \
  -H "Content-Type: application/json" \
  -d '{
    "case":             <case.json contents>,
    "registry_snapshot": <registry_snapshot.json contents>,
    "routing_config":   <config.json contents>
  }'
```

Response:

```json
{
  "receipt": { ... },
  "derived_policy": { ... }
}
```

### Verify a receipt — pilot path

```bash
curl -s -X POST http://localhost:8080/verify \
  -H "Content-Type: application/json" \
  -d '{
    "receipt": <receipt.json contents>,
    "case":    <case.json contents>,
    "policy":  <derived_policy.json contents>
  }'
```

Response on success: `{"result":"VERIFIED"}`

### Route a case — registry-backed

```bash
curl -s -X POST http://localhost:8080/route-case-from-registry \
  -H "Content-Type: application/json" \
  -d '{
    "case":     <case.json contents>,
    "registry": <registry_snapshot.json contents>,
    "config":   <config.json contents>
  }'
```

Response:

```json
{
  "receipt": { ... },
  "derived_policy": { ... }
}
```

### Verify a receipt

```bash
curl -s -X POST http://localhost:8080/verify-receipt \
  -H "Content-Type: application/json" \
  -d '{
    "receipt": <receipt.json contents>,
    "case":    <case.json contents>,
    "policy":  <derived_policy.json contents>
  }'
```

Response on success:

```json
{ "result": "VERIFIED" }
```

Response on failure (HTTP 422):

```json
{ "result": "FAILED", "error": { "code": "registry_snapshot_hash_mismatch", "message": "..." } }
```

### Protocol manifest

```bash
curl -s http://localhost:8080/protocol-manifest
```

---

## Startup Output

On startup the node prints the protocol version summary:

```
============================================
 PostCAD Protocol Node
============================================
{"protocol_version":"postcad-v1","protocol_semver":"1.0","routing_kernel":"postcad-routing-v1","routing_kernel_semver":"1.0","receipt_schema_hash":"37a025b6...","proof_schema_hash":"ebd5a82f...","refusal_code_set_hash":"4ebf952c...","manifest_fingerprint":"a46b5519..."}
--------------------------------------------
Starting service on 0.0.0.0:8080...
```

---

## Pilot Demo (inside the container)

The `postcad-demo` binary is included in the image. Run it to execute the full protocol demo:

```bash
docker run --rm postcad-node postcad-demo
```

Expected output includes the selected candidate (`mfr-de-001`) and `Verification: OK`.

---

## Image Contents

| Path | Description |
|------|-------------|
| `/usr/local/bin/postcad-cli` | CLI with all subcommands. |
| `/usr/local/bin/postcad-service` | HTTP service binary (the default entrypoint process). |
| `/usr/local/bin/postcad-demo` | Standalone demo binary. |
| `/entrypoint.sh` | Startup script (prints info, execs service). |

---

## Versioning

The image encodes a fixed protocol version. Routing receipts emitted by this node carry:

- `protocol_version: "postcad-v1"`
- `routing_kernel_version: "postcad-routing-v1"`

Receipts verified against a node running a different protocol version will fail with `protocol_version_mismatch`.
