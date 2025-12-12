use greentic_component::{CompatPolicy, ComponentStore};
use tempfile::tempdir;
use tokio::runtime::Runtime;

#[test]
fn cache_hit_and_compat_fail() {
    Runtime::new().unwrap().block_on(async {
        let td = tempdir().expect("tempdir");
        let component_path = td.path().join("comp.wasm");
        std::fs::write(&component_path, b"FAKE_WASM_BYTES").expect("write component");

        let cache_dir = td.path().join("cache");
        let mut store = ComponentStore::with_cache_dir(
            Some(cache_dir.clone()),
            CompatPolicy {
                required_abi_prefix: "greentic-abi-0".to_string(),
                required_capabilities: Vec::new(),
            },
        );

        store.add_fs("fake", &component_path);

        let first = store.get("fake").await.expect("first fetch");
        assert!(
            cache_dir.exists(),
            "cache directory should exist after fetch"
        );

        let second = store.get("fake").await.expect("second fetch");
        assert_eq!(first.id, second.id);

        let mut stricter = ComponentStore::with_cache_dir(
            Some(cache_dir.clone()),
            CompatPolicy {
                required_abi_prefix: "greentic-abi-0".to_string(),
                required_capabilities: vec!["needs:x".to_string()],
            },
        );
        stricter.add_fs("fake", &component_path);

        let err = stricter.get("fake").await.expect_err("compat should fail");
        assert!(
            err.to_string().contains("Missing capabilities"),
            "compat error should mention missing capabilities: {err}"
        );
    });
}
