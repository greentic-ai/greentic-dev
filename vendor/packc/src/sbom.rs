use crate::flows::FlowAsset;
use crate::manifest::SpecBundle;
use crate::templates::TemplateAsset;
use serde::Serialize;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

#[derive(Debug, Serialize)]
pub struct CycloneDxBom {
    #[serde(rename = "bomFormat")]
    pub bom_format: &'static str,
    #[serde(rename = "specVersion")]
    pub spec_version: &'static str,
    pub version: u32,
    pub metadata: Metadata,
    pub components: Vec<Component>,
}

#[derive(Debug, Serialize)]
pub struct Metadata {
    pub timestamp: String,
    pub component: ComponentSummary,
}

#[derive(Debug, Serialize)]
pub struct ComponentSummary {
    pub name: String,
    pub version: String,
    #[serde(rename = "type")]
    pub component_type: &'static str,
}

#[derive(Debug, Serialize)]
pub struct Component {
    pub name: String,
    #[serde(rename = "type")]
    pub component_type: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hashes: Option<Vec<HashEntry>>,
}

#[derive(Debug, Serialize)]
pub struct HashEntry {
    pub alg: &'static str,
    pub content: String,
}

pub fn generate(
    spec: &SpecBundle,
    flows: &[FlowAsset],
    templates: &[TemplateAsset],
) -> CycloneDxBom {
    let timestamp = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string());

    let mut components = Vec::new();
    for flow in flows {
        components.push(Component {
            name: flow.bundle.id.clone(),
            component_type: "file",
            version: None,
            hashes: Some(vec![HashEntry {
                alg: "SHA-256",
                content: flow.sha256.clone(),
            }]),
        });
    }

    for template in templates {
        components.push(Component {
            name: template.logical_path.clone(),
            component_type: "file",
            version: None,
            hashes: Some(vec![HashEntry {
                alg: "SHA-256",
                content: template.sha256.clone(),
            }]),
        });
    }

    CycloneDxBom {
        bom_format: "CycloneDX",
        spec_version: "1.5",
        version: 1,
        metadata: Metadata {
            timestamp,
            component: ComponentSummary {
                name: spec.spec.id.clone(),
                version: spec.spec.version.clone(),
                component_type: "application",
            },
        },
        components,
    }
}
