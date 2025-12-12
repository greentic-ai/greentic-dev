#![cfg(feature = "cli")]

use std::borrow::Cow;
use std::collections::HashSet;
use std::env;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};
use std::str;

use directories::BaseDirs;
use handlebars::{Handlebars, no_escape};
use include_dir::{Dir, DirEntry, include_dir};
use serde::{Deserialize, Serialize, Serializer};
use thiserror::Error;
use time::OffsetDateTime;
use walkdir::WalkDir;

use super::deps::{self, DependencyMode};
use super::validate::{self, ValidationError};
use super::write::{GeneratedFile, WriteError, Writer};

static BUILTIN_COMPONENT_TEMPLATES: Dir<'_> =
    include_dir!("$CARGO_MANIFEST_DIR/assets/templates/component");

pub const DEFAULT_WIT_WORLD: &str = "greentic:component/component@0.5.0";

const METADATA_FILE: &str = "template.json";
const TEMPLATE_HOME_ENV: &str = "GREENTIC_TEMPLATE_ROOT";
const TEMPLATE_YEAR_ENV: &str = "GREENTIC_TEMPLATE_YEAR";

#[derive(Debug, Clone, Default)]
pub struct ScaffoldEngine;

impl ScaffoldEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn templates(&self) -> Result<Vec<TemplateDescriptor>, ScaffoldError> {
        let mut templates = self.builtin_templates();
        templates.extend(self.user_templates()?);
        templates.sort();
        Ok(templates)
    }

    pub fn resolve_template(&self, id: &str) -> Result<TemplateDescriptor, ScaffoldError> {
        let list = self.templates()?;
        list.into_iter()
            .find(|tpl| tpl.id == id)
            .ok_or_else(|| ScaffoldError::TemplateNotFound(id.to_owned()))
    }

    pub fn scaffold(&self, request: ScaffoldRequest) -> Result<ScaffoldOutcome, ScaffoldError> {
        let descriptor = self.resolve_template(&request.template_id)?;
        validate::ensure_path_available(&request.path)?;
        let package = self.load_template(&descriptor)?;
        let context = TemplateContext::from_request(&request);
        let rendered = self.render_files(&package, &context)?;
        let created = Writer::new().write_all(&request.path, &rendered)?;

        if matches!(request.dependency_mode, DependencyMode::CratesIo) {
            deps::ensure_cratesio_manifest_clean(&request.path)?;
        }

        Ok(ScaffoldOutcome {
            name: request.name,
            template: package.metadata.id.clone(),
            template_description: descriptor.description.clone(),
            template_tags: descriptor.tags.clone(),
            path: request.path,
            created,
        })
    }

    fn builtin_templates(&self) -> Vec<TemplateDescriptor> {
        BUILTIN_COMPONENT_TEMPLATES
            .dirs()
            .filter_map(|dir| {
                let fallback_id = dir.path().file_name()?.to_string_lossy().to_string();
                let metadata = match embedded_metadata(dir, &fallback_id) {
                    Ok(meta) => meta,
                    Err(_) => ResolvedTemplateMetadata::fallback(fallback_id.clone()),
                };
                Some(TemplateDescriptor {
                    id: metadata.id,
                    location: TemplateLocation::BuiltIn,
                    path: None,
                    description: metadata.description,
                    tags: metadata.tags,
                })
            })
            .collect()
    }

    fn user_templates(&self) -> Result<Vec<TemplateDescriptor>, ScaffoldError> {
        let Some(root) = Self::user_templates_root() else {
            return Ok(Vec::new());
        };
        let metadata = match fs::metadata(&root) {
            Ok(meta) => meta,
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(err) => return Err(ScaffoldError::UserTemplatesIo(root.clone(), err)),
        };
        if !metadata.is_dir() {
            return Ok(Vec::new());
        }
        let mut templates = Vec::new();
        let iter =
            fs::read_dir(&root).map_err(|err| ScaffoldError::UserTemplatesIo(root.clone(), err))?;
        for entry in iter {
            let entry = entry.map_err(|err| ScaffoldError::UserTemplatesIo(root.clone(), err))?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let fallback_id = match path.file_name() {
                Some(id) => id.to_string_lossy().to_string(),
                None => continue,
            };
            if !validate::is_valid_name(&fallback_id) {
                continue;
            }
            let metadata = match user_metadata(&path, &fallback_id) {
                Ok(meta) => meta,
                Err(_) => ResolvedTemplateMetadata::fallback(fallback_id.clone()),
            };
            templates.push(TemplateDescriptor {
                id: metadata.id,
                location: TemplateLocation::User,
                path: Some(path),
                description: metadata.description,
                tags: metadata.tags,
            });
        }
        templates.sort();
        Ok(templates)
    }

    fn load_template(
        &self,
        descriptor: &TemplateDescriptor,
    ) -> Result<TemplatePackage, ScaffoldError> {
        let id = descriptor.id.clone();
        match descriptor.location {
            TemplateLocation::BuiltIn => {
                let dir = BUILTIN_COMPONENT_TEMPLATES
                    .get_dir(&descriptor.id)
                    .ok_or_else(|| ScaffoldError::TemplateNotFound(descriptor.id.clone()))?;
                TemplatePackage::from_embedded(dir)
                    .map_err(|source| ScaffoldError::TemplateLoad { id, source })
            }
            TemplateLocation::User => {
                let path = descriptor
                    .path
                    .as_ref()
                    .ok_or_else(|| ScaffoldError::TemplateNotFound(descriptor.id.clone()))?;
                TemplatePackage::from_disk(path)
                    .map_err(|source| ScaffoldError::TemplateLoad { id, source })
            }
        }
    }

    fn render_files(
        &self,
        package: &TemplatePackage,
        context: &TemplateContext,
    ) -> Result<Vec<GeneratedFile>, ScaffoldError> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);
        handlebars.register_escape_fn(no_escape);

        let template_id = package.metadata.id.clone();
        let executable_paths: HashSet<PathBuf> = package
            .metadata
            .executables
            .iter()
            .map(|path| render_path(path, &handlebars, context))
            .collect::<Result<_, _>>()
            .map_err(|source| ScaffoldError::Render {
                id: template_id.clone(),
                source,
            })?;

        let mut rendered = Vec::with_capacity(package.entries.len());
        for entry in &package.entries {
            let path_template = entry.path_template();
            let target_path =
                render_path(path_template, &handlebars, context).map_err(|source| {
                    ScaffoldError::Render {
                        id: template_id.clone(),
                        source,
                    }
                })?;
            let contents = if entry.templated {
                let source =
                    str::from_utf8(&entry.contents).map_err(|source| ScaffoldError::Render {
                        id: template_id.clone(),
                        source: RenderError::Utf8 {
                            path: entry.relative_path.clone(),
                            source,
                        },
                    })?;
                handlebars
                    .render_template(source, context)
                    .map(|value| value.into_bytes())
                    .map_err(|source| ScaffoldError::Render {
                        id: template_id.clone(),
                        source: RenderError::Handlebars {
                            path: entry.relative_path.clone(),
                            source,
                        },
                    })?
            } else {
                entry.contents.clone()
            };

            let executable =
                executable_paths.contains(&target_path) || is_executable_heuristic(&target_path);

            rendered.push(GeneratedFile {
                relative_path: target_path,
                contents,
                executable,
            });
        }

        Ok(rendered)
    }

    fn user_templates_root() -> Option<PathBuf> {
        if let Some(root) = env::var_os(TEMPLATE_HOME_ENV) {
            return Some(PathBuf::from(root));
        }
        BaseDirs::new().map(|dirs| {
            dirs.home_dir()
                .join(".greentic")
                .join("templates")
                .join("component")
        })
    }
}

