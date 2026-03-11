//! Filesystem-backed verification result store.
//!
//! Verification results are persisted at `{dir}/{receipt_hash}.json`.
//! Writes always overwrite — verification can be re-run.

use std::fs;
use std::path::PathBuf;

/// Filesystem-backed store for dispatch verification results.
pub struct VerificationStore {
    dir: PathBuf,
}

/// Errors that can be returned by [`VerificationStore`] operations.
#[derive(Debug)]
pub enum VerificationStoreError {
    /// An underlying I/O error occurred.
    Io(std::io::Error),
}

impl std::fmt::Display for VerificationStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerificationStoreError::Io(e) => write!(f, "io error: {e}"),
        }
    }
}

impl VerificationStore {
    /// Create a store rooted at `dir`. The directory is created lazily on
    /// first write.
    pub fn new<P: Into<PathBuf>>(dir: P) -> Self {
        Self { dir: dir.into() }
    }

    fn path_for(&self, receipt_hash: &str) -> PathBuf {
        self.dir.join(format!("{receipt_hash}.json"))
    }

    /// Persist `result_json` as `{dir}/{receipt_hash}.json`, overwriting any
    /// previous result.
    pub fn store(
        &self,
        receipt_hash: &str,
        result_json: &str,
    ) -> Result<(), VerificationStoreError> {
        fs::create_dir_all(&self.dir).map_err(VerificationStoreError::Io)?;
        fs::write(self.path_for(receipt_hash), result_json).map_err(VerificationStoreError::Io)?;
        Ok(())
    }
}
