//! Dependency closure of an image, computed from the REAL workspace.
//!
//! Rust: recursive walk over `path = "…"` dependencies in Cargo.toml files
//! (all dependency sections, including per-target). Node: resolution of
//! `workspace:*` through the package-name map of the root pnpm workspace,
//! plus `file:` dependencies resolved path-wise.
//!
//! A dead path is a HARD error naming the referencing manifest — today that
//! class of rot survives silently until a CI build fails an hour later.

use crate::core::config::MirrorConfig;
use crate::core::image::{ImageKind, ImageSpec};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum GraphError {
    #[error("image '{0}' is not declared in any manifest ([[images]])")]
    UnknownImage(String),
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
}

/// The computed closure of one image.
#[derive(Debug, Clone, Serialize)]
pub struct ImageGraph {
    pub image: String,
    pub kind: ImageKind,
    /// Workspace-relative app dir.
    pub app: String,
    /// Workspace-relative dirs of every crate/package in the closure
    /// (the app itself included), sorted.
    pub units: Vec<String>,
    /// Repositories covering the closure: mirror path -> git URL, sorted.
    pub repos: BTreeMap<String, String>,
}

/// Computes the closure of `image` against the workspace at `root`.
pub fn image_graph(
    root: &Path,
    config: &MirrorConfig,
    spec: &ImageSpec,
) -> Result<ImageGraph, GraphError> {
    let app_dir = root.join(&spec.app);
    if !app_dir.is_dir() {
        return Err(GraphError::DeadPath {
            manifest: PathBuf::from("[[images]]"),
            reference: spec.app.clone(),
            resolved: app_dir,
        });
    }

    let units = match spec.kind {
        ImageKind::RustBin => rust_closure(root, &app_dir)?,
        ImageKind::NodeTsx => node_closure(root, &app_dir)?,
        // Front z etapami WASM ma DWA domknięcia: paczki JS i kraty Rusta,
        // z których powstają moduły importowane przez JS.
        ImageKind::ViteStatic => {
            let mut units = node_closure(root, &app_dir)?;
            for stage in &spec.wasm {
                let crate_dir = root.join(&stage.crate_dir);
                if !crate_dir.is_dir() {
                    return Err(GraphError::DeadPath {
                        manifest: PathBuf::from("[[images.wasm]]"),
                        reference: stage.crate_dir.clone(),
                        resolved: crate_dir,
                    });
                }
                units.extend(rust_closure(root, &crate_dir)?);
            }
            units
        }
    };

    let mut rel_units = BTreeSet::new();
    for unit in &units {
        rel_units.insert(relative_to(root, unit)?);
    }

    let repos = map_to_repos(config, &rel_units)?;

    Ok(ImageGraph {
        image: spec.name.clone(),
        kind: spec.kind,
        app: spec.app.clone(),
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
                    let resolved = dir.join(rel);
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

    Ok(visited
        .into_iter()
        .filter(|d| d.starts_with(root))
        .collect())
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
