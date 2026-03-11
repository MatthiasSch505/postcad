//! Filesystem-backed case store.
//!
//! Cases are persisted as canonical JSON files under a configurable base
//! directory. The canonical path for a case is `{dir}/{case_id}.json`.
//!
//! All operations are synchronous and safe to call from an async context
//! provided the caller does not hold async locks across them.

use std::fs;
use std::path::PathBuf;

use serde_json::Value;

/// Filesystem-backed store for pilot cases.
///
/// The store is intentionally stateless beyond the directory path; it issues
/// no locks and relies on filesystem atomicity for single-file writes.
pub struct CaseStore {
    dir: PathBuf,
}

/// Outcome of a successful [`CaseStore::store`] call.
pub enum StoreOutcome {
    /// New file written.
    Created,
    /// Identical content was already present; no write performed.
    Identical,
}

/// Errors that can be returned by [`CaseStore`] operations.
#[derive(Debug)]
pub enum CaseStoreError {
    /// A different case JSON is already stored under the same `case_id`.
    Conflict,
    /// An underlying I/O error occurred.
    Io(std::io::Error),
    /// Stored file contained invalid JSON.
    InvalidJson(serde_json::Error),
}

impl std::fmt::Display for CaseStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CaseStoreError::Conflict => write!(f, "case_id already stored with different content"),
            CaseStoreError::Io(e) => write!(f, "io error: {e}"),
            CaseStoreError::InvalidJson(e) => write!(f, "invalid json in store: {e}"),
        }
    }
}

impl CaseStore {
    /// Create a store rooted at `dir`. The directory is created lazily on
    /// first write.
    pub fn new<P: Into<PathBuf>>(dir: P) -> Self {
        Self { dir: dir.into() }
    }

    fn path_for(&self, case_id: &str) -> PathBuf {
        self.dir.join(format!("{case_id}.json"))
    }

    /// Persist `canonical_json` as `{dir}/{case_id}.json`.
    ///
    /// - If the file does not exist, writes it and returns [`StoreOutcome::Created`].
    /// - If the file exists with identical JSON value, returns [`StoreOutcome::Identical`].
    /// - If the file exists with different JSON value, returns [`CaseStoreError::Conflict`].
    pub fn store(
        &self,
        case_id: &str,
        canonical_json: &str,
    ) -> Result<StoreOutcome, CaseStoreError> {
        fs::create_dir_all(&self.dir).map_err(CaseStoreError::Io)?;
        let path = self.path_for(case_id);

        if path.exists() {
            let existing_raw = fs::read_to_string(&path).map_err(CaseStoreError::Io)?;
            let existing: Value =
                serde_json::from_str(&existing_raw).map_err(CaseStoreError::InvalidJson)?;
            let incoming: Value =
                serde_json::from_str(canonical_json).map_err(CaseStoreError::InvalidJson)?;
            return if existing == incoming {
                Ok(StoreOutcome::Identical)
            } else {
                Err(CaseStoreError::Conflict)
            };
        }

        fs::write(&path, canonical_json).map_err(CaseStoreError::Io)?;
        Ok(StoreOutcome::Created)
    }

    /// List stored case IDs in ascending order.
    pub fn list(&self) -> Result<Vec<String>, CaseStoreError> {
        if !self.dir.exists() {
            return Ok(vec![]);
        }
        let mut ids = Vec::new();
        for entry in fs::read_dir(&self.dir).map_err(CaseStoreError::Io)? {
            let entry = entry.map_err(CaseStoreError::Io)?;
            let name = entry.file_name();
            let s = name.to_string_lossy();
            if s.ends_with(".json") {
                ids.push(s.trim_end_matches(".json").to_string());
            }
        }
        ids.sort();
        Ok(ids)
    }

    /// Return the stored JSON value for `case_id`, or `None` if absent.
    pub fn get(&self, case_id: &str) -> Result<Option<Value>, CaseStoreError> {
        let path = self.path_for(case_id);
        if !path.exists() {
            return Ok(None);
        }
        let raw = fs::read_to_string(&path).map_err(CaseStoreError::Io)?;
        let value: Value = serde_json::from_str(&raw).map_err(CaseStoreError::InvalidJson)?;
        Ok(Some(value))
    }
}
