//! Shared deployment context primitives for Greentic runtimes.

use alloc::string::String;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub use crate::TenantCtx;

/// Cloud provider locations supported by Greentic deployments.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Cloud {
    /// Amazon Web Services.
    Aws,
    /// Google Cloud Platform.
    Gcp,
    /// Microsoft Azure.
    Azure,
    /// Hetzner Cloud.
    Hetzner,
    /// Local/self-hosted environments.
    Local,
    /// Any other cloud provider not covered above.
    Other,
}

/// Platform-level schedulers supported by Greentic deployments.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Platform {
    /// Kubernetes workloads.
    K8s,
    /// Nomad workloads.
    Nomad,
    /// Systemd services.
    Systemd,
    /// Cloudflare Workers.
    CfWorkers,
    /// AWS Lambda functions.
    Lambda,
    /// Bare-metal deployments.
    Baremetal,
    /// Any other platform not captured above.
    Other,
}

/// Deployment metadata propagated to Greentic surfaces.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DeploymentCtx {
    /// Cloud provider.
    pub cloud: Cloud,
    /// Optional region identifier (for example `us-east-1`).
    #[cfg_attr(feature = "serde", serde(default))]
    pub region: Option<String>,
    /// Platform or scheduler running the deployment.
    pub platform: Platform,
    /// Optional runtime engine backing the deployment (for example `wasmtime`).
    #[cfg_attr(feature = "serde", serde(default))]
    pub runtime: Option<String>,
}
