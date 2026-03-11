//! Filesystem-backed receipt store.
//!
//! Receipts are persisted as canonical JSON files under a configurable base
//! directory. The canonical path for a receipt is `{dir}/{receipt_hash}.json`.
//!
//! Because the file name is derived from the content hash, storing the same
//! receipt twice is always idempotent.

use std::fs;
use std::path::PathBuf;

/// Filesystem-backed store for routing receipts.
///
/// Receipts are keyed by their `receipt_hash`, making every file
/// content-addressed. The store issues no locks and relies on filesystem
/// atomicity for single-file writes.
pub struct ReceiptStore {
    dir: PathBuf,
}

/// Errors that can be returned by [`ReceiptStore`] operations.
#[derive(Debug)]
pub enum ReceiptStoreError {
    /// An underlying I/O error occurred.
    Io(std::io::Error),
}

impl std::fmt::Display for ReceiptStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReceiptStoreError::Io(e) => write!(f, "io error: {e}"),
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
}
