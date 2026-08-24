#[test]
fn server_run_function_is_exposed() {
    let _run = crate::server::run;
}

#[test]
fn builds_manager_from_configuration() {
    let root = std::env::temp_dir().join(format!("resonique-server-test-{}", std::process::id()));

    let config = crate::model::ServerConfig {
        storage_root: root.to_string_lossy().into_owned(),
        node_id: "node-1".into(),
        nodes: vec![],
        collections: vec![],
    };

    let manager = crate::server::build_manager(&config).unwrap();

    assert_eq!(manager.node_id, "node-1");
    assert!(manager.collections.is_empty());

    let _ = std::fs::remove_dir_all(root);
}
