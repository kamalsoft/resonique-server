use crate::storage::wal::Wal;
use std::fs::{self, OpenOptions};
use std::io::Write;

fn wal_path(name: &str) -> (std::path::PathBuf, std::path::PathBuf) {
    let root = std::env::temp_dir().join(format!("resonique-wal-{}-{}", std::process::id(), name));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    (root.clone(), root.join("records.wal"))
}

#[test]
fn replay_missing_and_blank_files() {
    let (root, path) = wal_path("blank");

    assert!(Wal::replay(&path).unwrap().is_empty());
    fs::write(&path, "\n\n").unwrap();
    assert!(Wal::replay(&path).unwrap().is_empty());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn rejects_complete_invalid_record() {
    let (root, path) = wal_path("invalid");

    fs::write(&path, "{not-json}\n").unwrap();

    assert!(Wal::replay(&path).is_err());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn ignores_incomplete_trailing_record() {
    let (root, path) = wal_path("truncated");
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)
        .unwrap();

    writeln!(file, r#"{{"vector_id":1,"payload":[1,2,3,4]}}"#).unwrap();
    write!(file, r#"{{"vector_id":2"#).unwrap();

    let records = Wal::replay(&path).unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].vector_id, 1);

    let _ = fs::remove_dir_all(root);
}
