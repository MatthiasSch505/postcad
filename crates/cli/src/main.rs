use std::fs;
use std::process;

use postcad_cli::{route_case_from_json, CliError};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Dispatch subcommand.
    match args.get(1).map(String::as_str) {
        Some("route-case") => run_route_case(&args[2..]),
        Some("--help") | Some("-h") | Some("help") => print_help(),
        Some(other) => {
            eprintln!("error: unknown subcommand '{}'", other);
            print_help();
            process::exit(1);
        }
        None => {
            print_help();
            process::exit(1);
        }
    }
}

fn run_route_case(args: &[String]) {
    let mut case_path: Option<&str> = None;
    let mut candidates_path: Option<&str> = None;
    let mut snapshot_path: Option<&str> = None;
    let mut json_output = false;

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
                json_output = true;
                i += 1;
            }
            other => {
                eprintln!("error: unknown flag '{}'", other);
                process::exit(1);
            }
        }
    }

    let case_path = require_arg(case_path, "--case");
    let candidates_path = require_arg(candidates_path, "--candidates");
    let snapshot_path = require_arg(snapshot_path, "--snapshot");

    let case_json = read_file(case_path);
    let candidates_json = read_file(candidates_path);
    let snapshots_json = read_file(snapshot_path);

    let output = match route_case_from_json(&case_json, &candidates_json, &snapshots_json) {
        Ok(o) => o,
        Err(e) => {
            if json_output {
                println!("{}", error_to_json(&e));
            } else {
                eprintln!("error: {}", e);
            }
            process::exit(1);
        }
    };

    if json_output {
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    } else {
        println!("outcome:              {}", output.outcome);
        println!(
            "selected_candidate:   {}",
            output.selected_candidate_id.as_deref().unwrap_or("—")
        );
        println!("routing_proof_hash:   {}", output.routing_proof_hash);
        println!("policy_fingerprint:   {}", output.policy_fingerprint);
        println!("case_fingerprint:     {}", output.case_fingerprint);
        if let Some(r) = &output.refusal {
            println!("refusal_code:         {}", r.code);
            println!("refusal_message:      {}", r.message);
        }
    }
}

fn print_help() {
    println!("postcad-cli — deterministic dental case routing");
    println!();
    println!("USAGE:");
    println!("    postcad-cli <subcommand> [options]");
    println!();
    println!("SUBCOMMANDS:");
    println!("    route-case    Route a dental case through the compliance pipeline");
    println!("    help          Print this help message");
    println!();
    println!("OPTIONS (route-case):");
    println!("    --case <path>         Path to case JSON file");
    println!("    --candidates <path>   Path to candidates JSON array file");
    println!("    --snapshot <path>     Path to compliance snapshots JSON array file");
    println!("    --json                Emit output as JSON");
    println!();
    println!("CASE JSON FIELDS:");
    println!("    case_id              (optional) UUID string; generated if absent");
    println!("    jurisdiction         (optional) string; default \"global\"");
    println!("    routing_policy       (optional) allow_domestic_only | allow_domestic_and_cross_border");
    println!("    patient_country      united_states | germany | france | japan | united_kingdom | other:<name>");
    println!("    manufacturer_country same variants as patient_country");
    println!("    material             zirconia | pmma | emax | cobalt_chrome | titanium | other:<name>");
    println!("    procedure            crown | bridge | veneer | implant | denture | other:<name>");
    println!("    file_type            stl | obj | ply | three_mf | other:<name>");
}

fn error_to_json(e: &CliError) -> String {
    let code = match e {
        CliError::ParseError(_) => "invalid_input",
        CliError::InvalidField(_) => "invalid_input",
        CliError::SnapshotValidation(_) => "invalid_snapshot",
    };
    serde_json::json!({
        "code": code,
        "message": e.to_string()
    })
    .to_string()
}

fn require_arg<'a>(val: Option<&'a str>, flag: &str) -> &'a str {
    match val {
        Some(v) => v,
        None => {
            eprintln!("error: missing required argument {}", flag);
            process::exit(1);
        }
    }
}

fn read_file(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("error: cannot read '{}': {}", path, e);
        process::exit(1);
    })
}
