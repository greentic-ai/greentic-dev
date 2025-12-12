//! State key and JSON pointer helpers.

use alloc::string::String;
use alloc::vec::Vec;

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Unique key referencing a persisted state blob.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct StateKey(pub String);

impl StateKey {
    /// Returns the key as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Creates a new state key from the provided value.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl From<String> for StateKey {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for StateKey {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl core::fmt::Display for StateKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Hierarchical pointer addressing a nested state path.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct StatePath {
    /// Pointer segments making up the path.
    #[cfg_attr(feature = "serde", serde(default))]
    pub segments: Vec<String>,
}

impl StatePath {
    /// Creates an empty root path.
    pub fn root() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    /// Pushes a new segment to the path.
    pub fn push(&mut self, segment: impl Into<String>) {
        self.segments.push(segment.into());
    }

    /// Returns a JSON pointer representation (`/a/b/c`).
    pub fn to_pointer(&self) -> String {
        if self.segments.is_empty() {
            return "/".to_owned();
        }

        let mut pointer = String::new();
        for segment in &self.segments {
            pointer.push('/');
            pointer.push_str(&escape_segment(segment));
        }
        pointer
    }

    /// Creates a path from a JSON pointer representation.
    pub fn from_pointer(pointer: &str) -> Self {
        if pointer == "/" || pointer.is_empty() {
            return Self::root();
        }

        let segments = pointer
            .split('/')
            .skip_while(|s| s.is_empty())
            .map(unescape_segment)
            .collect::<Vec<_>>();

        Self { segments }
    }
}

impl Default for StatePath {
    fn default() -> Self {
        Self::root()
    }
}

fn escape_segment(segment: &str) -> String {
    segment.replace('~', "~0").replace('/', "~1")
}

fn unescape_segment(segment: &str) -> String {
    segment.replace("~1", "/").replace("~0", "~")
}
