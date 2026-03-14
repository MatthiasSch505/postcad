# PostCAD Case Input Format

A PostCAD case input file is a JSON document that describes the dental case to be routed.

## Fields

| Field | Type | Required | Description |
|---|---|---|---|
| `case_id` | string (UUID) | yes | Unique identifier for this case |
| `jurisdiction` | string | yes | Two-letter country code for routing jurisdiction (e.g. `"DE"`, `"US"`, `"JP"`) |
| `routing_policy` | string | yes | Routing policy name (e.g. `"allow_domestic_and_cross_border"`) |
| `patient_country` | string | yes | Patient country (lowercase, e.g. `"germany"`) |
| `manufacturer_country` | string | yes | Preferred manufacturer country (lowercase) |
| `material` | string | yes | Material type (e.g. `"zirconia"`, `"pmma"`) |
| `procedure` | string | yes | Procedure type (e.g. `"crown"`, `"bridge"`, `"implant"`) |
| `file_type` | string | yes | CAD file type (e.g. `"stl"`, `"3mf"`) |

## Example

```json
{
  "case_id": "f1000001-0000-0000-0000-000000000001",
  "jurisdiction": "DE",
  "routing_policy": "allow_domestic_and_cross_border",
  "patient_country": "germany",
  "manufacturer_country": "germany",
  "material": "zirconia",
  "procedure": "crown",
  "file_type": "stl"
}
```

This is the canonical pilot case used for deterministic routing tests. The routing result for this input is always `pilot-de-001` with receipt hash `0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb`.

## Running a case

```bash
./examples/pilot/run_case.sh <case_file.json>
```

The service must be running on `http://localhost:8080` before executing the script.

## Notes

- `case_id` must be a valid UUID.
- `jurisdiction` drives which compliance rules apply (EU MDR, FDA 510k, MHLW).
- `routing_policy` must match a policy known to the registry snapshot.
- All field values are case-sensitive.
