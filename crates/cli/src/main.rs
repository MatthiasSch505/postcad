use std::fs;
use std::process;

use postcad_cli::{
    build_manifest, export_registry, normalize_pilot_case_json, route_case_from_json,
    route_case_from_registry_json, verify_receipt_from_inputs, verify_receipt_from_policy_json,
    POSTCAD_PROTOCOL_VERSION, PROTOCOL_VERSION, ROUTING_KERNEL_SEMVER,
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    // Pre-scan for --json so top-level error paths can emit the envelope.
    let json_output = args.iter().any(|a| a.as_str() == "--json");

    match args.get(1).map(String::as_str) {
        Some("route-case") => run_route_case(&args[2..]),
        Some("route-case-from-registry") => run_route_case_from_registry(&args[2..]),
        Some("registry-export") => run_registry_export(&args[2..]),
        Some("verify-receipt") => run_verify_receipt(&args[2..]),
        Some("protocol-manifest") => run_protocol_manifest(),
        Some("protocol-info") => run_protocol_info(),
        Some("demo-run") | Some("demo") => run_demo_v1(&args[2..]),
        Some("pilot-route-normalized") => run_pilot_route_normalized(&args[2..]),
        Some("--help") | Some("-h") | Some("help") => print_help(),
        Some(other) => emit_error_and_exit(
            json_output,
            "invalid_arguments",
            &format!("unknown subcommand '{}'", other),
        ),
        None => emit_error_and_exit(
            json_output,
            "invalid_arguments",
            "no subcommand provided; run with 'help' for usage",
        ),
    }
}

fn run_route_case(args: &[String]) {
    // Pre-scan for --json before touching any other flag so all error paths
    // in this function can emit the JSON envelope.
    let json_output = args.iter().any(|a| a.as_str() == "--json");

    let mut case_path: Option<&str> = None;
    let mut candidates_path: Option<&str> = None;
    let mut snapshot_path: Option<&str> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--case" => {
                case_path = args.get(i + 1).map(String::as_str);
                i += 2;
            }
            "--candidates" => {
                candidates_path = args.get(i + 1).map(String::as_str);
                i += 2;
            }
            "--snapshot" => {
                snapshot_path = args.get(i + 1).map(String::as_str);
                i += 2;
            }
            "--json" => {
                i += 1;
            }
            other => emit_error_and_exit(
                json_output,
                "invalid_arguments",
                &format!("unknown flag '{}'", other),
            ),
        }
    }

    let case_path = case_path.unwrap_or_else(|| {
        emit_error_and_exit(
            json_output,
            "invalid_arguments",
            "missing required argument --case",
        )
    });
    let candidates_path = candidates_path.unwrap_or_else(|| {
        emit_error_and_exit(
            json_output,
            "invalid_arguments",
            "missing required argument --candidates",
        )
    });
    let snapshot_path = snapshot_path.unwrap_or_else(|| {
        emit_error_and_exit(
            json_output,
            "invalid_arguments",
            "missing required argument --snapshot",
        )
    });

    let case_json = read_file_or_exit(json_output, case_path);
    let candidates_json = read_file_or_exit(json_output, candidates_path);
    let snapshots_json = read_file_or_exit(json_output, snapshot_path);

    let receipt = match route_case_from_json(&case_json, &candidates_json, &snapshots_json) {
        Ok(r) => r,
        Err(e) => emit_error_and_exit(json_output, e.code(), &e.to_string()),
    };

    if json_output {
        println!("{}", serde_json::to_string_pretty(&receipt).unwrap());
    } else {
        println!("outcome:              {}", receipt.outcome);
        println!(
            "selected_candidate:   {}",
            receipt
                .selected_candidate_id
                .as_deref()
                .unwrap_or("\u{2014}")
        );
        println!(
            "refusal_code:         {}",
            receipt.refusal_code.as_deref().unwrap_or("\u{2014}")
        );
        println!("routing_proof_hash:   {}", receipt.routing_proof_hash);
        println!("policy_fingerprint:   {}", receipt.policy_fingerprint);
        println!("case_fingerprint:     {}", receipt.case_fingerprint);
        println!("audit_seq:            {}", receipt.audit_seq);
        println!("audit_entry_hash:     {}", receipt.audit_entry_hash);
        println!("audit_previous_hash:  {}", receipt.audit_previous_hash);
        if let Some(detail) = &receipt.refusal {
            println!("refusal_message:      {}", detail.message);
            println!("failed_constraint:    {}", detail.failed_constraint);
        }
    }
}

