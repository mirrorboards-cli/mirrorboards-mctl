//! Dependency closure of a workspace directory, computed from the REAL files.
//!
//! Rust: recursive walk over `path = "…"` dependencies in Cargo.toml files
//! (all dependency sections, including per-target). Node: resolution of
//! `workspace:*` through the package-name map of the root pnpm workspace,
//! plus `file:` dependencies resolved path-wise.
//!
//! A dead path is a HARD error naming the referencing manifest — today that
//! class of rot survives silently until a CI build fails an hour later.

use crate::core::config::MirrorConfig;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum GraphError {
    #[error("cannot read {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("cannot parse {path}: {message}")]
    Parse { path: PathBuf, message: String },
    #[error("{manifest} references '{reference}', which resolves to {resolved} — the path does not exist (stale layout?)")]
    DeadPath {
        manifest: PathBuf,
        reference: String,
        resolved: PathBuf,
    },
    #[error("workspace dependency '{name}' of {manifest} matches no package in the pnpm workspace")]
    UnknownWorkspacePackage { name: String, manifest: PathBuf },
    #[error("{dir} lies outside every repository declared in the mirror manifests")]
    UnmappedDir { dir: String },
    #[error("{dir} has neither Cargo.toml nor package.json — nothing to compute a closure from")]
    NoManifest { dir: String },
}

/// What a directory is built from — wykryte z obecności manifestów,
/// nie deklarowane: `Cargo.toml` znaczy kraty, `package.json` paczki.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Language {
    Rust,
    Node,
    /// Oba naraz — front z rdzeniem WASM w tym samym drzewie.
    Mixed,
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::Rust => write!(f, "rust"),
            Language::Node => write!(f, "node"),
            Language::Mixed => write!(f, "mixed"),
        }
    }
}

/// The computed closure of one workspace directory.
#[derive(Debug, Clone, Serialize)]
pub struct Closure {
    pub language: Language,
    /// Workspace-relative directory the closure was computed for.
    pub root_unit: String,
    /// Workspace-relative dirs of every crate/package in the closure
    /// (the app itself included), sorted.
    pub units: Vec<String>,
    /// Repositories covering the closure: mirror path -> git URL, sorted.
    pub repos: BTreeMap<String, String>,
}

/// Computes the closure of `dir` (workspace-relative) at `root`.
///
/// Rodzaj wykrywany jest z plików: obecność `Cargo.toml` włącza chodzenie po
/// path-depach, `package.json` po `workspace:*` i `file:`. Katalog z obydwoma
/// (front z rdzeniem WASM) daje oba domknięcia naraz.
pub fn closure(root: &Path, config: &MirrorConfig, dir: &str) -> Result<Closure, GraphError> {
    let unit_dir = root.join(dir);
    if !unit_dir.is_dir() {
        return Err(GraphError::DeadPath {
            manifest: PathBuf::from("(argument)"),
            reference: dir.to_string(),
            resolved: unit_dir,
        });
    }

    let has_cargo = unit_dir.join("Cargo.toml").is_file();
    let has_package = unit_dir.join("package.json").is_file();
    let language = match (has_cargo, has_package) {
        (true, true) => Language::Mixed,
        (true, false) => Language::Rust,
        (false, true) => Language::Node,
        (false, false) => {
            return Err(GraphError::NoManifest {
                dir: dir.to_string(),
            })
        }
    };

    let mut units = BTreeSet::new();
    if has_cargo {
        units.extend(rust_closure(root, &unit_dir)?);
    }
    if has_package {
        units.extend(node_closure(root, &unit_dir)?);
    }

    let mut rel_units = BTreeSet::new();
    for unit in &units {
        rel_units.insert(relative_to(root, unit)?);
    }

    let repos = map_to_repos(config, &rel_units)?;

    Ok(Closure {
        language,
        root_unit: dir.to_string(),
        units: rel_units.into_iter().collect(),
        repos,
    })
}

fn relative_to(root: &Path, dir: &Path) -> Result<String, GraphError> {
    let rel = dir.strip_prefix(root).map_err(|_| GraphError::UnmappedDir {
        dir: dir.display().to_string(),
    })?;
    Ok(rel.to_string_lossy().replace('\\', "/"))
}

/// Longest-prefix mapping of unit dirs onto mirror repositories.
fn map_to_repos(
    config: &MirrorConfig,
    units: &BTreeSet<String>,
) -> Result<BTreeMap<String, String>, GraphError> {
    let mut repos: Vec<(&str, &str)> = config
        .repositories
        .iter()
        .map(|r| (r.path.as_str(), r.git.as_str()))
        .collect();
    repos.sort_by_key(|(path, _)| std::cmp::Reverse(path.len()));

    let mut out = BTreeMap::new();
    for unit in units {
        let hit = repos
            .iter()
            .find(|(path, _)| unit == path || unit.starts_with(&format!("{path}/")));
        match hit {
            Some((path, git)) => {
                out.insert((*path).to_string(), (*git).to_string());
            }
            None => {
                return Err(GraphError::UnmappedDir { dir: unit.clone() });
            }
        }
    }
    Ok(out)
}

