//! Resource binding hints shared across Greentic tooling.
//! This module codifies the canonical host binding invariants (network allowlists, secrets,
//! environment passthrough, and MCP server stubs) so packs, hints generators, and runtime hosts
//! agree on the same schema.

extern crate alloc;

/// Shared binding hints emitted by pack generators and consumed by the runner host.
pub mod hints {
    use alloc::{string::String, vec::Vec};

    #[cfg(feature = "schemars")]
    use schemars::JsonSchema;
    #[cfg(feature = "serde")]
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[cfg_attr(feature = "schemars", derive(JsonSchema))]
    /// Hints that describe the binding requirements a pack emits for the runner.
    pub struct BindingsHints {
        /// Explicit network endpoints that the pack plans to call. Default is deny-all.
        #[cfg_attr(feature = "serde", serde(default))]
        pub network: NetworkHints,
        /// Secrets referenced by the pack. Only listed secrets are allowed; others are denied.
        #[cfg_attr(feature = "serde", serde(default))]
        pub secrets: SecretsHints,
        /// Environment variables the pack needs surfaced. Each listed key is forwarded through
        /// the runner; unspecified keys are not available to the host.
        #[cfg_attr(feature = "serde", serde(default))]
        pub env: EnvHints,
        /// MCP servers (name + endpoint) referenced by the flows. These entries let the runner
        /// prepare tool bindings before execution.
        #[cfg_attr(feature = "serde", serde(default))]
        pub mcp: McpHints,
    }

    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[cfg_attr(feature = "schemars", derive(JsonSchema))]
    /// Network-specific allowlists declared by a pack.
    pub struct NetworkHints {
        /// Allowlisted host:port entries required by the flows.
        #[cfg_attr(feature = "serde", serde(default))]
        pub allow: Vec<String>,
    }

    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[cfg_attr(feature = "schemars", derive(JsonSchema))]
    /// Secrets referenced by the pack that the runner must provide.
    pub struct SecretsHints {
        /// Secrets that flows declare as required. The host must supply these keys.
        #[cfg_attr(feature = "serde", serde(default))]
        pub required: Vec<String>,
    }

    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[cfg_attr(feature = "schemars", derive(JsonSchema))]
    /// Environment variables that need to be forwarded to the pack.
    pub struct EnvHints {
        /// Environment variables the pack expects the host to forward.
        #[cfg_attr(feature = "serde", serde(default))]
        pub passthrough: Vec<String>,
    }

    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[cfg_attr(feature = "schemars", derive(JsonSchema))]
    /// Metadata for MCP servers that will be bound into the runtime toolkit.
    pub struct McpHints {
        /// Stubbed MCP tool servers referenced by the flows.
        #[cfg_attr(feature = "serde", serde(default))]
        pub servers: Vec<McpServer>,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[cfg_attr(feature = "schemars", derive(JsonSchema))]
    /// Descriptor for a single MCP server the host must expose.
    pub struct McpServer {
        /// Logical name referenced by flows.
        pub name: String,
        /// Transport mechanism (e.g. `http`, `grpc`, etc.).
        pub transport: String,
        /// Endpoint exposed by the host for this MCP server.
        pub endpoint: String,
        /// Optional capability tags; useful when the runner enforces tool-specific policies.
        #[cfg_attr(feature = "serde", serde(default))]
        pub caps: Vec<String>,
    }
}