fn run_route_case_from_registry(args: &[String]) {
    let json_output = args.iter().any(|a| a.as_str() == "--json");

    let mut case_path: Option<&str> = None;
    let mut registry_path: Option<&str> = None;
    let mut config_path: Option<&str> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--case" => {
                case_path = args.get(i + 1).map(String::as_str);
                i += 2;
            }
            "--registry" => {
                registry_path = args.get(i + 1).map(String::as_str);
                i += 2;
            }
            "--config" => {
                config_path = args.get(i + 1).map(String::as_str);
                i += 2;
            }
            "--json" => {
                i += 1;
            }
            other => emit_error_and_exit(
                json_output,
                "invalid_arguments",
                &format!("unknown flag '{}'", other),
            ),
        }
    }

    let case_path = case_path.unwrap_or_else(|| {
        emit_error_and_exit(
            json_output,
            "invalid_arguments",
            "missing required argument --case",
        )
    });
    let registry_path = registry_path.unwrap_or_else(|| {
        emit_error_and_exit(
            json_output,
            "invalid_arguments",
            "missing required argument --registry",
        )
    });
    let config_path = config_path.unwrap_or_else(|| {
        emit_error_and_exit(
            json_output,
            "invalid_arguments",
            "missing required argument --config",
        )
    });

    let case_json = read_file_or_exit(json_output, case_path);
    let registry_json = read_file_or_exit(json_output, registry_path);
    let config_json = read_file_or_exit(json_output, config_path);

    let result = match route_case_from_registry_json(&case_json, &registry_json, &config_json) {
        Ok(r) => r,
        Err(e) => emit_error_and_exit(json_output, e.code(), &e.to_string()),
    };

    // Verify the receipt immediately as a self-check, then emit.
    if let Err(f) = verify_receipt_from_policy_json(
        &serde_json::to_string(&result.receipt).unwrap(),
        &case_json,
        &result.derived_policy_json,
    ) {
        emit_error_and_exit(json_output, &f.code, &f.message);
    }

    if json_output {
        println!("{}", serde_json::to_string_pretty(&result.receipt).unwrap());
    } else {
        println!("outcome:              {}", result.receipt.outcome);
        println!(
            "selected_candidate:   {}",
            result
                .receipt
                .selected_candidate_id
                .as_deref()
                .unwrap_or("\u{2014}")
        );
        println!(
            "refusal_code:         {}",
            result.receipt.refusal_code.as_deref().unwrap_or("\u{2014}")
        );
        println!(
            "routing_proof_hash:   {}",
            result.receipt.routing_proof_hash
        );
        println!(
            "policy_fingerprint:   {}",
            result.receipt.policy_fingerprint
        );
        println!("case_fingerprint:     {}", result.receipt.case_fingerprint);
        println!("audit_seq:            {}", result.receipt.audit_seq);
        println!("audit_entry_hash:     {}", result.receipt.audit_entry_hash);
        println!(
            "audit_previous_hash:  {}",
            result.receipt.audit_previous_hash
        );
        if let Some(detail) = &result.receipt.refusal {
            println!("refusal_message:      {}", detail.message);
            println!("failed_constraint:    {}", detail.failed_constraint);
        }
    }
}

