#![cfg(feature = "serde")]

use greentic_types::{
    DefaultPipeline, DidContext, DidService, DistributorTarget, EnabledPacks,
    IdentityProviderOption, RepoAuth, RepoConfigFeatures, RepoSkin, RepoSkinLayout, RepoSkinLinks,
    RepoSkinTheme, RepoTenantConfig, RepoWorkerPanel, StoreTarget, TenantDidDocument,
    VerificationMethod,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::json;
use std::collections::BTreeMap;

fn assert_roundtrip<T>(value: &T)
where
    T: Serialize + DeserializeOwned + PartialEq + core::fmt::Debug,
{
    let json = serde_json::to_string_pretty(value).expect("serialize");
    let roundtrip: T = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(&roundtrip, value, "{json}");
}

#[test]
fn repo_skin_roundtrip() {
    let skin = RepoSkin {
        tenant_id: "tenant-1".into(),
        tenant_name: Some("Nutanix".into()),
        product_name: Some("Greentic Repo".into()),
        theme: RepoSkinTheme {
            logo_url: "https://cdn.greentic.ai/logo.svg".into(),
            favicon_url: Some("https://cdn.greentic.ai/favicon.ico".into()),
            hero_image_url: Some("https://cdn.greentic.ai/hero.png".into()),
            primary_color: "#00cc88".into(),
            accent_color: "#ff9900".into(),
            background_color: Some("#0b1021".into()),
            background_gradient: Some("linear-gradient(#0b1021,#0f1630)".into()),
            font_family: Some("Inter, sans-serif".into()),
            success_color: Some("#1dd1a1".into()),
            warning_color: Some("#feca57".into()),
            danger_color: Some("#ff6b6b".into()),
        },
        layout: Some(RepoSkinLayout {
            show_dashboard: true,
            show_repositories: true,
            show_pipeline: false,
            show_packs: true,
            show_trust_access: true,
            show_audit_compliance: false,
            show_admin_config: Some(true),
            show_hero_band: Some(true),
            hero_title: Some("Secure your supply chain".into()),
            hero_subtitle: Some("Multi-tenant, multi-region".into()),
        }),
        worker_panel: Some(RepoWorkerPanel {
            enabled: true,
            title: Some("Repo Assistant".into()),
            default_open: Some(true),
            position: Some("right".into()),
        }),
        links: Some(RepoSkinLinks {
            docs_url: Some("https://docs.greentic.ai".into()),
            support_url: Some("https://support.greentic.ai".into()),
            status_url: Some("https://status.greentic.ai".into()),
        }),
    };

    assert_roundtrip(&skin);
}

#[test]
fn repo_auth_roundtrip() {
    let providers = vec![
        IdentityProviderOption {
            id: "github-enterprise-login".into(),
            kind: Some("identity-provider".into()),
            label: "GitHub Enterprise".into(),
            icon: Some("github".into()),
            button_style: Some("dark".into()),
            order: Some(1),
            login_url: "https://auth.greentic.ai/github".into(),
            description: Some("Sign in with GitHub".into()),
            recommended: Some(true),
        },
        IdentityProviderOption {
            id: "azure-ad".into(),
            kind: None,
            label: "Azure AD".into(),
            icon: Some("azure".into()),
            button_style: None,
            order: Some(2),
            login_url: "https://auth.greentic.ai/azure".into(),
            description: None,
            recommended: None,
        },
    ];

    let auth = RepoAuth {
        tenant_id: "tenant-1".into(),
        identity_providers: providers,
    };

    assert_roundtrip(&auth);
}

#[test]
fn repo_tenant_config_roundtrip() {
    let mut handlers = BTreeMap::new();
    handlers.insert("repositories".into(), "repo-ui-repositories".into());
    handlers.insert("trust".into(), "repo-ui-advanced-trust".into());

    let config = RepoTenantConfig {
        tenant_id: "tenant-1".into(),
        enabled_tabs: vec![
            "dashboard".into(),
            "repositories".into(),
            "pipeline".into(),
            "packs".into(),
        ],
        enabled_packs: EnabledPacks {
            identity_providers: Some(vec!["github-enterprise-login".into()]),
            source_providers: Some(vec!["github-enterprise".into()]),
            scanners: Some(vec!["trivy".into()]),
            signing: Some(vec!["cosign".into()]),
            attestation: Some(vec!["in-toto".into()]),
            policy_engines: Some(vec!["rego-engine".into()]),
            oci_providers: Some(vec!["ecr".into()]),
        },
        default_pipeline: Some(DefaultPipeline {
            scanners: Some(vec!["trivy".into()]),
            signing: Some("cosign".into()),
            attestation: Some("in-toto".into()),
            policy_engine: Some("rego-engine".into()),
            oci_provider: Some("ecr".into()),
        }),
        stores: Some(vec![StoreTarget {
            id: "primary-store".into(),
            label: "Primary Store".into(),
            url: "https://store.greentic.ai".into(),
            description: Some("Public artifact store".into()),
        }]),
        distributors: Some(vec![DistributorTarget {
            id: "edge-distributor".into(),
            label: "Edge Distributor".into(),
            url: "https://distributor.greentic.ai".into(),
            description: Some("Edge locations".into()),
        }]),
        features: Some(RepoConfigFeatures {
            allow_manual_approve: Some(true),
            show_advanced_scan_views: Some(false),
            show_experimental_modules: Some(true),
        }),
        page_handlers: Some(handlers),
    };

    assert_roundtrip(&config);
}

#[test]
fn tenant_did_document_roundtrip() {
    let doc_single = TenantDidDocument {
        raw_context: Some(DidContext::Single("https://www.w3.org/ns/did/v1".into())),
        id: "did:web:repos.did.greentic.ai:tenants:tenant-1".into(),
        verification_method: Some(vec![VerificationMethod {
            id: "#key-1".into(),
            r#type: "JsonWebKey2020".into(),
            controller: "did:web:repos.did.greentic.ai:tenants:tenant-1".into(),
            public_key_jwk: Some(json!({"kty": "EC", "crv": "P-256"})),
            public_key_multibase: None,
        }]),
        authentication: Some(vec!["#key-1".into()]),
        service: vec![DidService {
            id: "#repo-api".into(),
            r#type: "RepoApi".into(),
            service_endpoint: "https://repo.greentic.ai/api".into(),
        }],
    };

    assert_eq!(doc_single.context(), vec!["https://www.w3.org/ns/did/v1"]);
    assert_roundtrip(&doc_single);

    let doc_multi = TenantDidDocument {
        raw_context: Some(DidContext::Multiple(vec![
            "https://www.w3.org/ns/did/v1".into(),
            "https://greentic.ai/did/v1".into(),
        ])),
        service: doc_single.service.clone(),
        ..doc_single
    };

    assert_eq!(
        doc_multi.context(),
        vec!["https://www.w3.org/ns/did/v1", "https://greentic.ai/did/v1"]
    );
    assert_roundtrip(&doc_multi);
}
