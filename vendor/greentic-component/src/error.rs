use displaydoc::Display;

#[cfg(feature = "abi")]
use crate::abi::AbiError;
use crate::capabilities::CapabilityError;
#[cfg(feature = "describe")]
use crate::describe::DescribeError;
use crate::limits::LimitError;
#[cfg(feature = "loader")]
use crate::loader::LoadError;
use crate::manifest::ManifestError;
use crate::schema::SchemaIntrospectionError;
use crate::signing::SigningError;

#[derive(Debug, Display, thiserror::Error)]
pub enum ComponentError {
    /// manifest failure: {0}
    Manifest(#[from] ManifestError),
    /// schema introspection failure: {0}
    SchemaIntrospection(#[from] SchemaIntrospectionError),
    /// ABI failure: {0}
    #[cfg(feature = "abi")]
    Abi(#[from] AbiError),
    /// describe failure: {0}
    #[cfg(feature = "describe")]
    Describe(#[from] DescribeError),
    /// load failure: {0}
    #[cfg(feature = "loader")]
    Load(#[from] LoadError),
    /// capability failure: {0}
    Capability(#[from] CapabilityError),
    /// limit failure: {0}
    Limits(#[from] LimitError),
    /// signing failure: {0}
    Signing(#[from] SigningError),
    /// io failure: {0}
    Io(#[from] std::io::Error),
}

impl ComponentError {
    pub fn code(&self) -> &'static str {
        match self {
            ComponentError::Manifest(_) => "manifest-invalid",
            ComponentError::SchemaIntrospection(_) => "schema-introspection",
            #[cfg(feature = "abi")]
            ComponentError::Abi(err) => match err {
                crate::abi::AbiError::WorldMismatch { .. } => "world-mismatch",
                crate::abi::AbiError::MissingWasiTarget => "wasi-target-missing",
                _ => "abi-error",
            },
            #[cfg(feature = "describe")]
            ComponentError::Describe(err) => match err {
                crate::describe::DescribeError::NotFound(_) => "describe-missing",
                crate::describe::DescribeError::Json { .. } => "describe-invalid",
                crate::describe::DescribeError::Io { .. } => "describe-io",
                crate::describe::DescribeError::Metadata(_) => "describe-metadata",
            },
            #[cfg(feature = "loader")]
            ComponentError::Load(_) => "component-load",
            ComponentError::Capability(_) => "capability-error",
            ComponentError::Limits(_) => "limits-error",
            ComponentError::Signing(_) => "hash-mismatch",
            ComponentError::Io(_) => "io-error",
        }
    }
}
