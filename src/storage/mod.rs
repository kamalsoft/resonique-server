pub mod index;
pub mod search;
pub mod segment;
pub mod wal;

use anyhow::Result;
use std::path::{Path, PathBuf};

/// The StorageEngine is responsible for managing the root directory
/// where all collections, partitions, segments, and WAL files live.
pub struct StorageEngine {
    root: PathBuf,
}

impl StorageEngine {
    /// Create a new storage engine pointing at a directory path.
    pub fn new(root: impl Into<PathBuf>) -> Result<Self> {
        Ok(Self { root: root.into() })
    }

    /// Ensure the storage root exists.
    pub fn init(&self) -> Result<()> {
        std::fs::create_dir_all(&self.root)?;
        Ok(())
    }

    /// Resolve a path inside the storage root, filtering out parent directory components.
    pub fn resolve(&self, path: impl AsRef<Path>) -> PathBuf {
        let mut clean = PathBuf::new();

        for component in path.as_ref().components() {
            if let std::path::Component::Normal(component) = component {
                clean.push(component);
            }
        }

        self.root.join(clean)
    }
}
