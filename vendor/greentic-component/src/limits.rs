use serde::{Deserialize, Serialize};
use serde_with::rust::double_option;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Limits {
    pub memory_mb: u32,
    pub wall_time_ms: u64,
    #[serde(default)]
    pub fuel: Option<u64>,
    #[serde(default)]
    pub files: Option<u32>,
}

impl Limits {
    pub fn validate(&self) -> Result<(), LimitError> {
        if self.memory_mb == 0 {
            return Err(LimitError::NonZero {
                field: "memory_mb",
                value: self.memory_mb as u128,
            });
        }
        if self.wall_time_ms == 0 {
            return Err(LimitError::NonZero {
                field: "wall_time_ms",
                value: self.wall_time_ms as u128,
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct LimitOverrides {
    #[serde(default)]
    pub memory_mb: Option<u32>,
    #[serde(default)]
    pub wall_time_ms: Option<u64>,
    #[serde(default, with = "double_option")]
    pub fuel: Option<Option<u64>>,
    #[serde(default, with = "double_option")]
    pub files: Option<Option<u32>>,
}

pub fn defaults_dev() -> Limits {
    Limits {
        memory_mb: 256,
        wall_time_ms: 30_000,
        fuel: Some(50_000),
        files: Some(128),
    }
}

pub fn merge(user: Option<&LimitOverrides>, defaults: &Limits) -> Limits {
    let mut merged = defaults.clone();
    if let Some(overrides) = user {
        if let Some(memory_mb) = overrides.memory_mb {
            merged.memory_mb = memory_mb;
        }
        if let Some(wall_time_ms) = overrides.wall_time_ms {
            merged.wall_time_ms = wall_time_ms;
        }
        if let Some(fuel) = overrides.fuel {
            merged.fuel = fuel;
        }
        if let Some(files) = overrides.files {
            merged.files = files;
        }
    }
    merged
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum LimitError {
    #[error("limit `{field}` must be greater than zero (got {value})")]
    NonZero { field: &'static str, value: u128 },
}