// ── Rust ────────────────────────────────────────────────────────────────────

/// Walks `path =` dependencies recursively, starting from the app manifest
/// and the app's own workspace members.
///
/// Dev-dependencies are followed ONLY for manifests inside the app dir:
/// cargo requires dev-deps of the workspace it builds, but not of external
/// path-dependencies.
fn rust_closure(root: &Path, app_dir: &Path) -> Result<BTreeSet<PathBuf>, GraphError> {
    let mut visited: BTreeSet<PathBuf> = BTreeSet::new();
    let mut queue: Vec<PathBuf> = vec![canonical(app_dir)?];

    while let Some(dir) = queue.pop() {
        if !visited.insert(dir.clone()) {
            continue;
        }
        let manifest_path = dir.join("Cargo.toml");
        let manifest = read_toml(&manifest_path)?;
        let inside_app = dir.starts_with(app_dir);

        // Workspace members of the app's own workspace (relative subdirs).
        if inside_app {
            if let Some(members) = manifest
                .get("workspace")
                .and_then(|w| w.get("members"))
                .and_then(|m| m.as_array())
            {
                for member in members.iter().filter_map(|m| m.as_str()) {
                    push_resolved(&dir, member, &manifest_path, &mut queue)?;
                }
            }
        }

        for dep_path in path_dependencies(&manifest, inside_app) {
            push_resolved(&dir, &dep_path, &manifest_path, &mut queue)?;
        }
    }

    // Keep only dirs under the workspace root; the app dir stays too.
    Ok(visited
        .into_iter()
        .filter(|d| d.starts_with(root))
        .collect())
}

/// All `path = …` entries from every dependency section of a manifest.
fn path_dependencies(manifest: &toml::Value, include_dev: bool) -> Vec<String> {
    let mut sections: Vec<&toml::Value> = Vec::new();
    for key in ["dependencies", "build-dependencies"] {
        if let Some(section) = manifest.get(key) {
            sections.push(section);
        }
    }
    if include_dev {
        if let Some(section) = manifest.get("dev-dependencies") {
            sections.push(section);
        }
    }
    if let Some(targets) = manifest.get("target").and_then(|t| t.as_table()) {
        for target in targets.values() {
            for key in ["dependencies", "build-dependencies", "dev-dependencies"] {
                if key == "dev-dependencies" && !include_dev {
                    continue;
                }
                if let Some(section) = target.get(key) {
                    sections.push(section);
                }
            }
        }
    }

    let mut out = Vec::new();
    for section in sections {
        if let Some(table) = section.as_table() {
            for dep in table.values() {
                if let Some(path) = dep.get("path").and_then(|p| p.as_str()) {
                    out.push(path.to_string());
                }
            }
        }
    }
    out
}

fn push_resolved(
    base: &Path,
    reference: &str,
    manifest: &Path,
    queue: &mut Vec<PathBuf>,
) -> Result<(), GraphError> {
    let resolved = base.join(reference);
    let canonical = resolved
        .canonicalize()
        .map_err(|_| GraphError::DeadPath {
            manifest: manifest.to_path_buf(),
            reference: reference.to_string(),
            resolved,
        })?;
    queue.push(canonical);
    Ok(())
}

// ── Node ────────────────────────────────────────────────────────────────────

