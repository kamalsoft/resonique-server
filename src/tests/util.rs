use std::fs;

#[test]
fn loads_valid_configuration() {
    let path = std::env::temp_dir().join("resonique-valid-config.json");

    fs::write(
        &path,
        r#"{
            "storage_root": "/tmp/resonique",
            "collections": []
        }"#,
    )
    .unwrap();

    let config = crate::util::load_config(path.to_str().unwrap()).unwrap();

    assert_eq!(config.storage_root, "/tmp/resonique");
    assert_eq!(config.node_id, "node-0");

    let _ = fs::remove_file(path);
}

#[test]
fn rejects_missing_configuration() {
    let result = crate::util::load_config("/tmp/resonique-config-that-does-not-exist.json");

    assert!(result.is_err());
}

#[test]
fn rejects_invalid_configuration_json() {
    let path = std::env::temp_dir().join("resonique-invalid-config.json");
    fs::write(&path, "{invalid json").unwrap();

    assert!(crate::util::load_config(path.to_str().unwrap()).is_err());

    let _ = fs::remove_file(path);
}

#[test]
fn initializes_logging_without_panicking() {
    crate::util::init_logging();
}
