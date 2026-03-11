//! Filesystem-backed dispatch store.
//!
//! Dispatch records are persisted as JSON files under a configurable base
//! directory. The canonical path for a dispatch record is
//! `{dir}/{receipt_hash}.json`.
//!
//! Because the file name is derived from the receipt hash, a dispatch record
//! is unique per receipt. Attempting to store a second record for the same
//! receipt hash returns [`DispatchStoreError::AlreadyExists`].

use std::fs;
use std::path::PathBuf;

/// Filesystem-backed store for dispatch records.
pub struct DispatchStore {
    dir: PathBuf,
}

/// Errors that can be returned by [`DispatchStore`] operations.
#[derive(Debug)]
pub enum DispatchStoreError {
    /// An underlying I/O error occurred.
    Io(std::io::Error),
    /// A dispatch record already exists for this receipt hash.
    AlreadyExists,
}

impl std::fmt::Display for DispatchStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DispatchStoreError::Io(e) => write!(f, "io error: {e}"),
            DispatchStoreError::AlreadyExists => write!(f, "dispatch record already exists"),
        }
    }
}

impl DispatchStore {
    /// Create a store rooted at `dir`. The directory is created lazily on
    /// first write.
    pub fn new<P: Into<PathBuf>>(dir: P) -> Self {
        Self { dir: dir.into() }
    }

    fn path_for(&self, receipt_hash: &str) -> PathBuf {
        self.dir.join(format!("{receipt_hash}.json"))
    }

    /// Return `true` if a dispatch record for `receipt_hash` exists on disk.
    pub fn exists(&self, receipt_hash: &str) -> bool {
        self.path_for(receipt_hash).exists()
    }

    /// Persist `record_json` as `{dir}/{receipt_hash}.json`.
    ///
    /// Returns [`DispatchStoreError::AlreadyExists`] if a file already exists
    /// for this receipt hash.
    pub fn store(&self, receipt_hash: &str, record_json: &str) -> Result<(), DispatchStoreError> {
        fs::create_dir_all(&self.dir).map_err(DispatchStoreError::Io)?;
        let path = self.path_for(receipt_hash);
        if path.exists() {
            return Err(DispatchStoreError::AlreadyExists);
        }
        fs::write(&path, record_json).map_err(DispatchStoreError::Io)?;
        Ok(())
    }
}
