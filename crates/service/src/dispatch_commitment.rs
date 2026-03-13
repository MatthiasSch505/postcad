//! Dispatch Commitment Layer — domain object and filesystem-backed store.
//!
//! A `DispatchRecord` is created only from a verification-passed routing receipt.
//! It goes through a minimal state machine: draft → approved → exported.
//!
//! Invariants:
//! - One dispatch per receipt (enforced by `receipt_hash` uniqueness scan).
//! - `verification_passed` is always `true` — the handler rejects if verification fails.
//! - Once approved, routing/receipt fields are immutable; only `status` can advance.
//! - `dispatch_id` is a UUID generated at creation and used as the storage key.

use std::fs;
use std::path::PathBuf;

use chrono::Utc;
use serde::{Deserialize, Serialize};

// ── Domain object ─────────────────────────────────────────────────────────────

/// Immutable commitment record produced after a verified routing result is
/// approved for dispatch to the selected manufacturer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchRecord {
    /// Unique identifier for this dispatch commitment (UUID v4).
    pub dispatch_id: String,
    /// Routing case identifier extracted from the receipt.
    pub case_id: String,
    /// Manufacturer selected by the routing kernel (`None` for refused receipts).
    pub selected_candidate_id: Option<String>,
    /// SHA-256 receipt hash — binds the commitment to a specific routing artifact.
    pub receipt_hash: String,
    /// Always `true` — dispatch creation is gated on successful verification.
    pub verification_passed: bool,
    /// Lifecycle state: `"draft"` → `"approved"` → `"exported"`.
    pub status: String,
    /// Identity of the operator who approved this dispatch (set at approval time).
    pub approved_by: Option<String>,
    /// ISO-8601 timestamp when the dispatch was approved.
    pub approved_at: Option<String>,
    /// ISO-8601 timestamp when the dispatch record was created.
    pub created_at: String,
    /// Optional opaque JSON payload forwarded to the manufacturer on export.
    pub manufacturer_payload_json: Option<String>,
}

// ── Store ─────────────────────────────────────────────────────────────────────

/// Filesystem-backed store for [`DispatchRecord`]s.
///
/// Records are persisted at `{dir}/{dispatch_id}.json`.
pub struct DispatchCommitmentStore {
    dir: PathBuf,
}

/// Errors returned by [`DispatchCommitmentStore`] operations.
#[derive(Debug)]
pub enum DispatchCommitmentError {
    Io(std::io::Error),
    Parse(serde_json::Error),
    /// A dispatch already exists for this receipt hash.
    ReceiptAlreadyDispatched,
    /// No record found for the given `dispatch_id`.
    NotFound,
    /// Approve requires status == `"draft"`.
    NotDraft,
    /// Export requires status == `"approved"` or `"exported"`.
    NotApproved,
}

impl std::fmt::Display for DispatchCommitmentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "io error: {e}"),
            Self::Parse(e) => write!(f, "parse error: {e}"),
            Self::ReceiptAlreadyDispatched => {
                write!(f, "a dispatch commitment already exists for this receipt")
            }
            Self::NotFound => write!(f, "dispatch record not found"),
            Self::NotDraft => write!(f, "dispatch is not in draft state"),
            Self::NotApproved => write!(f, "dispatch must be approved before export"),
        }
    }
}

impl DispatchCommitmentStore {
    /// Create a store rooted at `dir`. The directory is created lazily on first write.
    pub fn new<P: Into<PathBuf>>(dir: P) -> Self {
        Self { dir: dir.into() }
    }

    fn path_for(&self, dispatch_id: &str) -> PathBuf {
        self.dir.join(format!("{dispatch_id}.json"))
    }

    /// Scan all records for one whose `receipt_hash` matches.
    ///
    /// O(n) scan — acceptable for the small volumes expected in this layer.
    fn find_by_receipt(
        &self,
        receipt_hash: &str,
    ) -> Result<Option<DispatchRecord>, DispatchCommitmentError> {
        if !self.dir.exists() {
            return Ok(None);
        }
        let entries = fs::read_dir(&self.dir).map_err(DispatchCommitmentError::Io)?;
        for entry in entries {
            let entry = entry.map_err(DispatchCommitmentError::Io)?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let content = fs::read_to_string(&path).map_err(DispatchCommitmentError::Io)?;
            let record: DispatchRecord =
                serde_json::from_str(&content).map_err(DispatchCommitmentError::Parse)?;
            if record.receipt_hash == receipt_hash {
                return Ok(Some(record));
            }
        }
        Ok(None)
    }

    /// Persist a new dispatch record. Fails if one already exists for the receipt.
    pub fn create(&self, record: &DispatchRecord) -> Result<(), DispatchCommitmentError> {
        fs::create_dir_all(&self.dir).map_err(DispatchCommitmentError::Io)?;
        if self.find_by_receipt(&record.receipt_hash)?.is_some() {
            return Err(DispatchCommitmentError::ReceiptAlreadyDispatched);
        }
        let json =
            serde_json::to_string_pretty(record).map_err(DispatchCommitmentError::Parse)?;
        fs::write(self.path_for(&record.dispatch_id), json)
            .map_err(DispatchCommitmentError::Io)?;
        Ok(())
    }

    /// Read a dispatch record by `dispatch_id`.
    pub fn read(
        &self,
        dispatch_id: &str,
    ) -> Result<Option<DispatchRecord>, DispatchCommitmentError> {
        let path = self.path_for(dispatch_id);
        if !path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(path).map_err(DispatchCommitmentError::Io)?;
        let record = serde_json::from_str(&content).map_err(DispatchCommitmentError::Parse)?;
        Ok(Some(record))
    }

    /// Transition `draft` → `approved`. Returns the updated record.
    ///
    /// Fails with [`DispatchCommitmentError::NotDraft`] if already approved,
    /// enforcing immutability of the approval decision.
    pub fn approve(
        &self,
        dispatch_id: &str,
        approved_by: &str,
    ) -> Result<DispatchRecord, DispatchCommitmentError> {
        let mut record = self
            .read(dispatch_id)?
            .ok_or(DispatchCommitmentError::NotFound)?;
        if record.status != "draft" {
            return Err(DispatchCommitmentError::NotDraft);
        }
        record.status = "approved".to_string();
        record.approved_by = Some(approved_by.to_string());
        record.approved_at = Some(Utc::now().to_rfc3339());
        let json =
            serde_json::to_string_pretty(&record).map_err(DispatchCommitmentError::Parse)?;
        fs::write(self.path_for(dispatch_id), json).map_err(DispatchCommitmentError::Io)?;
        Ok(record)
    }

    /// Transition `approved` → `exported`. Idempotent if already `exported`.
    ///
    /// Fails with [`DispatchCommitmentError::NotApproved`] if status is `draft`.
    pub fn mark_exported(
        &self,
        dispatch_id: &str,
    ) -> Result<DispatchRecord, DispatchCommitmentError> {
        let mut record = self
            .read(dispatch_id)?
            .ok_or(DispatchCommitmentError::NotFound)?;
        if record.status == "draft" {
            return Err(DispatchCommitmentError::NotApproved);
        }
        if record.status == "approved" {
            record.status = "exported".to_string();
            let json =
                serde_json::to_string_pretty(&record).map_err(DispatchCommitmentError::Parse)?;
            fs::write(self.path_for(dispatch_id), json).map_err(DispatchCommitmentError::Io)?;
        }
        Ok(record)
    }
}