fn run_verify_receipt(args: &[String]) {
    let json_output = args.iter().any(|a| a.as_str() == "--json");

    let mut receipt_path: Option<&str> = None;
    let mut case_path: Option<&str> = None;
    let mut policy_path: Option<&str> = None;
    let mut candidates_path: Option<&str> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--receipt" => {
                receipt_path = args.get(i + 1).map(String::as_str);
                i += 2;
            }
            "--case" => {
                case_path = args.get(i + 1).map(String::as_str);
                i += 2;
            }
            "--policy" => {
                policy_path = args.get(i + 1).map(String::as_str);
                i += 2;
            }
            "--candidates" => {
                candidates_path = args.get(i + 1).map(String::as_str);
                i += 2;
            }
            "--json" => {
                i += 1;
            }
            other => emit_error_and_exit(
                json_output,
                "invalid_arguments",
                &format!("unknown flag '{}'", other),
            ),
        }
    }

    let receipt_path = receipt_path.unwrap_or_else(|| {
        emit_error_and_exit(
            json_output,
            "invalid_arguments",
            "missing required argument --receipt",
        )
    });
    let case_path = case_path.unwrap_or_else(|| {
        emit_error_and_exit(
            json_output,
            "invalid_arguments",
            "missing required argument --case",
        )
    });
    let policy_path = policy_path.unwrap_or_else(|| {
        emit_error_and_exit(
            json_output,
            "invalid_arguments",
            "missing required argument --policy",
        )
    });
    let candidates_path = candidates_path.unwrap_or_else(|| {
        emit_error_and_exit(
            json_output,
            "invalid_arguments",
            "missing required argument --candidates",
        )
    });

    let receipt_json = read_file_or_exit(json_output, receipt_path);
    let case_json = read_file_or_exit(json_output, case_path);
    let policy_json = read_file_or_exit(json_output, policy_path);
    let candidates_json = read_file_or_exit(json_output, candidates_path);

    match verify_receipt_from_inputs(&receipt_json, &case_json, &policy_json, &candidates_json) {
        Ok(()) => {
            if json_output {
                println!("{}", serde_json::json!({"result": "VERIFIED"}));
            } else {
                println!("VERIFIED");
            }
        }
        Err(reason) => {
            if json_output {
                println!(
                    "{}",
                    serde_json::json!({"result": "VERIFICATION FAILED", "code": reason.code, "reason": reason.to_string()})
                );
            } else {
                println!("VERIFICATION FAILED: {}", reason);
            }
            process::exit(1);
        }
    }
}

// ── Frozen v1 demo fixtures (embedded at compile time) ───────────────────────

// Registry-backed pilot v1 demo fixtures (protocol vector v01).
const DEMO_CASE_JSON: &str =
    include_str!("../../../tests/protocol_vectors/v01_basic_routing/case.json");
const DEMO_REGISTRY_JSON: &str =
    include_str!("../../../tests/protocol_vectors/v01_basic_routing/registry_snapshot.json");
const DEMO_CONFIG_JSON: &str =
    include_str!("../../../tests/protocol_vectors/v01_basic_routing/policy.json");

// ── Pilot normalized route fixtures (embedded at compile time) ────────────────

/// Canonical normalized pilot case: case-001, crown / zirconia / DE jurisdiction.
const PILOT_NORM_CASE_JSON: &str = r#"{
  "case_id": "c0000001-0000-0000-0000-000000000001",
  "restoration_type": "crown",
  "material": "zirconia",
  "jurisdiction": "DE"
}"#;
const PILOT_NORM_REGISTRY_JSON: &str =
    include_str!("../../../examples/pilot/registry_snapshot.json");
const PILOT_NORM_CONFIG_JSON: &str = include_str!("../../../examples/pilot/config.json");

/// `pilot-route-normalized` — routes the canonical normalized pilot case.
///
/// Normalizes a minimal 4-field pilot input (case_id, restoration_type,
/// material, jurisdiction) via the input adapter, routes it through the
/// registry-backed kernel, self-verifies the receipt, and prints the result.
///
/// Uses only embedded compile-time fixtures; no file I/O, no flags required.
/// `--json` emits a compact summary object instead of human-readable output.
fn run_pilot_route_normalized(args: &[String]) {
    let json_output = args.iter().any(|a| a.as_str() == "--json");

    // Normalize the 4-field pilot input into a CaseInput-compatible JSON string.
    let case_json = match normalize_pilot_case_json(PILOT_NORM_CASE_JSON) {
        Ok(j) => j,
        Err(e) => emit_error_and_exit(json_output, e.code(), &e.to_string()),
    };

    // Route using the registry-backed kernel.
    let result =
        match route_case_from_registry_json(&case_json, PILOT_NORM_REGISTRY_JSON, PILOT_NORM_CONFIG_JSON) {
            Ok(r) => r,
            Err(e) => emit_error_and_exit(json_output, e.code(), &e.to_string()),
        };

    let receipt = &result.receipt;
    let receipt_json = serde_json::to_string(receipt).unwrap();

    // Self-verify: replay the routing decision before emitting output.
    match verify_receipt_from_policy_json(&receipt_json, &case_json, &result.derived_policy_json) {
        Ok(()) => {
            if json_output {
                println!(
                    "{}",
                    serde_json::json!({
                        "result":               "VERIFIED",
                        "outcome":              receipt.outcome,
                        "selected_candidate_id": receipt.selected_candidate_id,
                        "receipt_hash":         receipt.receipt_hash,
                    })
                );
            } else {
                println!("pilot-route-normalized");
                println!("----------------------");
                println!(
                    "Selected Candidate:   {}",
                    receipt.selected_candidate_id.as_deref().unwrap_or("\u{2014}")
                );
                println!("Receipt Hash:         {}", receipt.receipt_hash);
                println!("Verification:         VERIFIED");
            }
        }
        Err(f) => emit_error_and_exit(json_output, &f.code, &f.message),
    }
}