#[derive(Debug, Error)]
pub enum ScaffoldError {
    #[error("template `{0}` not found")]
    TemplateNotFound(String),
    #[error("failed to read user templates from {0}: {1}")]
    UserTemplatesIo(PathBuf, #[source] io::Error),
    #[error("failed to load template `{id}`: {source}")]
    TemplateLoad {
        id: String,
        #[source]
        source: TemplateLoadError,
    },
    #[error("failed to render template `{id}`: {source}")]
    Render {
        id: String,
        #[source]
        source: RenderError,
    },
    #[error(transparent)]
    Write(#[from] WriteError),
    #[error(transparent)]
    Validation(#[from] ValidationError),
    #[error(transparent)]
    Dependency(#[from] deps::DependencyError),
}

#[derive(Debug, Clone)]
pub struct ScaffoldRequest {
    pub name: String,
    pub path: PathBuf,
    pub template_id: String,
    pub org: String,
    pub version: String,
    pub license: String,
    pub wit_world: String,
    pub non_interactive: bool,
    pub year_override: Option<i32>,
    pub dependency_mode: DependencyMode,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScaffoldOutcome {
    pub name: String,
    pub template: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub template_tags: Vec<String>,
    #[serde(serialize_with = "serialize_path")]
    pub path: PathBuf,
    pub created: Vec<String>,
}

impl ScaffoldOutcome {
    pub fn human_summary(&self) -> String {
        format!(
            "Scaffolded component `{}` in {} ({} files)",
            self.name,
            self.path.display(),
            self.created.len()
        )
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct TemplateDescriptor {
    pub id: String,
    pub location: TemplateLocation,
    #[serde(serialize_with = "serialize_optional_path")]
    pub path: Option<PathBuf>,
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl TemplateDescriptor {
    pub fn display_path(&self) -> Cow<'_, str> {
        match &self.path {
            Some(path) => Cow::Owned(path.display().to_string()),
            None => Cow::Borrowed("<embedded>"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum TemplateLocation {
    #[serde(rename = "built-in")]
    BuiltIn,
    User,
}

impl fmt::Display for TemplateLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TemplateLocation::BuiltIn => write!(f, "built-in"),
            TemplateLocation::User => write!(f, "user"),
        }
    }
}

#[derive(Debug, Error)]
pub enum TemplateLoadError {
    #[error("failed to parse metadata {path}: {source}")]
    Metadata {
        path: String,
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to read {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("template `{path}` is not valid UTF-8: {source}")]
    Utf8 {
        path: String,
        #[source]
        source: str::Utf8Error,
    },
    #[error("failed to render `{path}`: {source}")]
    Handlebars {
        path: String,
        #[source]
        source: handlebars::RenderError,
    },
    #[error("rendered path `{0}` escapes the target directory")]
    Traversal(String),
}

struct TemplatePackage {
    metadata: ResolvedTemplateMetadata,
    entries: Vec<TemplateEntry>,
}

impl TemplatePackage {
    fn from_embedded(dir: &Dir<'_>) -> Result<Self, TemplateLoadError> {
        let fallback_id = dir
            .path()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let metadata = embedded_metadata(dir, &fallback_id)?;
        let mut entries = Vec::new();
        collect_embedded_entries(dir, "", &mut entries);
        Ok(Self { metadata, entries })
    }

    fn from_disk(path: &Path) -> Result<Self, TemplateLoadError> {
        let fallback_id = path
            .file_name()
            .map(|id| id.to_string_lossy().to_string())
            .unwrap_or_else(|| "user".into());
        let metadata = user_metadata(path, &fallback_id)?;
        let mut entries = Vec::new();
        collect_fs_entries(path, &mut entries)?;
        Ok(Self { metadata, entries })
    }
}

#[derive(Debug, Clone)]
struct TemplateEntry {
    relative_path: String,
    contents: Vec<u8>,
    templated: bool,
}

impl TemplateEntry {
    fn path_template(&self) -> &str {
        if self.templated && self.relative_path.ends_with(".hbs") {
            &self.relative_path[..self.relative_path.len() - 4]
        } else {
            &self.relative_path
        }
    }
}

#[derive(Debug, Clone)]
struct ResolvedTemplateMetadata {
    id: String,
    description: Option<String>,
    tags: Vec<String>,
    executables: Vec<String>,
}

impl ResolvedTemplateMetadata {
    fn fallback(id: String) -> Self {
        Self {
            id,
            description: None,
            tags: Vec::new(),
            executables: Vec::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct TemplateMetadataFile {
    id: Option<String>,
    description: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    executables: Vec<String>,
}

fn embedded_metadata(
    dir: &Dir<'_>,
    fallback_id: &str,
) -> Result<ResolvedTemplateMetadata, TemplateLoadError> {
    let path = dir.path().join(METADATA_FILE);
    let metadata = match dir.get_file(&path) {
        Some(file) => deserialize_metadata(file.contents(), path.to_string_lossy().as_ref())?,
        None => None,
    };
    Ok(resolve_metadata(metadata, fallback_id))
}

fn user_metadata(
    path: &Path,
    fallback_id: &str,
) -> Result<ResolvedTemplateMetadata, TemplateLoadError> {
    let metadata_path = path.join(METADATA_FILE);
    if !metadata_path.exists() {
        return Ok(ResolvedTemplateMetadata::fallback(fallback_id.to_string()));
    }
    let contents = fs::read(&metadata_path).map_err(|source| TemplateLoadError::Io {
        path: metadata_path.clone(),
        source,
    })?;
    let metadata = deserialize_metadata(&contents, metadata_path.to_string_lossy().as_ref())?;
    Ok(resolve_metadata(metadata, fallback_id))
}

fn deserialize_metadata<T: AsRef<[u8]>>(
    bytes: T,
    path: &str,
) -> Result<Option<TemplateMetadataFile>, TemplateLoadError> {
    if bytes.as_ref().is_empty() {
        return Ok(None);
    }
    serde_json::from_slice(bytes.as_ref())
        .map(Some)
        .map_err(|source| TemplateLoadError::Metadata {
            path: path.to_string(),
            source,
        })
}

fn resolve_metadata(
    metadata: Option<TemplateMetadataFile>,
    fallback_id: &str,
) -> ResolvedTemplateMetadata {
    match metadata {
        Some(file) => ResolvedTemplateMetadata {
            id: file.id.unwrap_or_else(|| fallback_id.to_string()),
            description: file.description,
            tags: file.tags,
            executables: file.executables,
        },
        None => ResolvedTemplateMetadata::fallback(fallback_id.to_string()),
    }
}

fn collect_embedded_entries(dir: &Dir<'_>, prefix: &str, entries: &mut Vec<TemplateEntry>) {
    for entry in dir.entries() {
        match entry {
            DirEntry::Dir(sub) => {
                let new_prefix = if prefix.is_empty() {
                    sub.path()
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .to_string()
                } else {
                    format!(
                        "{}/{}",
                        prefix,
                        sub.path().file_name().unwrap().to_string_lossy()
                    )
                };
                collect_embedded_entries(sub, &new_prefix, entries);
            }
            DirEntry::File(file) => {
                if file.path().ends_with(METADATA_FILE) {
                    continue;
                }
                entries.push(TemplateEntry {
                    relative_path: join_relative(
                        prefix,
                        file.path().file_name().unwrap().to_string_lossy().as_ref(),
                    ),
                    contents: file.contents().to_vec(),
                    templated: file.path().extension().and_then(|ext| ext.to_str()) == Some("hbs"),
                });
            }
        }
    }
}

fn collect_fs_entries(
    root: &Path,
    entries: &mut Vec<TemplateEntry>,
) -> Result<(), TemplateLoadError> {
    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_dir() {
            continue;
        }
        let path = entry.path();
        if path.file_name().and_then(|f| f.to_str()) == Some(METADATA_FILE) {
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .map_err(|source| TemplateLoadError::Io {
                path: path.to_path_buf(),
                source: io::Error::other(source),
            })?;
        let contents = fs::read(path).map_err(|source| TemplateLoadError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        entries.push(TemplateEntry {
            relative_path: relative.to_string_lossy().replace('\\', "/"),
            contents,
            templated: relative.extension().and_then(|ext| ext.to_str()) == Some("hbs"),
        });
    }
    Ok(())
}

fn join_relative(prefix: &str, name: &str) -> String {
    if prefix.is_empty() {
        name.to_string()
    } else {
        format!("{prefix}/{name}")
    }
}

#[derive(Serialize)]
struct TemplateContext {
    name: String,
    name_snake: String,
    name_kebab: String,
    package_id: String,
    namespace_wit: String,
    org: String,
    version: String,
    license: String,
    wit_world: String,
    year: i32,
    repo: String,
    author: Option<String>,
    dependency_mode: &'static str,
    greentic_interfaces_dep: String,
    greentic_interfaces_guest_dep: String,
    greentic_types_dep: String,
    relative_patch_path: Option<String>,
}

impl TemplateContext {
    fn from_request(request: &ScaffoldRequest) -> Self {
        let name_snake = request.name.replace('-', "_");
        let name_kebab = request.name.replace('_', "-");
        let package_id = format!("{}.{}", request.org, name_snake);
        let namespace_wit = sanitize_namespace(&request.org);
        let year = request.year_override.unwrap_or_else(template_year);
        let deps = deps::resolve_dependency_templates(request.dependency_mode, &request.path);
        Self {
            name: request.name.clone(),
            name_snake,
            name_kebab,
            package_id,
            namespace_wit,
            org: request.org.clone(),
            version: request.version.clone(),
            license: request.license.clone(),
            wit_world: request.wit_world.clone(),
            year,
            repo: request.name.clone(),
            author: detect_author(),
            dependency_mode: request.dependency_mode.as_str(),
            greentic_interfaces_dep: deps.greentic_interfaces,
            greentic_interfaces_guest_dep: deps.greentic_interfaces_guest,
            greentic_types_dep: deps.greentic_types,
            relative_patch_path: deps.relative_patch_path,
        }
    }
}

fn template_year() -> i32 {
    if let Ok(value) = env::var(TEMPLATE_YEAR_ENV)
        && let Ok(parsed) = value.parse()
    {
        return parsed;
    }
    OffsetDateTime::now_utc().year()
}

fn sanitize_namespace(value: &str) -> String {
    value
        .chars()
        .map(|c| {
            let lower = c.to_ascii_lowercase();
            if lower.is_ascii_lowercase() || lower.is_ascii_digit() || lower == '-' {
                lower
            } else {
                '-'
            }
        })
        .collect()
}

fn detect_author() -> Option<String> {
    for key in ["GIT_AUTHOR_NAME", "GIT_COMMITTER_NAME", "USER", "USERNAME"] {
        if let Ok(value) = env::var(key) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

fn render_path(
    template: &str,
    handlebars: &Handlebars<'_>,
    context: &TemplateContext,
) -> Result<PathBuf, RenderError> {
    let rendered = handlebars
        .render_template(template, context)
        .map_err(|source| RenderError::Handlebars {
            path: template.to_string(),
            source,
        })?;
    normalize_relative(&rendered)
}

fn normalize_relative(value: &str) -> Result<PathBuf, RenderError> {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        return Err(RenderError::Traversal(value.to_string()));
    }
    for component in path.components() {
        match component {
            Component::ParentDir | Component::Prefix(_) | Component::RootDir => {
                return Err(RenderError::Traversal(value.to_string()));
            }
            _ => {}
        }
    }
    Ok(path)
}

fn is_executable_heuristic(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("sh" | "bash" | "zsh" | "ps1")
    ) || path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name == "Makefile")
        .unwrap_or(false)
}

fn serialize_path<S>(path: &Path, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&path.display().to_string())
}

fn serialize_optional_path<S>(path: &Option<PathBuf>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match path {
        Some(value) => serializer.serialize_some(&value.display().to_string()),
        None => serializer.serialize_none(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::TempDir;
    use std::fs;

    #[test]
    fn lists_built_in_template_ids() {
        let engine = ScaffoldEngine::new();
        let templates = engine.templates().unwrap();
        assert!(!templates.is_empty());
        assert!(templates.iter().any(|tpl| tpl.id == "rust-wasi-p2-min"));
    }

    #[test]
    fn resolves_template() {
        let engine = ScaffoldEngine::new();
        let descriptor = engine.resolve_template("rust-wasi-p2-min").unwrap();
        assert_eq!(descriptor.id, "rust-wasi-p2-min");
    }

    #[test]
    fn builtin_metadata_is_available() {
        let dir = BUILTIN_COMPONENT_TEMPLATES
            .get_dir("rust-wasi-p2-min")
            .expect("template dir");
        let meta_path = dir.path().join(METADATA_FILE);
        assert!(dir.get_file(&meta_path).is_some());
        let metadata = embedded_metadata(dir, "rust-wasi-p2-min").expect("metadata");
        assert_eq!(
            metadata.description.as_deref(),
            Some("Minimal Rust + WASI-P2 component starter")
        );
        assert_eq!(metadata.tags, vec!["rust", "wasi-p2", "component"]);
    }

    #[test]
    fn scaffolds_into_empty_directory() {
        let temp = TempDir::new().unwrap();
        let target = temp.path().join("demo-component");
        let engine = ScaffoldEngine::new();
        let request = ScaffoldRequest {
            name: "demo-component".into(),
            path: target.clone(),
            template_id: "rust-wasi-p2-min".into(),
            org: "ai.greentic".into(),
            version: "0.1.0".into(),
            license: "MIT".into(),
            wit_world: DEFAULT_WIT_WORLD.into(),
            non_interactive: true,
            year_override: Some(2030),
            dependency_mode: DependencyMode::Local,
        };
        let outcome = engine.scaffold(request).unwrap();
        assert!(target.join("Cargo.toml").exists());
        assert!(
            outcome
                .created
                .iter()
                .any(|path| path.contains("Cargo.toml"))
        );
    }

    #[test]
    fn refuses_non_empty_directory() {
        let temp = TempDir::new().unwrap();
        let target = temp.path().join("demo");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("file"), "data").unwrap();
        let engine = ScaffoldEngine::new();
        let request = ScaffoldRequest {
            name: "demo".into(),
            path: target.clone(),
            template_id: "rust-wasi-p2-min".into(),
            org: "ai.greentic".into(),
            version: "0.1.0".into(),
            license: "MIT".into(),
            wit_world: DEFAULT_WIT_WORLD.into(),
            non_interactive: true,
            year_override: None,
            dependency_mode: DependencyMode::Local,
        };
        let err = engine.scaffold(request).unwrap_err();
        assert!(matches!(err, ScaffoldError::Validation(_)));
    }
}