/// Resolves `workspace:*` through the root pnpm workspace's name map and
/// `file:` dependencies path-wise, recursively.
fn node_closure(root: &Path, app_dir: &Path) -> Result<BTreeSet<PathBuf>, GraphError> {
    let name_map = pnpm_package_map(root)?;

    let mut visited: BTreeSet<PathBuf> = BTreeSet::new();
    // Kraty Rusta wciągnięte przez zależności `file:` na artefakty budowania.
    let mut rust: BTreeSet<PathBuf> = BTreeSet::new();
    let mut queue: Vec<PathBuf> = vec![canonical(app_dir)?];

    while let Some(dir) = queue.pop() {
        if !visited.insert(dir.clone()) {
            continue;
        }
        let manifest_path = dir.join("package.json");
        let manifest = read_json(&manifest_path)?;

        for section in ["dependencies", "devDependencies"] {
            let Some(deps) = manifest.get(section).and_then(|d| d.as_object()) else {
                continue;
            };
            for (name, version) in deps {
                let Some(version) = version.as_str() else { continue };
                if version.starts_with("workspace:") {
                    let target = name_map.get(name).ok_or_else(|| {
                        GraphError::UnknownWorkspacePackage {
                            name: name.clone(),
                            manifest: manifest_path.clone(),
                        }
                    })?;
                    queue.push(target.clone());
                } else if let Some(rel) = version.strip_prefix("file:") {
                    let resolved = normalize(&dir.join(rel));
                    // Kolejność ma znaczenie: krata-producent SPRAWDZANA
                    // PIERWSZA, zanim spytamy o istnienie katalogu. `pkg`
                    // wasm-packa bywa na dysku po dawnym buildzie, a wynik
                    // grafu nie może zależeć od tego, co ktoś kiedyś zbudował.
                    if let Some(crate_dir) = producing_crate(root, &resolved) {
                        rust.extend(rust_closure(root, &crate_dir)?);
                    } else {
                        let canonical =
                            resolved.canonicalize().map_err(|_| GraphError::DeadPath {
                                manifest: manifest_path.clone(),
                                reference: format!("{name} -> file:{rel}"),
                                resolved,
                            })?;
                        queue.push(canonical);
                    }
                }
            }
        }
    }

    visited.extend(rust);
    Ok(visited
        .into_iter()
        .filter(|d| d.starts_with(root))
        .collect())
}

/// Najbliższa krata NAD katalogiem — zależność `file:` wskazująca w głąb
/// kraty Rusta jest artefaktem jej budowania (`pkg` wasm-packa), nie paczką.
fn producing_crate(root: &Path, missing: &Path) -> Option<PathBuf> {
    let mut candidate = missing.parent()?;
    // Zatrzymujemy się PRZED korzeniem: on też ma Cargo.toml (workspace),
    // więc bez tego warunku każda zależność `file:` wyglądałaby na artefakt.
    while candidate != root && candidate.starts_with(root) {
        if candidate.join("Cargo.toml").is_file() {
            return Some(candidate.to_path_buf());
        }
        candidate = candidate.parent()?;
    }
    None
}

/// Package-name -> dir map of the root pnpm workspace.
///
/// Globs are expanded shallowly: an entry ending in `/*` lists the parent's
/// direct children — the only glob form the workspace uses.
fn pnpm_package_map(root: &Path) -> Result<BTreeMap<String, PathBuf>, GraphError> {
    let workspace_path = root.join("pnpm-workspace.yaml");
    let content = std::fs::read_to_string(&workspace_path).map_err(|e| GraphError::Io {
        path: workspace_path.clone(),
        source: e,
    })?;

    let mut dirs: Vec<PathBuf> = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        let Some(entry) = line.strip_prefix("- ") else { continue };
        let entry = entry.trim_matches('"').trim_matches('\'');
        if let Some(parent) = entry.strip_suffix("/*") {
            let parent_dir = root.join(parent);
            let Ok(children) = std::fs::read_dir(&parent_dir) else { continue };
            for child in children.flatten() {
                if child.path().is_dir() {
                    dirs.push(child.path());
                }
            }
        } else {
            dirs.push(root.join(entry));
        }
    }

    let mut map = BTreeMap::new();
    for dir in dirs {
        let manifest_path = dir.join("package.json");
        if !manifest_path.is_file() {
            continue;
        }
        let manifest = read_json(&manifest_path)?;
        if let Some(name) = manifest.get("name").and_then(|n| n.as_str()) {
            map.insert(name.to_string(), canonical(&dir)?);
        }
    }
    Ok(map)
}

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Normalizacja LEKSYKALNA — cel może jeszcze nie istnieć (katalog
/// wytwarzany przez etap budowania), więc `canonicalize` odpada.
fn normalize(path: &Path) -> PathBuf {
    let mut parts: Vec<std::ffi::OsString> = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                parts.pop();
            }
            std::path::Component::CurDir => {}
            other => parts.push(other.as_os_str().to_os_string()),
        }
    }
    parts.iter().collect()
}

fn canonical(path: &Path) -> Result<PathBuf, GraphError> {
    path.canonicalize().map_err(|e| GraphError::Io {
        path: path.to_path_buf(),
        source: e,
    })
}

fn read_toml(path: &Path) -> Result<toml::Value, GraphError> {
    let content = std::fs::read_to_string(path).map_err(|e| GraphError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;
    toml::from_str(&content).map_err(|e| GraphError::Parse {
        path: path.to_path_buf(),
        message: e.to_string(),
    })
}

fn read_json(path: &Path) -> Result<serde_json::Value, GraphError> {
    let content = std::fs::read_to_string(path).map_err(|e| GraphError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;
    serde_json::from_str(&content).map_err(|e| GraphError::Parse {
        path: path.to_path_buf(),
        message: e.to_string(),
    })
}
