//! PostCAD Protocol v1 — standalone demo runner.
//!
//! Executes the full registry-backed pilot loop using frozen protocol-vector
//! v01 fixtures embedded at compile time:
//!
//!   1. Derive candidates from registry snapshot.
//!   2. Route case deterministically.
//!   3. Verify receipt against derived policy.
//!   4. Print result.
//!
//! Exit 0 on VERIFIED, exit 1 on any failure.

use postcad_cli::{
    route_case_from_registry_json, verify_receipt_from_policy_json, PROTOCOL_VERSION,
};

const CASE: &str = include_str!("../../../../tests/protocol_vectors/v01_basic_routing/case.json");
const REGISTRY: &str =
    include_str!("../../../../tests/protocol_vectors/v01_basic_routing/registry_snapshot.json");
const CONFIG: &str =
    include_str!("../../../../tests/protocol_vectors/v01_basic_routing/policy.json");

fn main() {
    println!("PostCAD Demo");
    println!();

    println!("Routing case...");
    let result = match route_case_from_registry_json(CASE, REGISTRY, CONFIG) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("routing failed: {}", e);
            std::process::exit(1);
        }
    };

    let receipt = &result.receipt;
    let outcome = receipt.outcome.as_str();

    if outcome == "routed" {
        println!("Result: ROUTED");
        println!();
        println!(
            "Selected candidate: {}",
            receipt.selected_candidate_id.as_deref().unwrap_or("—")
        );
        println!("Receipt hash: {}", receipt.receipt_hash);
    } else {
        println!("Result: REFUSED");
        println!();
        println!(
            "Refusal code: {}",
            receipt.refusal_code.as_deref().unwrap_or("—")
        );
        std::process::exit(0);
    }

    println!();
    println!("Verifying receipt...");
    let receipt_json = serde_json::to_string(receipt).unwrap();
    match verify_receipt_from_policy_json(&receipt_json, CASE, &result.derived_policy_json) {
        Ok(()) => {
            println!("Verification: OK");
            println!();
            println!("Protocol version: {}", PROTOCOL_VERSION);
        }
        Err(f) => {
            eprintln!("verification failed: {} — {}", f.code, f.message);
            std::process::exit(1);
        }
    }
}
