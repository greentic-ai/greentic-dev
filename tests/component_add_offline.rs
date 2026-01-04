use greentic_dev::component_add::run_component_add;
use greentic_dev::pack_init::PackInitIntent;
use once_cell::sync::Lazy;
use std::fs;
use std::sync::Mutex;

static WORKDIR_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

fn set_env(key: &str, value: &str) {
    unsafe { std::env::set_var(key, value) }
}

fn remove_env(key: &str) {
    unsafe { std::env::remove_var(key) }
}

#[test]
fn component_add_uses_stub_when_offline_and_config_absent() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().to_path_buf();
    let artifact = root.join("artifact.wasm");
    fs::write(&artifact, b"00").unwrap();

    let stub = serde_json::json!({
        "artifact_path": artifact.display().to_string(),
        "digest": "sha256:stub"
    });
    let stub_path = root.join("stub.json");
    fs::write(&stub_path, serde_json::to_string(&stub).unwrap()).unwrap();

    let _guard = WORKDIR_LOCK.lock().unwrap();
    let prev_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&root).unwrap();

    let prev_offline = std::env::var("GREENTIC_DEV_OFFLINE").ok();
    let prev_stub = std::env::var("GREENTIC_DEV_RESOLVE_STUB").ok();
    let prev_cfg = std::env::var("GREENTIC_DEV_CONFIG_FILE").ok();
    let prev_profile = std::env::var("GREENTIC_DISTRIBUTOR_PROFILE").ok();

    set_env("GREENTIC_DEV_OFFLINE", "1");
    set_env(
        "GREENTIC_DEV_RESOLVE_STUB",
        stub_path.to_string_lossy().as_ref(),
    );
    remove_env("GREENTIC_DEV_CONFIG_FILE");
    remove_env("GREENTIC_DISTRIBUTOR_PROFILE");

    let cache_dir = run_component_add("component://greentic/example@^1", None, PackInitIntent::Dev)
        .expect("stubbed resolve should succeed offline");
    assert!(cache_dir.exists(), "cache dir should exist");

    // restore env and cwd
    if let Some(val) = prev_offline {
        set_env("GREENTIC_DEV_OFFLINE", &val);
    } else {
        remove_env("GREENTIC_DEV_OFFLINE");
    }
    if let Some(val) = prev_stub {
        set_env("GREENTIC_DEV_RESOLVE_STUB", &val);
    } else {
        remove_env("GREENTIC_DEV_RESOLVE_STUB");
    }
    if let Some(val) = prev_cfg {
        set_env("GREENTIC_DEV_CONFIG_FILE", &val);
    } else {
        remove_env("GREENTIC_DEV_CONFIG_FILE");
    }
    if let Some(val) = prev_profile {
        set_env("GREENTIC_DISTRIBUTOR_PROFILE", &val);
    } else {
        remove_env("GREENTIC_DISTRIBUTOR_PROFILE");
    }
    std::env::set_current_dir(prev_dir).unwrap();
}
