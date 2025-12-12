#![cfg(feature = "cli")]

use std::env;
use std::io;
use std::path::Path;
use std::process::{Command, Stdio};

use serde::Serialize;

use crate::scaffold::engine::ScaffoldOutcome;

const DEFAULT_GIT_NAME: &str = "Greentic Scaffold";
const DEFAULT_GIT_EMAIL: &str = "builders@greentic.ai";

#[derive(Debug, Serialize)]
pub struct PostInitReport {
    pub git: GitInitReport,
    pub next_steps: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<PostHookEvent>,
}

#[derive(Debug, Serialize)]
pub struct PostHookEvent {
    pub stage: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl PostHookEvent {
    fn new(stage: &str, status: &str, message: Option<String>) -> Self {
        Self {
            stage: stage.into(),
            status: status.into(),
            message,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct GitInitReport {
    pub status: GitInitStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl GitInitReport {
    fn initialized(commit: String) -> Self {
        Self {
            status: GitInitStatus::Initialized,
            commit: Some(commit),
            message: None,
        }
    }

    fn already_present(reason: impl Into<String>) -> Self {
        Self {
            status: GitInitStatus::AlreadyPresent,
            commit: None,
            message: Some(reason.into()),
        }
    }

    fn inside_worktree() -> Self {
        Self {
            status: GitInitStatus::InsideWorktree,
            commit: None,
            message: Some("target directory is already inside a git worktree".into()),
        }
    }

    fn skipped(reason: impl Into<String>) -> Self {
        Self {
            status: GitInitStatus::Skipped,
            commit: None,
            message: Some(reason.into()),
        }
    }

    fn failed(reason: impl Into<String>) -> Self {
        Self {
            status: GitInitStatus::Failed,
            commit: None,
            message: Some(reason.into()),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum GitInitStatus {
    Initialized,
    AlreadyPresent,
    InsideWorktree,
    Skipped,
    Failed,
}

pub fn run_post_init(outcome: &ScaffoldOutcome, skip_git: bool) -> PostInitReport {
    let mut events = Vec::new();
    let git = if skip_git {
        events.push(PostHookEvent::new(
            "git-init",
            "skipped",
            Some("git scaffolding disabled via flag/env".into()),
        ));
        GitInitReport::skipped("git scaffolding disabled via flag/env")
    } else {
        initialize_git_repo(&outcome.path, &outcome.template, &mut events)
    };
    let next_steps = default_next_steps(&outcome.path);
    PostInitReport {
        git,
        next_steps,
        events,
    }
}

fn default_next_steps(path: &Path) -> Vec<String> {
    let cd = format!("cd {}", path.display());
    vec![
        cd,
        "component-doctor .".into(),
        "component-inspect component.manifest.json --json".into(),
        "git status".into(),
    ]
}

fn initialize_git_repo(
    path: &Path,
    template: &str,
    events: &mut Vec<PostHookEvent>,
) -> GitInitReport {
    if !path.exists() {
        events.push(PostHookEvent::new(
            "git-init",
            "failed",
            Some("target directory is missing".into()),
        ));
        return GitInitReport::failed("target directory is missing");
    }
    let git_dir = path.join(".git");
    if git_dir.exists() {
        events.push(PostHookEvent::new(
            "git-detect",
            "already-present",
            Some("directory already contains .git".into()),
        ));
        return GitInitReport::already_present("directory already contains .git");
    }
    let git = env::var("GIT").unwrap_or_else(|_| "git".to_owned());
    match detect_existing_worktree(&git, path) {
        Ok(true) => {
            events.push(PostHookEvent::new(
                "git-detect",
                "inside-worktree",
                Some("target directory belongs to an existing git worktree".into()),
            ));
            return GitInitReport::inside_worktree();
        }
        Ok(false) => {}
        Err(GitProbeError::MissingBinary) => {
            events.push(PostHookEvent::new(
                "git-detect",
                "skipped",
                Some("git binary not found in PATH".into()),
            ));
            return GitInitReport::skipped("git binary not found in PATH");
        }
        Err(GitProbeError::Io(err)) => {
            let msg = format!("git rev-parse failed: {err}");
            events.push(PostHookEvent::new(
                "git-detect",
                "failed",
                Some(msg.clone()),
            ));
            return GitInitReport::failed(msg);
        }
    }

    match git_init(&git, path) {
        Ok(()) => {}
        Err(GitInitError::MissingBinary) => {
            events.push(PostHookEvent::new(
                "git-init",
                "skipped",
                Some("git binary not found in PATH".into()),
            ));
            return GitInitReport::skipped("git binary not found in PATH");
        }
        Err(GitInitError::CommandFailed(cmd, output)) => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let message = if stderr.is_empty() {
                format!("{cmd} failed with exit code {}", output.status)
            } else {
                format!("{cmd} failed: {stderr}")
            };
            events.push(PostHookEvent::new(
                "git-init",
                "failed",
                Some(message.clone()),
            ));
            return GitInitReport::failed(message);
        }
        Err(GitInitError::Io(err)) => {
            let msg = format!("failed to run git command: {err}");
            events.push(PostHookEvent::new("git-init", "failed", Some(msg.clone())));
            return GitInitReport::failed(msg);
        }
    }
    events.push(PostHookEvent::new("git-init", "ok", None));

    if let Err(err) = git_add_all(&git, path) {
        events.push(PostHookEvent::new("git-add", "failed", Some(err.clone())));
        return GitInitReport::failed(err);
    }
    events.push(PostHookEvent::new("git-add", "ok", None));
    match git_commit_initial(&git, path, template) {
        Ok(commit) => {
            events.push(PostHookEvent::new("git-commit", "ok", None));
            GitInitReport::initialized(commit)
        }
        Err(err) => {
            events.push(PostHookEvent::new(
                "git-commit",
                "failed",
                Some(err.clone()),
            ));
            GitInitReport::failed(err)
        }
    }
}

fn detect_existing_worktree(git: &str, path: &Path) -> Result<bool, GitProbeError> {
    let output = Command::new(git)
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .current_dir(path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output();
    match output {
        Ok(out) => Ok(out.status.success()),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Err(GitProbeError::MissingBinary),
        Err(err) => Err(GitProbeError::Io(err)),
    }
}

fn git_init(git: &str, path: &Path) -> Result<(), GitInitError> {
    let output = Command::new(git).arg("init").current_dir(path).output();
    match output {
        Ok(out) if out.status.success() => Ok(()),
        Ok(out) => Err(GitInitError::CommandFailed("git init".into(), out)),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Err(GitInitError::MissingBinary),
        Err(err) => Err(GitInitError::Io(err)),
    }
}

fn git_add_all(git: &str, path: &Path) -> Result<(), String> {
    let output = Command::new(git)
        .arg("add")
        .arg("--all")
        .current_dir(path)
        .output();
    match output {
        Ok(out) if out.status.success() => Ok(()),
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
            let msg = if stderr.is_empty() {
                format!("git add --all failed with exit code {}", out.status)
            } else {
                format!("git add --all failed: {stderr}")
            };
            Err(msg)
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            Err("git binary not found in PATH".into())
        }
        Err(err) => Err(format!("failed to run git add: {err}")),
    }
}

fn git_commit_initial(git: &str, path: &Path, template: &str) -> Result<String, String> {
    let mut cmd = Command::new(git);
    cmd.arg("commit")
        .arg("-m")
        .arg(format!("chore(init): scaffold component from {template}"))
        .current_dir(path);
    ensure_git_identity(&mut cmd);
    let output = cmd.output();
    let output = match output {
        Ok(out) => out,
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            return Err("git binary not found in PATH".into());
        }
        Err(err) => return Err(format!("failed to run git commit: {err}")),
    };
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let message = if stderr.is_empty() {
            "git commit failed".to_string()
        } else {
            format!("git commit failed: {stderr}")
        };
        return Err(message);
    }
    read_head_hash(git, path)
}

fn read_head_hash(git: &str, path: &Path) -> Result<String, String> {
    let output = Command::new(git)
        .arg("rev-parse")
        .arg("HEAD")
        .current_dir(path)
        .output();
    let output = match output {
        Ok(out) => out,
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            return Err("git binary not found in PATH".into());
        }
        Err(err) => return Err(format!("failed to read git HEAD: {err}")),
    };
    if !output.status.success() {
        return Err("git rev-parse HEAD failed".into());
    }
    let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(hash)
}

fn ensure_git_identity(cmd: &mut Command) {
    if env::var_os("GIT_AUTHOR_NAME").is_none() {
        cmd.env("GIT_AUTHOR_NAME", DEFAULT_GIT_NAME);
    }
    if env::var_os("GIT_AUTHOR_EMAIL").is_none() {
        cmd.env("GIT_AUTHOR_EMAIL", DEFAULT_GIT_EMAIL);
    }
    if env::var_os("GIT_COMMITTER_NAME").is_none() {
        cmd.env("GIT_COMMITTER_NAME", DEFAULT_GIT_NAME);
    }
    if env::var_os("GIT_COMMITTER_EMAIL").is_none() {
        cmd.env("GIT_COMMITTER_EMAIL", DEFAULT_GIT_EMAIL);
    }
}

enum GitProbeError {
    MissingBinary,
    Io(io::Error),
}

enum GitInitError {
    MissingBinary,
    Io(io::Error),
    CommandFailed(String, std::process::Output),
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::TempDir;

    #[test]
    fn creates_git_repo_with_commit() {
        let temp = TempDir::new().expect("tempdir");
        let project = temp.path().join("demo");
        std::fs::create_dir_all(&project).expect("mkdir");
        std::fs::write(project.join("README.md"), "# Demo\n").expect("write");

        let outcome = ScaffoldOutcome {
            name: "demo".into(),
            template: "rust-wasi-p2-min".into(),
            template_description: Some("demo template".into()),
            template_tags: vec!["test".into()],
            path: project.clone(),
            created: vec!["README.md".into()],
        };

        let report = run_post_init(&outcome, false);
        assert_eq!(report.git.status, GitInitStatus::Initialized);
        assert!(project.join(".git").exists());
        assert!(report.git.commit.is_some());
        assert!(report.next_steps.iter().any(|step| step.contains("cd ")));
        assert!(
            report
                .events
                .iter()
                .any(|event| event.stage == "git-commit" && event.status == "ok")
        );
    }

    #[test]
    fn skips_when_inside_existing_repo() {
        let temp = TempDir::new().expect("tempdir");
        let project = temp.path().join("outer");
        std::fs::create_dir_all(project.join(".git")).expect("fake git dir");

        let outcome = ScaffoldOutcome {
            name: "demo".into(),
            template: "rust-wasi-p2-min".into(),
            template_description: None,
            template_tags: vec![],
            path: project.clone(),
            created: vec![],
        };
        let report = run_post_init(&outcome, false);
        assert!(matches!(
            report.git.status,
            GitInitStatus::AlreadyPresent | GitInitStatus::InsideWorktree
        ));
        assert!(
            report
                .events
                .iter()
                .any(|event| event.stage == "git-detect")
        );
    }

    #[test]
    fn honors_skip_flag() {
        let temp = TempDir::new().expect("tempdir");
        let project = temp.path().join("demo-skip");
        std::fs::create_dir_all(&project).expect("mkdir");

        let outcome = ScaffoldOutcome {
            name: "demo-skip".into(),
            template: "rust-wasi-p2-min".into(),
            template_description: None,
            template_tags: vec![],
            path: project.clone(),
            created: vec![],
        };
        let report = run_post_init(&outcome, true);
        assert_eq!(report.git.status, GitInitStatus::Skipped);
        assert!(
            report
                .events
                .iter()
                .any(|event| event.stage == "git-init" && event.status == "skipped")
        );
    }
}
