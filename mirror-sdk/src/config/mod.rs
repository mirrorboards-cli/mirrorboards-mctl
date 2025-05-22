//! Configuration handling for the Mirror SDK.

pub mod paths;
pub mod settings;

pub use paths::{get_config_path, get_home_dir, ENV_MIRROR_CONFIG, DEFAULT_CONFIG_FILENAME};
pub use settings::ConfigSettings;