/// `demo-run` — executes the frozen PostCAD Protocol v1 flow in one command.
///
/// Demonstrates the full registry-backed pilot loop:
///   1. Derive candidates from the registry snapshot.
///   2. Route the case deterministically.
///   3. Emit the routing receipt.
///   4. Verify the receipt against the same inputs.
///   5. Print VERIFIED (or exit 1 on failure).
///
/// Uses only embedded compile-time fixtures; no file I/O, no flags required.
/// `--json` emits a compact summary object instead of human-readable output.
fn run_demo_v1(args: &[String]) {
    let json_output = args.iter().any(|a| a.as_str() == "--json");

    // Step 1 + 2: derive candidates from registry and route.
    let result =
        match route_case_from_registry_json(DEMO_CASE_JSON, DEMO_REGISTRY_JSON, DEMO_CONFIG_JSON) {
            Ok(r) => r,
            Err(e) => emit_error_and_exit(json_output, e.code(), &e.to_string()),
        };

    let receipt = &result.receipt;
    let receipt_json = serde_json::to_string(receipt).unwrap();

    // Step 3 + 4: verify using the derived policy bundle.
    match verify_receipt_from_policy_json(
        &receipt_json,
        DEMO_CASE_JSON,
        &result.derived_policy_json,
    ) {
        Ok(()) => {
            if json_output {
                println!(
                    "{}",
                    serde_json::json!({
                        "result": "VERIFIED",
                        "protocol_version": PROTOCOL_VERSION,
                        "outcome": receipt.outcome,
                        "selected_candidate_id": receipt.selected_candidate_id,
                        "receipt_hash": receipt.receipt_hash,
                    })
                );
            } else {
                println!("postcad protocol v1 demo");
                println!("------------------------");
                println!("outcome:              {}", receipt.outcome);
                println!(
                    "selected_candidate:   {}",
                    receipt
                        .selected_candidate_id
                        .as_deref()
                        .unwrap_or("\u{2014}")
                );
                println!("receipt_hash:         {}", receipt.receipt_hash);
                println!("protocol_version:     {}", PROTOCOL_VERSION);
                println!("verify:               VERIFIED");
            }
        }
        Err(f) => emit_error_and_exit(json_output, &f.code, &f.message),
    }
}

fn run_registry_export(args: &[String]) {
    let json_output = args.iter().any(|a| a.as_str() == "--json");

    let mut input_path: Option<&str> = None;
    let mut output_path: Option<&str> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--input" => {
                input_path = args.get(i + 1).map(String::as_str);
                i += 2;
            }
            "--output" => {
                output_path = args.get(i + 1).map(String::as_str);
                i += 2;
            }
            "--json" => {
                i += 1;
            }
            other => emit_error_and_exit(
                json_output,
                "invalid_arguments",
                &format!("unknown flag '{}'", other),
            ),
        }
    }

    let input_path = input_path.unwrap_or_else(|| {
        emit_error_and_exit(
            json_output,
            "invalid_arguments",
            "missing required argument --input",
        )
    });
    let output_path = output_path.unwrap_or_else(|| {
        emit_error_and_exit(
            json_output,
            "invalid_arguments",
            "missing required argument --output",
        )
    });

    let source_json = read_file_or_exit(json_output, input_path);
    let snapshot_json = match export_registry(&source_json) {
        Ok(j) => j,
        Err(e) => emit_error_and_exit(json_output, e.code(), &e.to_string()),
    };

    std::fs::write(output_path, &snapshot_json).unwrap_or_else(|e| {
        emit_error_and_exit(
            json_output,
            "io_error",
            &format!("cannot write '{}': {}", output_path, e),
        )
    });

    if json_output {
        println!(
            "{}",
            serde_json::json!({
                "result": "ok",
                "output": output_path,
            })
        );
    } else {
        println!("registry snapshot written to: {}", output_path);
    }
}

