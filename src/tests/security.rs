#[cfg(test)]
mod tests {
    use crate::storage::StorageEngine;

    #[test]
    fn test_path_traversal_prevention() {
        let engine = StorageEngine::new("/tmp/db_root").unwrap();

        // Escape attempt
        let resolved = engine.resolve("../../etc/passwd");

        // The resolved path must remain inside the storage root /tmp/db_root
        assert!(resolved.starts_with("/tmp/db_root"));
        assert_ne!(resolved, std::path::Path::new("/etc/passwd"));
    }
}
