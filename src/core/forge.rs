//! Build-context assembly and image building from a computed graph.
//!
//! The context is a staging directory mirroring the REAL workspace layout,
//! reduced to the image's closure, with root manifests GENERATED from the
//! graph — the slim roots stop being hand-maintained files that rot on
//! every layout migration.

use crate::core::graph::ImageGraph;
use crate::core::image::{ImageKind, ImageSpec};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, thiserror::Error)]
pub enum ForgeError {
    #[error("io on {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("cannot parse {path}: {message}")]
    Parse { path: PathBuf, message: String },
    #[error("workspace root {0} has no {1} — is this the assembled workspace?")]
    MissingRootFile(PathBuf, &'static str),
    #[error("docker failed with status {0}")]
    Docker(i32),
    #[error("cannot run docker: {0}")]
    DockerSpawn(std::io::Error),
}

const RUST_BIN_TEMPLATE: &str = include_str!("../../templates/rust-bin.Dockerfile");
const NODE_TSX_TEMPLATE: &str = include_str!("../../templates/node-tsx.Dockerfile");
const VITE_STATIC_TEMPLATE: &str = include_str!("../../templates/vite-static.Dockerfile");

/// Dirs never copied into a context: VCS metadata and build products.
const EXCLUDED_DIRS: &[&str] = &[".git", "target", "node_modules", "dist", ".github-images"];

/// A record of exactly what went into the image — written into the context
/// and baked into the image as a label. Rebuildability is a property of the
/// receipt, not of anyone's memory.
#[derive(Debug, Serialize)]
pub struct ForgeReceipt {
    pub image: String,
    pub app: String,
    /// repo mirror-path -> HEAD commit at context time.
    pub repos: std::collections::BTreeMap<String, String>,
}

/// Assembles the build context for `spec` into `out_dir`.
pub fn assemble_context(
    root: &Path,
    spec: &ImageSpec,
    graph: &ImageGraph,
    out_dir: &Path,
) -> Result<ForgeReceipt, ForgeError> {
    if out_dir.exists() {
        std::fs::remove_dir_all(out_dir).map_err(|e| io_err(out_dir, e))?;
    }
    std::fs::create_dir_all(out_dir).map_err(|e| io_err(out_dir, e))?;

    // 1. Closure dirs, real layout preserved.
    for unit in &graph.units {
        copy_tree(&root.join(unit), &out_dir.join(unit))?;
    }

    // 2. Generated root manifests.
    match spec.kind {
        ImageKind::RustBin => generate_rust_root(root, graph, out_dir)?,
        ImageKind::NodeTsx => generate_node_root(root, graph, out_dir)?,
        // Front potrzebuje OBU korzeni: pnpm dla paczek i Cargo dla krat,
        // z których wasm-pack robi moduły.
        ImageKind::ViteStatic => {
            generate_node_root(root, graph, out_dir)?;
            if !spec.wasm.is_empty() {
                generate_rust_root(root, graph, out_dir)?;
            }
        }
    }

    // 3. Rendered Dockerfile — values baked in, no build args to drift.
    let dockerfile = render_dockerfile(spec);
    write(&out_dir.join("Dockerfile"), &dockerfile)?;

    // 4. Receipt: repo -> HEAD.
    let receipt = receipt(root, spec, graph);
    write(
        &out_dir.join("forge-receipt.json"),
        &serde_json::to_string_pretty(&receipt).expect("receipt serializes"),
    )?;

    Ok(receipt)
}

/// Builds (and optionally pushes) the image from an assembled context.
#[allow(clippy::too_many_arguments)]
pub fn build_image(
    context: &Path,
    spec: &ImageSpec,
    receipt: &ForgeReceipt,
    tag: &str,
    push: bool,
    load: bool,
    no_cache_store: bool,
) -> Result<(), ForgeError> {
    let image_ref = format!("{}:{}", spec.registry, tag);
    let cache_ref = format!("{}:forge-cache", spec.registry);

    let mut cmd = Command::new("docker");
    cmd.arg("buildx")
        .arg("build")
        .arg(context)
        .arg("--tag")
        .arg(&image_ref)
        .arg("--cache-from")
        .arg(format!("type=registry,ref={cache_ref}"))
        .arg("--label")
        .arg(format!(
            "io.mirrorboards.forge.receipt={}",
            serde_json::to_string(receipt).expect("receipt serializes")
        ));
    // Zapis cache'u wymaga prawa pushu do rejestru — lokalny build bez
    // logowania do ghcr nadal MOŻE czytać cache i budować z --load.
    if !no_cache_store {
        cmd.arg("--cache-to")
            .arg(format!("type=registry,ref={cache_ref},mode=max"));
    }
    if push {
        cmd.arg("--push");
    }
    if load {
        cmd.arg("--load");
    }

    let status = cmd.status().map_err(ForgeError::DockerSpawn)?;
    if !status.success() {
        return Err(ForgeError::Docker(status.code().unwrap_or(-1)));
    }
    Ok(())
}

// ── Root manifest generation ────────────────────────────────────────────────

/// Root Cargo.toml for the context: the REAL root's workspace sections
/// (package inheritance, shared dependencies, profiles) with `members`
/// narrowed to the closure and `exclude` covering closure dirs that are
/// workspace roots of their own (the app included).
fn generate_rust_root(
    root: &Path,
    graph: &ImageGraph,
    out_dir: &Path,
) -> Result<(), ForgeError> {
    let root_manifest_path = root.join("Cargo.toml");
    if !root_manifest_path.is_file() {
        return Err(ForgeError::MissingRootFile(root.to_path_buf(), "Cargo.toml"));
    }
    let mut manifest: toml::Value = read_toml(&root_manifest_path)?;

    let mut members = Vec::new();
    let mut excludes = Vec::new();
    for unit in &graph.units {
        // W obrazie mieszanym (vite-static) większość jednostek to paczki
        // Node — do workspace'u Rusta wchodzą tylko te z manifestem.
        let unit_manifest_path = root.join(unit).join("Cargo.toml");
        if !unit_manifest_path.is_file() {
            continue;
        }
        let unit_manifest = read_toml(&unit_manifest_path)?;
        let own_workspace = unit_manifest.get("workspace");
        let empty_marker = own_workspace
            .and_then(|w| w.as_table())
            .map(|t| t.is_empty())
            .unwrap_or(false);
        if own_workspace.is_some() && !empty_marker {
            // Prawdziwy własny workspace (np. vendorowany zitadel) — wyłączamy.
            excludes.push(unit.clone());
        } else {
            // Pusty znacznik `[workspace]` służył tylko odcięciu od korzenia
            // NA DYSKU; w kontekście zdejmujemy go (patrz strip w kopii)
            // i krata zostaje zwykłym członkiem jednego workspace'u.
            if empty_marker {
                strip_empty_workspace_marker(&out_dir.join(unit).join("Cargo.toml"))?;
            }
            members.push(unit.clone());
        }
    }

    if let Some(workspace) = manifest.get_mut("workspace").and_then(|w| w.as_table_mut()) {
        workspace.insert(
            "members".to_string(),
            toml::Value::Array(members.into_iter().map(toml::Value::String).collect()),
        );
        workspace.insert(
            "exclude".to_string(),
            toml::Value::Array(excludes.into_iter().map(toml::Value::String).collect()),
        );
    }

    write(
        &out_dir.join("Cargo.toml"),
        &toml::to_string_pretty(&manifest).map_err(|e| ForgeError::Parse {
            path: root_manifest_path,
            message: e.to_string(),
        })?,
    )?;

    // Root lock pins third-party versions; extra entries are pruned by cargo.
    let lock = root.join("Cargo.lock");
    if lock.is_file() {
        std::fs::copy(&lock, out_dir.join("Cargo.lock")).map_err(|e| io_err(&lock, e))?;
    }
    Ok(())
}

/// Root pnpm-workspace.yaml + stub package.json for the context.
fn generate_node_root(
    root: &Path,
    graph: &ImageGraph,
    out_dir: &Path,
) -> Result<(), ForgeError> {
    let mut yaml = String::from(
        "# WYGENEROWANE przez mctl z grafu obrazu — nie edytować ręcznie.\npackages:\n",
    );
    for unit in &graph.units {
        yaml.push_str(&format!("  - \"{unit}\"\n"));
    }
    // Ustawienia budowania skryptów przenoszą się z prawdziwego korzenia.
    let real = root.join("pnpm-workspace.yaml");
    if real.is_file() {
        let content = std::fs::read_to_string(&real).map_err(|e| io_err(&real, e))?;
        let mut in_section = false;
        for line in content.lines() {
            if line.starts_with("onlyBuiltDependencies:") {
                in_section = true;
                yaml.push_str(line);
                yaml.push('\n');
            } else if in_section {
                if line.starts_with("  ") {
                    yaml.push_str(line);
                    yaml.push('\n');
                } else {
                    in_section = false;
                }
            }
        }
    }
    write(&out_dir.join("pnpm-workspace.yaml"), &yaml)?;

    let package_manager = read_json(&root.join("package.json"))
        .ok()
        .and_then(|p| {
            p.get("packageManager")
                .and_then(|v| v.as_str())
                .map(String::from)
        })
        .unwrap_or_else(|| "pnpm@10.15.0".to_string());
    write(
        &out_dir.join("package.json"),
        &format!(
            "{{\n\t\"name\": \"{}-image-root\",\n\t\"private\": true,\n\t\"packageManager\": \"{}\"\n}}\n",
            "forge", package_manager
        ),
    )?;
    Ok(())
}

// ── Dockerfile rendering ────────────────────────────────────────────────────

fn render_dockerfile(spec: &ImageSpec) -> String {
    let template = match spec.kind {
        ImageKind::RustBin => RUST_BIN_TEMPLATE,
        ImageKind::NodeTsx => NODE_TSX_TEMPLATE,
        ImageKind::ViteStatic => VITE_STATIC_TEMPLATE,
    };
    let cmd_json = serde_json::to_string(&spec.command()).expect("cmd serializes");
    let env_lines: String = spec
        .env
        .iter()
        .map(|e| format!("ENV {}={}\n", e.name, e.value))
        .collect();
    // Etapy WASM: jeden blok build + jeden COPY do etapu JS na sztukę.
    let wasm_stages: String = spec
        .wasm
        .iter()
        .enumerate()
        .map(|(index, stage)| {
            format!(
                "FROM wasm AS wasm{index}\nWORKDIR /workspace/{}\nRUN wasm-pack build --target web --release --out-dir /out --out-name {}\n\n",
                stage.crate_dir, stage.out_name
            )
        })
        .collect();
    let wasm_copies: String = spec
        .wasm
        .iter()
        .enumerate()
        .map(|(index, stage)| {
            format!(
                "COPY --from=wasm{index} /out /workspace/{}\n",
                stage.out_dir
            )
        })
        .collect();
    // Vite WKLEJA `import.meta.env.PUBLIC_*` w buildzie, więc te wartości
    // muszą stać w środowisku etapu budowania — ustawienie ich w kontenerze
    // nie ma żadnego skutku.
    let public_lines: String = spec
        .public
        .iter()
        .map(|e| format!("ENV {}={}\n", e.name, e.value))
        .collect();
    let runtime_copies: String = spec
        .runtime_files
        .iter()
        .map(|f| {
            let name = f.rsplit('/').next().unwrap_or(f);
            format!("COPY --from=builder /workspace/{}/{f} /app/{name}\n", spec.app)
        })
        .collect();

    template
        .replace("{{WASM_STAGES}}", &wasm_stages)
        .replace("{{WASM_COPIES}}", &wasm_copies)
        .replace("{{PUBLIC_LINES}}", &public_lines)
        .replace("{{RUNTIME_COPIES}}", &runtime_copies)
        .replace("{{APP_DIR}}", &spec.app)
        .replace("{{BIN}}", spec.bin_name())
        .replace("{{PORT}}", &spec.port.to_string())
        .replace("{{CMD_JSON}}", &cmd_json)
        .replace("{{ENV_LINES}}", &env_lines)
}

// ── Receipt ────────────────────────────────────────────────────────────────

fn receipt(root: &Path, spec: &ImageSpec, graph: &ImageGraph) -> ForgeReceipt {
    let mut repos = std::collections::BTreeMap::new();
    for path in graph.repos.keys() {
        let head = Command::new("git")
            .arg("-C")
            .arg(root.join(path))
            .arg("rev-parse")
            .arg("HEAD")
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        repos.insert(path.clone(), head);
    }
    ForgeReceipt {
        image: spec.name.clone(),
        app: spec.app.clone(),
        repos,
    }
}

// ── Copy helpers ────────────────────────────────────────────────────────────

fn copy_tree(from: &Path, to: &Path) -> Result<(), ForgeError> {
    std::fs::create_dir_all(to).map_err(|e| io_err(to, e))?;
    for entry in std::fs::read_dir(from).map_err(|e| io_err(from, e))? {
        let entry = entry.map_err(|e| io_err(from, e))?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        let path = entry.path();
        if path.is_dir() {
            if EXCLUDED_DIRS.contains(&name_str.as_ref()) {
                continue;
            }
            copy_tree(&path, &to.join(&name))?;
        } else {
            std::fs::copy(&path, to.join(&name)).map_err(|e| io_err(&path, e))?;
        }
    }
    Ok(())
}

/// Zdejmuje PUSTY znacznik `[workspace]` ze skopiowanego manifestu.
fn strip_empty_workspace_marker(path: &Path) -> Result<(), ForgeError> {
    let mut manifest = read_toml(path)?;
    if let Some(table) = manifest.as_table_mut() {
        let is_empty_marker = table
            .get("workspace")
            .and_then(|w| w.as_table())
            .map(|t| t.is_empty())
            .unwrap_or(false);
        if is_empty_marker {
            table.remove("workspace");
            write(
                path,
                &toml::to_string_pretty(&manifest).map_err(|e| ForgeError::Parse {
                    path: path.to_path_buf(),
                    message: e.to_string(),
                })?,
            )?;
        }
    }
    Ok(())
}

fn write(path: &Path, content: &str) -> Result<(), ForgeError> {
    std::fs::write(path, content).map_err(|e| io_err(path, e))
}

fn io_err(path: &Path, source: std::io::Error) -> ForgeError {
    ForgeError::Io {
        path: path.to_path_buf(),
        source,
    }
}

fn read_toml(path: &Path) -> Result<toml::Value, ForgeError> {
    let content = std::fs::read_to_string(path).map_err(|e| io_err(path, e))?;
    toml::from_str(&content).map_err(|e| ForgeError::Parse {
        path: path.to_path_buf(),
        message: e.to_string(),
    })
}

fn read_json(path: &Path) -> Result<serde_json::Value, ForgeError> {
    let content = std::fs::read_to_string(path).map_err(|e| io_err(path, e))?;
    serde_json::from_str(&content).map_err(|e| ForgeError::Parse {
        path: path.to_path_buf(),
        message: e.to_string(),
    })
}
