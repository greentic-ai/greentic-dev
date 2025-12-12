use std::collections::HashSet;

use crate::capabilities::CapabilityError;
use crate::capabilities::{
    Capabilities, FilesystemCapabilities, FilesystemMode, HostCapabilities, TelemetryScope,
    WasiCapabilities,
};
use crate::manifest::ComponentManifest;

/// Host profile describing the maximum capabilities granted to a component.
#[derive(Debug, Clone, Default)]
pub struct Profile {
    pub allowed: Capabilities,
}

impl Profile {
    pub fn new(allowed: Capabilities) -> Self {
        Self { allowed }
    }
}

pub fn enforce_capabilities(
    manifest: &ComponentManifest,
    profile: Profile,
) -> Result<(), CapabilityError> {
    ensure_wasi(&manifest.capabilities.wasi, &profile.allowed.wasi)?;
    ensure_host(&manifest.capabilities.host, &profile.allowed.host)
}

fn ensure_wasi(
    requested: &WasiCapabilities,
    allowed: &WasiCapabilities,
) -> Result<(), CapabilityError> {
    if let Some(fs) = &requested.filesystem {
        let policy = allowed.filesystem.as_ref().ok_or_else(|| {
            CapabilityError::invalid("wasi.filesystem", "filesystem access denied")
        })?;
        ensure_filesystem(fs, policy)?;
    }

    if let Some(env) = &requested.env {
        let policy = allowed
            .env
            .as_ref()
            .ok_or_else(|| CapabilityError::invalid("wasi.env", "environment access denied"))?;
        let allowed_vars: HashSet<_> = policy.allow.iter().collect();
        for var in &env.allow {
            if !allowed_vars.contains(var) {
                return Err(CapabilityError::invalid(
                    "wasi.env.allow",
                    format!("env `{var}` not permitted by profile"),
                ));
            }
        }
    }

    if requested.random && !allowed.random {
        return Err(CapabilityError::invalid(
            "wasi.random",
            "profile denies random number generation",
        ));
    }
    if requested.clocks && !allowed.clocks {
        return Err(CapabilityError::invalid(
            "wasi.clocks",
            "profile denies clock access",
        ));
    }

    Ok(())
}

fn ensure_filesystem(
    requested: &FilesystemCapabilities,
    allowed: &FilesystemCapabilities,
) -> Result<(), CapabilityError> {
    if mode_rank(&requested.mode) > mode_rank(&allowed.mode) {
        return Err(CapabilityError::invalid(
            "wasi.filesystem.mode",
            "requested mode exceeds profile allowance",
        ));
    }

    let allowed_mounts: HashSet<_> = allowed
        .mounts
        .iter()
        .map(|mount| (&mount.name, &mount.host_class, &mount.guest_path))
        .collect();
    for mount in &requested.mounts {
        let key = (&mount.name, &mount.host_class, &mount.guest_path);
        if !allowed_mounts.contains(&key) {
            return Err(CapabilityError::invalid(
                "wasi.filesystem.mounts",
                format!("mount `{}` is not available in this profile", mount.name),
            ));
        }
    }
    Ok(())
}

fn mode_rank(mode: &FilesystemMode) -> u8 {
    match mode {
        FilesystemMode::None => 0,
        FilesystemMode::ReadOnly => 1,
        FilesystemMode::Sandbox => 2,
    }
}

fn ensure_host(
    requested: &HostCapabilities,
    allowed: &HostCapabilities,
) -> Result<(), CapabilityError> {
    if let Some(secrets) = &requested.secrets {
        let policy = allowed
            .secrets
            .as_ref()
            .ok_or_else(|| CapabilityError::invalid("host.secrets", "secrets access denied"))?;
        let allowed_set: HashSet<_> = policy.required.iter().collect();
        for key in &secrets.required {
            if !allowed_set.contains(key) {
                return Err(CapabilityError::invalid(
                    "host.secrets.required",
                    format!("secret `{key}` is not available"),
                ));
            }
        }
    }

    if let Some(state) = &requested.state {
        let policy = allowed
            .state
            .as_ref()
            .ok_or_else(|| CapabilityError::invalid("host.state", "state access denied"))?;
        if state.read && !policy.read {
            return Err(CapabilityError::invalid(
                "host.state.read",
                "profile denies state reads",
            ));
        }
        if state.write && !policy.write {
            return Err(CapabilityError::invalid(
                "host.state.write",
                "profile denies state writes",
            ));
        }
    }

    ensure_io_capability(
        requested
            .messaging
            .as_ref()
            .map(|m| (m.inbound, m.outbound)),
        allowed.messaging.as_ref().map(|m| (m.inbound, m.outbound)),
        "host.messaging",
    )?;
    ensure_io_capability(
        requested.events.as_ref().map(|m| (m.inbound, m.outbound)),
        allowed.events.as_ref().map(|m| (m.inbound, m.outbound)),
        "host.events",
    )?;
    ensure_io_capability(
        requested.http.as_ref().map(|h| (h.client, h.server)),
        allowed.http.as_ref().map(|h| (h.client, h.server)),
        "host.http",
    )?;

    if let Some(telemetry) = &requested.telemetry {
        let policy = allowed
            .telemetry
            .as_ref()
            .ok_or_else(|| CapabilityError::invalid("host.telemetry", "telemetry access denied"))?;
        if !telemetry_scope_allowed(&policy.scope, &telemetry.scope) {
            return Err(CapabilityError::invalid(
                "host.telemetry.scope",
                format!(
                    "requested scope `{:?}` exceeds profile allowance `{:?}`",
                    telemetry.scope, policy.scope
                ),
            ));
        }
    }

    if let Some(iac) = &requested.iac {
        let policy = allowed
            .iac
            .as_ref()
            .ok_or_else(|| CapabilityError::invalid("host.iac", "iac access denied"))?;
        if iac.write_templates && !policy.write_templates {
            return Err(CapabilityError::invalid(
                "host.iac.write_templates",
                "profile denies template writes",
            ));
        }
        if iac.execute_plans && !policy.execute_plans {
            return Err(CapabilityError::invalid(
                "host.iac.execute_plans",
                "profile denies plan execution",
            ));
        }
    }

    Ok(())
}

fn ensure_io_capability(
    requested: Option<(bool, bool)>,
    allowed: Option<(bool, bool)>,
    label: &'static str,
) -> Result<(), CapabilityError> {
    if let Some((req_in, req_out)) = requested {
        let Some((allow_in, allow_out)) = allowed else {
            return Err(CapabilityError::invalid(
                label,
                "profile denies this capability",
            ));
        };
        if req_in && !allow_in {
            return Err(CapabilityError::invalid(
                label,
                "inbound access denied by profile",
            ));
        }
        if req_out && !allow_out {
            return Err(CapabilityError::invalid(
                label,
                "outbound access denied by profile",
            ));
        }
    }
    Ok(())
}

fn telemetry_scope_allowed(allowed: &TelemetryScope, requested: &TelemetryScope) -> bool {
    scope_rank(allowed) >= scope_rank(requested)
}

fn scope_rank(scope: &TelemetryScope) -> u8 {
    match scope {
        TelemetryScope::Tenant => 0,
        TelemetryScope::Pack => 1,
        TelemetryScope::Node => 2,
    }
}
