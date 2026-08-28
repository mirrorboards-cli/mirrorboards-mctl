//! Image declarations — `[[images]]` in mirror manifests.
//!
//! An image is declared next to the family that owns it, in the family's
//! manifest file. The unit of a build is a SUBGRAPH of the real workspace,
//! computed mechanically (see `graph`), never a hand-maintained copy of the
//! layout — hand-maintained copies rot on every layout migration.

use serde::{Deserialize, Serialize};

/// How an image is built. One generic, tool-owned Dockerfile per kind;
/// an app never carries its own copy of the build recipe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ImageKind {
    /// Rust binary: cargo-chef cook (deps layer) + cargo build in the app dir.
    RustBin,
    /// Node process executed by tsx straight from TypeScript — no build step.
    NodeTsx,
}

impl std::fmt::Display for ImageKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageKind::RustBin => write!(f, "rust-bin"),
            ImageKind::NodeTsx => write!(f, "node-tsx"),
        }
    }
}

/// A single `[[images]]` entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageSpec {
    /// Image name; also the default ghcr repository name.
    pub name: String,

    /// Workspace-relative path of the application directory.
    pub app: String,

    /// Build recipe kind.
    pub kind: ImageKind,

    /// Port the process listens on (EXPOSE + PORT env).
    pub port: u16,

    /// Registry repository, e.g. `ghcr.io/mirrorboards-xbooks/xbooks-api`.
    /// Required: the git org of the app repo is not derivable from a path.
    pub registry: String,

    /// Rust: binary name if it differs from `name`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bin: Option<String>,

    /// Node: command to exec, e.g. ["./node_modules/.bin/tsx", "src/index.ts", "--http"].
    /// Defaults to tsx on src/index.ts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cmd: Option<Vec<String>>,

    /// Extra environment baked into the image (non-secret only).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub env: Vec<EnvVar>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnvVar {
    pub name: String,
    pub value: String,
}

impl ImageSpec {
    pub fn bin_name(&self) -> &str {
        self.bin.as_deref().unwrap_or(&self.name)
    }

    pub fn command(&self) -> Vec<String> {
        self.cmd.clone().unwrap_or_else(|| {
            vec![
                "./node_modules/.bin/tsx".to_string(),
                "src/index.ts".to_string(),
            ]
        })
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("image name cannot be empty".to_string());
        }
        if self.app.is_empty() || self.app.starts_with('/') {
            return Err(format!(
                "image '{}': app must be a workspace-relative path",
                self.name
            ));
        }
        if self.registry.is_empty() {
            return Err(format!("image '{}': registry is required", self.name));
        }
        Ok(())
    }
}
