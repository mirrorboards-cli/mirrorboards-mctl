use serde::{Deserialize, Serialize};

/// Configuration for central config repository
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConfigRepo {
    /// Git URL for the config repository
    pub git: String,

    /// Branch to use (default: "main")
    #[serde(default = "default_branch")]
    pub branch: String,

    /// Path to mirror.toml within the repo
    #[serde(default = "default_config_path")]
    pub config_path: String,

    /// Directory for snapshots within the repo
    #[serde(default = "default_snapshots_dir")]
    pub snapshots_dir: String,
}

fn default_branch() -> String {
    "main".to_string()
}

fn default_config_path() -> String {
    "mirror.toml".to_string()
}

fn default_snapshots_dir() -> String {
    "snapshots".to_string()
}

impl Default for ConfigRepo {
    fn default() -> Self {
        Self {
            git: String::new(),
            branch: default_branch(),
            config_path: default_config_path(),
            snapshots_dir: default_snapshots_dir(),
        }
    }
}
