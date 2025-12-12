use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Provenance {
    pub builder: String,
    pub git_commit: String,
    pub toolchain: String,
    #[serde(with = "time::serde::rfc3339")]
    pub built_at_utc: OffsetDateTime,
}

impl Provenance {
    pub fn validate(&self) -> Result<(), ProvenanceError> {
        if self.builder.trim().is_empty() {
            return Err(ProvenanceError::EmptyField("builder"));
        }
        if self.toolchain.trim().is_empty() {
            return Err(ProvenanceError::EmptyField("toolchain"));
        }
        if !GIT_COMMIT_RE.is_match(&self.git_commit) {
            return Err(ProvenanceError::InvalidGit(self.git_commit.clone()));
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum ProvenanceError {
    #[error("provenance field `{0}` cannot be empty")]
    EmptyField(&'static str),
    #[error("git commit `{0}` must be lowercase hex (min 7 chars)")]
    InvalidGit(String),
}

static GIT_COMMIT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[0-9a-f]{7,40}$").expect("git commit regex compile should never fail")
});
