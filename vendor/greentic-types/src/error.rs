//! Shared error types for Greentic crates.

use alloc::string::String;

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(feature = "std")]
use std::error::Error as StdError;

/// Canonical error codes used across the Greentic platform.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum ErrorCode {
    /// Unclassified error.
    Unknown,
    /// Invalid input supplied by the caller.
    InvalidInput,
    /// Required entity was not found.
    NotFound,
    /// Operation conflicts with existing data.
    Conflict,
    /// Operation timed out.
    Timeout,
    /// Caller is not authenticated.
    Unauthenticated,
    /// Caller lacks permissions.
    PermissionDenied,
    /// Requests throttled by rate limits.
    RateLimited,
    /// External dependency unavailable.
    Unavailable,
    /// Internal platform error.
    Internal,
}

/// Error type carrying a code and message.
#[derive(Debug, Error)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[error("{code:?}: {message}")]
pub struct GreenticError {
    /// Machine-readable error code.
    pub code: ErrorCode,
    /// Human-readable error message.
    pub message: String,
    /// Optional source error for debugging.
    #[cfg(feature = "std")]
    #[cfg_attr(feature = "serde", serde(skip, default = "default_source"))]
    #[cfg_attr(feature = "schemars", schemars(skip))]
    #[source]
    source: Option<Box<dyn StdError + Send + Sync>>,
}

impl GreenticError {
    /// Creates a new error with the provided code and message.
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            #[cfg(feature = "std")]
            source: None,
        }
    }

    /// Attaches a source error to the `GreenticError`.
    #[cfg(feature = "std")]
    pub fn with_source<E>(mut self, source: E) -> Self
    where
        E: StdError + Send + Sync + 'static,
    {
        self.source = Some(Box::new(source));
        self
    }
}

#[cfg(feature = "std")]
fn default_source() -> Option<Box<dyn StdError + Send + Sync>> {
    None
}

#[cfg(feature = "time")]
impl From<time::error::ComponentRange> for GreenticError {
    fn from(err: time::error::ComponentRange) -> Self {
        Self::new(ErrorCode::InvalidInput, err.to_string())
    }
}

#[cfg(feature = "time")]
impl From<time::error::Parse> for GreenticError {
    fn from(err: time::error::Parse) -> Self {
        Self::new(ErrorCode::InvalidInput, err.to_string())
    }
}

#[cfg(feature = "uuid")]
impl From<uuid::Error> for GreenticError {
    fn from(err: uuid::Error) -> Self {
        Self::new(ErrorCode::InvalidInput, err.to_string())
    }
}

/// Convenient result alias for Greentic APIs.
pub type GResult<T> = core::result::Result<T, GreenticError>;
