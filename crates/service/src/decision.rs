use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DecisionType {
    Proceed,
    ProceedWithRisk,
    RequestCorrection,
}

impl DecisionType {
    pub fn is_routable(&self) -> bool {
        matches!(self, DecisionType::Proceed | DecisionType::ProceedWithRisk)
    }

    pub fn requires_reason_code(&self) -> bool {
        matches!(
            self,
            DecisionType::ProceedWithRisk | DecisionType::RequestCorrection
        )
    }
}

impl fmt::Display for DecisionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecisionType::Proceed => write!(f, "proceed"),
            DecisionType::ProceedWithRisk => write!(f, "proceed_with_risk"),
            DecisionType::RequestCorrection => write!(f, "request_correction"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DecisionReasonCode {
    IncompleteScan,
    UnclearMargin,
    PrepUncertainty,
    TimePressure,
    Other,
}

impl fmt::Display for DecisionReasonCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecisionReasonCode::IncompleteScan => write!(f, "incomplete_scan"),
            DecisionReasonCode::UnclearMargin => write!(f, "unclear_margin"),
            DecisionReasonCode::PrepUncertainty => write!(f, "prep_uncertainty"),
            DecisionReasonCode::TimePressure => write!(f, "time_pressure"),
            DecisionReasonCode::Other => write!(f, "other"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRecord {
    pub decision_id: String,
    pub case_id: String,
    pub timestamp: String,
    pub actor_role: String,
    pub actor_id: String,
    pub decision_type: DecisionType,
    pub reason_code: Option<DecisionReasonCode>,
    /// SHA-256 of the compact canonical JSON of the case at decision time.
    pub input_hash: String,
    /// SHA-256 of all other fields (excluding this one), pipe-delimited.
    pub decision_hash: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateDecisionRequest {
    pub case_id: String,
    pub actor_role: String,
    pub actor_id: String,
    pub decision_type: DecisionType,
    pub reason_code: Option<DecisionReasonCode>,
}

#[derive(Debug)]
pub enum DecisionValidationError {
    MissingReasonCode(String),
}

impl fmt::Display for DecisionValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecisionValidationError::MissingReasonCode(dt) => {
                write!(f, "decision_type '{dt}' requires a reason_code")
            }
        }
    }
}

pub fn validate_decision(
    decision_type: &DecisionType,
    reason_code: &Option<DecisionReasonCode>,
) -> Result<(), DecisionValidationError> {
    if decision_type.requires_reason_code() && reason_code.is_none() {
        return Err(DecisionValidationError::MissingReasonCode(
            decision_type.to_string(),
        ));
    }
    Ok(())
}

/// SHA-256 of compact JSON of the case value — used as input_hash.
pub fn compute_case_input_hash(case_value: &serde_json::Value) -> String {
    let compact = serde_json::to_string(case_value).expect("case json serialization");
    let digest = Sha256::digest(compact.as_bytes());
    format!("{:x}", digest)
}

fn compute_decision_hash(record: &DecisionRecord) -> String {
    let canonical = format!(
        "{}|{}|{}|{}|{}|{}|{}|{}",
        record.decision_id,
        record.case_id,
        record.timestamp,
        record.actor_role,
        record.actor_id,
        record.decision_type,
        record
            .reason_code
            .as_ref()
            .map(|r| r.to_string())
            .unwrap_or_default(),
        record.input_hash,
    );
    let digest = Sha256::digest(canonical.as_bytes());
    format!("{:x}", digest)
}

pub fn build_decision_record(req: CreateDecisionRequest, input_hash: String) -> DecisionRecord {
    let decision_id = Uuid::new_v4().to_string();
    let timestamp = Utc::now().to_rfc3339();

    let mut record = DecisionRecord {
        decision_id,
        case_id: req.case_id,
        timestamp,
        actor_role: req.actor_role,
        actor_id: req.actor_id,
        decision_type: req.decision_type,
        reason_code: req.reason_code,
        input_hash,
        decision_hash: String::new(),
    };
    record.decision_hash = compute_decision_hash(&record);
    record
}

// ── Decision Store ────────────────────────────────────────────────────────────

pub struct DecisionStore {
    dir: PathBuf,
}

#[derive(Debug)]
pub enum DecisionStoreError {
    Io(std::io::Error),
    InvalidJson(serde_json::Error),
}

impl fmt::Display for DecisionStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecisionStoreError::Io(e) => write!(f, "io error: {e}"),
            DecisionStoreError::InvalidJson(e) => write!(f, "invalid json: {e}"),
        }
    }
}

impl DecisionStore {
    pub fn new<P: Into<PathBuf>>(dir: P) -> Self {
        Self { dir: dir.into() }
    }

    pub fn store(&self, record: &DecisionRecord) -> Result<(), DecisionStoreError> {
        fs::create_dir_all(&self.dir).map_err(DecisionStoreError::Io)?;
        let path = self.dir.join(format!("{}.json", record.decision_id));
        let json = serde_json::to_string_pretty(record).expect("decision serialization");
        fs::write(path, json).map_err(DecisionStoreError::Io)
    }

    pub fn get_by_case_id(&self, case_id: &str) -> Result<Option<DecisionRecord>, DecisionStoreError> {
        if !self.dir.exists() {
            return Ok(None);
        }
        for entry in fs::read_dir(&self.dir)
            .map_err(DecisionStoreError::Io)?
            .flatten()
        {
            let name = entry.file_name();
            if !name.to_string_lossy().ends_with(".json") {
                continue;
            }
            let raw = fs::read_to_string(entry.path()).map_err(DecisionStoreError::Io)?;
            let record: DecisionRecord =
                serde_json::from_str(&raw).map_err(DecisionStoreError::InvalidJson)?;
            if record.case_id == case_id {
                return Ok(Some(record));
            }
        }
        Ok(None)
    }
}
