use greentic_component::limits::{LimitError, LimitOverrides, Limits, defaults_dev, merge};

#[test]
fn merge_respects_overrides() {
    let defaults = defaults_dev();
    let overrides = LimitOverrides {
        memory_mb: Some(defaults.memory_mb * 2),
        wall_time_ms: None,
        fuel: Some(Some(0)),
        files: Some(None),
    };

    let merged = merge(Some(&overrides), &defaults);
    assert_eq!(merged.memory_mb, defaults.memory_mb * 2);
    assert_eq!(merged.wall_time_ms, defaults.wall_time_ms);
    assert_eq!(merged.fuel, Some(0));
    assert!(merged.files.is_none());
}

#[test]
fn validate_rejects_zero_limits() {
    let limits = Limits {
        memory_mb: 0,
        wall_time_ms: 10,
        fuel: None,
        files: None,
    };
    match limits.validate() {
        Err(LimitError::NonZero { field, .. }) => assert_eq!(field, "memory_mb"),
        other => panic!("expected NonZero error, got {other:?}"),
    }
}