/// Emits a stable JSON error envelope (in --json mode) or a plain error line,
/// then exits with code 1. Returns `!` so it can appear in `unwrap_or_else`.
fn emit_error_and_exit(json_output: bool, code: &str, message: &str) -> ! {
    if json_output {
        println!(
            "{}",
            serde_json::json!({
                "outcome": "error",
                "code": code,
                "message": message
            })
        );
    } else {
        eprintln!("error: {}", message);
    }
    process::exit(1)
}

fn read_file_or_exit(json_output: bool, path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| {
        emit_error_and_exit(
            json_output,
            "io_error",
            &format!("cannot read '{}': {}", path, e),
        )
    })
}

fn run_protocol_manifest() {
    println!(
        "{}",
        serde_json::to_string_pretty(&build_manifest()).unwrap()
    );
}

/// `protocol-info` — compact semantic version summary of the protocol.
///
/// Outputs a JSON object with the semver identifiers and the three schema
/// hashes that uniquely identify this protocol configuration.  Uses the same
/// manifest data as `protocol-manifest` but presents only the semver surface.
fn run_protocol_info() {
    let m = build_manifest();
    let info = serde_json::json!({
        "manifest_fingerprint":    m.manifest_fingerprint,
        "proof_schema_hash":       m.proof_schema_hash,
        "protocol_version":        POSTCAD_PROTOCOL_VERSION,
        "receipt_schema_hash":     m.receipt_schema_hash,
        "refusal_code_set_hash":   m.refusal_code_set_hash,
        "routing_kernel_version":  ROUTING_KERNEL_SEMVER,
    });
    println!("{}", serde_json::to_string_pretty(&info).unwrap());
}

fn print_help() {
    println!("postcad-cli \u{2014} deterministic dental case routing");
    println!();
    println!("USAGE:");
    println!("    postcad-cli <subcommand> [options]");
    println!();
    println!("SUBCOMMANDS:");
    println!("    route-case                Route a dental case through the compliance pipeline");
    println!("    verify-receipt            Verify a routing receipt against its original inputs");
    println!("    pilot-route-normalized    Route the canonical normalized pilot case (4-field input)");
    println!("    demo-run                  Execute the frozen protocol v1 demo in one command");
    println!("    help                      Print this help message");
    println!();
    println!("OPTIONS (route-case):");
    println!("    --case <path>         Path to case JSON file");
    println!("    --candidates <path>   Path to candidates JSON array file");
    println!("    --snapshot <path>     Path to compliance snapshots JSON array file");
    println!("    --json                Emit output as JSON");
    println!();
    println!("OPTIONS (verify-receipt):");
    println!("    --receipt <path>      Path to the receipt JSON file to verify");
    println!("    --case <path>         Path to the original case JSON file");
    println!("    --policy <path>       Path to the policy JSON file (jurisdiction,");
    println!("                          routing_policy, compliance_profile, snapshots)");
    println!("    --candidates <path>   Path to the candidates JSON array file");
    println!("    --json                Emit output as JSON");
    println!();
    println!("CASE JSON FIELDS:");
    println!("    case_id              (optional) UUID string; generated if absent");
    println!("    jurisdiction         (optional) string; default \"global\"");
    println!(
        "    routing_policy       (optional) allow_domestic_only | allow_domestic_and_cross_border"
    );
    println!("    patient_country      united_states | germany | france | japan | united_kingdom | other:<name>");
    println!("    manufacturer_country same variants as patient_country");
    println!(
        "    material             zirconia | pmma | emax | cobalt_chrome | titanium | other:<name>"
    );
    println!("    procedure            crown | bridge | veneer | implant | denture | other:<name>");
    println!("    file_type            stl | obj | ply | three_mf | other:<name>");
}
