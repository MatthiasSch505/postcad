//! Filesystem-backed receipt store.
//!
//! Receipts are persisted as canonical JSON files under a configurable base
//! directory. The canonical path for a receipt is `{dir}/{receipt_hash}.json`.
//!
//! Because the file name is derived from the content hash, storing the same
//! receipt twice is always idempotent.

use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use chrono::{DateTime, Utc};
use serde_json::Value;

/// Filesystem-backed store for routing receipts.
///
/// Receipts are keyed by their `receipt_hash`, making every file
/// content-addressed. The store issues no locks and relies on filesystem
/// atomicity for single-file writes.
pub struct ReceiptStore {
    dir: PathBuf,
}

/// A single entry in the route history, derived from a stored receipt file.
#[derive(Debug)]
pub struct RouteEntry {
    /// The case identifier extracted from `routing_input.case_id`.
    pub case_id: String,
    /// SHA-256 hash of the receipt (also the file stem).
    pub receipt_hash: String,
    /// The selected candidate, or `None` for refused outcomes.
    pub selected_candidate_id: Option<String>,
    /// File modification time formatted as RFC 3339 (UTC, second precision).
    pub timestamp: String,
}

/// Errors that can be returned by [`ReceiptStore`] operations.
#[derive(Debug)]
pub enum ReceiptStoreError {
    /// An underlying I/O error occurred.
    Io(std::io::Error),
    /// A stored receipt file could not be parsed.
    ParseError(String),
}

impl std::fmt::Display for ReceiptStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReceiptStoreError::Io(e) => write!(f, "io error: {e}"),
            ReceiptStoreError::ParseError(msg) => write!(f, "receipt parse error: {msg}"),
        }
    }
}

impl ReceiptStore {
    /// Create a store rooted at `dir`. The directory is created lazily on
    /// first write.
    pub fn new<P: Into<PathBuf>>(dir: P) -> Self {
        Self { dir: dir.into() }
    }

    fn path_for(&self, receipt_hash: &str) -> PathBuf {
        self.dir.join(format!("{receipt_hash}.json"))
    }

    /// Persist `canonical_json` as `{dir}/{receipt_hash}.json`.
    ///
    /// Idempotent: if a file already exists under the same hash it is left
    /// unchanged and `Ok(())` is returned. Since the file name is the content
    /// hash an existing file is guaranteed to carry identical content.
    pub fn store(&self, receipt_hash: &str, canonical_json: &str) -> Result<(), ReceiptStoreError> {
        fs::create_dir_all(&self.dir).map_err(ReceiptStoreError::Io)?;
        let path = self.path_for(receipt_hash);
        if path.exists() {
            return Ok(());
        }
        fs::write(&path, canonical_json).map_err(ReceiptStoreError::Io)?;
        Ok(())
    }

    /// Return `true` if a receipt file for `receipt_hash` exists on disk.
    pub fn exists(&self, receipt_hash: &str) -> bool {
        self.path_for(receipt_hash).exists()
    }

    /// Return the sorted list of all stored receipt hashes (file stems).
    ///
    /// Returns an empty list if the directory does not yet exist.
    pub fn list_hashes(&self) -> Result<Vec<String>, ReceiptStoreError> {
        if !self.dir.exists() {
            return Ok(vec![]);
        }
        let mut hashes: Vec<String> = Vec::new();
        for dir_entry in fs::read_dir(&self.dir).map_err(ReceiptStoreError::Io)? {
            let dir_entry = dir_entry.map_err(ReceiptStoreError::Io)?;
            let name = dir_entry.file_name();
            let s = name.to_string_lossy();
            if s.ends_with(".json") {
                hashes.push(s.trim_end_matches(".json").to_string());
            }
        }
        hashes.sort();
        Ok(hashes)
    }

    /// Read and parse a stored receipt. Returns `Ok(None)` if the file does
    /// not exist.
    pub fn read(&self, receipt_hash: &str) -> Result<Option<Value>, ReceiptStoreError> {
        let path = self.path_for(receipt_hash);
        if !path.exists() {
            return Ok(None);
        }
        let raw = fs::read_to_string(&path).map_err(ReceiptStoreError::Io)?;
        let v: Value = serde_json::from_str(&raw)
            .map_err(|e| ReceiptStoreError::ParseError(format!("{receipt_hash}: {e}")))?;
        Ok(Some(v))
    }

    /// Read all stored receipts and return them as [`RouteEntry`] items.
    ///
    /// Each entry's `timestamp` is derived from the file's last-modified time.
    /// Entries are sorted by **timestamp descending**, with **receipt_hash
    /// ascending** as a tiebreaker to guarantee a deterministic order when
    /// multiple files share the same modification time (common in tests).
    ///
    /// Returns an empty list if the directory does not yet exist.
    /// Returns [`ReceiptStoreError::ParseError`] if any receipt file contains
    /// invalid JSON or is missing the required `receipt_hash` or
    /// `routing_input.case_id` fields.
    pub fn list_routes(&self) -> Result<Vec<RouteEntry>, ReceiptStoreError> {
        if !self.dir.exists() {
            return Ok(vec![]);
        }

        let mut entries: Vec<(String, RouteEntry)> = Vec::new(); // (timestamp, entry)

        for dir_entry in fs::read_dir(&self.dir).map_err(ReceiptStoreError::Io)? {
            let dir_entry = dir_entry.map_err(ReceiptStoreError::Io)?;
            let name = dir_entry.file_name();
            let s = name.to_string_lossy();
            if !s.ends_with(".json") {
                continue;
            }
            let receipt_hash = s.trim_end_matches(".json").to_string();

            // Derive timestamp from file modification time.
            let modified: SystemTime = dir_entry
                .metadata()
                .map_err(ReceiptStoreError::Io)?
                .modified()
                .unwrap_or(SystemTime::UNIX_EPOCH);
            let dt: DateTime<Utc> = modified.into();
            let timestamp = dt.format("%Y-%m-%dT%H:%M:%SZ").to_string();

            // Parse the receipt JSON.
            let raw = fs::read_to_string(dir_entry.path()).map_err(ReceiptStoreError::Io)?;
            let v: Value = serde_json::from_str(&raw)
                .map_err(|e| ReceiptStoreError::ParseError(format!("{receipt_hash}: {e}")))?;

            let case_id = v["routing_input"]["case_id"]
                .as_str()
                .ok_or_else(|| {
                    ReceiptStoreError::ParseError(format!(
                        "{receipt_hash}: missing routing_input.case_id"
                    ))
                })?
                .to_string();

            let stored_hash = v["receipt_hash"]
                .as_str()
                .ok_or_else(|| {
                    ReceiptStoreError::ParseError(format!("{receipt_hash}: missing receipt_hash"))
                })?
                .to_string();

            let selected_candidate_id = v["selected_candidate_id"].as_str().map(|s| s.to_string());

            entries.push((
                timestamp.clone(),
                RouteEntry {
                    case_id,
                    receipt_hash: stored_hash,
                    selected_candidate_id,
                    timestamp,
                },
            ));
        }

        // Sort: timestamp descending, then receipt_hash ascending.
        entries.sort_by(|(ts_a, e_a), (ts_b, e_b)| {
            ts_b.cmp(ts_a)
                .then_with(|| e_a.receipt_hash.cmp(&e_b.receipt_hash))
        });

        Ok(entries.into_iter().map(|(_, e)| e).collect())
    }
}
