//! Integration tests for the CLI interface
//!
//! These tests validate the command-line interface functionality
//! by executing the actual binary with various arguments and checking
//! the output and exit codes.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tempfile::{tempdir, TempDir};

// Helper function to create a new temporary config file
fn create_test_config(dir: &Path) -> std::io::Result<String> {
    let config_path = dir.join("mirror.toml");
    let mut file = File::create(&config_path)?;
    
    write!(file, r#"
# MCTL Configuration File

[global]
base_dir = "./repos"

[auth]
ssh_key_path = "~/.ssh/id_rsa"

[logging]
level = "info"
file = "~/.config/mctl/logs/mctl.log"

[commands.sync]
parallel = true
recursive = true

# Repository definitions
[[repositories]]
path = "test-repo-1"
origin = "https://github.com/example/repo1.git"
branch = "main"
enabled = true
tags = ["core", "documentation"]

[[repositories]]
path = "test-repo-2"
origin = "https://github.com/example/repo2.git"
branch = "develop"
enabled = true
tags = ["plugin"]
"#)?;
    
    Ok(config_path.to_string_lossy().to_string())
}

#[test]
fn test_version_flag() {
    let mut cmd = Command::cargo_bin("mctl").unwrap();
    
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_help_flag() {
    let mut cmd = Command::cargo_bin("mctl").unwrap();
    
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("Commands:"))
        .stdout(predicate::str::contains("Options:"));
}

#[test]
fn test_invalid_command() {
    let mut cmd = Command::cargo_bin("mctl").unwrap();
    
    cmd.arg("invalid-command");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("error:"));
}

#[test]
fn test_verbose_flags() {
    let mut cmd = Command::cargo_bin("mctl").unwrap();
    
    cmd.args(&["-v", "status"]);
    cmd.assert().success();
    
    let mut cmd = Command::cargo_bin("mctl").unwrap();
    cmd.args(&["-vv", "status"]);
    cmd.assert().success();
    
    let mut cmd = Command::cargo_bin("mctl").unwrap();
    cmd.args(&["-vvv", "status"]);
    cmd.assert().success();
}

#[test]
fn test_init_command() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let output_path = temp_dir.path().join("test-config.toml");
    
    // Test init command with output path
    let mut cmd = Command::cargo_bin("mctl").unwrap();
    cmd.args(&["init", "--output", output_path.to_str().unwrap()]);
    cmd.assert().success();
    
    // Verify file was created
    assert!(output_path.exists(), "Config file was not created");
    
    // Check content
    let content = fs::read_to_string(output_path).unwrap();
    assert!(content.contains("[global]"), "Config file missing global section");
    assert!(content.contains("[[repositories]]"), "Config file missing repositories section");
}

#[test]
fn test_status_command_with_config() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().unwrap();
    let config_path = create_test_config(temp_dir.path()).unwrap();
    
    // Test status command with config file
    let mut cmd = Command::cargo_bin("mctl").unwrap();
    cmd.args(&["--config-path", &config_path, "status"]);
    
    // Since we don't have actual repositories to check, we expect a controlled error
    // about repositories not being found, not a crash
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_sync_command_args() {
    // Test sync command with various arguments to ensure they're parsed correctly
    let mut cmd = Command::cargo_bin("mctl").unwrap();
    cmd.args(&["sync", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--recursive"))
        .stdout(predicate::str::contains("--parallel"))
        .stdout(predicate::str::contains("--depth"));
    
    // Ensure we can pass repository names
    let mut cmd = Command::cargo_bin("mctl").unwrap();
    cmd.args(&["sync", "repo1", "repo2"]);
    // We expect a controlled failure since these repos don't exist
    cmd.assert().failure();
    
    // Ensure we can use flags
    let mut cmd = Command::cargo_bin("mctl").unwrap();
    cmd.args(&["sync", "--recursive", "--parallel"]);
    cmd.assert().failure(); // Expected failure since repos don't exist
}

#[test]
fn test_save_command_args() {
    // Test save command with various arguments
    let mut cmd = Command::cargo_bin("mctl").unwrap();
    cmd.args(&["save", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--message"))
        .stdout(predicate::str::contains("--push"))
        .stdout(predicate::str::contains("--sign"));
    
    // Ensure we can specify a commit message
    let mut cmd = Command::cargo_bin("mctl").unwrap();
    cmd.args(&["save", "--message", "Test commit message"]);
    cmd.assert().failure(); // Expected failure since repos don't exist
}

#[test]
fn test_output_formats() {
    // Test different output formats
    let mut cmd = Command::cargo_bin("mctl").unwrap();
    cmd.args(&["--format", "json", "status"]);
    cmd.assert().failure(); // Expected failure since repos don't exist
    
    let mut cmd = Command::cargo_bin("mctl").unwrap();
    cmd.args(&["--format", "text", "status"]);
    cmd.assert().failure(); // Expected failure since repos don't exist
    
    let mut cmd = Command::cargo_bin("mctl").unwrap();
    cmd.args(&["--format", "compact", "status"]);
    cmd.assert().failure(); // Expected failure since repos don't exist
}

#[test]
fn test_color_modes() {
    // Test different color modes
    let mut cmd = Command::cargo_bin("mctl").unwrap();
    cmd.args(&["--color", "always", "status"]);
    cmd.assert().failure(); // Expected failure since repos don't exist
    
    let mut cmd = Command::cargo_bin("mctl").unwrap();
    cmd.args(&["--color", "never", "status"]);
    cmd.assert().failure(); // Expected failure since repos don't exist
    
    let mut cmd = Command::cargo_bin("mctl").unwrap();
    cmd.args(&["--color", "auto", "status"]);
    cmd.assert().failure(); // Expected failure since repos don't exist
}