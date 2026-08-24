use crate::model::NodeConfig;
use crate::tests::helpers::manager;

#[test]
fn routes_owned_partition_and_rejects_unowned_partition() {
    let (manager, root) = manager();
    let mut manager = manager.blocking_lock();

    assert_eq!(manager.route_partition("default", 1).unwrap().name, "p0");
    assert!(manager.route_partition("missing", 1).is_err());

    manager.collections.get_mut("default").unwrap().partitions[0].node_id = "node-1".into();

    assert!(manager.route_partition("default", 1).is_err());

    drop(manager);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn resolves_partition_owner() {
    let (manager, root) = manager();
    let mut manager = manager.blocking_lock();

    manager.set_nodes(vec![NodeConfig {
        id: "node-0".into(),
        address: "127.0.0.1:3000".into(),
    }]);

    assert_eq!(
        manager.partition_owner("default", 10).unwrap(),
        ("node-0".into(), "127.0.0.1:3000".into())
    );

    assert!(manager.partition_owner("missing", 1).is_err());

    let _ = std::fs::remove_dir_all(root);
}
