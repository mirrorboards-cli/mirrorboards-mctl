use thiserror::Error;

#[derive(Error, Debug)]
pub enum MirrorError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Include cycle detected: {0}")]
    IncludeCycle(String),

    #[error("Duplicate path found: {0}")]
    DuplicatePath(String),

    #[error("Git operation failed: {0}")]
    Git(#[from] git2::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("TOML serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("Config repo not configured")]
    ConfigRepoNotConfigured,

    #[error("Snapshot not found: {0}")]
    SnapshotNotFound(String),

    #[error("Repository not found: {0}")]
    RepositoryNotFound(String),

    #[error("Workspace not found: {0}")]
    WorkspaceNotFound(String),

    #[error("Invalid RefSpec: cannot specify multiple of branch/tag/rev")]
    InvalidRefSpec,
}

pub type Result<T> = std::result::Result<T, MirrorError>;
