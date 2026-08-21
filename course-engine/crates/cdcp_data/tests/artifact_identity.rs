use cdcp_data::verify_identity_manifest;
use std::fs;
use std::path::PathBuf;

fn engine_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("cdcp_data is crates/cdcp_data")
        .to_path_buf()
}

#[test]
fn shipped_web_identity_manifest_matches_bytes() {
    let root = engine_root();
    let manifest = fs::read_to_string(root.join("web/data/artifact_identity.toml")).unwrap();
    let actual = verify_identity_manifest(&root, &manifest).unwrap();
    assert_eq!(actual.len(), 4);
}

#[test]
fn shipped_golden_identity_manifest_matches_bytes() {
    let root = engine_root();
    let manifest = fs::read_to_string(root.join("goldens/artifact_identity.toml")).unwrap();
    let actual = verify_identity_manifest(&root, &manifest).unwrap();
    assert_eq!(actual.len(), 4);
}
