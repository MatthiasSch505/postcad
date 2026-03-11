# PostCAD Service — Local Run

---

## Prerequisites

- Docker 20.10+ (preferred) **or** Rust toolchain with `cargo`
- `curl`

---

## Build

```bash
docker build -t postcad-node -f docker/Dockerfile .
```

Or via Make:

```bash
make docker-build
```

---

## Run

```bash
docker compose up
```

Or via Make:

```bash
make docker-compose-up
```

The service binds to `0.0.0.0:8080` by default. Set `POSTCAD_ADDR` to override:

```bash
POSTCAD_ADDR=127.0.0.1:9000 docker compose up
```

The `examples/pilot/` directory is mounted read-only at `/data/pilot` inside the container.

---

## Health check

```bash
curl -s http://localhost:8080/health
```

Expected:

```json
{"status":"ok"}
```

---

## Version check

```bash
curl -s http://localhost:8080/version
```

Expected:

```json
{"protocol_version":"postcad-v1","routing_kernel_version":"postcad-routing-v1","service":"postcad-service"}
```

---

## Route — pilot fixture

```bash
curl -s -X POST http://localhost:8080/route \
  -H 'Content-Type: application/json' \
  -d "{
    \"case\":             $(cat examples/pilot/case.json),
    \"registry_snapshot\": $(cat examples/pilot/registry_snapshot.json),
    \"routing_config\":   $(cat examples/pilot/config.json)
  }"
```

Expected response contains:

```json
{
  "receipt": {
    "outcome": "routed",
    "selected_candidate_id": "pilot-de-001",
    "receipt_hash": "0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb"
  },
  "derived_policy": { ... }
}
```

The full locked receipt is in `examples/pilot/expected_routed.json`.

---

## Verify — pilot fixture

```bash
curl -s -X POST http://localhost:8080/verify \
  -H 'Content-Type: application/json' \
  -d "{
    \"receipt\": $(cat examples/pilot/expected_routed.json),
    \"case\":    $(cat examples/pilot/case.json),
    \"policy\":  $(cat examples/pilot/derived_policy.json)
  }"
```

Expected:

```json
{"result":"VERIFIED"}
```

---

## Case intake

Store a case:

```bash
curl -s -X POST http://localhost:8080/cases \
  -H 'Content-Type: application/json' \
  -d "$(cat examples/pilot/case.json)"
```

Expected (first store):

```json
{"case_id":"f1000001-0000-0000-0000-000000000001","stored":true}
```

List stored cases:

```bash
curl -s http://localhost:8080/cases
```

Expected:

```json
{"case_ids":["f1000001-0000-0000-0000-000000000001"]}
```

Retrieve a case by ID:

```bash
curl -s http://localhost:8080/cases/f1000001-0000-0000-0000-000000000001
```

Expected: the original case JSON object.

Cases are stored under `data/cases/{case_id}.json` relative to the working directory (or `/data/cases/` inside the container).

---

## Dispatch

```bash
curl -s -X POST http://localhost:8080/dispatch/<receipt_hash>
```

Expected:

```json
{"dispatched":true,"receipt_hash":"<receipt_hash>"}
```

---

## Dispatch verification

```bash
curl -s -X POST http://localhost:8080/dispatch/<receipt_hash>/verify
```

Expected:

```json
{"receipt_hash":"<receipt_hash>","result":"VERIFIED"}
```

---

## Stop

```bash
docker compose down
```

---

## Without Docker

Build and run the service binary directly:

```bash
cargo build --bin postcad-service
./target/debug/postcad-service
```

The same `curl` commands above apply against `http://localhost:8080`.

---

## Smoke test (in-process, no Docker required)

```bash
cargo test -p postcad-service --test pilot_bundle_smoke_test
cargo test -p postcad-service --test pilot_http_tests
```

These tests run in-process against the service router without binding a port. They cover health, version, route (value-for-value against `expected_routed.json`), and verify (value-for-value against `expected_verify.json`).
