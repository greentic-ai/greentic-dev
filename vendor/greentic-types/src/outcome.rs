//! Normalized execution outcomes.

use alloc::string::String;
use alloc::vec::Vec;

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::ErrorCode;

/// Outcome of a node, adapter, or tool invocation.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(serialize = "T: Serialize", deserialize = "T: DeserializeOwned"))
)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "schemars", schemars(bound = "T: JsonSchema"))]
pub enum Outcome<T> {
    /// Execution finished with a value.
    Done(T),
    /// Execution is pending external input.
    Pending {
        /// Human-readable wait reason.
        reason: String,
        /// Optional list of expected input channels or identifiers.
        #[cfg_attr(
            feature = "serde",
            serde(default, skip_serializing_if = "Option::is_none")
        )]
        expected_input: Option<Vec<String>>,
    },
    /// Execution produced an error.
    Error {
        /// Machine-readable error code.
        code: ErrorCode,
        /// Human-readable message.
        message: String,
    },
}

impl<T> Outcome<T> {
    /// Returns `true` when the outcome is [`Outcome::Done`].
    pub fn is_done(&self) -> bool {
        matches!(self, Self::Done(_))
    }

    /// Returns `true` when the outcome is [`Outcome::Pending`].
    pub fn is_pending(&self) -> bool {
        matches!(self, Self::Pending { .. })
    }

    /// Returns `true` when the outcome is [`Outcome::Error`].
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error { .. })
    }

    /// Maps a [`Outcome::Done`] value with the provided function.
    pub fn map<U, F>(self, mut f: F) -> Outcome<U>
    where
        F: FnMut(T) -> U,
    {
        match self {
            Outcome::Done(inner) => Outcome::Done(f(inner)),
            Outcome::Pending {
                reason,
                expected_input,
            } => Outcome::Pending {
                reason,
                expected_input,
            },
            Outcome::Error { code, message } => Outcome::Error { code, message },
        }
    }
}
