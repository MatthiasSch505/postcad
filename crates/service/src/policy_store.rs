//! Filesystem-backed policy store.
//!
//! Derived policy bundles are persisted at `{dir}/{receipt_hash}.json`.
//! Storing the same policy twice is idempotent (the bundle is immutable for a
//! given receipt hash).

use std::fs;
use std::path::PathBuf;

/// Filesystem-backed store for derived routing policy bundles.
pub struct PolicyStore {
    dir: PathBuf,
}

/// Errors that can be returned by [`PolicyStore`] operations.
#[derive(Debug)]
pub enum PolicyStoreError {
    /// An underlying I/O error occurred.
    Io(std::io::Error),
    /// A stored policy file could not be parsed.
    ParseError(String),
}

impl std::fmt::Display for PolicyStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PolicyStoreError::Io(e) => write!(f, "io error: {e}"),
            PolicyStoreError::ParseError(msg) => write!(f, "policy parse error: {msg}"),
        }
    }
}

impl PolicyStore {
    /// Create a store rooted at `dir`. The directory is created lazily on
    /// first write.
    pub fn new<P: Into<PathBuf>>(dir: P) -> Self {
        Self { dir: dir.into() }
    }

    fn path_for(&self, receipt_hash: &str) -> PathBuf {
        self.dir.join(format!("{receipt_hash}.json"))
    }

    /// Persist `policy_json` as `{dir}/{receipt_hash}.json`.
    ///
    /// Idempotent: if a file already exists it is left unchanged. Because the
    /// file name is the receipt hash, an existing file is guaranteed to carry
    /// the same policy bundle.
    pub fn store(&self, receipt_hash: &str, policy_json: &str) -> Result<(), PolicyStoreError> {
        fs::create_dir_all(&self.dir).map_err(PolicyStoreError::Io)?;
        let path = self.path_for(receipt_hash);
        if path.exists() {
            return Ok(());
        }
        fs::write(&path, policy_json).map_err(PolicyStoreError::Io)?;
        Ok(())
    }

    /// Read a stored policy bundle. Returns `Ok(None)` if not present.
    pub fn read(&self, receipt_hash: &str) -> Result<Option<String>, PolicyStoreError> {
        let path = self.path_for(receipt_hash);
        if !path.exists() {
            return Ok(None);
        }
        let raw = fs::read_to_string(&path).map_err(PolicyStoreError::Io)?;
        Ok(Some(raw))
    }
}
