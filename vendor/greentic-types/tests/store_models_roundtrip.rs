#![cfg(feature = "serde")]

use greentic_types::{
    ArtifactSelector, BundleSpec, CapabilityMap, Collection, ConnectionKind, DesiredState,
    DesiredStateExportSpec, DesiredSubscriptionEntry, Environment, LayoutSection,
    LayoutSectionKind, PlanLimits, PriceModel, ProductOverride, StoreFront, StorePlan,
    StoreProduct, StoreProductKind, Subscription, SubscriptionStatus, Theme, VersionStrategy,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::json;
use std::collections::BTreeMap;

fn map(value: serde_json::Value) -> BTreeMap<String, serde_json::Value> {
    serde_json::from_value(value).expect("object to map")
}

fn assert_roundtrip<T>(value: &T)
where
    T: Serialize + DeserializeOwned + PartialEq + core::fmt::Debug,
{
    let json = serde_json::to_string_pretty(value).expect("serialize");
    let roundtrip: T = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(&roundtrip, value, "{json}");
}

#[test]
fn storefront_and_theme_roundtrip() {
    let theme = Theme {
        primary_color: "#000000".into(),
        secondary_color: "#111111".into(),
        accent_color: "#00ff99".into(),
        background_color: "#ffffff".into(),
        text_color: "#000000".into(),
        font_family: "CustomSans".into(),
        logo_url: Some("https://example.test/logo.png".into()),
        favicon_url: None,
        hero_image_url: None,
        hero_title: Some("Welcome".into()),
        hero_subtitle: Some("Greentic Store".into()),
        card_radius: Some(8),
        card_elevation: Some(2),
        button_style: Some("rounded".into()),
    };

    let collections = vec![Collection {
        id: "col-1".parse().unwrap(),
        storefront_id: "storefront-1".parse().unwrap(),
        title: "Featured".into(),
        product_ids: vec!["prod-1".parse().unwrap()],
        slug: Some("featured".into()),
        description: None,
        sort_order: 0,
        metadata: map(json!({"layout": "grid"})),
    }];

    let sections = vec![LayoutSection {
        id: "hero-1".into(),
        kind: LayoutSectionKind::Hero,
        collection_id: None,
        title: Some("Hero".into()),
        subtitle: Some("Subtitle".into()),
        sort_order: 0,
        metadata: map(json!({"cta": "Get started"})),
    }];

    let overrides = vec![ProductOverride {
        storefront_id: "storefront-1".parse().unwrap(),
        product_id: "prod-1".parse().unwrap(),
        display_name: Some("Featured Product".into()),
        short_description: Some("Short desc".into()),
        badges: vec!["new".into()],
        metadata: map(json!({"color": "green"})),
    }];

    let storefront = StoreFront {
        id: "storefront-1".parse().unwrap(),
        slug: "greentic".into(),
        name: "Greentic Default".into(),
        theme,
        sections,
        collections,
        overrides,
        worker_id: Some("storefront-worker".into()),
        metadata: map(json!({"brand": "greentic"})),
    };

    assert_roundtrip(&storefront);
}

#[test]
fn store_product_and_subscription_roundtrip() {
    let mut capabilities = CapabilityMap::default();
    capabilities.insert("scan".into(), vec!["sast".into(), "deps".into()]);

    let product = StoreProduct {
        id: "prod-1".parse().unwrap(),
        kind: StoreProductKind::Component,
        name: "Scanner".into(),
        slug: "scanner".into(),
        description: "Security scanner".into(),
        source_repo: "repo-scanner".parse().unwrap(),
        component_ref: Some("component.scan".parse().unwrap()),
        pack_ref: None,
        category: Some("security".into()),
        tags: vec!["scan".into(), "security".into()],
        capabilities,
        version_strategy: VersionStrategy::Fixed {
            version: "1.0.0".into(),
        },
        default_plan_id: Some("plan-free".parse().unwrap()),
        is_free: true,
        metadata: map(json!({"ui_icon": "shield"})),
    };

    let plan = StorePlan {
        id: "plan-free".parse().unwrap(),
        name: "Free".into(),
        description: "Free plan".into(),
        price_model: PriceModel::Free,
        limits: PlanLimits {
            max_environments: Some(5),
            max_subscriptions: None,
            monthly_units_included: None,
            metadata: map(json!({"note": "beta"})),
        },
        tags: vec!["free".into()],
        metadata: map(json!({})),
    };

    let subscription = Subscription {
        id: "sub-1".parse().unwrap(),
        tenant_ctx: greentic_types::TenantCtx::new(
            "prod".parse().unwrap(),
            "tenant-1".parse().unwrap(),
        ),
        product_id: product.id.clone(),
        plan_id: plan.id.clone(),
        environment_ref: Some("env-1".parse().unwrap()),
        distributor_ref: Some("dist-1".parse().unwrap()),
        status: SubscriptionStatus::Active,
        metadata: map(json!({"priority": "high"})),
    };

    assert_roundtrip(&product);
    assert_roundtrip(&plan);
    assert_roundtrip(&subscription);
}

#[test]
fn desired_state_and_bundle_roundtrip() {
    let desired_entry = DesiredSubscriptionEntry {
        selector: ArtifactSelector::Component("component.scan".parse().unwrap()),
        version_strategy: VersionStrategy::Latest,
        config_overrides: map(json!({"setting": true})),
        policy_tags: vec!["strict".into()],
        metadata: map(json!({})),
    };

    let desired_state = DesiredState {
        tenant: greentic_types::TenantCtx::new(
            "prod".parse().unwrap(),
            "tenant-1".parse().unwrap(),
        ),
        environment_ref: "env-1".parse().unwrap(),
        entries: vec![desired_entry],
        version: 1,
        metadata: map(json!({"channel": "stable"})),
    };

    let bundle = BundleSpec {
        bundle_id: "bundle-1".parse().unwrap(),
        tenant: desired_state.tenant.clone(),
        environment_ref: desired_state.environment_ref.clone(),
        desired_state_version: desired_state.version,
        artifact_refs: vec!["artifact-1".parse().unwrap()],
        metadata_refs: vec!["meta-1".parse().unwrap()],
        additional_metadata: map(json!({"signed": true})),
    };

    let export_spec = DesiredStateExportSpec {
        tenant: desired_state.tenant.clone(),
        environment_ref: desired_state.environment_ref.clone(),
        desired_state_version: desired_state.version,
        include_artifacts: true,
        include_metadata: true,
        metadata: map(json!({"format": "bundle"})),
    };

    assert_roundtrip(&desired_state);
    assert_roundtrip(&bundle);
    assert_roundtrip(&export_spec);
}

#[test]
fn distribution_bundle_spec_roundtrip() {
    let tenant =
        greentic_types::TenantCtx::new("prod".parse().unwrap(), "tenant-2".parse().unwrap());
    let bundle = BundleSpec {
        bundle_id: "bundle-dist-1".parse().unwrap(),
        tenant,
        environment_ref: "env-dist".parse().unwrap(),
        desired_state_version: 7,
        artifact_refs: vec![
            "artifact-runner".parse().unwrap(),
            "artifact-components".parse().unwrap(),
        ],
        metadata_refs: vec!["sbom-1".parse().unwrap(), "attestation-1".parse().unwrap()],
        additional_metadata: map(json!({
            "pack_kind": "distribution-bundle",
            "notes": "offline rollout pack"
        })),
    };

    assert_roundtrip(&bundle);
}

#[test]
fn environment_roundtrip() {
    let env = Environment {
        id: "env-1".parse().unwrap(),
        tenant: greentic_types::TenantCtx::new(
            "prod".parse().unwrap(),
            "tenant-1".parse().unwrap(),
        ),
        distributor_ref: "dist-1".parse().unwrap(),
        name: "Primary".into(),
        connection_kind: ConnectionKind::Online,
        labels: BTreeMap::from([("region".into(), "eu-west".into())]),
        metadata: map(json!({"notes": "primary"})),
    };

    assert_roundtrip(&env);
}

#[test]
fn rollout_status_roundtrip() {
    let status = greentic_types::RolloutStatus {
        environment_ref: "env-1".parse().unwrap(),
        desired_state_version: Some(2),
        state: greentic_types::RolloutState::InProgress,
        bundle_id: Some("bundle-1".parse().unwrap()),
        message: Some("deploying".into()),
        metadata: map(json!({"wave": 1})),
    };

    assert_roundtrip(&status);
}

#[test]
fn version_strategy_compat() {
    let latest_json = "\"latest\"";
    let latest: VersionStrategy = serde_json::from_str(latest_json).expect("latest");
    assert!(matches!(latest, VersionStrategy::Latest));

    let pinned_json = r#"{"pinned":{"requirement":"^1.2"}}"#;
    let pinned: VersionStrategy = serde_json::from_str(pinned_json).expect("pinned");
    match pinned {
        VersionStrategy::Pinned { requirement } => {
            assert_eq!(requirement.to_string(), "^1.2")
        }
        other => panic!("unexpected variant {other:?}"),
    }

    let fixed_json = r#"{"kind":"fixed","version":"1.2.3"}"#;
    let fixed: VersionStrategy = serde_json::from_str(fixed_json).expect("fixed");
    assert!(matches!(fixed, VersionStrategy::Fixed { version } if version == "1.2.3"));
}
