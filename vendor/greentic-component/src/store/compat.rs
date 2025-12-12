use super::MetaInfo;
use thiserror::Error;

#[derive(Clone, Debug, Default)]
pub struct CompatPolicy {
    pub required_abi_prefix: String,
    pub required_capabilities: Vec<String>,
}

#[derive(Debug, Error)]
pub enum CompatError {
    #[error("ABI mismatch: required prefix '{required}', got '{got}'")]
    AbiMismatch { required: String, got: String },
    #[error("Missing capabilities: {0:?}")]
    MissingCapabilities(Vec<String>),
}

pub fn check(policy: &CompatPolicy, meta: &MetaInfo) -> Result<(), CompatError> {
    if !policy.required_abi_prefix.is_empty()
        && !meta.abi_version.starts_with(&policy.required_abi_prefix)
    {
        return Err(CompatError::AbiMismatch {
            required: policy.required_abi_prefix.clone(),
            got: meta.abi_version.clone(),
        });
    }

    if !policy.required_capabilities.is_empty() {
        let mut missing = Vec::new();
        for capability in &policy.required_capabilities {
            if !meta.capabilities.iter().any(|c| c == capability) {
                missing.push(capability.clone());
            }
        }
        if !missing.is_empty() {
            return Err(CompatError::MissingCapabilities(missing));
        }
    }

    Ok(())
}
