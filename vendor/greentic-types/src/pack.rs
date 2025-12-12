//! Pack reference metadata.

use alloc::string::String;
use alloc::vec::Vec;

use semver::Version;

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "serde")]
use serde_with::serde_as;

/// Reference to a pack stored in an OCI registry.
#[cfg_attr(feature = "serde", serde_as)]
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct PackRef {
    /// OCI reference pointing to the pack.
    pub oci_url: String,
    /// Semantic version of the pack.
    #[cfg_attr(
        feature = "serde",
        serde_as(as = "serde_with::formats::DisplayFromStr")
    )]
    #[cfg_attr(
        feature = "schemars",
        schemars(with = "String", description = "SemVer version")
    )]
    pub version: Version,
    /// Content digest of the pack.
    pub digest: String,
    /// Optional detached signatures.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub signatures: Vec<Signature>,
}

impl PackRef {
    /// Creates a new pack reference.
    pub fn new(oci_url: impl Into<String>, version: Version, digest: impl Into<String>) -> Self {
        Self {
            oci_url: oci_url.into(),
            version,
            digest: digest.into(),
            signatures: Vec::new(),
        }
    }
}

/// Detached signature accompanying a [`PackRef`].
#[cfg_attr(feature = "serde", serde_as)]
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct Signature {
    /// Identifier of the public key.
    pub key_id: String,
    /// Signature algorithm (for example `ed25519`).
    pub algorithm: SignatureAlgorithm,
    /// Raw signature bytes (base64 encoded when serialized).
    #[cfg_attr(feature = "serde", serde_as(as = "serde_with::base64::Base64"))]
    pub signature: Vec<u8>,
}

impl Signature {
    /// Creates a new signature entry.
    pub fn new(
        key_id: impl Into<String>,
        algorithm: SignatureAlgorithm,
        signature: Vec<u8>,
    ) -> Self {
        Self {
            key_id: key_id.into(),
            algorithm,
            signature,
        }
    }
}

/// Supported signature algorithms for packs.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum SignatureAlgorithm {
    /// Ed25519 signatures.
    Ed25519,
    /// Other algorithms identified by name.
    Other(String),
}
